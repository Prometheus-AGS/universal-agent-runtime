/// Integration tests for the AgentNode host-capability boundary.
///
/// Positive local and remote delegation run through `RunManager`, which owns
/// the otherwise-unconstructible thread capability. These direct graph tests
/// prove callers cannot bypass that trusted host with a driver or endpoint.
use std::sync::Arc;

use serde_json::json;
use universal_agent_runtime::{
    llm::mock_driver::MockLlmDriver,
    normalized::NormalizedEvent,
    uar::runtime::graph::{AgentGraph, AgentNode, GraphContext, GraphState},
};

fn make_ctx(run_id: &str) -> GraphContext {
    make_ctx_with_events(
        run_id,
        vec![NormalizedEvent::MessageDelta {
            text: "delegated".to_string(),
        }],
    )
}

fn make_ctx_with_events(run_id: &str, events: Vec<NormalizedEvent>) -> GraphContext {
    let driver = Arc::new(MockLlmDriver::new(vec![events]));

    GraphContext {
        run_id: run_id.to_string(),
        session_id: None,
        llm_config: universal_agent_runtime::config::LlmConfig::default(),
        driver,
        cache_strategy: None,
        persistence: None,
        thread_delegate: None,
        tool_host: None,
    }
}

#[tokio::test]
async fn test_agent_node_local_requires_host_thread_service() {
    let graph = AgentGraph::builder("rust-reviewer")
        .add_node(AgentNode::new("rust-reviewer", "rust-reviewer"))
        .build();
    let ctx = make_ctx_with_events("run-agent-node-empty", vec![NormalizedEvent::Done]);

    let final_state = graph.execute(GraphState::default(), &ctx).await;

    assert_eq!(
        final_state.get::<String>("_error").as_deref(),
        Some("Graph child execution requires a host thread service")
    );
    assert!(!final_state.data.contains_key("_agent_output_rust-reviewer"));
}

#[tokio::test]
async fn test_agent_node_local_does_not_fall_back_to_graph_driver() {
    let graph = AgentGraph::builder("rust-reviewer")
        .add_node(AgentNode::new("rust-reviewer", "rust-reviewer"))
        .build();
    let mut initial = GraphState::default();
    initial.set("_agent_input", "review this Rust boundary".to_string());

    let final_state = graph
        .execute(initial, &make_ctx("run-agent-node-local"))
        .await;

    assert_eq!(
        final_state.get::<String>("_error").as_deref(),
        Some("Graph child execution requires a host thread service")
    );
    assert!(!final_state.data.contains_key("_agent_output_rust-reviewer"));
}

#[tokio::test]
async fn test_agent_node_remote_requires_host_thread_service() {
    let graph = AgentGraph::builder("delegate")
        .add_node(AgentNode::new("delegate", "http://127.0.0.1:1/"))
        .build();

    let mut initial = GraphState::default();
    initial.set("_agent_input", "what is 2 + 2?".to_string());

    let ctx = make_ctx("run-agent-node-1");
    let final_state = graph.execute(initial, &ctx).await;

    assert_eq!(
        final_state.get::<String>("_error").as_deref(),
        Some("Remote graph child execution requires a host thread service")
    );
    assert!(!final_state.data.contains_key("_agent_result_delegate"));
    assert!(!final_state.data.contains_key("_agent_thread_id_delegate"));
    assert!(!final_state.data.contains_key("_agent_output_delegate"));
}

#[tokio::test]
async fn test_agent_node_remote_endpoint_does_not_bypass_host() {
    let graph = AgentGraph::builder("bad")
        .add_node(AgentNode::new("bad", "http://127.0.0.1:1/"))
        .build();

    let ctx = make_ctx("run-agent-node-err");
    let final_state = graph.execute(GraphState::default(), &ctx).await;

    assert_eq!(
        final_state.get::<String>("_error").as_deref(),
        Some("Remote graph child execution requires a host thread service")
    );
}

#[tokio::test]
async fn test_agent_node_message_fallback_does_not_bypass_host() {
    let graph = AgentGraph::builder("delegate")
        .add_node(AgentNode::new("delegate", "http://127.0.0.1:1/"))
        .build();

    // No _agent_input — should use the last message from state.messages
    let mut initial = GraphState::default();
    initial
        .messages
        .push(json!({"role": "user", "content": "from messages"}));

    let ctx = make_ctx("run-agent-node-2");
    let final_state = graph.execute(initial, &ctx).await;

    assert_eq!(
        final_state.get::<String>("_error").as_deref(),
        Some("Remote graph child execution requires a host thread service")
    );
    assert!(!final_state.data.contains_key("_agent_result_delegate"));
}
