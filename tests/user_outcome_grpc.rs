//! Installed-host A2A acceptance using generated tonic clients.
//!
//! No runtime, datastore, mock driver or optional green skip is created here.
//! The exact packet must provide a running real host, authenticated owner and
//! genuine configured model. Run the named test with a2a-transport enabled.
#![cfg(feature = "a2a-transport")]

use serde::Deserialize;
use serde_json::{Value, json};
use std::{error::Error, fs::OpenOptions, io::Write, time::Duration};
use tonic::{Request, transport::Channel};
use universal_agent_runtime::uar::api::a2a::{grpc::pb as legacy, grpc_v1::pb as v1};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Deserialize)]
struct Input {
    base_url: String,
    grpc_endpoint: Option<String>,
    authorization_env: String,
    agent_id: String,
    model: String,
    mode: String,
    turns: Vec<String>,
    expected: Vec<Value>,
    timeout_seconds: f64,
    max_bytes: usize,
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition { Ok(()) } else { Err(message.into()) }
}

fn authenticated<T>(body: T, authorization: &str, agent: &str, timeout: Duration) -> Result<Request<T>> {
    let mut request = Request::new(body);
    request.metadata_mut().insert("authorization", authorization.parse()?);
    request.metadata_mut().insert("x-uar-agent-id", agent.parse()?);
    request.set_timeout(timeout);
    Ok(request)
}

fn legacy_text(task: &legacy::TaskResponse) -> String {
    task.messages.iter().rev().find(|message| message.role == "agent")
        .map(|message| message.parts.iter().filter_map(|part| match &part.content {
            Some(legacy::part::Content::Text(text)) => Some(text.as_str()), _ => None,
        }).collect()).unwrap_or_default()
}

fn v1_text(task: &v1::Task) -> String {
    task.status.as_ref().and_then(|status| status.message.as_ref())
        .map(|message| message.parts.iter().filter_map(|part| match &part.content {
            Some(v1::part::Content::Text(text)) => Some(text.as_str()), _ => None,
        }).collect()).unwrap_or_default()
}

async fn task_receipt(http: &reqwest::Client, input: &Input, authorization: &str,
                      task_id: &str, official: bool) -> Result<Value> {
    let path = if official { "/a2a/v1/agents/" } else { "/a2a/agents/" };
    let method = if official { "GetTask" } else { "tasks/get" };
    let url = format!("{}{}{}", input.base_url.trim_end_matches('/'), path, input.agent_id);
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut request = http.post(url).header("Authorization", authorization)
        .json(&json!({"jsonrpc":"2.0","id":request_id,"method":method,"params":{"id":task_id}}));
    if official { request = request.header("a2a-version", "1.0"); }
    let response = request.send().await?;
    require(response.status().is_success(), "companion task lookup HTTP failed")?;
    let raw = response.bytes().await?;
    require(raw.len() <= input.max_bytes, "companion task receipt exceeded bound")?;
    let value: Value = serde_json::from_slice(&raw)?;
    require(value["id"] == request_id && value.get("error").is_none(), "companion task RPC failed")?;
    let task = value.get("result").ok_or("task result missing")?;
    let expected_state = if official { "TASK_STATE_COMPLETED" } else { "completed" };
    require(task["id"] == task_id && task["status"]["state"] == expected_state,
            "gRPC task does not have successful native companion state")?;
    require(task["metadata"]["agent_id"] == input.agent_id, "gRPC selected wrong persisted agent")?;
    require(task["metadata"]["run_id"].as_str().is_some_and(|value| !value.is_empty()),
            "gRPC native run correlation missing")?;
    Ok(task.clone())
}

