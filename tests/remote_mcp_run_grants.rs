//! Gate R: authenticated Boss host -> registered UAR agent -> remote MCP.
//!
//! This is deliberately one release-boundary scenario. It uses a real sidecar
//! process and a real streamable-HTTP transport; assertions retain only
//! non-secret identity labels and session identifiers.

#[path = "support/sidecar_process.rs"]
mod sidecar_process;
#[path = "integration/live/stub_llm.rs"]
mod stub_llm;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use reqwest::Method;
use serde_json::{Value, json};
use tokio::net::TcpListener;

use sidecar_process::{
    ConfigOptions, LaunchOptions, ServerProcess, Workspace, launch_sidecar, random_token,
    render_config, test_agent,
};
use stub_llm::{
    FixtureResponse, FixtureSet, RequestFingerprint, StubLlmServer, start_stub_llm,
};

const P1: &str = "boss-session-1";
const P2: &str = "boss-session-2";
const PRINCIPAL_HEADER: &str = "X-UAR-Principal";
const SERVER: &str = "tenant-tools";
const TOOL: &str = "tenant-tools__echo_identity";
const SCOPE: &str = "tenant-tools:invoke";

#[derive(Clone, Debug)]
struct ObservedCall {
    method: String,
    identity: &'static str,
    session: Option<String>,
    supplied_tenant: Option<String>,
}

#[derive(Default)]
struct PeerState {
    calls: Mutex<Vec<ObservedCall>>,
    next_session: std::sync::atomic::AtomicUsize,
    disconnect_once: Mutex<HashMap<&'static str, bool>>,
}

struct RemotePeer {
    url: String,
    state: Arc<PeerState>,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for RemotePeer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

fn identity(headers: &HeaderMap) -> &'static str {
    match headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        Some("Bearer gate-r-p1") => P1,
        Some("Bearer gate-r-p2") => P2,
        Some("Bearer gate-r-renewed") => "boss-session-1-renewed",
        _ => headers
            .get("x-boss-principal")
            .and_then(|value| value.to_str().ok())
            .map_or("anonymous", |value| match value {
                P1 => P1,
                P2 => P2,
                "boss-session-1-renewed" => "boss-session-1-renewed",
                "registry-reconnect" => "registry-reconnect",
                _ => "unrecognized",
            }),
    }
}

fn rpc_result(id: Value, result: Value, session: Option<&str>) -> Response {
    let mut response = Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    }))
    .into_response();
    if let Some(session) = session {
        response.headers_mut().insert(
            "mcp-session-id",
            HeaderValue::from_str(session).expect("session header"),
        );
    }
    response
}

async fn mcp_handler(
    State(state): State<Arc<PeerState>>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Response {
    let method = request["method"].as_str().unwrap_or_default().to_owned();
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let caller = identity(&headers);
    let session = headers
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    match method.as_str() {
        "initialize" => {
            let ordinal = state
                .next_session
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let session = format!("gate-r-session-{ordinal}");
            rpc_result(
                id,
                json!({
                    "protocolVersion": request
                        .pointer("/params/protocolVersion")
                        .cloned()
                        .unwrap_or_else(|| json!("2025-03-26")),
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "gate-r-peer", "version": "1.0.0" }
                }),
                Some(&session),
            )
        }
        "notifications/initialized" => StatusCode::ACCEPTED.into_response(),
        "tools/list" => rpc_result(
            id,
            json!({
                "tools": [
                    {
                        "name": "echo_identity",
                        "description": "Return the verified downstream identity",
                        "inputSchema": {
                            "type": "object",
                            "properties": { "tenant_id": { "type": "string" } }
                        }
                    },
                    {
                        "name": "disconnect_once",
                        "description": "Drop one call so the client reconnects without replay",
                        "inputSchema": { "type": "object", "properties": {} }
                    }
                ]
            }),
            None,
        ),
        "tools/call" => {
            let tool = request.pointer("/params/name").and_then(Value::as_str);
            let supplied_tenant = request
                .pointer("/params/arguments/tenant_id")
                .and_then(Value::as_str)
                .map(str::to_owned);
            state.calls.lock().expect("peer call log").push(ObservedCall {
                method: tool.unwrap_or_default().to_owned(),
                identity: caller,
                session,
                supplied_tenant,
            });
            if tool == Some("disconnect_once") {
                let mut failures = state.disconnect_once.lock().expect("disconnect state");
                if !failures.insert(caller, true).unwrap_or(false) {
                    return StatusCode::SERVICE_UNAVAILABLE.into_response();
                }
            }
            rpc_result(
                id,
                json!({
                    "content": [{
                        "type": "text",
                        "text": format!("verified downstream identity: {caller}")
                    }],
                    "isError": false
                }),
                None,
            )
        }
        _ => StatusCode::ACCEPTED.into_response(),
    }
}

