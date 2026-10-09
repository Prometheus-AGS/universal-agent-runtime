//! Real full-harness router/executor cursor contract with a controlled HTTP model.
//! Authentication identity is supplied by this fixture; this is not an auth gate.
use axum::{
    Extension, Json, Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use futures::StreamExt;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower::ServiceExt;
use universal_agent_runtime::{
    config::{LlmConfig, RunsConfig, ServiceInstanceConfig},
    mcp::registry::McpRegistry,
    session::SessionStore,
    uar::{
        api::{
            full_harness::{self, FullHarnessApiState, FullHarnessTaskAuthority},
            routes::{self, RunApiState},
        },
        compiler::collaboration::CollaborationCatalogService,
        defaults::default_agent,
        domain::events::NormalizedEvent,
        rag::embeddings::UnavailableEmbeddingBackend,
        runtime::{manager::RunManager, matching::VectorMatcher, skills::SkillRegistry},
        security::claims::{UserClaims, UserContext},
        service_instance::ServiceInstanceAuthority,
    },
};

const WAIT: Duration = Duration::from_secs(45);
const PROFILE: &str = "contiguous_cursor_frames_v1";

/// The actual Liter HTTP driver consumes these provider-shaped SSE deltas.
async fn model(Json(request): Json<Value>) -> Response {
    let input = request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|message| message["role"] == "user")
        .unwrap()["content"]
        .as_str()
        .unwrap()
        .to_owned();
    let count = if input == "retention-gap" { 700 } else { 3 };
    let mut body = String::new();
    for index in 0..count {
        let chunk = json!({"id":"cursor-model", "object":"chat.completion.chunk", "created":0,
            "model":"gpt-4o-mini", "choices":[{"index":0,
                "delta":if index == 0 { json!({"role":"assistant","content":"x"}) }
                    else { json!({"content":"x"}) }, "finish_reason":null}]});
        body.push_str(&format!("data: {chunk}\n\n"));
    }
    let terminal = json!({"id":"cursor-model", "object":"chat.completion.chunk", "created":0,
        "model":"gpt-4o-mini", "choices":[{"index":0,"delta":{},"finish_reason":"stop"}]});
    body.push_str(&format!("data: {terminal}\n\ndata: [DONE]\n\n"));
    ([("content-type", "text/event-stream")], body).into_response()
}

struct Fixture {
    app: Router,
    manager: Arc<RunManager>,
    model: tokio::task::JoinHandle<()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.model.abort();
    }
}

