//! Synthetic receiver and real sidecar router for the resource contract.
//! Token labels exercise transport isolation, never external JWT certification.
use super::{sidecar_process, stub_llm};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use reqwest::Method;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const SERVER: &str = "resource";
pub const TOOL: &str = "resource__effect";
pub const SCOPE: &str = "fixture:invoke";
pub const OWNER_A: &str = "resource-owner-a";
pub const OWNER_B: &str = "resource-owner-b";

#[derive(Clone, Debug)]
pub struct Effect {
    pub label: String,
    pub session: String,
    pub argument: String,
}

#[derive(Default)]
pub struct ReceiverState {
    pub effects: Mutex<Vec<Effect>>,
    sequence: AtomicUsize,
}

pub struct Peer {
    pub url: String,
    pub state: Arc<ReceiverState>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn reply(id: Value, result: Value) -> Response {
    Json(json!({"jsonrpc":"2.0", "id":id, "result":result})).into_response()
}

async fn receiver(
    State(state): State<Arc<ReceiverState>>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Response {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    match request["method"].as_str().unwrap_or_default() {
        "initialize" => {
            let mut response = reply(
                id,
                json!({
                    "protocolVersion":request["params"]["protocolVersion"],
                    "capabilities":{"tools":{}}, "serverInfo":{"name":"resource-fixture","version":"1"}
                }),
            );
            let session = format!(
                "resource-session-{}",
                state.sequence.fetch_add(1, Ordering::SeqCst)
            );
            response
                .headers_mut()
                .insert("mcp-session-id", HeaderValue::from_str(&session).unwrap());
            response
        }
        "tools/list" => reply(
            id,
            json!({"tools":[{
                "name":"effect", "description":"Record one synthetic fixture effect",
                "inputSchema":{"type":"object","properties":{"argument":{"type":"string"}},"required":["argument"]}
            }]}),
        ),
        "tools/call" => {
            let label = match headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
            {
                Some("Bearer resource-a1") => "a1",
                Some("Bearer resource-a2") => "a2",
                Some("Bearer resource-b1") => "b1",
                Some("Bearer resource-renewed") => "renewed",
                _ => return StatusCode::UNAUTHORIZED.into_response(),
            };
            if request["params"]["name"] != "effect" {
                return StatusCode::BAD_REQUEST.into_response();
            }
            state.effects.lock().unwrap().push(Effect {
                label: label.to_owned(),
                session: headers
                    .get("mcp-session-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_owned(),
                argument: request["params"]["arguments"]["argument"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
            });
            reply(
                id,
                json!({"content":[{"type":"text","text":format!("effect:{label}")}],"isError":false}),
            )
        }
        _ => StatusCode::ACCEPTED.into_response(),
    }
}

impl Peer {
    pub async fn start() -> Self {
        let state = Arc::new(ReceiverState::default());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/mcp", listener.local_addr().unwrap());
        let router = Router::new()
            .route("/mcp", post(receiver))
            .with_state(state.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self { url, state, task }
    }

    pub fn effects(&self) -> Vec<Effect> {
        self.state.effects.lock().unwrap().clone()
    }
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn grant(label: &str, revision: &str, expiry: u64) -> Value {
    json!({"name":SERVER,"grant":{"credential_revision":revision,"scopes":[SCOPE],
        "expires_at_unix":expiry,"headers":{"Authorization":format!("Bearer resource-{label}")}}})
}

pub struct Host {
    pub process: sidecar_process::ServerProcess,
    pub workspace: sidecar_process::Workspace,
    pub model: stub_llm::StubLlmServer,
    token: String,
    client: reqwest::Client,
}

impl Host {
    pub async fn start(peer: &Peer) -> Self {
        let mut fixtures = stub_llm::FixtureSet::new();
        for input in [
            "owner-a-first",
            "owner-a-second",
            "owner-b-first",
            "renewed",
            "expired-at-effect",
            "revoked-at-effect",
            "denied-tool",
        ] {
            for has_tools in [false, true] {
                for has_tool_result in [false, true] {
                    fixtures = fixtures.with(
                        stub_llm::RequestFingerprint {
                            model: sidecar_process::STUB_MODEL.to_owned(),
                            last_user_message: input.to_owned(),
                            has_tools,
                            has_tool_result,
                        },
                        if has_tool_result {
                            stub_llm::FixtureResponse::Content("finished".to_owned())
                        } else {
                            stub_llm::FixtureResponse::ToolCall {
                                name: TOOL.to_owned(),
                                arguments: json!({"argument":input}).to_string(),
                            }
                        },
                    );
                }
            }
        }
        let model = stub_llm::start_stub_llm(fixtures).await;
        let workspace = sidecar_process::Workspace::new();
        let entry = |destination: &str| {
            json!({"url":peer.url,"grant_policy":{
                "destination_id":destination,"trusted_hosts":["sidecar-launch-host"],
                "required_scopes":[SCOPE],"allow_private_http":true
            }})
        };
        std::fs::write(
            workspace.work().join("mcp.json"),
            serde_json::to_vec(&json!({
                "mcpServers":{"resource":entry("resource-one"),"alternate":entry("resource-two")}
            }))
            .unwrap(),
        )
        .unwrap();
        let options = sidecar_process::ConfigOptions::new(&model.base_url);
        let config = workspace.write_config(
            "config.yaml",
            &sidecar_process::render_config(&workspace, &options),
        );
        let token = sidecar_process::random_token();
        let process = sidecar_process::launch_sidecar(
            &workspace,
            &sidecar_process::LaunchOptions::new(config, "resource-boundary").with_token(&token),
        )
        .await;
        Self {
            process,
            workspace,
            model,
            token,
            client: reqwest::Client::new(),
        }
    }

    fn request(&self, method: Method, path: &str, owner: &str) -> reqwest::RequestBuilder {
        self.client
            .request(method, format!("{}{path}", self.process.base_url()))
            .bearer_auth(&self.token)
            .header("X-UAR-Principal", owner)
    }

    pub async fn call(&self, method: Method, path: &str, owner: &str, body: Value) -> (u16, Value) {
        let response = self
            .request(method, path, owner)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = response.status().as_u16();
        let body = response.text().await.unwrap();
        (
            status,
            serde_json::from_str(&body).unwrap_or_else(|_| json!({"body":body})),
        )
    }

    pub async fn create(
        &self,
        owner: &str,
        input: &str,
        resource: Value,
        agent: &str,
    ) -> (u16, Value) {
        self.call(
            Method::POST,
            "/api/uar/runs",
            owner,
            json!({"agent_id":agent,"input":input,"mcp_servers":[resource]}),
        )
        .await
    }

    pub async fn open(&self, owner: &str, run: &str) -> Transcript {
        let response = self
            .request(Method::GET, &format!("/api/uar/runs/{run}/stream"), owner)
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        Transcript {
            response,
            text: String::new(),
            answered: BTreeSet::new(),
            deadline: tokio::time::Instant::now() + Duration::from_secs(90),
        }
    }

    pub async fn approve(&self, owner: &str, run: &str, id: &str) {
        let (status, body) = self
            .call(
                Method::POST,
                &format!("/api/uar/runs/{run}/tool-approval"),
                owner,
                json!({"approval_id":id,"approved":true}),
            )
            .await;
        assert_eq!(status, 200, "exact approval rejected: {body}");
    }
}

pub struct Transcript {
    response: reqwest::Response,
    pub text: String,
    pub answered: BTreeSet<String>,
    deadline: tokio::time::Instant,
}

impl Transcript {
    async fn chunk(&mut self) {
        let chunk = tokio::time::timeout_at(self.deadline, self.response.chunk())
            .await
            .expect("resource run timed out")
            .expect("stream read")
            .expect("stream ended without terminal");
        self.text.push_str(&String::from_utf8_lossy(&chunk));
    }

    pub async fn pending(&mut self, run: &str) -> String {
        loop {
            if let Some(id) = sidecar_process::approval_ids(&self.text, run).first() {
                return id.clone();
            }
            assert!(
                !self.terminal(),
                "run ended before its controlled approval wait: {}",
                self.text
            );
            self.chunk().await;
        }
    }

    fn terminal(&self) -> bool {
        [
            "event: agui.done",
            "event: agui.error",
            "event: agui.cancelled",
        ]
        .iter()
        .any(|marker| self.text.contains(marker))
    }

    pub async fn finish(mut self, host: &Host, owner: &str, run: &str) -> String {
        loop {
            if self.terminal() {
                return self.text;
            }
            for id in sidecar_process::approval_ids(&self.text, run) {
                if self.answered.insert(id.clone()) {
                    host.approve(owner, run, &id).await;
                }
            }
            self.chunk().await;
        }
    }
}
