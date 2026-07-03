/// Integration test for the A2A v0.3 gRPC transport (CH-01 a2a-grpc-enable).
///
/// Starts a real `GrpcAgentService` on an ephemeral port, connects a tonic
/// client, and round-trips `MessageSend` + `TaskGet` — the same server-side
/// state (`CompilerService`, `TaskStore`) the JSON-RPC binding uses, so this
/// exercises the actual production service implementation, not a mock.
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;
use std::time::Duration;

use universal_agent_runtime::uar::api::a2a::grpc::GrpcAgentService;
use universal_agent_runtime::uar::api::a2a::grpc::pb::agent_service_client::AgentServiceClient;
use universal_agent_runtime::uar::api::a2a::grpc::pb::{
    GetTaskRequest, Message, Part, SendMessageRequest, part,
};
use universal_agent_runtime::uar::api::a2a::{A2AState, TaskStore};
use universal_agent_runtime::uar::compiler::service::CompilerService;

fn find_free_port() -> u16 {
    StdTcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Start a real `GrpcAgentService` on a free port and return its base URL.
fn start_grpc_server() -> String {
    let port = find_free_port();
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    let state = Arc::new(A2AState {
        compiler_service: Arc::new(CompilerService::in_memory()),
        task_store: TaskStore::new(),
        base_url: format!("http://127.0.0.1:{port}"),
    });
    let service = GrpcAgentService::new(state);

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(service.into_server())
            .serve(addr)
            .await
            .unwrap();
    });

    // The listener bind above races the server's own bind; retry-connect
    // instead of a fixed sleep to avoid flakiness under load.
    format!("http://127.0.0.1:{port}")
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

#[tokio::test]
async fn grpc_message_send_and_task_get_round_trip() {
    let url = start_grpc_server();
    let mut client = connect(&url).await;

    let send_resp = client
        .message_send(SendMessageRequest {
            task_id: String::new(),
            message: Some(text_message("hello from the gRPC integration test")),
        })
        .await
        .expect("message_send should succeed")
        .into_inner();

    assert!(!send_resp.task_id.is_empty(), "task_id should be assigned");
    assert!(!send_resp.status.is_empty(), "status should be set");

    let get_resp = client
        .task_get(GetTaskRequest {
            task_id: send_resp.task_id.clone(),
        })
        .await
        .expect("task_get should succeed")
        .into_inner();

    assert_eq!(get_resp.task_id, send_resp.task_id);
}

#[tokio::test]
async fn grpc_task_get_missing_task_returns_not_found() {
    let url = start_grpc_server();
    let mut client = connect(&url).await;

    let err = client
        .task_get(GetTaskRequest {
            task_id: "does-not-exist".to_string(),
        })
        .await
        .expect_err("unknown task_id should error");

    assert_eq!(err.code(), tonic::Code::NotFound);
}