impl Fixture {
    async fn new() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}/v1", listener.local_addr().unwrap());
        let model = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route("/v1/chat/completions", post(model)),
            )
            .await
            .unwrap();
        });
        let manager = Arc::new(
            RunManager::new(
                LlmConfig {
                    model: "openai/gpt-4o-mini".to_owned(),
                    base_url: Some(base),
                    api_key: Some("cursor-fixture-provider".to_owned()),
                    host_supplied_connection: true,
                    ..LlmConfig::default()
                },
                Arc::new(McpRegistry::new_empty()),
                SessionStore::new(),
                Arc::new(RwLock::new(SkillRegistry::default())),
                Arc::new(VectorMatcher::new(
                    Arc::new(UnavailableEmbeddingBackend::new(
                        384,
                        "cursor fixture does not use embeddings",
                    )),
                    0.75,
                )),
                None,
            )
            .await,
        );
        let runs = Arc::new(RunApiState {
            manager: manager.clone(),
            collaboration_catalog: Arc::new(CollaborationCatalogService::in_memory()),
            service_instance: Arc::new(ServiceInstanceAuthority::new(
                &ServiceInstanceConfig::default(),
                "cursor-instance",
                "http://127.0.0.1",
                false,
                &[],
            )),
        });
        let full = Arc::new(FullHarnessApiState {
            authority: Arc::new(FullHarnessTaskAuthority::new(
                manager.clone(),
                RunsConfig::default(),
            )),
            runs: runs.clone(),
        });
        let user = UserContext {
            user_id: "cursor-owner".to_owned(),
            tenant_id: None,
            authority: None,
            host_authority: None,
            claims: UserClaims {
                sub: "cursor-owner".to_owned(),
                name: None,
                roles: None,
                tenant_id: None,
                uar_instance_id: None,
                exp: usize::MAX,
            },
        };
        let app = Router::new()
            .nest(
                "/api/uar/full-harness/v1",
                full_harness::build_router().with_state(full),
            )
            .nest("/api/uar", routes::build_router().with_state(runs))
            .layer(Extension(user));
        Self {
            app,
            manager,
            model,
        }
    }

    async fn request(&self, path: &str, body: Option<Value>, cursor: Option<u64>) -> Response {
        let mut request = Request::builder()
            .uri(path)
            .method(if body.is_some() { "POST" } else { "GET" })
            .header("x-uar-workspace-id", "cursor-workspace")
            .header("content-type", "application/json");
        if let Some(cursor) = cursor {
            request = request.header("last-event-id", cursor.to_string());
        }
        tokio::time::timeout(
            WAIT,
            self.app.clone().oneshot(
                request
                    .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                    .unwrap(),
            ),
        )
        .await
        .expect("router deadline")
        .unwrap()
    }

    async fn admit(&self, input: &str) -> Value {
        let mut artifact = default_agent();
        artifact.id = format!("cursor-{input}");
        let response = self
            .request(
                "/api/uar/full-harness/v1/tasks",
                Some(json!({
                    "admission_id":uuid::Uuid::new_v4().to_string(),
                    "native_task_id":format!("native-{input}"), "artifact":artifact, "input":input,
                })),
                None,
            )
            .await;
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap()
    }

    async fn settled(&self, run: &str) {
        tokio::time::timeout(WAIT, async {
            loop {
                if self
                    .manager
                    .history_since(run, None)
                    .await
                    .is_some_and(|history| {
                        history.iter().any(|event| {
                            matches!(
                                event.event,
                                NormalizedEvent::RunDone { .. }
                                    | NormalizedEvent::RunDoneWithUsage { .. }
                                    | NormalizedEvent::Error { .. }
                                    | NormalizedEvent::Cancelled { .. }
                            )
                        })
                    })
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("real model/executor terminal deadline");
    }
}

#[derive(Debug, PartialEq)]
struct Frame {
    id: u64,
    name: String,
    payload: Value,
}

fn frames(text: &str) -> Vec<Frame> {
    let complete = text.rfind("\n\n").map_or("", |end| &text[..end]);
    complete
        .split("\n\n")
        .filter_map(|frame| {
            let mut id = None;
            let mut name = None;
            let mut data = None;
            for line in frame.lines() {
                if let Some(value) = line.strip_prefix("id:") {
                    id = Some(value.trim().parse().unwrap());
                }
                if let Some(value) = line.strip_prefix("event:") {
                    name = Some(value.trim().to_owned());
                }
                if let Some(value) = line.strip_prefix("data:") {
                    data = Some(serde_json::from_str(value.trim()).unwrap());
                }
            }
            Some(Frame {
                id: id?,
                name: name?,
                payload: data?,
            })
        })
        .collect()
}

async fn stream(response: Response) -> Vec<Frame> {
    assert_eq!(response.status(), StatusCode::OK);
    tokio::time::timeout(WAIT, async {
        let mut data = response.into_body().into_data_stream();
        let mut text = String::new();
        while let Some(chunk) = data.next().await {
            text.push_str(std::str::from_utf8(&chunk.unwrap()).unwrap());
            let parsed = frames(&text);
            if parsed.last().is_some_and(|frame| {
                matches!(
                    frame.name.as_str(),
                    "agui.done" | "agui.error" | "agui.cancelled"
                )
            }) {
                return parsed;
            }
        }
        panic!("SSE ended before a terminal frame");
    })
    .await
    .expect("SSE terminal deadline")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn filtered_runtime_steps_keep_original_cursors_and_reconnect_without_gaps() {
    let fixture = Fixture::new().await;
    let response = fixture
        .request("/api/uar/full-harness/v1/capabilities", None, None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let descriptor: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert_eq!(descriptor["event_cursor_profile"], PROFILE);
    assert_eq!(descriptor["recovery"], "unsupported_after_restart");
    let receipt = fixture.admit("normal").await;
    let run = receipt["run_id"].as_str().unwrap();
    fixture.settled(run).await;
    let history = fixture.manager.history_since(run, None).await.unwrap();
    let runtime_ids: Vec<_> = history
        .iter()
        .filter_map(|event| {
            matches!(event.event, NormalizedEvent::RuntimeStep { .. }).then_some(event.id)
        })
        .collect();
    assert!(
        !runtime_ids.is_empty(),
        "positive control requires actual executor RuntimeStep output"
    );
    let path = receipt["links"]["stream"].as_str().unwrap();
    let full = stream(fixture.request(path, None, None).await).await;
    assert_eq!(full.last().unwrap().name, "agui.done");
    assert_eq!(
        full.iter().map(|frame| frame.id).collect::<Vec<_>>(),
        history.iter().map(|event| event.id).collect::<Vec<_>>()
    );
    assert!(full.windows(2).all(|pair| pair[1].id == pair[0].id + 1));
    for id in &runtime_ids {
        let frame = full.iter().find(|frame| frame.id == *id).unwrap();
        assert_eq!(frame.name, "uar.cursor");
        assert_eq!(frame.payload, json!({"request_id":run}));
    }
    let cursor = runtime_ids[0];
    let resumed = stream(fixture.request(path, None, Some(cursor)).await).await;
    assert_eq!(
        resumed,
        full.into_iter()
            .filter(|frame| frame.id > cursor)
            .collect::<Vec<_>>()
    );
    assert_eq!(resumed.first().unwrap().id, cursor + 1);
    let ordinary = stream(
        fixture
            .request(&format!("/api/uar/runs/{run}/stream"), None, None)
            .await,
    )
    .await;
    assert!(ordinary.iter().all(|frame| frame.name != "uar.cursor"));
    assert!(
        ordinary
            .iter()
            .all(|frame| !runtime_ids.contains(&frame.id)),
        "ordinary AG-UI suppression remains unchanged"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lost_retained_history_remains_an_explicit_stream_gap() {
    let fixture = Fixture::new().await;
    let receipt = fixture.admit("retention-gap").await;
    let run = receipt["run_id"].as_str().unwrap();
    fixture.settled(run).await;
    let retained = fixture.manager.history_since(run, None).await.unwrap();
    assert!(
        retained.first().unwrap().id > 2,
        "real provider deltas must exceed retained history"
    );
    let result = stream(
        fixture
            .request(receipt["links"]["stream"].as_str().unwrap(), None, Some(1))
            .await,
    )
    .await;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, 2);
    assert_eq!(result[0].name, "agui.error");
    assert_eq!(result[0].payload["code"], "STREAM_GAP");
    assert_eq!(result[0].payload["request_id"], run);
}