async fn start_remote_peer() -> RemotePeer {
    let state = Arc::new(PeerState::default());
    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .with_state(Arc::clone(&state));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind remote MCP peer");
    let address = listener.local_addr().expect("remote MCP address");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("remote MCP peer");
    });
    RemotePeer {
        url: format!("http://{address}/mcp"),
        state,
        handle,
    }
}

fn fixture(input: &str, response: FixtureResponse) -> (RequestFingerprint, FixtureResponse) {
    (
        RequestFingerprint {
            model: sidecar_process::STUB_MODEL.to_owned(),
            last_user_message: input.to_owned(),
            has_tools: true,
            has_tool_result: false,
        },
        response,
    )
}

fn tool_fixtures(inputs: &[(&str, &str)]) -> FixtureSet {
    let mut fixtures = FixtureSet::new();
    for (input, forged_tenant) in inputs {
        let (fingerprint, response) = fixture(
            input,
            FixtureResponse::ToolCall {
                name: TOOL.to_owned(),
                arguments: json!({ "tenant_id": forged_tenant }).to_string(),
            },
        );
        fixtures = fixtures.with(fingerprint, response).with(
            RequestFingerprint {
                model: sidecar_process::STUB_MODEL.to_owned(),
                last_user_message: (*input).to_owned(),
                has_tools: true,
                has_tool_result: true,
            },
            FixtureResponse::Content("done".to_owned()),
        );
    }
    fixtures.with(
        RequestFingerprint {
            model: sidecar_process::STUB_MODEL.to_owned(),
            last_user_message: "authenticated without MCP grant".to_owned(),
            has_tools: true,
            has_tool_result: false,
        },
        FixtureResponse::Content("no remote authority".to_owned()),
    )
}

struct GateHost {
    process: ServerProcess,
    token: String,
    workspace: Workspace,
    stub: StubLlmServer,
    client: reqwest::Client,
}

