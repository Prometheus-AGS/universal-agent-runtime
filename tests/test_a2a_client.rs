/// Integration tests for the A2A JSON-RPC 2.0 client.
///
/// Spins up a minimal Axum mock server that returns canned JSON-RPC responses,
/// then verifies A2AClient parses them correctly.

use std::net::TcpListener;
use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::post};
use serde_json::json;
use tokio::net::TcpListener as TokioListener;
use universal_agent_runtime::uar::api::a2a::{
    A2AClient,
    types::TaskState,
};

// ── Mock server helpers ───────────────────────────────────────────────────────

fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Start a minimal mock A2A server on a free port.
/// `response_factory` returns the JSON-RPC `result` value for any request.
async fn start_mock_server(result: serde_json::Value) -> String {
    let port = find_free_port();
    let result = Arc::new(result);

    let app = Router::new()
        .route("/", post(mock_rpc_handler))
        .with_state(result);

    let listener = TokioListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://127.0.0.1:{port}/")
}

async fn mock_rpc_handler(
    State(result): State<Arc<serde_json::Value>>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let id = req.get("id").cloned().unwrap_or(json!(1));
    Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": *result,
    }))
}

fn sample_task() -> serde_json::Value {
    json!({
        "id": "task-abc",
        "context_id": "ctx-1",
        "status": {
            "state": "completed",
            "timestamp": "2026-04-12T08:00:00Z"
        },
        "artifacts": [],
        "history": [],
        "metadata": {}
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_send_message_returns_task() {
    let url = start_mock_server(sample_task()).await;

    let client = A2AClient::new();
    let msg = universal_agent_runtime::uar::api::a2a::types::Message::user_text("hello");

    let task = client
        .send_message(&url, &msg)
        .await
        .expect("send_message should succeed");

    assert_eq!(task.id, "task-abc");
    assert!(matches!(task.status.state, TaskState::Completed));
}

#[tokio::test]
async fn test_get_task_returns_task() {
    let url = start_mock_server(sample_task()).await;

    let client = A2AClient::new();
    let task = client
        .get_task(&url, "task-abc")
        .await
        .expect("get_task should succeed");

    assert_eq!(task.id, "task-abc");
}

#[tokio::test]
async fn test_rpc_error_propagated() {
    let port = find_free_port();

    let app = Router::new().route(
        "/",
        post(|| async {
            Json(json!({
                "jsonrpc": "2.0",
                "id": "1",
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                }
            }))
        }),
    );

    let listener = TokioListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let url = format!("http://127.0.0.1:{port}/");
    let client = A2AClient::new();
    let msg = universal_agent_runtime::uar::api::a2a::types::Message::user_text("hello");

    let err = client.send_message(&url, &msg).await;
    assert!(err.is_err(), "should propagate JSON-RPC error");
    let msg = err.unwrap_err().to_string();
    assert!(msg.contains("-32601") || msg.contains("Method not found"), "error: {msg}");
}
