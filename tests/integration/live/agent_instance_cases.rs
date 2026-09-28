//! C06 completed-phase gate: real UAR process, HTTP administration and SurrealKV.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Duration;

use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};

use super::backend::{BACKEND_ENV_VAR, resolve};
use super::harness::{
    HARNESS_JWT_SECRET, ServiceNeeds, boot_test_server_process_with_instance_id,
    mint_harness_peer_token,
};
use super::stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint};

const MODEL: &str = "gpt-5.4-mini";
const FIRST_WORKSPACE: &str = "workspace:c03";
const SECOND_WORKSPACE: &str = "workspace:c06-second";
const FIRST_BINDING: &str = "urn:uar:c03:binding";
const SECOND_BINDING: &str = "urn:uar:c06:binding-second";
const SERVICE_INSTANCE_ID: &str = "agent-instance:c03";
const SKILL_ROOT: &str = "tests/fixtures/collaboration/builtin-skills";
const AGENT: &str = include_str!("../../fixtures/collaboration/agent-definition.json");
const TEAM: &str = include_str!("../../fixtures/collaboration/team-definition.json");
const WORKFLOW: &str = include_str!("../../fixtures/collaboration/workflow-definition.json");
const MANIFEST: &str = include_str!("../../fixtures/collaboration/package-manifest.json");
const GRANT: &str = include_str!("../../fixtures/collaboration/representation-grant-v1.json");
const BINDING: &str = include_str!("../../fixtures/collaboration/deployment-binding.json");

fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), canonical(value)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

fn finalize(document: &mut Value) {
    document
        .as_object_mut()
        .expect("binding fixture object")
        .remove("contentDigest");
    let bytes = serde_json::to_vec(&canonical(document)).expect("canonical binding JSON");
    let mut digest = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(&mut digest, "{byte:02x}").expect("write digest to String");
    }
    document["contentDigest"] = Value::String(digest);
}

fn binding(workspace: &str, id: &str, model: &str) -> Value {
    let mut binding: Value = serde_json::from_str(BINDING).expect("binding fixture JSON");
    binding["id"] = json!(id);
    binding["workspaceId"] = json!(workspace);
    binding["runtimeInstanceId"] = json!(SERVICE_INSTANCE_ID);
    binding["modelBindings"][0]["modelId"] =
        json!(model.split_once('/').map_or(model, |(_, model)| model));
    if workspace == SECOND_WORKSPACE {
        binding["representationGrantRefs"] = json!([]);
    }
    finalize(&mut binding);
    binding
}

fn copy_cold_datastore(source: &std::path::Path, destination: &std::path::Path) {
    std::fs::create_dir(destination).expect("create C06 backup datastore directory");
    for entry in std::fs::read_dir(source).expect("read closed C06 SurrealKV datastore") {
        let entry = entry.expect("read SurrealKV directory entry");
        let target = destination.join(entry.file_name());
        let kind = entry.file_type().expect("read SurrealKV entry kind");
        if kind.is_dir() {
            copy_cold_datastore(&entry.path(), &target);
        } else if kind.is_file() {
            std::fs::copy(entry.path(), target).expect("copy closed SurrealKV file");
        } else {
            panic!("SurrealKV backup contains an unsupported entry kind");
        }
    }
}

async fn request(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    workspace: &str,
    method: Method,
    path: &str,
    body: Option<Value>,
    expected: StatusCode,
) -> Value {
    let mut builder = client
        .request(method, format!("{base_url}{path}"))
        .bearer_auth(token)
        .header("x-uar-workspace-id", workspace);
    if let Some(body) = body {
        builder = builder.json(&body);
    }
    let response = builder.send().await.expect("C06 real HTTP request");
    let status = response.status();
    let text = response.text().await.expect("C06 HTTP response body");
    assert!(
        !text.contains("api_route_not_found"),
        "C06 route {path} was not mounted: {status} {text}"
    );
    let value = serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text));
    assert_eq!(status, expected, "C06 {path}: {value}");
    value
}

async fn get(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    workspace: &str,
    path: &str,
    expected: StatusCode,
) -> Value {
    request(
        client,
        base_url,
        token,
        workspace,
        Method::GET,
        path,
        None,
        expected,
    )
    .await
}

async fn post(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    workspace: &str,
    path: &str,
    body: Value,
    expected: StatusCode,
) -> Value {
    request(
        client,
        base_url,
        token,
        workspace,
        Method::POST,
        path,
        Some(body),
        expected,
    )
    .await
}

async fn read_sse_until(response: &mut reqwest::Response, expected: &str) -> String {
    tokio::time::timeout(Duration::from_secs(90), async {
        let mut body = String::new();
        while let Some(chunk) = response.chunk().await.expect("C06 instance stream chunk") {
            body.push_str(&String::from_utf8_lossy(&chunk));
            if body.contains(expected) {
                return body;
            }
        }
        panic!("C06 instance stream ended before {expected}: {body}");
    })
    .await
    .unwrap_or_else(|_| panic!("C06 instance stream timed out waiting for {expected}"))
}

