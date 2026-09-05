/// Integration test for the A2A v0.3 gRPC transport (CH-01 a2a-grpc-enable).
///
/// Starts a real `GrpcAgentService` on an ephemeral port, connects a tonic
/// client, and round-trips `MessageSend` + `TaskGet` — the same server-side
/// persisted thread service the JSON-RPC binding uses, so this
/// exercises the actual production service implementation, not a mock.
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tonic::Request;
use universal_agent_runtime::config::{LlmConfig, SecurityConfig};
use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::api::a2a::grpc::GrpcAgentService;
use universal_agent_runtime::uar::api::a2a::grpc::pb::agent_service_client::AgentServiceClient;
use universal_agent_runtime::uar::api::a2a::grpc::pb::{
    CancelTaskRequest, GetTaskRequest, Message, Part, SendMessageRequest, part,
};
use universal_agent_runtime::uar::api::a2a::{A2AState, thread_service::A2AThreadService};
use universal_agent_runtime::uar::persistence::{
    PersistenceLayer, providers::surreal::SurrealDbProvider,
};
use universal_agent_runtime::uar::rag::embeddings::{
    EmbeddingBackend, UnavailableEmbeddingBackend,
};
use universal_agent_runtime::uar::runtime::{
    actor::system::ActorCollaboration, manager::RunManager, matching::VectorMatcher,
    skills::SkillRegistry,
};
use universal_agent_runtime::uar::security::api_keys::{
    ApiKeyService, CreateKeyRequest, InMemoryApiKeyStorage,
};

fn find_free_port() -> u16 {
    StdTcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn integration_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()
        .expect("A2A gRPC integration runtime builds")
}

/// Start a real `GrpcAgentService` on a free port and return its base URL.
async fn start_grpc_server() -> (String, tempfile::TempDir, String) {
    let port = find_free_port();
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let database = tempfile::tempdir().expect("A2A gRPC test database directory");
    let endpoint = format!(
        "surrealkv://{}",
        database.path().join("a2a-grpc.db").display()
    );
    let persistence: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(
            &endpoint,
            None,
            None,
            Some("a2a-grpc-test"),
            Some("a2a-grpc-test"),
        )
        .await
        .expect("A2A gRPC test database opens"),
    );
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by A2A gRPC tests",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig::default(),
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            Some(persistence),
        )
        .await
        .with_llm_driver(Arc::new(MockLlmDriver::echo())),
    );
    let state = Arc::new(A2AState {
        threads: Arc::new(A2AThreadService::new(Arc::new(ActorCollaboration::new(
            manager,
        )))),
        security: SecurityConfig {
            jwt_required: false,
            jwt_secret: "test-secret".to_owned().into(),
            jwks_url: None,
            jwt_issuer: None,
            jwt_audience: None,
            jwt_validate_nbf: true,
            settings_mutation_auth_required: true,
            settings_admin_key: Some("test-admin-key".to_string().into()),
        },
        base_url: format!("http://127.0.0.1:{port}"),
    });
    let service = GrpcAgentService::new(state);
    let api_keys = ApiKeyService::new(Arc::new(InMemoryApiKeyStorage::new()), "test-secret");
    let key = api_keys
        .create_key(
            "a2a-grpc-user",
            CreateKeyRequest {
                name: "A2A gRPC integration".to_owned(),
                roles: Some(vec!["user".to_owned()]),
                expires_in_secs: None,
            },
        )
        .await
        .expect("A2A gRPC test API key is created");
    let token = api_keys
        .exchange_for_jwt(&key.raw_key)
        .await
        .expect("A2A gRPC test API key exchange succeeds")
        .expect("A2A gRPC test API key returns a token");

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(service.into_server())
            .serve(addr)
            .await
            .unwrap();
    });

    // The listener bind above races the server's own bind; retry-connect
    // instead of a fixed sleep to avoid flakiness under load.
    (format!("http://127.0.0.1:{port}"), database, token)
}

