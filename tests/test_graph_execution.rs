/// Integration tests for graph-based agent orchestration.
///
/// Verifies that `AgentGraph` correctly routes state through nodes and
/// terminates with the expected final state.  Uses `MockLlmDriver` for
/// deterministic LLM responses so no network is required.
use std::sync::Arc;

use universal_agent_runtime::{
    llm::{anthropic_cache::CacheStrategy, mock_driver::MockLlmDriver},
    mcp::registry::McpRegistry,
    normalized::NormalizedEvent,
    uar::runtime::graph::{
        AgentGraph, GraphContext, GraphNode, GraphState, LlmNode, NodeResult, RouterNode,
    },
};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// A simple "finish" node that sets a marker in state and signals completion.
struct FinishNode {
    id: String,
    marker: String,
}

#[async_trait::async_trait]
impl GraphNode for FinishNode {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&self, mut state: GraphState, _ctx: &GraphContext) -> NodeResult {
        state.set("finished_by", self.marker.clone());
        NodeResult::Finished(state)
    }
}

/// A passthrough node that writes to state and continues.
struct PassthroughNode {
    id: String,
    key: String,
    value: String,
}

#[async_trait::async_trait]
impl GraphNode for PassthroughNode {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&self, mut state: GraphState, _ctx: &GraphContext) -> NodeResult {
        state.set(self.key.as_str(), self.value.clone());
        NodeResult::Continue(state)
    }
}

/// Build a minimal `GraphContext` backed by `MockLlmDriver`.
fn make_ctx(run_id: &str) -> GraphContext {
    let driver = Arc::new(MockLlmDriver::new(vec![
        // One canned response: empty assistant message
        vec![NormalizedEvent::MessageDelta {
            text: "hello".to_string(),
        }],
    ]));

    GraphContext {
        run_id: run_id.to_string(),
        session_id: None,
        mcp: Arc::new(McpRegistry::new_empty()),
        llm_config: universal_agent_runtime::config::LlmConfig::default(),
        driver,
        cache_strategy: None,
        persistence: None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_simple_two_node_graph() {
    // Graph: passthrough → finish
    let graph = AgentGraph::builder("step1")
        .add_node(PassthroughNode {
            id: "step1".to_string(),
            key: "step1_ran".to_string(),
            value: "yes".to_string(),
        })
        .add_node(FinishNode {
            id: "finish".to_string(),
            marker: "finish_node".to_string(),
        })
        .add_edge("step1", "finish")
        .build();

    let ctx = make_ctx("run-test-1");
    let state = graph.execute(GraphState::default(), &ctx).await;

    assert_eq!(state.get::<String>("step1_ran").as_deref(), Some("yes"));
    assert_eq!(
        state.get::<String>("finished_by").as_deref(),
        Some("finish_node")
    );
}

#[tokio::test]
async fn policy_bearing_graph_llm_requests_inherit_cache_strategy() {
    let driver = Arc::new(MockLlmDriver::new(vec![
        vec![
            NormalizedEvent::MessageDelta {
                text: "answer".to_string(),
            },
            NormalizedEvent::Done,
        ],
        vec![
            NormalizedEvent::MessageDelta {
                text: "route-a".to_string(),
            },
            NormalizedEvent::Done,
        ],
    ]));
    let ctx = GraphContext {
        run_id: "run-cache-policy".to_string(),
        session_id: Some("session-cache-policy".to_string()),
        mcp: Arc::new(McpRegistry::new_empty()),
        llm_config: universal_agent_runtime::config::LlmConfig::default(),
        driver: driver.clone(),
        cache_strategy: Some(CacheStrategy::default()),
        persistence: None,
    };
    let mut llm_state = GraphState::default();
    llm_state
        .messages
        .push(serde_json::json!({"role": "user", "content": "hello"}));
    let _ = LlmNode::new("llm").execute(llm_state, &ctx).await;

    let mut router_state = GraphState::default();
    router_state.set("_agent_input", "choose");
    let _ = RouterNode::new(
        "router",
        "Choose a route",
        vec!["route-a".to_string(), "route-b".to_string()],
    )
    .execute(router_state, &ctx)
    .await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 2);
    assert!(
        requests
            .iter()
            .all(|request| request.cache_strategy.is_some())
    );
}

#[tokio::test]
async fn test_state_flows_through_multiple_nodes() {
    let graph = AgentGraph::builder("a")
        .add_node(PassthroughNode {
            id: "a".to_string(),
            key: "a".to_string(),
            value: "1".to_string(),
        })
        .add_node(PassthroughNode {
            id: "b".to_string(),
            key: "b".to_string(),
            value: "2".to_string(),
        })
        .add_node(FinishNode {
            id: "c".to_string(),
            marker: "done".to_string(),
        })
        .add_edge("a", "b")
        .add_edge("b", "c")
        .build();

    let ctx = make_ctx("run-test-2");
    let state = graph.execute(GraphState::default(), &ctx).await;

    assert_eq!(state.get::<String>("a").as_deref(), Some("1"));
    assert_eq!(state.get::<String>("b").as_deref(), Some("2"));
    assert_eq!(state.get::<String>("finished_by").as_deref(), Some("done"));
    assert_eq!(state.iteration, 3, "three nodes should have run");
    assert_eq!(
        state.get::<Vec<String>>("_graph_trace"),
        Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
}

#[tokio::test]
async fn test_conditional_edge_routing() {
    // Graph: router → (if flag=="a" → node_a, else → node_b)
    let graph = AgentGraph::builder("router")
        .add_node(PassthroughNode {
            id: "router".to_string(),
            key: "flag".to_string(),
            value: "a".to_string(),
        })
        .add_node(FinishNode {
            id: "node_a".to_string(),
            marker: "chosen_a".to_string(),
        })
        .add_node(FinishNode {
            id: "node_b".to_string(),
            marker: "chosen_b".to_string(),
        })
        .add_conditional_edge("router", |state: &GraphState| {
            if state.get::<String>("flag").as_deref() == Some("a") {
                "node_a".to_string()
            } else {
                "node_b".to_string()
            }
        })
        .build();

    let ctx = make_ctx("run-test-3");
    let state = graph.execute(GraphState::default(), &ctx).await;

    assert_eq!(
        state.get::<String>("finished_by").as_deref(),
        Some("chosen_a"),
        "conditional edge should have routed to node_a"
    );
}

#[tokio::test]
async fn test_graph_terminates_naturally_without_finish_node() {
    // A single passthrough with no outgoing edge — natural termination.
    let graph = AgentGraph::builder("only")
        .add_node(PassthroughNode {
            id: "only".to_string(),
            key: "ran".to_string(),
            value: "true".to_string(),
        })
        .build();

    let ctx = make_ctx("run-test-4");
    let state = graph.execute(GraphState::default(), &ctx).await;

    assert_eq!(state.get::<String>("ran").as_deref(), Some("true"));
    // No _error key — graph terminated cleanly.
    assert!(!state.data.contains_key("_error"));
}