async fn legacy_turn(client: &mut legacy::agent_service_client::AgentServiceClient<Channel>,
                     input: &Input, authorization: &str, task_id: &str,
                     text: &str, timeout: Duration) -> Result<legacy::TaskResponse> {
    let body = legacy::SendMessageRequest {
        task_id: task_id.to_owned(),
        message: Some(legacy::Message { role: "user".into(), parts: vec![legacy::Part {
            content: Some(legacy::part::Content::Text(text.to_owned())), content_type: "text/plain".into(),
        }] }),
    };
    if input.mode == "json" {
        let initial = client.message_send(authenticated(body, authorization, &input.agent_id, timeout)?).await?.into_inner();
        let mut task = initial;
        for _ in 0..40 {
            require(!matches!(task.status.as_str(), "failed" | "canceled" | "inputrequired"), "legacy gRPC task failed")?;
            if task.status == "completed" { return Ok(task); }
            task = client.task_get(authenticated(legacy::GetTaskRequest { task_id: task.task_id },
                authorization, &input.agent_id, timeout)?).await?.into_inner();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Err("legacy gRPC task poll allowance exhausted".into())
    } else {
        let mut events = client.message_stream(authenticated(body, authorization, &input.agent_id, timeout)?).await?.into_inner();
        let mut completed = None;
        let mut stream_task = None;
        let mut count = 0usize;
        while let Some(event) = events.message().await? {
            count += 1;
            require(count <= 4096, "gRPC event allowance exhausted")?;
            if let Some(previous) = &stream_task { require(previous == &event.task_id, "gRPC stream changed task")?; }
            stream_task = Some(event.task_id.clone());
            if let Some(task) = event.state {
                require(!matches!(task.status.as_str(), "failed" | "canceled" | "inputrequired"), "legacy gRPC stream failed")?;
                if task.status == "completed" { completed = Some(task); }
            }
        }
        if let Some(task) = completed { return Ok(task); }
        // The mounted legacy transport emits one status snapshot, rather than
        // a live text stream. Resolve that exact task with the actual TaskGet
        // method; never treat submitted/working as successful completion.
        require(count == 1, "legacy gRPC snapshot contract changed")?;
        let mut task_id = stream_task.ok_or("legacy stream task identity missing")?;
        for _ in 0..40 {
            let task = client.task_get(authenticated(legacy::GetTaskRequest { task_id },
                authorization, &input.agent_id, timeout)?).await?.into_inner();
            require(!matches!(task.status.as_str(), "failed" | "canceled" | "inputrequired"), "legacy gRPC streamed task failed")?;
            if task.status == "completed" { return Ok(task); }
            task_id = task.task_id;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Err("legacy gRPC streamed task poll bound exhausted".into())
    }
}

async fn v1_turn(client: &mut v1::a2a_service_client::A2aServiceClient<Channel>, input: &Input,
                 authorization: &str, task_id: &str, context: &str, text: &str,
                 timeout: Duration) -> Result<v1::Task> {
    let body = v1::SendMessageRequest {
        message: Some(v1::Message { message_id: uuid::Uuid::new_v4().to_string(),
            task_id: task_id.to_owned(), context_id: context.to_owned(), role: 1,
            parts: vec![v1::Part { content: Some(v1::part::Content::Text(text.to_owned())),
                media_type: "text/plain".into(), ..Default::default() }], ..Default::default() }),
        configuration: Some(v1::SendMessageConfiguration { return_immediately: false, ..Default::default() }),
        ..Default::default()
    };
    if input.mode == "json" {
        let response = client.send_message(authenticated(body, authorization, &input.agent_id, timeout)?).await?.into_inner();
        match response.payload {
            Some(v1::send_message_response::Payload::Task(task)) => Ok(task),
            _ => Err("official gRPC did not return a task".into()),
        }
    } else {
        let mut stream = client.send_streaming_message(authenticated(body, authorization, &input.agent_id, timeout)?).await?.into_inner();
        let mut id = None;
        let mut complete = false;
        let mut count = 0usize;
        while let Some(event) = stream.message().await? {
            count += 1;
            require(count <= 4096, "official gRPC event allowance exhausted")?;
            match event.payload {
                Some(v1::stream_response::Payload::Task(task)) => {
                    if let Some(previous) = &id { require(previous == &task.id, "official gRPC changed task")?; }
                    id = Some(task.id.clone());
                    let state = task.status.as_ref().map(|status| status.state).unwrap_or(0);
                    require(!matches!(state, 4..=8), "official gRPC task failed or interrupted")?;
                    complete = state == 3;
                }
                Some(v1::stream_response::Payload::StatusUpdate(update)) => {
                    if let Some(previous) = &id { require(previous == &update.task_id, "official gRPC changed task")?; }
                    id = Some(update.task_id);
                    let state = update.status.as_ref().map(|status| status.state).unwrap_or(0);
                    require(!matches!(state, 4..=8), "official gRPC stream failed or interrupted")?;
                    complete = state == 3;
                }
                _ => {}
            }
        }
        require(complete, "official gRPC stream lacks successful terminal")?;
        let task_id = id.ok_or("official gRPC task identity missing")?;
        let task = client.get_task(authenticated(v1::GetTaskRequest { id: task_id, ..Default::default() },
            authorization, &input.agent_id, timeout)?).await?.into_inner();
        Ok(task)
    }
}

async fn acceptance(input: &Input, official: bool) -> Result<Value> {
    require(input.turns.len() == 2 && input.expected.len() == 2, "exact two-turn input required")?;
    require(matches!(input.mode.as_str(), "json" | "stream"), "unknown gRPC mode")?;
    require(input.timeout_seconds.is_finite() && input.timeout_seconds > 0.0, "finite timeout required")?;
    let timeout = Duration::from_secs_f64(input.timeout_seconds);
    let authorization = std::env::var(&input.authorization_env)?;
    require(authorization.starts_with("Bearer ") && authorization.len() > 7, "authenticated gRPC credential required")?;
    let endpoint = input.grpc_endpoint.as_ref().filter(|value| !value.is_empty()).ok_or("installed gRPC endpoint required")?;
    let channel = Channel::from_shared(endpoint.clone())?.connect_timeout(timeout).timeout(timeout).connect().await?;
    let http = reqwest::Client::builder().timeout(timeout).build()?;
    let mut old_client = legacy::agent_service_client::AgentServiceClient::new(channel.clone());
    let mut new_client = v1::a2a_service_client::A2aServiceClient::new(channel);
    let mut task_id = String::new();
    let mut context = String::new();
    let mut turns = Vec::new();
    for (text, expected) in input.turns.iter().zip(&input.expected) {
        let (id, content) = if official {
            let task = tokio::time::timeout(timeout, v1_turn(&mut new_client, input, &authorization, &task_id, &context, text, timeout)).await??;
            require(task.status.as_ref().map(|status| status.state) == Some(3), "official gRPC task did not complete")?;
            (task.id.clone(), v1_text(&task))
        } else {
            let task = tokio::time::timeout(timeout, legacy_turn(&mut old_client, input, &authorization, &task_id, text, timeout)).await??;
            require(task.status == "completed", "legacy gRPC task did not complete")?;
            (task.task_id.clone(), legacy_text(&task))
        };
        require(content.len() <= input.max_bytes && serde_json::from_str::<Value>(&content)? == *expected,
                "gRPC useful-result predicate failed")?;
        let receipt = task_receipt(&http, input, &authorization, &id, official).await?;
        let key = if official { "contextId" } else { "context_id" };
        let next_context = receipt[key].as_str().ok_or("gRPC context missing")?;
        if !task_id.is_empty() { require(task_id == id && context == next_context, "gRPC continuation changed task/context")?; }
        task_id = id;
        context = next_context.to_owned();
        turns.push(json!({"content":content,"run_id":receipt["metadata"]["run_id"],
            "context_id":context,"task_id":task_id,"protocol_terminal":"success"}));
    }
    Ok(json!({"status":"PASS","client":if official {"grpc_v1"} else {"grpc_legacy"},
              "mode":input.mode,"model":input.model,"turns":turns}))
}

#[tokio::test]
async fn installed_grpc_user_outcomes() -> Result<()> {
    let output = std::env::var("UAR_ACCEPTANCE_CLIENT_OUTPUT")?;
    let raw = std::env::var("UAR_ACCEPTANCE_CLIENT_INPUT")?;
    let input: Input = serde_json::from_str(&raw)?;
    let version = std::env::var("UAR_ACCEPTANCE_GRPC_VERSION")?;
    require(matches!(version.as_str(), "legacy" | "v1"), "exact gRPC version required")?;
    let result = acceptance(&input, version == "v1").await;
    let receipt = result.as_ref().map_or_else(|error| json!({"status":"FAIL","client":if version == "v1" {"grpc_v1"} else {"grpc_legacy"},
        "error":error.to_string().chars().take(8192).collect::<String>()}), Value::clone);
    let mut file = OpenOptions::new().write(true).create_new(true).open(output)?;
    file.write_all(serde_json::to_string(&receipt)?.as_bytes())?;
    file.sync_all()?;
    result.map(|_| ())
}