impl GateHost {
    async fn boot(peer: &RemotePeer, fixtures: FixtureSet) -> Self {
        let stub = start_stub_llm(fixtures).await;
        let workspace = Workspace::new();
        std::fs::write(
            workspace.work().join("mcp.json"),
            serde_json::to_vec_pretty(&json!({
                "mcpServers": {
                    SERVER: {
                        "url": peer.url,
                        "grant_policy": {
                            "destination_id": "gate-r-tenant-tools",
                            "trusted_hosts": ["sidecar-launch-host"],
                            "required_scopes": [SCOPE],
                            "allow_private_http": true
                        }
                    }
                }
            }))
            .expect("serialize mcp config"),
        )
        .expect("write mcp config");
        let options = ConfigOptions::new(&stub.base_url);
        let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));
        let token = random_token();
        let process = launch_sidecar(
            &workspace,
            &LaunchOptions::new(config, "gate-r").with_token(&token),
        )
        .await;
        Self {
            process,
            token,
            workspace,
            stub,
            client: reqwest::Client::new(),
        }
    }

    fn request(&self, method: Method, path: &str, principal: Option<&str>) -> reqwest::RequestBuilder {
        let request = self
            .client
            .request(method, format!("{}{path}", self.process.base_url()))
            .bearer_auth(&self.token);
        match principal {
            Some(principal) => request.header(PRINCIPAL_HEADER, principal),
            None => request,
        }
    }

    async fn call(
        &self,
        method: Method,
        path: &str,
        principal: Option<&str>,
        body: Option<Value>,
    ) -> (u16, String) {
        let mut request = self.request(method, path, principal);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.expect("send sidecar request");
        let status = response.status().as_u16();
        (status, response.text().await.unwrap_or_default())
    }

    async fn follow(&self, principal: &str, run_id: &str) -> String {
        let mut response = self
            .request(
                Method::GET,
                &format!("/api/uar/runs/{run_id}/stream"),
                Some(principal),
            )
            .send()
            .await
            .expect("open run stream");
        assert!(response.status().is_success(), "run stream: {}", response.status());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
        let mut text = String::new();
        let mut approvals = 0;
        loop {
            if text.matches("event: agui.tool_call.approval_required").count() > approvals {
                let (status, body) = self
                    .call(
                        Method::POST,
                        &format!("/api/uar/runs/{run_id}/tool-approval"),
                        Some(principal),
                        Some(json!({ "approved": true })),
                    )
                    .await;
                assert_eq!(status, 200, "approve remote tool call: {body}");
                approvals += 1;
                continue;
            }
            if ["event: agui.done", "event: agui.error", "event: agui.cancelled"]
                .iter()
                .any(|marker| text.contains(marker))
            {
                return text;
            }
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            assert!(!remaining.is_zero(), "run {run_id} timed out\n{text}");
            match tokio::time::timeout(remaining, response.chunk()).await {
                Ok(Ok(Some(chunk))) => text.push_str(&String::from_utf8_lossy(&chunk)),
                other => panic!("run {run_id} stream ended before terminal event: {other:?}\n{text}"),
            }
        }
    }
}

fn grant(revision: &str, bearer: &str, principal: &str, expires_at_unix: u64, scopes: &[&str]) -> Value {
    json!({
        "name": SERVER,
        "grant": {
            "credential_revision": revision,
            "scopes": scopes,
            "expires_at_unix": expires_at_unix,
            "headers": {
                "Authorization": format!("Bearer {bearer}"),
                "X-Boss-Principal": principal
            }
        }
    })
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock")
        .as_secs()
}

