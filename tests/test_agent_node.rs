/// Integration test for AgentNode — verifies remote A2A delegation.
///
/// Spins up a minimal mock A2A server and verifies that an AgentNode with a
/// remote URL successfully delegates and stores the task result in state.

use std::net::TcpListener;
use std::sync::Arc;

use axum::{Json, Router, routing::post};
use serde_json::json;
use tokio::net::TcpListener as TokioListener;
use universal_agent_runtime::{
    llm::mock_driver::MockLlmDriver,
    mcp::registry::McpRegistry,
    normalized::NormalizedEvent,
    uar::runtime::graph::{
        AgentGraph, AgentNode, GraphContext, GraphState,
    },
};

fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Start a minimal mock A2A server that returns a completed task.
async fn start_a2a_mock() -> String {
    let port = find_free_port();

    let app = Router::new().route(
        "/",
        post(|| async {
            Json(json!({
                "jsonrpc": "2.0",
                "id": "1",
                "result": {
                    "id": "task-delegated",
                    "context_id": "ctx-test",
                    "status": {
                        "state": "completed",
                        "timestamp": "2026-04-12T08:00:00Z"
                    },
                    "artifacts": [],
                    "history": [],
                    "metadata": {}
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

    format!("http://127.0.0.1:{port}/")
}

fn make_ctx(run_id: &str) -> GraphContext {
    let driver = Arc::new(MockLlmDriver::new(vec![vec![NormalizedEvent::MessageDelta {
        text: "delegated".to_string(),
    }]]));

    GraphContext {
        run_id: run_id.to_string(),
        session_id: None,
        mcp: Arc::new(McpRegistry::new_empty()),
        llm_config: universal_agent_runtime::config::LlmConfig::default(),
        driver,
        persistence: None,
    }
}

#[tokio::test]
async fn test_agent_node_remote_delegation() {
    let url = start_a2a_mock().await;

    let graph = AgentGraph::builder("delegate")
        .add_node(AgentNode::new("delegate", &url))
        .build();

    let mut initial = GraphState::default();
    initial.set("_agent_input", "what is 2 + 2?".to_string());

    let ctx = make_ctx("run-agent-node-1");
    let final_state = graph.execute(initial, &ctx).await;

    // The result should be stored under _agent_result_delegate
    let result: serde_json::Value = final_state
        .get("_agent_result_delegate")
        .expect("_agent_result_delegate should be in state");

    assert_eq!(
        result.get("id").and_then(|v| v.as_str()),
        Some("task-delegated")
    );

    let task_id: String = final_state
        .get("_agent_task_id_delegate")
        .expect("task_id should be stored");
    assert_eq!(task_id, "task-delegated");
}

#[tokio::test]
async fn test_agent_node_error_on_unreachable_url() {
    // Port 1 is always refused — simulates network failure.
    let graph = AgentGraph::builder("bad")
        .add_node(AgentNode::new("bad", "http://127.0.0.1:1/"))
        .build();

    let ctx = make_ctx("run-agent-node-err");
    let final_state = graph.execute(GraphState::default(), &ctx).await;

    assert!(
        final_state.data.contains_key("_error"),
        "unreachable agent should set _error in state"
    );
}

#[tokio::test]
async fn test_agent_node_falls_back_to_last_message() {
    let url = start_a2a_mock().await;

    let graph = AgentGraph::builder("delegate")
        .add_node(AgentNode::new("delegate", &url))
        .build();

    // No _agent_input — should use the last message from state.messages
    let mut initial = GraphState::default();
    initial.messages.push(json!({"role": "user", "content": "from messages"}));

    let ctx = make_ctx("run-agent-node-2");
    let final_state = graph.execute(initial, &ctx).await;

    // Task should still be returned (mock ignores input content)
    assert!(final_state.data.contains_key("_agent_result_delegate"));
    assert!(!final_state.data.contains_key("_error"));
}