async fn wait_for_command(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    workspace: &str,
    instance_id: &str,
    command_id: &str,
) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    loop {
        let view = get(
            client,
            base_url,
            token,
            workspace,
            &format!("/api/uar/agent-instances/v1/{instance_id}"),
            StatusCode::OK,
        )
        .await;
        let command = view["commands"]
            .as_array()
            .and_then(|items| items.iter().find(|item| item["commandId"] == command_id))
            .cloned()
            .expect("accepted command remains in durable retention window");
        match command["status"].as_str() {
            Some("completed") => return command,
            Some("failed" | "cancelled" | "uncertain") => {
                panic!("C06 command terminated without completion: {view}")
            }
            _ => {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "C06 command did not settle within 90s: {view}"
                );
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn wait_for_run(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    run_id: &str,
) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    loop {
        let run = get(
            client,
            base_url,
            token,
            FIRST_WORKSPACE,
            &format!("/api/uar/runs/{run_id}"),
            StatusCode::OK,
        )
        .await;
        if matches!(run["status"].as_str(), Some("done" | "error" | "cancelled")) {
            return run;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "ordinary run did not settle within 90s: {run}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_model_request(base_url: &str, prompt: &str, client: &reqwest::Client) {
    let stub_url = format!("{}/_stub/requests", base_url.trim_end_matches("/v1"));
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let observed: Value = client
            .get(&stub_url)
            .send()
            .await
            .expect("recorded model request log")
            .json()
            .await
            .expect("recorded model request JSON");
        if observed["requests"].as_array().is_some_and(|requests| {
            requests
                .iter()
                .any(|request| request.to_string().contains(prompt))
        }) {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "UAR did not dispatch the occupied turn to the model: {observed}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn model_request_count(base_url: &str, prompt: &str, client: &reqwest::Client) -> usize {
    let stub_url = format!("{}/_stub/requests", base_url.trim_end_matches("/v1"));
    let observed: Value = client
        .get(&stub_url)
        .send()
        .await
        .expect("recorded model request log")
        .json()
        .await
        .expect("recorded model request JSON");
    observed["requests"]
        .as_array()
        .expect("recorded model requests")
        .iter()
        .filter(|request| {
            request["messages"]
                .as_array()
                .and_then(|messages| {
                    messages
                        .iter()
                        .rev()
                        .find(|message| message["role"] == "user")
                })
                .and_then(|message| message["content"].as_str())
                == Some(prompt)
        })
        .count()
}

fn operator_token() -> String {
    let claims = universal_agent_runtime::uar::security::claims::UserClaims {
        sub: "live-harness-peer".to_owned(),
        name: Some("Live Harness Operator".to_owned()),
        roles: Some(vec!["operator".to_owned()]),
        tenant_id: None,
        uar_instance_id: None,
        exp: usize::MAX,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(HARNESS_JWT_SECRET.as_bytes()),
    )
    .expect("mint harness operator JWT")
}

async fn wait_for_tool_approval(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    instance_id: &str,
) -> (String, Value) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let view = get(
            client,
            base_url,
            token,
            FIRST_WORKSPACE,
            &format!("/api/uar/agent-instances/v1/{instance_id}"),
            StatusCode::OK,
        )
        .await;
        if let Some(run_id) = view["activeRunId"].as_str() {
            let pending = get(
                client,
                base_url,
                token,
                FIRST_WORKSPACE,
                &format!("/api/uar/runs/{run_id}/tool-approval/pending"),
                StatusCode::OK,
            )
            .await;
            if pending["pending"]["approvalId"].as_str().is_some() {
                return (run_id.to_owned(), pending);
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "C06 file tool did not reach the real approval waiter: {view}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_cancelled_command(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    workspace: &str,
    instance_id: &str,
    command_id: &str,
) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let view = get(
            client,
            base_url,
            token,
            workspace,
            &format!("/api/uar/agent-instances/v1/{instance_id}"),
            StatusCode::OK,
        )
        .await;
        let command = view["commands"]
            .as_array()
            .and_then(|items| items.iter().find(|item| item["commandId"] == command_id))
            .expect("occupied command remains retained");
        if command["status"] == "cancelled" {
            return view;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "occupied command did not cancel: {view}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// One phase-boundary scenario, not a per-edit or mock-only regression loop.
#[tokio::test]
#[serial]
async fn c06_durable_instances_complete_live_boundary() {
    assert!(
        include_str!("../../../versions.toml").contains("surrealdb = \"3.3.0\""),
        "C06 gate requires the pinned SurrealDB 3.3.0 SDK"
    );
    let original_skill_root = std::env::var_os("UAR_BUILTIN_SKILLS_DIR");
    // SAFETY: this serial integration case restores its process-global fixture root.
    unsafe { std::env::set_var("UAR_BUILTIN_SKILLS_DIR", SKILL_ROOT) };
    let scratch = tempfile::tempdir().expect("C06 SurrealKV scratch");
    let db_path = scratch.path().join("surrealkv");
    let file_root = scratch.path().join("file-workspace");
    std::fs::create_dir(&file_root).expect("create C06 disposable tool workspace");
    let blocked_tool_file = file_root.join("blocked-write.txt");
    let prompts = [
        ("C06_FIRST_TURN", "C06_FIRST_RESULT"),
        ("C06_SECOND_TURN", "C06_SECOND_RESULT"),
        ("C06_STREAM_TURN", "C06_STREAM_RESULT"),
        ("C06_ORDINARY_RUN", "C06_ORDINARY_RESULT"),
        ("C06_AFTER_RECONCILE", "C06_RECOVERED_RESULT"),
    ];
    let mut fixtures = FixtureSet::new();
    for (prompt, response) in prompts {
        fixtures = fixtures.with(
            RequestFingerprint {
                model: MODEL.to_owned(),
                last_user_message: prompt.to_owned(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::Content(response.to_owned()),
        );
    }
    for prompt in ["C06_BLOCKED_MODEL", "C06_CANCEL_MODEL", "C06_CRASH_MODEL"] {
        fixtures = fixtures.with(
            RequestFingerprint {
                model: MODEL.to_owned(),
                last_user_message: prompt.to_owned(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::DelayedContent {
                delay_ms: 30_000,
                text: "C06_LATE_RESULT".to_owned(),
            },
        );
    }
    fixtures = fixtures.with(
        RequestFingerprint {
            model: MODEL.to_owned(),
            last_user_message: "C06_BLOCKED_TOOL".to_owned(),
            has_tools: true,
            has_tool_result: false,
        },
        FixtureResponse::ToolCall {
            name: "file_write".to_owned(),
            arguments: json!({
                "path":blocked_tool_file.to_string_lossy(),
                "content":"C06_PROTECTED_EFFECT_MUST_NOT_APPEAR"
            })
            .to_string(),
        },
    );
    let recorded = std::env::var(BACKEND_ENV_VAR).as_deref() != Ok("live");
    let backend = resolve(fixtures).await;
    let reserved = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("reserve C06 HTTP port for both process boots");
    let fixed_http_port = reserved.local_addr().expect("reserved C06 address").port();
    drop(reserved);
    let first = boot_test_server_process_with_instance_id(
        &backend.base_url,
        &backend.model,
        ServiceNeeds::default(),
        &db_path,
        Some(SERVICE_INSTANCE_ID),
        Some(fixed_http_port),
        Some(&file_root),
    )
    .await;
    let token = mint_harness_peer_token();
    let client = reqwest::Client::new();

    let capabilities = get(
        &client,
        &first.base_url,
        &token,
        FIRST_WORKSPACE,
        "/api/uar/capabilities",
        StatusCode::OK,
    )
    .await;
    assert!(
        capabilities["capabilities"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| item == "durable_agent_instances_v1")),
        "SurrealKV-backed runtime did not advertise its durable profile: {capabilities}"
    );
    post(
        &client,
        &first.base_url,
        &token,
        FIRST_WORKSPACE,
        "/api/v1/collaboration/packages:install",
        json!({"commandId":"c06-package","manifest":MANIFEST,"files":{
            "agent-definition.json":AGENT,
            "team-definition.json":TEAM,
            "workflow-definition.json":WORKFLOW,
        }}),
        StatusCode::CREATED,
    )
    .await;
    post(
        &client,
        &first.base_url,
        &token,
        FIRST_WORKSPACE,
        "/api/v1/collaboration/representation-grants",
        json!({"commandId":"c06-grant","expectedRevision":0,
            "grant":serde_json::from_str::<Value>(GRANT).expect("grant fixture")}),
        StatusCode::CREATED,
    )
    .await;
    for (workspace, id, command_id) in [
        (FIRST_WORKSPACE, FIRST_BINDING, "c06-binding-a"),
        (SECOND_WORKSPACE, SECOND_BINDING, "c06-binding-b"),
    ] {
        let installed = post(
            &client,
            &first.base_url,
            &token,
            workspace,
            "/api/v1/collaboration/deployment-bindings",
            json!({"commandId":command_id,"expectedRevision":0,
                "binding":binding(workspace,id,&backend.model)}),
            StatusCode::CREATED,
        )
        .await;
        assert_eq!(
            installed["preflight"]["activationSupported"], true,
            "C06 binding {id} in {workspace} failed preflight: {}",
            installed["preflight"]
        );
    }
    let limits = json!({
        "max_inbox": 2,
        "retained_commands": 16,
        "retained_events": 2,
        "max_restart_attempts": 3,
        "idle_timeout_secs": 300,
    });
    let first_instance = post(
        &client,
        &first.base_url,
        &token,
        FIRST_WORKSPACE,
        "/api/uar/agent-instances/v1",
        json!({"deploymentBindingId":FIRST_BINDING,"profile":"on_demand","limits":limits}),
        StatusCode::CREATED,
    )
    .await;
    let first_id = first_instance["instanceId"]
        .as_str()
        .expect("first logical instance ID")
        .to_owned();
    let second_instance = post(
        &client,
        &first.base_url,
        &token,
        SECOND_WORKSPACE,
        "/api/uar/agent-instances/v1",
        json!({"deploymentBindingId":SECOND_BINDING,"profile":"on_demand"}),
        StatusCode::CREATED,
    )
    .await;
    let second_id = second_instance["instanceId"]
        .as_str()
        .expect("second logical instance ID")
        .to_owned();
    assert_ne!(first_id, second_id);
    get(
        &client,
        &first.base_url,
        &token,
        SECOND_WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{first_id}"),
        StatusCode::NOT_FOUND,
    )
    .await;
    let isolated = get(
        &client,
        &first.base_url,
        &token,
        SECOND_WORKSPACE,
        "/api/uar/agent-instances/v1",
        StatusCode::OK,
    )
    .await;
    assert_eq!(isolated.as_array().map(Vec::len), Some(1));
    assert_eq!(isolated[0]["instanceId"], second_id);

    let stream_path = format!("/api/uar/agent-instances/v1/{second_id}/events/stream");
    let mut snapshot_stream = client
        .get(format!("{}{stream_path}", first.base_url))
        .bearer_auth(&token)
        .header("x-uar-workspace-id", SECOND_WORKSPACE)
        .send()
        .await
        .expect("C06 instance snapshot stream");
    assert_eq!(snapshot_stream.status(), StatusCode::OK);
    let snapshot_frame = read_sse_until(&mut snapshot_stream, "event: instance.snapshot").await;
    assert!(snapshot_frame.contains(&second_id));
    assert!(
        !snapshot_frame.contains("id: 0\n"),
        "empty snapshot invented event cursor"
    );
    drop(snapshot_stream);
    post(
        &client,
        &first.base_url,
        &token,
        SECOND_WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{second_id}/activate"),
        json!({"commandId":"c06-stream-activation"}),
        StatusCode::OK,
    )
    .await;
    let mut replay_stream = client
        .get(format!("{}{stream_path}", first.base_url))
        .bearer_auth(&token)
        .header("x-uar-workspace-id", SECOND_WORKSPACE)
        .header("Last-Event-ID", "0")
        .send()
        .await
        .expect("C06 instance replay stream");
    assert_eq!(replay_stream.status(), StatusCode::OK);
    let replay_frame = read_sse_until(&mut replay_stream, "\"delivery\":\"replay\"").await;
    assert!(replay_frame.contains("event: instance.event"));
    post(
        &client,
        &first.base_url,
        &token,
        SECOND_WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{second_id}/turns"),
        json!({"commandId":"c06-stream-turn","prompt":"C06_STREAM_TURN"}),
        StatusCode::ACCEPTED,
    )
    .await;
    let live_frame = read_sse_until(&mut replay_stream, "\"delivery\":\"live\"").await;
    assert!(live_frame.contains("event: instance.event"));
    wait_for_command(
        &client,
        &first.base_url,
        &token,
        SECOND_WORKSPACE,
        &second_id,
        "c06-stream-turn",
    )
    .await;
    drop(replay_stream);

    let turn_path = format!("/api/uar/agent-instances/v1/{first_id}/turns");
    let first_turn = post(
        &client,
        &first.base_url,
        &token,
        FIRST_WORKSPACE,
        &turn_path,
        json!({"commandId":"c06-turn-1","prompt":"C06_FIRST_TURN"}),
        StatusCode::ACCEPTED,
    )
    .await;
    let duplicate = post(
        &client,
        &first.base_url,
        &token,
        FIRST_WORKSPACE,
        &turn_path,
        json!({"commandId":"c06-turn-1","prompt":"C06_FIRST_TURN"}),
        StatusCode::ACCEPTED,
    )
    .await;
    assert_eq!(duplicate["commandId"], first_turn["commandId"]);
    post(
        &client,
        &first.base_url,
        &token,
        FIRST_WORKSPACE,
        &turn_path,
        json!({"commandId":"c06-turn-1","prompt":"changed input"}),
        StatusCode::CONFLICT,
    )
    .await;
    let completed_first = wait_for_command(
        &client,
        &first.base_url,
        &token,
        FIRST_WORKSPACE,
        &first_id,
        "c06-turn-1",
    )
    .await;
    let root_one = completed_first["rootRunId"]
        .as_str()
        .expect("first fresh root ID")
        .to_owned();

    let barrier = first.shutdown_to_pre_exit_barrier("TERM").await;
    let restarted = boot_test_server_process_with_instance_id(
        &backend.base_url,
        &backend.model,
        ServiceNeeds::default(),
        &db_path,
        Some(SERVICE_INSTANCE_ID),
        Some(fixed_http_port),
        Some(&file_root),
    )
    .await;
    barrier.allow_exit().await;
    let restored = get(
        &client,
        &restarted.base_url,
        &token,
        FIRST_WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{first_id}"),
        StatusCode::OK,
    )
    .await;
    assert_eq!(restored["instanceId"], first_id);
    assert_eq!(restored["workspaceId"], FIRST_WORKSPACE);
    assert_eq!(restored["commands"][0]["rootRunId"], root_one);
    assert_eq!(
        restored["recovery"], "ready",
        "cold recovery blocked: {restored}"
    );
    assert_eq!(
        restored["lifecycle"], "dormant",
        "stale activation survived: {restored}"
    );
    assert_eq!(
        restored["commands"][0]["status"], "completed",
        "first receipt changed: {restored}"
    );
    assert!(
        restored["activeAttemptId"].is_null(),
        "stale attempt survived: {restored}"
    );
    assert_eq!(
        restored["queueDepth"], 0,
        "cold inbox remained occupied: {restored}"
    );

    post(
        &client,
        &restarted.base_url,
        &token,
        FIRST_WORKSPACE,
        &turn_path,
        json!({"commandId":"c06-turn-2","prompt":"C06_SECOND_TURN"}),
        StatusCode::ACCEPTED,
    )
    .await;
    let completed_second = wait_for_command(
        &client,
        &restarted.base_url,
        &token,
        FIRST_WORKSPACE,
        &first_id,
        "c06-turn-2",
    )
    .await;
    let root_two = completed_second["rootRunId"]
        .as_str()
        .expect("second fresh root ID");
    assert_ne!(root_one, root_two);

    let instance_path = format!("/api/uar/agent-instances/v1/{first_id}");
    let lifecycle = [
        ("passivate", "dormant"),
        ("activate", "active"),
        ("restart", "active"),
        ("drain", "dormant"),
        ("disable", "disabled"),
    ];
    let before_lifecycle = get(
        &client,
        &restarted.base_url,
        &token,
        FIRST_WORKSPACE,
        &instance_path,
        StatusCode::OK,
    )
    .await;
    let mut last_revision = before_lifecycle["revision"]
        .as_u64()
        .expect("instance revision before lifecycle");
    for (operation, lifecycle_state) in lifecycle {
        let command_id = format!("c06-lifecycle-{operation}");
        post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{instance_path}/{operation}"),
            json!({"commandId":command_id}),
            StatusCode::OK,
        )
        .await;
        wait_for_command(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &first_id,
            &command_id,
        )
        .await;
        let state = get(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &instance_path,
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            state["lifecycle"], lifecycle_state,
            "{operation} did not settle: {state}"
        );
        let revision = state["revision"]
            .as_u64()
            .expect("revisioned lifecycle receipt");
        assert!(revision > last_revision, "{operation} was not revisioned");
        assert!(state["commands"].as_array().is_some_and(|items| {
            items
                .iter()
                .any(|item| item["commandId"] == command_id && item["status"] == "completed")
        }));
        last_revision = revision;
    }

    let events = get(
        &client,
        &restarted.base_url,
        &token,
        FIRST_WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{first_id}/events?after=0"),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        events["gap"], true,
        "retained cursor gap was hidden: {events}"
    );
    assert_eq!(events["snapshot"]["instanceId"], first_id);
    let mut gap_stream = client
        .get(format!(
            "{}/api/uar/agent-instances/v1/{first_id}/events/stream?after=0",
            restarted.base_url
        ))
        .bearer_auth(&token)
        .header("x-uar-workspace-id", FIRST_WORKSPACE)
        .send()
        .await
        .expect("C06 instance gap stream");
    assert_eq!(gap_stream.status(), StatusCode::OK);
    let gap_frame = read_sse_until(&mut gap_stream, "event: instance.gap").await;
    assert!(gap_frame.contains("\"gap\":true"));
    drop(gap_stream);

    let ordinary = post(
        &client,
        &restarted.base_url,
        &token,
        FIRST_WORKSPACE,
        "/api/uar/runs",
        json!({"deployment_binding_id":FIRST_BINDING,"input":"C06_ORDINARY_RUN"}),
        StatusCode::OK,
    )
    .await;
    let ordinary_run_id = ordinary["run_id"].as_str().expect("ordinary run ID");
    let ordinary_result = wait_for_run(&client, &restarted.base_url, &token, ordinary_run_id).await;
    assert_eq!(
        ordinary_result["status"], "done",
        "ordinary run regressed: {ordinary_result}"
    );
    if recorded {
        let occupied = post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            "/api/uar/agent-instances/v1",
            json!({"deploymentBindingId":FIRST_BINDING,"profile":"on_demand",
                "limits":{"max_inbox":1,"retained_commands":4,"retained_events":8,
                    "max_restart_attempts":3,"idle_timeout_secs":300}}),
            StatusCode::CREATED,
        )
        .await;
        let occupied_id = occupied["instanceId"]
            .as_str()
            .expect("occupied instance ID");
        let occupied_path = format!("/api/uar/agent-instances/v1/{occupied_id}");
        post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{occupied_path}/turns"),
            json!({"commandId":"c06-blocked-turn","prompt":"C06_BLOCKED_MODEL"}),
            StatusCode::ACCEPTED,
        )
        .await;
        wait_for_model_request(&backend.base_url, "C06_BLOCKED_MODEL", &client).await;
        let while_blocked = get(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &occupied_path,
            StatusCode::OK,
        )
        .await;
        assert_eq!(while_blocked["queueDepth"], 1);
        assert!(while_blocked["activeRunId"].as_str().is_some());
        post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{occupied_path}/turns"),
            json!({"commandId":"c06-over-capacity","prompt":"must not dispatch"}),
            StatusCode::TOO_MANY_REQUESTS,
        )
        .await;
        let epoch_before_restart = while_blocked["epoch"].as_u64().expect("active epoch");
        let restarting = post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{occupied_path}/restart"),
            json!({"commandId":"c06-restart-occupied"}),
            StatusCode::OK,
        )
        .await;
        if restarting["commands"].as_array().is_some_and(|items| {
            items.iter().any(|item| {
                item["commandId"] == "c06-restart-occupied" && item["status"] == "accepted"
            })
        }) {
            assert_eq!(
                restarting["epoch"].as_u64(),
                Some(epoch_before_restart),
                "restart advanced epoch before the occupied turn settled"
            );
        }
        wait_for_cancelled_command(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            occupied_id,
            "c06-blocked-turn",
        )
        .await;
        wait_for_command(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            occupied_id,
            "c06-restart-occupied",
        )
        .await;
        let after_restart = get(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &occupied_path,
            StatusCode::OK,
        )
        .await;
        assert!(
            after_restart["epoch"]
                .as_u64()
                .is_some_and(|epoch| epoch > epoch_before_restart),
            "restart did not advance epoch after the prior turn settled: {after_restart}"
        );
        assert_eq!(after_restart["lifecycle"], "active");
        post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{occupied_path}/turns"),
            json!({"commandId":"c06-cancel-turn","prompt":"C06_CANCEL_MODEL"}),
            StatusCode::ACCEPTED,
        )
        .await;
        wait_for_model_request(&backend.base_url, "C06_CANCEL_MODEL", &client).await;
        let cancellation = post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{occupied_path}/cancel"),
            json!({"commandId":"c06-cancel-active"}),
            StatusCode::OK,
        )
        .await;
        assert!(cancellation["commands"].as_array().is_some_and(|items| {
            items
                .iter()
                .any(|item| item["commandId"] == "c06-cancel-active")
        }));
        let cancelled = wait_for_cancelled_command(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            occupied_id,
            "c06-cancel-turn",
        )
        .await;
        assert_eq!(cancelled["queueDepth"], 0);

        let tool_instance = post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            "/api/uar/agent-instances/v1",
            json!({"deploymentBindingId":FIRST_BINDING,"profile":"on_demand",
                "limits":{"max_inbox":1,"retained_commands":4,"retained_events":8,
                    "max_restart_attempts":3,"idle_timeout_secs":300}}),
            StatusCode::CREATED,
        )
        .await;
        let tool_instance_id = tool_instance["instanceId"]
            .as_str()
            .expect("tool instance ID");
        let tool_instance_path = format!("/api/uar/agent-instances/v1/{tool_instance_id}");
        post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{tool_instance_path}/turns"),
            json!({"commandId":"c06-blocked-tool-turn","prompt":"C06_BLOCKED_TOOL"}),
            StatusCode::ACCEPTED,
        )
        .await;
        let (tool_run_id, pending) =
            wait_for_tool_approval(&client, &restarted.base_url, &token, tool_instance_id).await;
        assert_eq!(
            pending["pending"]["name"], "file_write",
            "wrong approval: {pending}"
        );
        let approval_id = pending["pending"]["approvalId"]
            .as_str()
            .expect("stable protected-tool approval ID");
        let evidence_path = format!("/api/uar/runs/{tool_run_id}/tool-admission-evidence");
        let prepared_evidence = get(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &evidence_path,
            StatusCode::OK,
        )
        .await;
        assert!(
            prepared_evidence["records"]
                .as_array()
                .is_some_and(|records| {
                    records.iter().any(|record| {
                        record["tool_name"] == "file_write"
                            && record["state"] == "awaiting_approval"
                    })
                }),
            "protected tool never reached durable approval evidence: {prepared_evidence}"
        );
        assert!(!blocked_tool_file.exists(), "tool wrote before approval");
        let approval_view = get(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &tool_instance_path,
            StatusCode::OK,
        )
        .await;
        let epoch_at_approval = approval_view["epoch"]
            .as_u64()
            .expect("epoch at protected-tool approval");
        post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{tool_instance_path}/restart"),
            json!({"commandId":"c06-restart-blocked-tool"}),
            StatusCode::OK,
        )
        .await;
        wait_for_cancelled_command(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            tool_instance_id,
            "c06-blocked-tool-turn",
        )
        .await;
        wait_for_command(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            tool_instance_id,
            "c06-restart-blocked-tool",
        )
        .await;
        let tool_after_restart = get(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &tool_instance_path,
            StatusCode::OK,
        )
        .await;
        assert!(
            tool_after_restart["epoch"]
                .as_u64()
                .is_some_and(|epoch| epoch > epoch_at_approval),
            "blocked tool restart did not replace activation: {tool_after_restart}"
        );
        let pending_after_restart = get(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("/api/uar/runs/{tool_run_id}/tool-approval/pending"),
            StatusCode::OK,
        )
        .await;
        assert!(
            pending_after_restart["pending"].is_null(),
            "old epoch retained a live approval waiter: {pending_after_restart}"
        );
        post(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("/api/uar/runs/{tool_run_id}/tool-approval"),
            json!({"approved":true,"approval_id":approval_id}),
            StatusCode::NOT_FOUND,
        )
        .await;
        let terminal_evidence = get(
            &client,
            &restarted.base_url,
            &token,
            FIRST_WORKSPACE,
            &evidence_path,
            StatusCode::OK,
        )
        .await;
        assert!(
            !terminal_evidence["records"]
                .as_array()
                .is_some_and(|records| {
                    records.iter().any(|record| {
                        record["tool_name"] == "file_write"
                            && matches!(
                                record["state"].as_str(),
                                Some("claim_intent" | "succeeded")
                            )
                    })
                }),
            "cancelled approval reached a protected effect claim: {terminal_evidence}"
        );
        assert!(
            !blocked_tool_file.exists(),
            "stale approval executed a protected file write"
        );
    }
    let final_view = get(
        &client,
        &restarted.base_url,
        &token,
        FIRST_WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{first_id}"),
        StatusCode::OK,
    )
    .await;
    let projected = final_view.to_string();
    assert!(!projected.contains("C06_FIRST_TURN"));
    assert!(!projected.contains("protected-credential://c03/model"));
    assert!(
        final_view["commands"]
            .as_array()
            .is_some_and(|items| items.len() >= 7)
    );

    restarted.shutdown().await;
    if recorded {
        // A process crash must latch a running attempt as uncertain. The
        // operator can attest to its effect; the old turn is never replayed.
        let crash_host = boot_test_server_process_with_instance_id(
            &backend.base_url,
            &backend.model,
            ServiceNeeds::default(),
            &db_path,
            Some(SERVICE_INSTANCE_ID),
            Some(fixed_http_port),
            Some(&file_root),
        )
        .await;
        let crash_instance = post(
            &client,
            &crash_host.base_url,
            &token,
            FIRST_WORKSPACE,
            "/api/uar/agent-instances/v1",
            json!({"deploymentBindingId":FIRST_BINDING,"profile":"on_demand",
                "limits":{"max_inbox":2,"retained_commands":8,"retained_events":16,
                    "max_restart_attempts":3,"idle_timeout_secs":300}}),
            StatusCode::CREATED,
        )
        .await;
        let crash_id = crash_instance["instanceId"]
            .as_str()
            .expect("crash-recovery instance ID");
        let crash_path = format!("/api/uar/agent-instances/v1/{crash_id}");
        post(
            &client,
            &crash_host.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{crash_path}/turns"),
            json!({"commandId":"c06-crash-turn","prompt":"C06_CRASH_MODEL"}),
            StatusCode::ACCEPTED,
        )
        .await;
        wait_for_model_request(&backend.base_url, "C06_CRASH_MODEL", &client).await;
        let active = get(
            &client,
            &crash_host.base_url,
            &token,
            FIRST_WORKSPACE,
            &crash_path,
            StatusCode::OK,
        )
        .await;
        assert_eq!(active["activeCommandId"], "c06-crash-turn");
        let attempt_id = active["activeAttemptId"]
            .as_str()
            .expect("persisted active attempt before crash")
            .to_owned();
        assert!(
            active["commands"].as_array().is_some_and(|commands| {
                commands.iter().any(|command| {
                    command["commandId"] == "c06-crash-turn" && command["status"] == "running"
                })
            }),
            "crashed attempt was not durably running: {active}"
        );
        let initial_model_requests =
            model_request_count(&backend.base_url, "C06_CRASH_MODEL", &client).await;
        assert_eq!(
            initial_model_requests, 1,
            "old turn dispatched more than once"
        );
        crash_host.crash().await;

        let recovery_host = boot_test_server_process_with_instance_id(
            &backend.base_url,
            &backend.model,
            ServiceNeeds::default(),
            &db_path,
            Some(SERVICE_INSTANCE_ID),
            Some(fixed_http_port),
            Some(&file_root),
        )
        .await;
        let uncertain = get(
            &client,
            &recovery_host.base_url,
            &token,
            FIRST_WORKSPACE,
            &crash_path,
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            uncertain["recovery"], "effect_uncertain",
            "crash lost effect posture: {uncertain}"
        );
        assert_eq!(
            uncertain["lifecycle"], "failed",
            "crash resumed stale activation: {uncertain}"
        );
        assert!(
            uncertain["activeAttemptId"].is_null(),
            "old attempt remains active: {uncertain}"
        );
        assert!(
            uncertain["commands"].as_array().is_some_and(|commands| {
                commands.iter().any(|command| {
                    command["commandId"] == "c06-crash-turn"
                        && command["attemptId"] == attempt_id
                        && command["status"] == "uncertain"
                })
            }),
            "exact command/attempt did not latch uncertainty: {uncertain}"
        );
        assert_eq!(
            model_request_count(&backend.base_url, "C06_CRASH_MODEL", &client).await,
            initial_model_requests,
            "cold recovery replayed the old model turn"
        );
        post(
            &client,
            &recovery_host.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{crash_path}/turns"),
            json!({"commandId":"c06-premature-turn","prompt":"must not dispatch"}),
            StatusCode::CONFLICT,
        )
        .await;
        let reconcile_path = format!("{crash_path}/reconcile");
        let exact_receipt = json!({
            "targetCommandId":"c06-crash-turn",
            "targetAttemptId":attempt_id,
            "effectDisposition":"confirmed_not_applied",
            "receiptId":"c06-operator-attestation-1",
        });
        post(
            &client,
            &recovery_host.base_url,
            &token,
            FIRST_WORKSPACE,
            &reconcile_path,
            exact_receipt.clone(),
            StatusCode::FORBIDDEN,
        )
        .await;
        let operator = operator_token();
        post(
            &client,
            &recovery_host.base_url,
            &operator,
            FIRST_WORKSPACE,
            &reconcile_path,
            json!({"targetCommandId":"c06-crash-turn","targetAttemptId":"wrong-attempt",
                "effectDisposition":"confirmed_not_applied","receiptId":"wrong-attestation"}),
            StatusCode::CONFLICT,
        )
        .await;
        let reconciled = post(
            &client,
            &recovery_host.base_url,
            &operator,
            FIRST_WORKSPACE,
            &reconcile_path,
            exact_receipt.clone(),
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            reconciled["recovery"], "ready",
            "attestation did not release instance: {reconciled}"
        );
        assert_eq!(reconciled["lifecycle"], "dormant");
        assert_eq!(
            reconciled["reconciliationReceipt"],
            "c06-operator-attestation-1"
        );
        assert!(
            reconciled["commands"].as_array().is_some_and(|commands| {
                commands.iter().any(|command| {
                    command["commandId"] == "c06-crash-turn" && command["status"] == "failed"
                })
            }),
            "old command was not closed without replay: {reconciled}"
        );
        let replayed_receipt = post(
            &client,
            &recovery_host.base_url,
            &operator,
            FIRST_WORKSPACE,
            &reconcile_path,
            exact_receipt,
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            replayed_receipt["revision"], reconciled["revision"],
            "same attestation changed state on retry"
        );
        post(
            &client,
            &recovery_host.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{crash_path}/turns"),
            json!({"commandId":"c06-after-reconcile","prompt":"C06_AFTER_RECONCILE"}),
            StatusCode::ACCEPTED,
        )
        .await;
        wait_for_command(
            &client,
            &recovery_host.base_url,
            &token,
            FIRST_WORKSPACE,
            crash_id,
            "c06-after-reconcile",
        )
        .await;
        assert_eq!(
            model_request_count(&backend.base_url, "C06_CRASH_MODEL", &client).await,
            initial_model_requests,
            "new activation replayed the uncertain old command"
        );

        // An installed binding can advance independently of an instance's
        // immutable activation snapshot. Each failed activation must consume
        // one retry and leave a durable, truthful command receipt.
        let retry_instance = post(
            &client,
            &recovery_host.base_url,
            &token,
            FIRST_WORKSPACE,
            "/api/uar/agent-instances/v1",
            json!({"deploymentBindingId":FIRST_BINDING,"profile":"request",
                "limits":{"max_inbox":2,"retained_commands":8,"retained_events":16,
                    "max_restart_attempts":2,"idle_timeout_secs":300}}),
            StatusCode::CREATED,
        )
        .await;
        let retry_id = retry_instance["instanceId"]
            .as_str()
            .expect("activation-retry instance ID");
        let retry_path = format!("/api/uar/agent-instances/v1/{retry_id}");
        let mut newer_binding = binding(FIRST_WORKSPACE, FIRST_BINDING, &backend.model);
        newer_binding["revision"] = json!(2);
        newer_binding["effectiveBudget"]["maxTokens"] = json!(4000);
        finalize(&mut newer_binding);
        post(
            &client,
            &recovery_host.base_url,
            &token,
            FIRST_WORKSPACE,
            "/api/v1/collaboration/deployment-bindings",
            json!({"commandId":"c06-binding-invalidates-pinned-instance",
                "expectedRevision":1,"binding":newer_binding}),
            StatusCode::CREATED,
        )
        .await;
        for attempt in 1..=2 {
            post(
                &client,
                &recovery_host.base_url,
                &token,
                FIRST_WORKSPACE,
                &format!("{retry_path}/activate"),
                json!({"commandId":format!("c06-failed-activation-{attempt}")}),
                StatusCode::CONFLICT,
            )
            .await;
            let failed_attempt = get(
                &client,
                &recovery_host.base_url,
                &token,
                FIRST_WORKSPACE,
                &retry_path,
                StatusCode::OK,
            )
            .await;
            assert_eq!(
                failed_attempt["restartAttempts"].as_u64(),
                Some(attempt),
                "activation failure was not counted: {failed_attempt}"
            );
            assert_eq!(failed_attempt["lastErrorCode"], "activation_failed");
            assert_eq!(
                failed_attempt["lifecycle"],
                if attempt == 2 { "failed" } else { "dormant" }
            );
            let command_id = format!("c06-failed-activation-{attempt}");
            assert!(
                failed_attempt["commands"]
                    .as_array()
                    .is_some_and(|commands| {
                        commands.iter().any(|command| {
                            command["commandId"] == command_id && command["status"] == "failed"
                        })
                    }),
                "failed activation has no durable failed receipt: {failed_attempt}"
            );
        }
        recovery_host.shutdown().await;
        let exhausted_host = boot_test_server_process_with_instance_id(
            &backend.base_url,
            &backend.model,
            ServiceNeeds::default(),
            &db_path,
            Some(SERVICE_INSTANCE_ID),
            Some(fixed_http_port),
            Some(&file_root),
        )
        .await;
        let exhausted = get(
            &client,
            &exhausted_host.base_url,
            &token,
            FIRST_WORKSPACE,
            &retry_path,
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            exhausted["restartAttempts"], 2,
            "cold restart lost the activation retry count: {exhausted}"
        );
        assert_eq!(exhausted["lifecycle"], "failed");
        assert_eq!(exhausted["lastErrorCode"], "activation_failed");
        post(
            &client,
            &exhausted_host.base_url,
            &token,
            FIRST_WORKSPACE,
            &format!("{retry_path}/activate"),
            json!({"commandId":"c06-activation-over-budget"}),
            StatusCode::CONFLICT,
        )
        .await;
        let still_exhausted = get(
            &client,
            &exhausted_host.base_url,
            &token,
            FIRST_WORKSPACE,
            &retry_path,
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            still_exhausted["revision"], exhausted["revision"],
            "exhausted retry changed durable instance state"
        );
        exhausted_host.shutdown().await;
    }
    let backup_path = scratch.path().join("surrealkv-cold-backup");
    copy_cold_datastore(&db_path, &backup_path);
    let backup_host = boot_test_server_process_with_instance_id(
        &backend.base_url,
        &backend.model,
        ServiceNeeds::default(),
        &backup_path,
        Some(SERVICE_INSTANCE_ID),
        Some(fixed_http_port),
        Some(&file_root),
    )
    .await;
    let backed_up = get(
        &client,
        &backup_host.base_url,
        &token,
        FIRST_WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{first_id}"),
        StatusCode::OK,
    )
    .await;
    assert_eq!(backed_up["instanceId"], first_id);
    assert!(
        backed_up["commands"].as_array().is_some_and(|commands| {
            commands.iter().any(|command| {
                command["commandId"] == "c06-turn-1"
                    && command["status"] == "completed"
                    && command["rootRunId"] == root_one
            })
        }),
        "cold datastore copy lost a completed C06 command receipt: {backed_up}"
    );
    get(
        &client,
        &backup_host.base_url,
        &token,
        FIRST_WORKSPACE,
        &format!("/api/uar/agent-instances/v1/{ordinary_run_id}"),
        StatusCode::NOT_FOUND,
    )
    .await;
    // Ordinary run inspection is process-local; its old run ID is not a
    // durable logical instance or command receipt in the copied datastore.
    get(
        &client,
        &backup_host.base_url,
        &token,
        FIRST_WORKSPACE,
        &format!("/api/uar/runs/{ordinary_run_id}"),
        StatusCode::NOT_FOUND,
    )
    .await;
    backup_host.shutdown().await;
    // SAFETY: paired restoration after the serial integration case above.
    unsafe {
        match original_skill_root {
            Some(root) => std::env::set_var("UAR_BUILTIN_SKILLS_DIR", root),
            None => std::env::remove_var("UAR_BUILTIN_SKILLS_DIR"),
        }
    }
    let receipt = json!({
        "provider":"SurrealKV",
        "surrealdbSdk":"3.3.0",
        "backendMode":if recorded { "recorded" } else { "live" },
        "workspaceIsolation":true,
        "duplicateReceipt":true,
        "changedRequestConflict":true,
        "coldRestart":true,
        "freshRootIds":[root_one,root_two],
        "replayGapAndSnapshot":true,
        "revisionedLifecycle":true,
        "ordinaryRunCompatibility":true,
        "capabilityAdvertised":true,
        "privateInputAbsentFromProjection":true,
        "occupiedModelTurnCapacityAndCancellation":recorded,
        "occupiedTurnTwoStageRestart":recorded,
        "blockedToolApprovalCancellation":recorded,
        "staleApprovalAfterEpochHandoff":recorded,
        "activeAttemptCrashAndOperatorReconciliation":recorded,
        "activationFailureRetryExhaustionAfterRestart":recorded,
        "coldSurrealKvBackupRestore":true,
        "ordinaryRunNotProjectedAsInstance":true,
        "ordinaryRunInspectionRemainsProcessLocal":true,
        "notCoveredByThisScenario":["stale-epoch protected-effect claim/dispatch race"]
    });
    println!("C06_INTEGRATION_RECEIPT={receipt}");
}