async fn create_run(host: &GateHost, principal: &str, input: &str, grant: Value) -> (u16, String) {
    host.call(
        Method::POST,
        "/api/uar/runs",
        Some(principal),
        Some(json!({
            "agent_id": "gate-r-agent",
            "input": input,
            "mcp_servers": [grant]
        })),
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn gate_r_registered_agent_remote_mcp_identity_and_lifecycle() {
    let peer = start_remote_peer().await;
    let inputs = [
        ("p1 remote identity", "forged-p2"),
        ("p2 remote identity", "forged-p1"),
        ("p1 renewed identity", "forged-after-renewal"),
    ];
    let host = GateHost::boot(&peer, tool_fixtures(&inputs)).await;

    let mut artifact = test_agent(None);
    artifact["id"] = json!("gate-r-agent");
    artifact["metadata"]["title"] = json!("Gate R v1");
    artifact["prompt"]["system"] = json!("gate-r-catalog-v1-marker");
    let (status, created) = host
        .call(Method::POST, "/api/agents", Some(P1), Some(artifact.clone()))
        .await;
    assert_eq!(status, 201, "register agent: {created}");

    let expiry = now_unix() + 10;
    let p1 = create_run(
        &host,
        P1,
        inputs[0].0,
        grant("p1-v1", "gate-r-p1", P1, expiry, &[SCOPE, "tenant-tools:optional"]),
    );
    let p2 = create_run(
        &host,
        P2,
        inputs[1].0,
        grant("p2-v1", "gate-r-p2", P2, now_unix() + 60, &[SCOPE]),
    );
    let ((p1_status, p1_body), (p2_status, p2_body)) = tokio::join!(p1, p2);
    assert_eq!(p1_status, 200, "P1 run: {p1_body}");
    assert_eq!(p2_status, 200, "P2 run: {p2_body}");
    let p1_run = serde_json::from_str::<Value>(&p1_body).expect("P1 run JSON")["run_id"]
        .as_str()
        .expect("P1 run id")
        .to_owned();
    let p2_run = serde_json::from_str::<Value>(&p2_body).expect("P2 run JSON")["run_id"]
        .as_str()
        .expect("P2 run id")
        .to_owned();
    let (p1_stream, p2_stream) = tokio::join!(host.follow(P1, &p1_run), host.follow(P2, &p2_run));
    assert!(p1_stream.contains("event: agui.done"), "P1 run failed\n{p1_stream}");
    assert!(p2_stream.contains("event: agui.done"), "P2 run failed\n{p2_stream}");

    let calls = peer.state.calls.lock().expect("peer calls").clone();
    let p1_call = calls
        .iter()
        .find(|call| call.method == "echo_identity" && call.identity == P1)
        .expect("P1 downstream call");
    let p2_call = calls
        .iter()
        .find(|call| call.method == "echo_identity" && call.identity == P2)
        .expect("P2 downstream call");
    assert_eq!(p1_call.supplied_tenant.as_deref(), Some("forged-p2"));
    assert_eq!(p2_call.supplied_tenant.as_deref(), Some("forged-p1"));
    assert_ne!(p1_call.session, p2_call.session, "principals reused an MCP session");

    artifact["metadata"]["title"] = json!("Gate R v2");
    artifact["prompt"]["system"] = json!("gate-r-catalog-v2-marker");
    let (status, updated) = host
        .call(
            Method::PUT,
            "/api/agents/gate-r-agent",
            Some(P1),
            Some(artifact),
        )
        .await;
    assert_eq!(status, 200, "update catalog agent: {updated}");

    tokio::time::sleep(Duration::from_secs(11)).await;
    let expired = grant("p1-v1", "gate-r-p1", P1, expiry, &[SCOPE]);
    let (status, body) = host
        .call(
            Method::POST,
            &format!("/api/uar/runs/{p1_run}/resume"),
            Some(P1),
            Some(json!({ "input": "expired grant", "mcp_servers": [expired] })),
        )
        .await;
    assert_eq!(status, 401, "expired grant did not require authentication: {body}");
    assert!(body.contains("run_mcp_grant_authentication_required"), "{body}");

    let renewed = grant(
        "p1-v2",
        "gate-r-renewed",
        "boss-session-1-renewed",
        now_unix() + 60,
        &[SCOPE],
    );
    let (status, body) = host
        .call(
            Method::POST,
            &format!("/api/uar/runs/{p1_run}/resume"),
            Some(P1),
            Some(json!({ "input": inputs[2].0, "mcp_servers": [renewed] })),
        )
        .await;
    assert_eq!(status, 200, "renew grant at resume boundary: {body}");
    let renewed_run = serde_json::from_str::<Value>(&body).expect("renewed run JSON")["run_id"]
        .as_str()
        .expect("renewed run id")
        .to_owned();
    let renewed_stream = host.follow(P1, &renewed_run).await;
    assert!(renewed_stream.contains("event: agui.done"), "renewed run failed\n{renewed_stream}");

    let requests: Value = host
        .client
        .get(format!("{}/_stub/requests", host.stub.base_url.trim_end_matches("/v1")))
        .send()
        .await
        .expect("read model request log")
        .json()
        .await
        .expect("model request log JSON");
    let resumed_requests = requests["requests"]
        .as_array()
        .expect("model request array")
        .iter()
        .filter(|request| request.to_string().contains(inputs[2].0))
        .map(Value::to_string)
        .collect::<Vec<_>>();
    assert!(!resumed_requests.is_empty(), "resumed model request absent");
    assert!(
        resumed_requests.iter().all(|request| request.contains("gate-r-catalog-v1-marker") && !request.contains("gate-r-catalog-v2-marker")),
        "resume did not retain the admitted catalog snapshot: {resumed_requests:?}"
    );

    let calls = peer.state.calls.lock().expect("peer calls").clone();
    let renewed_call = calls
        .iter()
        .find(|call| call.identity == "boss-session-1-renewed" && call.method == "echo_identity")
        .expect("renewed downstream call");
    assert_ne!(renewed_call.session, p1_call.session, "renewal reused the expired session");

    let mut unregistered = grant("bad-v1", "gate-r-p1", P1, now_unix() + 60, &[SCOPE]);
    unregistered["name"] = json!("not-registered");
    let (status, body) = host
        .call(
            Method::POST,
            "/api/uar/runs",
            Some(P1),
            Some(json!({
                "agent_id": "gate-r-agent",
                "input": "unregistered destination",
                "mcp_servers": [unregistered]
            })),
        )
        .await;
    assert_eq!(status, 422, "unregistered destination admitted: {body}");
    assert!(body.contains("run_mcp_destination_unregistered"), "{body}");

    let calls_before_ungranted_run = peer.state.calls.lock().expect("peer calls").len();
    let (status, body) = host
        .call(
            Method::POST,
            "/api/uar/runs",
            Some(P1),
            Some(json!({
                "agent_id": "gate-r-agent",
                "input": "authenticated without MCP grant"
            })),
        )
        .await;
    assert_eq!(status, 200, "authenticated ungranted run: {body}");
    let ungranted_run = serde_json::from_str::<Value>(&body).expect("ungranted run JSON")["run_id"]
        .as_str()
        .expect("ungranted run id")
        .to_owned();
    let ungranted_stream = host.follow(P1, &ungranted_run).await;
    assert!(
        ungranted_stream.contains("event: agui.done"),
        "ungranted run failed: {ungranted_stream}"
    );
    assert_eq!(
        peer.state.calls.lock().expect("peer calls").len(),
        calls_before_ungranted_run,
        "administrator catalog became executable without a run grant"
    );

    let response = host
        .client
        .post(format!("{}/api/uar/runs", host.process.base_url()))
        .json(&json!({ "agent_id": "gate-r-agent", "input": "unauthenticated" }))
        .send()
        .await
        .expect("unauthenticated request");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Exercise the same production transport's reconnect path. The failed call
    // is not replayed; a subsequent call uses a fresh session with the same
    // administrator-selected identity.
    let reconnect_config = universal_agent_runtime::mcp::config::McpConfig {
        mcp_servers: HashMap::from([(
            "reconnect".to_owned(),
            universal_agent_runtime::mcp::config::McpServerEntry::RemoteHttp {
                url: peer.url.clone(),
                env: HashMap::new(),
                headers: HashMap::from([(
                    "X-Boss-Principal".to_owned(),
                    universal_agent_runtime::mcp::config::McpHttpHeaderValue::Literal(
                        "registry-reconnect".to_owned(),
                    ),
                )]),
                grant_policy: None,
            },
        )]),
    };
    let registry = universal_agent_runtime::mcp::registry::McpRegistry::from_config(&reconnect_config)
        .await
        .expect("connect reconnect registry");
    assert!(
        registry
            .call_namespaced_tool("reconnect__disconnect_once", json!({}))
            .await
            .is_err(),
        "transport failure was replayed"
    );
    registry
        .call_namespaced_tool(
            "reconnect__echo_identity",
            json!({ "tenant_id": "still-forged" }),
        )
        .await
        .expect("subsequent call after reconnect");
    let calls = peer.state.calls.lock().expect("peer calls").clone();
    let reconnect_calls = calls
        .iter()
        .filter(|call| call.identity == "registry-reconnect")
        .collect::<Vec<_>>();
    assert_eq!(
        reconnect_calls
            .iter()
            .filter(|call| call.method == "disconnect_once")
            .count(),
        1,
        "failed tool call was replayed"
    );
    let reconnect_sessions = reconnect_calls
        .iter()
        .filter_map(|call| call.session.as_deref())
        .collect::<Vec<_>>();
    assert!(
        reconnect_sessions.windows(2).any(|pair| pair[0] != pair[1]),
        "reconnect did not establish a fresh transport session: {reconnect_sessions:?}"
    );

    let _keep_workspace_alive = host.workspace.path();
}