async fn connect(url: &str) -> AgentServiceClient<tonic::transport::Channel> {
    for _ in 0..50 {
        if let Ok(client) = AgentServiceClient::connect(url.to_string()).await {
            return client;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("gRPC server did not become ready in time");
}

fn text_message(text: &str) -> Message {
    Message {
        role: "user".to_string(),
        parts: vec![Part {
            content: Some(part::Content::Text(text.to_string())),
            content_type: "text/plain".to_string(),
        }],
    }
}

fn authenticated<T>(message: T, token: &str) -> Request<T> {
    let mut request = Request::new(message);
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {token}")
            .parse()
            .expect("authorization metadata parses"),
    );
    request
}

#[test]
fn grpc_message_send_and_task_get_round_trip() {
    integration_runtime().block_on(async {
        let (url, _database, token) = start_grpc_server().await;
        let mut client = connect(&url).await;

        let send_resp = client
            .message_send(authenticated(
                SendMessageRequest {
                    task_id: String::new(),
                    message: Some(text_message("hello from the gRPC integration test")),
                },
                &token,
            ))
            .await
            .expect("message_send should succeed")
            .into_inner();

        assert!(!send_resp.task_id.is_empty(), "task_id should be assigned");
        assert!(!send_resp.status.is_empty(), "status should be set");

        let get_resp = client
            .task_get(authenticated(
                GetTaskRequest {
                    task_id: send_resp.task_id.clone(),
                },
                &token,
            ))
            .await
            .expect("task_get should succeed")
            .into_inner();

        assert_eq!(get_resp.task_id, send_resp.task_id);
    });
}

#[test]
fn grpc_task_get_missing_task_returns_not_found() {
    integration_runtime().block_on(async {
        let (url, _database, token) = start_grpc_server().await;
        let mut client = connect(&url).await;

        let err = client
            .task_get(authenticated(
                GetTaskRequest {
                    task_id: "does-not-exist".to_string(),
                },
                &token,
            ))
            .await
            .expect_err("unknown task_id should error");

        assert_eq!(err.code(), tonic::Code::NotFound);
    });
}

/// Covers the streaming RPC (`MessageStream`) — previously untested; only
/// the three unary methods had coverage.
#[test]
fn grpc_message_stream_emits_a_status_update_event() {
    integration_runtime().block_on(async {
        let (url, _database, token) = start_grpc_server().await;
        let mut client = connect(&url).await;

        let mut stream = client
            .message_stream(authenticated(
                SendMessageRequest {
                    task_id: String::new(),
                    message: Some(text_message("hello via streaming")),
                },
                &token,
            ))
            .await
            .expect("message_stream should succeed")
            .into_inner();

        let event = stream
            .message()
            .await
            .expect("stream should yield a message without transport error")
            .expect("stream should yield at least one event");

        assert_eq!(event.event_type, "status_update");
        assert!(!event.task_id.is_empty(), "task_id should be assigned");
        let task_state = event.state.expect("event should carry task state");
        assert_eq!(task_state.task_id, event.task_id);

        // Current implementation emits exactly one event then completes (see
        // grpc.rs's message_stream doc comment) — assert that contract holds so
        // a future change to multi-event streaming updates this test instead of
        // silently changing observable behavior.
        let next = stream
            .message()
            .await
            .expect("no transport error on stream completion");
        assert!(next.is_none(), "stream should complete after one event");
    });
}

/// Covers `TaskCancel` — previously untested.
#[test]
fn grpc_task_cancel_transitions_task_to_canceled() {
    integration_runtime().block_on(async {
        let (url, _database, token) = start_grpc_server().await;
        let mut client = connect(&url).await;

        let send_resp = client
            .message_send(authenticated(
                SendMessageRequest {
                    task_id: String::new(),
                    message: Some(text_message("please cancel me")),
                },
                &token,
            ))
            .await
            .expect("message_send should succeed")
            .into_inner();

        let cancel_resp = client
            .task_cancel(authenticated(
                CancelTaskRequest {
                    task_id: send_resp.task_id.clone(),
                },
                &token,
            ))
            .await
            .expect("task_cancel should succeed")
            .into_inner();

        assert_eq!(cancel_resp.status, "canceled");

        let get_resp = client
            .task_get(authenticated(
                GetTaskRequest {
                    task_id: send_resp.task_id,
                },
                &token,
            ))
            .await
            .expect("task_get after cancel should succeed")
            .into_inner();
        assert_eq!(
            get_resp.status, "canceled",
            "cancellation must persist, not just be reflected in the cancel response"
        );
    });
}

#[test]
fn grpc_task_cancel_missing_task_returns_not_found() {
    integration_runtime().block_on(async {
        let (url, _database, token) = start_grpc_server().await;
        let mut client = connect(&url).await;

        let err = client
            .task_cancel(authenticated(
                CancelTaskRequest {
                    task_id: "does-not-exist".to_string(),
                },
                &token,
            ))
            .await
            .expect_err("cancelling an unknown task_id should error");

        assert_eq!(err.code(), tonic::Code::NotFound);
    });
}
