//! A2A v1.0.0 JSON-RPC binding. Legacy wire types stay confined to adapters.
//! Normative contract: https://github.com/a2aproject/A2A/tree/v1.0.0/specification
use std::{collections::HashMap, sync::Arc};

use axum::{Extension, Json, body::Bytes, extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{admission::A2aAdmissionError, handler::A2AState, thread_service::TaskError, types::{self, JsonRpcRequest, JsonRpcResponse, Task}};
use crate::uar::{runtime::actor::messages::ActorOwner, security::{authority::authorize_context, claims::UserContext, credential_capture::AuthenticatedCredentialCapture}};

/// Exact discovery shape from the pinned 1.0 schema.
pub fn agent_card(base_url: &str, grpc_url: Option<&str>) -> Value {
    let legacy = super::agent_card::build_agent_card(base_url);
    let mut interfaces = vec![json!({"url": format!("{}/a2a/v1/compiler", base_url.trim_end_matches('/')),
        "protocolBinding": "JSONRPC", "protocolVersion": "1.0"})];
    if let Some(grpc_url) = grpc_url.filter(|url| !url.trim().is_empty()) {
        interfaces.push(json!({"url": grpc_url, "protocolBinding": "GRPC", "protocolVersion": "1.0"}));
    }
    json!({
        "name": legacy.name, "description": legacy.description,
        "version": env!("CARGO_PKG_VERSION"), "documentationUrl": legacy.documentation_url,
        "supportedInterfaces": interfaces,
        "capabilities": {"streaming": true, "pushNotifications": false, "extendedAgentCard": false},
        "securitySchemes": {
            "bearer": {"httpAuthSecurityScheme": {"scheme": "Bearer", "bearerFormat": "JWT"}},
            "apiKey": {"apiKeySecurityScheme": {"location": "header", "name": "X-API-Key"}}
        },
        "securityRequirements": [{"schemes": {"bearer": {"list": []}}}, {"schemes": {"apiKey": {"list": []}}}],
        "defaultInputModes": ["text/plain"], "defaultOutputModes": ["text/plain", "application/json"],
        "skills": legacy.skills.into_iter().map(|skill| json!({
            "id": skill.id, "name": skill.name, "description": skill.description,
            "tags": skill.tags, "examples": skill.examples,
            "inputModes": ["text/plain"], "outputModes": ["text/plain", "application/json"]
        })).collect::<Vec<_>>()
    })
}

pub async fn handle_agent_card(State(state): State<Arc<A2AState>>) -> Json<Value> {
    Json(agent_card(&state.base_url, state.grpc_url.as_deref()))
}

pub async fn handle_rpc(
    State(state): State<Arc<A2AState>>, user: Option<Extension<UserContext>>,
    capture: Option<Extension<AuthenticatedCredentialCapture>>, headers: HeaderMap,
    Query(parameters): Query<HashMap<String, String>>, body: Bytes,
) -> Response {
    dispatch(&state, "compiler-agent", user, capture, &headers, &parameters, body).await
}

pub async fn handle_agent_rpc(
    State(state): State<Arc<A2AState>>, Path(agent): Path<String>, user: Option<Extension<UserContext>>,
    capture: Option<Extension<AuthenticatedCredentialCapture>>, headers: HeaderMap,
    Query(parameters): Query<HashMap<String, String>>, body: Bytes,
) -> Response {
    dispatch(&state, &agent, user, capture, &headers, &parameters, body).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendParams {
    message: WireMessage,
    #[serde(default)] configuration: Configuration,
    #[serde(default)] metadata: HashMap<String, Value>,
    #[serde(default)] tenant: String,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Configuration {
    #[serde(default)] return_immediately: bool,
    history_length: Option<usize>,
    task_push_notification_config: Option<Value>,
    #[serde(default)] accepted_output_modes: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireMessage {
    message_id: String,
    role: String,
    parts: Vec<Value>,
    task_id: Option<String>,
    context_id: Option<String>,
    #[serde(default)] metadata: HashMap<String, Value>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskParams {
    id: String,
    history_length: Option<usize>,
    #[serde(default)] tenant: String,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListParams {
    context_id: Option<String>, status: Option<String>, page_size: Option<usize>,
    page_token: Option<String>, history_length: Option<usize>, status_timestamp_after: Option<String>,
    #[serde(default)] include_artifacts: bool,
    #[serde(default)] tenant: String,
}

fn response(status: StatusCode, body: JsonRpcResponse) -> Response {
    (status, [("a2a-version", "1.0")], Json(body)).into_response()
}
fn error(id: Option<Value>, code: i32, message: &str) -> Response {
    response(StatusCode::OK, JsonRpcResponse::err(id, code, message))
}
fn task_error(id: Option<Value>, failure: TaskError, cancel: bool) -> Response {
    let (code, message) = match failure {
        TaskError::NotFound => (-32001, "Task not found"),
        TaskError::Conflict if cancel => (-32002, "Task cannot be canceled"),
        TaskError::Conflict => (-32004, "Task does not support this operation in its current state"),
        TaskError::Invalid(message) => (-32602, message),
        TaskError::Host(cause) => {
            tracing::error!(%cause, "A2A v1 host failure");
            (-32603, "Internal error")
        }
    };
    error(id, code, message)
}
fn admission_error(id: Option<Value>, failure: A2aAdmissionError) -> Response {
    match failure {
        A2aAdmissionError::Forbidden => response(
            StatusCode::FORBIDDEN,
            JsonRpcResponse::err(id, -32011, "Insufficient scope"),
        ),
        A2aAdmissionError::Unavailable => response(
            StatusCode::SERVICE_UNAVAILABLE,
            JsonRpcResponse::err(id, -32603, "Task admission unavailable"),
        ),
    }
}
fn parse<T: serde::de::DeserializeOwned>(value: Option<Value>) -> Result<T, ()> {
    serde_json::from_value(value.unwrap_or_else(|| json!({}))).map_err(|_| ())
}
fn tenant_matches(tenant: &str, user: &UserContext) -> bool {
    tenant.is_empty() || user.tenant_id.as_ref().is_some_and(|actual| actual.as_str() == tenant)
}

async fn dispatch(
    state: &A2AState, agent: &str, user: Option<Extension<UserContext>>,
    capture: Option<Extension<AuthenticatedCredentialCapture>>, headers: &HeaderMap,
    parameters: &HashMap<String, String>, body: Bytes,
) -> Response {
    let value: Value = match serde_json::from_slice(&body) {
        Ok(value) => value, Err(_) => return error(None, -32700, "Invalid JSON payload"),
    };
    let req: JsonRpcRequest = match serde_json::from_value(value) {
        Ok(req) => req, Err(_) => return error(None, -32600, "Request payload validation error"),
    };
    if req.jsonrpc != "2.0" || req.id.as_ref().is_none_or(|id| !id.is_string() && !id.is_number()) {
        return error(req.id, -32600, "Request payload validation error");
    }
    let Some(Extension(user)) = user else {
        return response(StatusCode::UNAUTHORIZED, JsonRpcResponse::err(req.id, -32010, "Authentication required"));
    };
    let owner = match ActorOwner::from_verified_context(&user) {
        Ok(owner) if owner.credential_principal().is_some() => owner,
        _ => return response(StatusCode::UNAUTHORIZED, JsonRpcResponse::err(req.id, -32010, "Authentication required")),
    };
    if authorize_context(&state.security, Some(&user), None).is_err() {
        return response(StatusCode::FORBIDDEN, JsonRpcResponse::err(req.id, -32011, "Insufficient scope"));
    }
    // Missing version means 0.3, which belongs to the explicit legacy endpoint.
    let version = headers.get("a2a-version").and_then(|value| value.to_str().ok())
        .or_else(|| parameters.get("A2A-Version").map(String::as_str)).unwrap_or("");
    if version.split('.').take(2).collect::<Vec<_>>() != ["1", "0"] {
        return error(req.id, -32009, "A2A-Version 1.0 is required by this interface");
    }
    match req.method.as_str() {
        "SendMessage" | "SendStreamingMessage" => {
            let Ok(params) = parse::<SendParams>(req.params) else { return error(req.id, -32602, "Invalid parameters"); };
            if !tenant_matches(&params.tenant, &user) {
                return response(StatusCode::FORBIDDEN, JsonRpcResponse::err(req.id, -32011, "Insufficient scope"));
            }
            if params.configuration.task_push_notification_config.is_some() {
                return error(req.id, -32003, "Push notifications are not supported");
            }
            if !params.configuration.accepted_output_modes.is_empty()
                && !params.configuration.accepted_output_modes.iter().any(|mode| matches!(mode.as_str(), "text/plain" | "application/json")) {
                return error(req.id, -32005, "Content type not supported");
            }
            if params.message.role != "ROLE_USER" || params.message.message_id.trim().is_empty() || params.message.parts.is_empty() {
                return error(req.id, -32602, "A user messageId and nonempty parts are required");
            }
            let mut parts = Vec::new();
            for part in &params.message.parts {
                let count = ["text", "raw", "url", "data"].iter().filter(|key| part.get(**key).is_some()).count();
                if count != 1 { return error(req.id, -32602, "Each Part requires exactly one content field"); }
                let Some(text) = part.get("text").and_then(Value::as_str) else {
                    return error(req.id, -32005, "This agent accepts text/plain parts");
                };
                if part.get("mediaType").and_then(Value::as_str).is_some_and(|mime| mime != "text/plain") {
                    return error(req.id, -32005, "This agent accepts text/plain parts");
                }
                parts.push(types::Part::text(text));
            }
            let mut message_metadata = params.message.metadata;
            message_metadata.insert("uar_a2a_message_id".into(), json!(params.message.message_id));
            let params_legacy = types::MessageSendParams {
                message: types::Message { role: types::Role::User, parts, metadata: message_metadata },
                task_id: params.message.task_id, context_id: params.message.context_id, metadata: params.metadata,
            };
            if let Err(failure) = state.admission.admit(&user, agent).await {
                return admission_error(req.id, failure);
            }
            let mut task = match state.threads.send_durable(&owner, &user, user.claims.uar_instance_id.as_deref(), agent,
                params_legacy, &params.message.message_id, capture.map(|Extension(value)| value).unwrap_or_default()).await {
                Ok(task) => task, Err(failure) => return task_error(req.id, failure, false),
            };
            if req.method == "SendStreamingMessage" {
                return task_stream(state, owner, agent.to_owned(), req.id, task, params.configuration.history_length).await;
            }
            if !params.configuration.return_immediately && is_active(&task) {
                task = match state.threads.wait_durable_until_settled(&owner, agent, &task.id, &task).await {
                    Ok(task) => task, Err(failure) => return task_error(req.id, failure, false),
                };
            }
            response(StatusCode::OK, JsonRpcResponse::ok(req.id, json!({"task": project_task(&task, params.configuration.history_length)})))
        }
        "GetTask" | "CancelTask" => {
            let Ok(params) = parse::<TaskParams>(req.params) else { return error(req.id, -32602, "Invalid parameters"); };
            if params.id.trim().is_empty() { return error(req.id, -32602, "Task id is required"); }
            if !tenant_matches(&params.tenant, &user) {
                return response(StatusCode::FORBIDDEN, JsonRpcResponse::err(req.id, -32011, "Insufficient scope"));
            }
            let cancel = req.method == "CancelTask";
            let result = if cancel { state.threads.cancel_durable(&owner, agent, &params.id).await }
                else { state.threads.get_durable(&owner, agent, &params.id).await };
            match result {
                Ok(task) => response(StatusCode::OK, JsonRpcResponse::ok(req.id, project_task(&task, params.history_length))),
                Err(failure) => task_error(req.id, failure, cancel),
            }
        }
        "ListTasks" => {
            let Ok(params) = parse::<ListParams>(req.params) else { return error(req.id, -32602, "Invalid parameters"); };
            if !tenant_matches(&params.tenant, &user) {
                return response(StatusCode::FORBIDDEN, JsonRpcResponse::err(req.id, -32011, "Insufficient scope"));
            }
            let size = params.page_size.unwrap_or(50);
            if !(1..=100).contains(&size) { return error(req.id, -32602, "pageSize must be between 1 and 100"); }
            let after = match params.status_timestamp_after.as_deref().map(chrono::DateTime::parse_from_rfc3339).transpose() {
                Ok(after) => after, Err(_) => return error(req.id, -32602, "Invalid statusTimestampAfter"),
            };
            let mut tasks = match state.threads.list_durable(&owner, agent).await {
                Ok(tasks) => tasks, Err(failure) => return task_error(req.id, failure, false),
            };
            tasks.retain(|task| params.context_id.as_ref().is_none_or(|id| task.context_id.as_ref() == Some(id))
                && params.status.as_ref().is_none_or(|status| task_state(task) == status)
                && after.as_ref().is_none_or(|after| task.status.timestamp.as_deref()
                    .and_then(|timestamp| chrono::DateTime::parse_from_rfc3339(timestamp).ok()).is_some_and(|time| &time > after)));
            tasks.sort_by(|a, b| b.status.timestamp.cmp(&a.status.timestamp).then_with(|| a.id.cmp(&b.id)));
            let total = tasks.len();
            let start = match params.page_token.as_deref().filter(|token| !token.is_empty()) {
                None => 0,
                Some(token) => match tasks.iter().position(|task| task.id == token) {
                    Some(index) => index + 1, None => return error(req.id, -32602, "Invalid pageToken"),
                },
            };
            let page = tasks.iter().skip(start).take(size).collect::<Vec<_>>();
            let next = if start + page.len() < total { page.last().map(|task| task.id.clone()).unwrap_or_default() } else { String::new() };
            let page = page.into_iter().map(|task| {
                let mut value = project_task(task, params.history_length);
                if !params.include_artifacts { if let Some(value) = value.as_object_mut() { value.remove("artifacts"); } }
                value
            }).collect::<Vec<_>>();
            response(StatusCode::OK, JsonRpcResponse::ok(req.id, json!({"tasks": page, "nextPageToken": next, "pageSize": size, "totalSize": total})))
        }
        "GetExtendedAgentCard" => error(req.id, -32007, "Extended agent card is not configured"),
        "CreateTaskPushNotificationConfig" | "GetTaskPushNotificationConfig" | "ListTaskPushNotificationConfigs" | "DeleteTaskPushNotificationConfig" => error(req.id, -32003, "Push notifications are not supported"),
        "SubscribeToTask" => {
            let Ok(params) = parse::<TaskParams>(req.params) else { return error(req.id, -32602, "Invalid parameters"); };
            if params.id.trim().is_empty() { return error(req.id, -32602, "Task id is required"); }
            if !tenant_matches(&params.tenant, &user) {
                return response(StatusCode::FORBIDDEN, JsonRpcResponse::err(req.id, -32011, "Insufficient scope"));
            }
            let task = match state.threads.get_durable(&owner, agent, &params.id).await {
                Ok(task) => task,
                Err(failure) => return task_error(req.id, failure, false),
            };
            if matches!(task_state(&task), "TASK_STATE_COMPLETED" | "TASK_STATE_FAILED" | "TASK_STATE_CANCELED" | "TASK_STATE_REJECTED") {
                return error(req.id, -32004, "Terminal tasks do not support subscription");
            }
            task_stream(state, owner, agent.to_owned(), req.id, task, None).await
        },
        _ => error(req.id, -32601, "Method not found"),
    }
}

async fn task_stream(
    state: &A2AState, owner: ActorOwner, agent: String, request_id: Option<Value>,
    initial: Task, history_length: Option<usize>,
) -> Response {
    let mut updates = match state.threads.subscribe_durable(&owner, &agent, &initial).await {
        Ok(updates) => updates,
        Err(failure) => return task_error(request_id, failure, false),
    };
    let stream = async_stream::stream! {
        let mut first = true;
        while let Some(update) = updates.recv().await {
            let task = match update {
                Ok(task) => task,
                Err(_) => {
                    let body = JsonRpcResponse::err(request_id.clone(), -32603, "Task stream unavailable; reconcile exact task before resubscribing");
                    yield Ok::<_, std::convert::Infallible>(axum::response::sse::Event::default().data(serde_json::to_string(&body).unwrap_or_default()));
                    break;
                }
            };
            let projected = project_task(&task, history_length);
            let payloads = if first {
                first = false;
                vec![json!({"task": projected})]
            } else {
                let mut payloads = projected["artifacts"].as_array().into_iter().flatten().map(|artifact| json!({
                    "artifactUpdate": {"taskId": task.id, "contextId": task.context_id,
                        "artifact": artifact, "append": false, "lastChunk": true, "metadata": task.metadata}
                })).collect::<Vec<_>>();
                payloads.push(json!({"statusUpdate": {"taskId": task.id, "contextId": task.context_id,
                    "status": projected["status"], "metadata": task.metadata}}));
                payloads
            };
            for payload in payloads {
                let body = JsonRpcResponse::ok(request_id.clone(), payload);
                yield Ok::<_, std::convert::Infallible>(axum::response::sse::Event::default().data(serde_json::to_string(&body).unwrap_or_default()));
            }
        }
    };
    ([("a2a-version", "1.0")], axum::response::Sse::new(stream)).into_response()
}

fn is_active(task: &Task) -> bool {
    matches!(task.status.state, types::TaskState::Submitted | types::TaskState::Working)
        && task.metadata.get("execution_unconfirmed") != Some(&Value::Bool(true))
}

/// Dot19 may set this only from a committed host interaction, never client metadata.
pub(crate) fn task_state(task: &Task) -> &str {
    if task.metadata.get("execution_unconfirmed") == Some(&Value::Bool(true)) { return "TASK_STATE_UNSPECIFIED"; }
    if task.metadata.get("uar_auth_required") == Some(&Value::Bool(true)) { return "TASK_STATE_AUTH_REQUIRED"; }
    if task.metadata.get("uar_denied") == Some(&Value::Bool(true))
        && !task.cleanup_unconfirmed()
        && matches!(task.status.state, types::TaskState::Canceled | types::TaskState::Failed | types::TaskState::Completed) {
        return "TASK_STATE_REJECTED";
    }
    match task.status.state {
        types::TaskState::Submitted => "TASK_STATE_SUBMITTED",
        types::TaskState::Working => "TASK_STATE_WORKING",
        types::TaskState::InputRequired => "TASK_STATE_INPUT_REQUIRED",
        types::TaskState::Completed => "TASK_STATE_COMPLETED",
        types::TaskState::Canceled => "TASK_STATE_CANCELED",
        types::TaskState::Failed => "TASK_STATE_FAILED",
    }
}
fn part(part: &types::Part) -> Value {
    match part {
        types::Part::Text { text } => json!({"text": text}),
        types::Part::Data { data } => json!({"data": data}),
        types::Part::File { name, mime_type, uri, bytes } => {
            let mut value = json!({});
            if let Some(bytes) = bytes { value["raw"] = json!(bytes); }
            else if let Some(uri) = uri { value["url"] = json!(uri); }
            if let Some(name) = name { value["filename"] = json!(name); }
            if let Some(mime) = mime_type { value["mediaType"] = json!(mime); }
            value
        }
    }
}
fn message(message: &types::Message, task: &Task, index: &str) -> Value {
    let id = message.metadata.get("uar_a2a_message_id").and_then(Value::as_str)
        .map(str::to_owned).unwrap_or_else(|| format!("{}:{index}", task.id));
    let mut metadata = message.metadata.clone();
    metadata.remove("uar_a2a_message_id");
    json!({"messageId": id, "taskId": task.id, "contextId": task.context_id,
        "role": match message.role { types::Role::User => "ROLE_USER", types::Role::Agent => "ROLE_AGENT" },
        "parts": message.parts.iter().map(part).collect::<Vec<_>>(), "metadata": metadata})
}
pub(crate) fn project_task(task: &Task, history_length: Option<usize>) -> Value {
    let mut status = json!({"state": task_state(task)});
    if let Some(timestamp) = task.status.timestamp.as_deref().and_then(|timestamp| chrono::DateTime::parse_from_rfc3339(timestamp).ok()) {
        status["timestamp"] = json!(timestamp.with_timezone(&chrono::Utc).to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
    }
    if let Some(value) = &task.status.message { status["message"] = message(value, task, "status"); }
    let history = task.history.iter().enumerate().skip(task.history.len().saturating_sub(history_length.unwrap_or(usize::MAX)))
        .map(|(index, value)| message(value, task, &index.to_string())).collect::<Vec<_>>();
    let mut artifacts = task.artifacts.iter().map(|artifact| {
        let mut value = json!({"artifactId": artifact.artifact_id, "parts": artifact.parts.iter().map(part).collect::<Vec<_>>(), "metadata": artifact.metadata});
        if let Some(name) = &artifact.name { value["name"] = json!(name); }
        if let Some(description) = &artifact.description { value["description"] = json!(description); }
        value
    }).collect::<Vec<_>>();
    if task.status.state == types::TaskState::Completed && let Some(output) = &task.status.message {
        artifacts.push(json!({"artifactId": format!("{}:output", task.id), "parts": output.parts.iter().map(part).collect::<Vec<_>>()}));
    }
    json!({"id": task.id, "contextId": task.context_id, "status": status, "history": history,
        "artifacts": artifacts, "metadata": task.metadata})
}
