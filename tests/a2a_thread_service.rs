//! Inbound A2A integration over the persisted actor/thread kernel.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use async_trait::async_trait;
use axum::{
    Extension, Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
    routing::post,
};
use futures::stream;
use tokio::sync::RwLock;
use tower::ServiceExt;
use universal_agent_runtime::{
    config::{LlmConfig, SecurityConfig},
    llm::{ExternalDriverStream, LlmDriver, LlmRequest, mock_driver::MockLlmDriver},
    mcp::registry::McpRegistry,
    session::SessionStore,
    uar::{
        api::{
            a2a::{
                A2AState, contract::UAR_CLEANUP_CLOSED_METADATA, handler::handle_agent_rpc,
                thread_service::A2AThreadService,
            },
            actors,
        },
        persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
        rag::embeddings::{EmbeddingBackend, UnavailableEmbeddingBackend},
        runtime::{
            actor::{messages::ActorOwner, system::ActorCollaboration},
            manager::RunManager,
            matching::VectorMatcher,
            skills::SkillRegistry,
        },
        security::claims::{UserClaims, UserContext},
    },
};

struct Harness {
    app: Router,
    unauthenticated_actor_app: Router,
    owner: ActorOwner,
    persistence: Arc<dyn PersistenceLayer>,
    _database: tempfile::TempDir,
}

fn integration_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()
        .expect("A2A integration runtime builds")
}

fn security() -> SecurityConfig {
    SecurityConfig {
        jwt_required: false,
        jwt_secret: "a2a-thread-test-secret".to_owned().into(),
        jwks_url: None,
        jwt_issuer: None,
        jwt_audience: None,
        jwt_validate_nbf: true,
        settings_mutation_auth_required: true,
        settings_admin_key: Some("a2a-thread-test-admin".to_owned().into()),
    }
}

fn user() -> UserContext {
    UserContext {
        user_id: "a2a-thread-user".to_owned(),
        tenant_id: None,
        claims: UserClaims {
            sub: "a2a-thread-user".to_owned(),
            name: None,
            roles: None,
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        },
    }
}

async fn harness(driver: Arc<dyn LlmDriver>) -> Harness {
    let database = tempfile::tempdir().expect("A2A thread database directory");
    let endpoint = format!(
        "surrealkv://{}",
        database.path().join("a2a-threads.db").display()
    );
    let persistence: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(
            &endpoint,
            None,
            None,
            Some("a2a-thread-test"),
            Some("a2a-thread-test"),
        )
        .await
        .expect("A2A thread database opens"),
    );
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by A2A thread tests",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig::default(),
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            Some(Arc::clone(&persistence)),
        )
        .await
        .with_llm_driver(driver),
    );
    let context = user();
    let owner = ActorOwner::from_verified_context(&context).expect("verified A2A owner");
    let collaboration = Arc::new(ActorCollaboration::new(manager));
    let state = Arc::new(A2AState {
        threads: Arc::new(A2AThreadService::new(Arc::clone(&collaboration))),
        security: security(),
        base_url: "http://127.0.0.1:3928".to_owned(),
    });
    let app = Router::new()
        .route("/a2a/agents/{agent_id}", post(handle_agent_rpc))
        .with_state(state)
        .layer(Extension(context));
    let unauthenticated_actor_app = Router::new().nest(
        "/api/uar/actors",
        actors::build_router().with_state(collaboration),
    );
    Harness {
        app,
        unauthenticated_actor_app,
        owner,
        persistence,
        _database: database,
    }
}

async fn rpc(
    app: &Router,
    agent_id: &str,
    request_id: &str,
    method: &str,
    params: serde_json::Value,
) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/a2a/agents/{agent_id}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "method": method,
                        "params": params,
                    })
                    .to_string(),
                ))
                .expect("A2A request builds"),
        )
        .await
        .expect("A2A route responds");
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("A2A response body reads"),
    )
    .expect("A2A response is JSON")
}

fn assert_success_envelope(response: &serde_json::Value, request_id: &str) {
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], request_id);
    assert!(response.get("result").is_some());
    assert!(response.get("error").is_none());
}

async fn get_until_terminal(app: &Router, agent_id: &str, task_id: &str) -> serde_json::Value {
    for index in 0..100 {
        let request_id = format!("get-{index}");
        let response = rpc(
            app,
            agent_id,
            &request_id,
            "tasks/get",
            serde_json::json!({"id": task_id}),
        )
        .await;
        assert_success_envelope(&response, &request_id);
        if matches!(
            response["result"]["status"]["state"].as_str(),
            Some("completed" | "canceled" | "failed")
        ) {
            return response;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("A2A task did not reach a terminal state");
}

#[test]
fn named_agent_send_and_get_project_the_persisted_thread() {
    integration_runtime().block_on(async {
        let harness = harness(Arc::new(MockLlmDriver::echo())).await;
        let agent_id = "rust-reviewer";
        let send = rpc(
            &harness.app,
            agent_id,
            "send-named-agent",
            "message/send",
            serde_json::json!({
                "message": {
                    "role": "user",
                    "parts": [{"type": "text", "text": "review this change"}]
                }
            }),
        )
        .await;
        assert_success_envelope(&send, "send-named-agent");
        assert_eq!(send["result"]["metadata"]["agent_id"], agent_id);
        assert_eq!(
            send["result"]["history"][0]["parts"][0]["text"],
            "review this change"
        );
        let task_id = send["result"]["id"]
            .as_str()
            .expect("send returns task id")
            .to_owned();

        let get = get_until_terminal(&harness.app, agent_id, &task_id).await;
        assert_eq!(get["result"]["id"], task_id);
        assert_eq!(get["result"]["status"]["state"], "completed");
        let thread_id = get["result"]["metadata"]["thread_id"]
            .as_str()
            .expect("terminal task exposes persisted thread id");
        let record = harness
            .persistence
            .load_agent_thread(harness.owner.user_id(), thread_id)
            .await
            .expect("persisted thread query succeeds")
            .expect("named-agent thread is persisted");
        assert_eq!(record.thread.artifact_id, agent_id);
        assert_eq!(
            record.thread.run_id.as_deref(),
            get["result"]["metadata"]["run_id"].as_str()
        );
    });
}

#[derive(Default)]
struct PendingLlmDriver {
    calls: AtomicUsize,
}

#[async_trait]
impl LlmDriver for PendingLlmDriver {
    async fn stream(&self, _request: LlmRequest) -> anyhow::Result<ExternalDriverStream> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Box::pin(stream::pending()))
    }
}

#[test]
fn tasks_cancel_stops_the_named_agent_thread_and_preserves_wire_shape() {
    integration_runtime().block_on(async {
        let driver = Arc::new(PendingLlmDriver::default());
        let harness = harness(driver.clone()).await;
        let agent_id = "general-purpose";
        let send = rpc(
            &harness.app,
            agent_id,
            "send-cancellable",
            "message/send",
            serde_json::json!({
                "message": {
                    "role": "user",
                    "parts": [{"type": "text", "text": "wait for cancellation"}]
                }
            }),
        )
        .await;
        assert_success_envelope(&send, "send-cancellable");
        assert_eq!(send["result"]["status"]["state"], "working");
        let task_id = send["result"]["id"]
            .as_str()
            .expect("send returns task id")
            .to_owned();
        for _ in 0..200 {
            if driver.calls.load(Ordering::SeqCst) > 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(
            driver.calls.load(Ordering::SeqCst),
            1,
            "named agent model call must start"
        );

        let cancel = rpc(
            &harness.app,
            agent_id,
            "cancel-cancellable",
            "tasks/cancel",
            serde_json::json!({"id": task_id}),
        )
        .await;
        assert_success_envelope(&cancel, "cancel-cancellable");
        assert_eq!(cancel["result"]["id"], task_id);
        assert_eq!(cancel["result"]["status"]["state"], "canceled");
        assert_eq!(
            cancel["result"]["metadata"][UAR_CLEANUP_CLOSED_METADATA],
            true
        );

        let get = rpc(
            &harness.app,
            agent_id,
            "get-canceled",
            "tasks/get",
            serde_json::json!({"id": task_id}),
        )
        .await;
        assert_success_envelope(&get, "get-canceled");
        assert_eq!(get["result"]["status"]["state"], "canceled");
        assert_eq!(get["result"]["metadata"][UAR_CLEANUP_CLOSED_METADATA], true);
    });
}

#[tokio::test]
async fn every_actor_endpoint_without_user_context_returns_401() {
    let harness = harness(Arc::new(MockLlmDriver::echo())).await;
    let requests = [
        (Method::GET, "/api/uar/actors", ""),
        (
            Method::POST,
            "/api/uar/actors",
            r#"{"agent_id":"rust-reviewer"}"#,
        ),
        (Method::DELETE, "/api/uar/actors/reviewer", ""),
        (
            Method::POST,
            "/api/uar/actors/reviewer/message",
            r#"{"content":"hello"}"#,
        ),
        (
            Method::POST,
            "/api/uar/actors/reviewer/collaborate",
            r#"{"from_actor":"source","task":"review"}"#,
        ),
    ];

    for (method, uri, body) in requests {
        let response = harness
            .unauthenticated_actor_app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("actor request builds"),
            )
            .await
            .expect("actor route responds");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
    }
}
