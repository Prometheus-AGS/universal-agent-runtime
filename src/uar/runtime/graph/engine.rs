//! Graph execution engine.
//!
//! [`AgentGraph`] holds a set of [`GraphNode`]s connected by [`GraphEdge`]s
//! and drives iterative execution from the entry node until either a node
//! returns [`NodeResult::Finished`], an error occurs, or the graph has no
//! outgoing edge from the current node (natural termination).

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::types::{GraphContext, GraphEdge, GraphNode, GraphState, NodeResult};

/// Maximum iterations before the engine hard-stops to prevent infinite loops.
const MAX_GRAPH_ITERATIONS: u32 = 1_000;

// ── AgentGraph ────────────────────────────────────────────────────────────────

/// A directed graph of processing nodes that can be executed as a single unit.
pub struct AgentGraph {
    nodes: HashMap<String, Box<dyn GraphNode>>,
    edges: Vec<GraphEdge>,
    entry_node: String,
}

impl AgentGraph {
    /// Create a builder starting from `entry`.
    pub fn builder(entry: impl Into<String>) -> AgentGraphBuilder {
        AgentGraphBuilder {
            entry_node: entry.into(),
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Execute the graph from the entry node.
    ///
    /// Returns the final [`GraphState`] after execution completes (normally,
    /// via a `Finished` signal, or after an error).
    pub async fn execute(&self, initial_state: GraphState, ctx: &GraphContext) -> GraphState {
        let mut state = initial_state;
        let mut current_id = self.entry_node.clone();

        loop {
            state.iteration += 1;

            if state.iteration > MAX_GRAPH_ITERATIONS {
                error!(
                    run_id = %ctx.run_id,
                    iteration = state.iteration,
                    "Graph exceeded max iterations — terminating"
                );
                state.set("_error", "Max graph iterations exceeded");
                break;
            }

            let node = match self.nodes.get(&current_id) {
                Some(n) => n,
                None => {
                    error!(run_id = %ctx.run_id, node_id = %current_id, "Unknown graph node");
                    state.set("_error", format!("Unknown node: {current_id}"));
                    break;
                }
            };

            debug!(
                run_id = %ctx.run_id,
                node_id = %current_id,
                iteration = state.iteration,
                "Executing graph node"
            );

            state = match node.execute(state, ctx).await {
                NodeResult::Continue(s) => s,
                NodeResult::Finished(s) => {
                    info!(run_id = %ctx.run_id, node_id = %current_id, "Graph finished");
                    return s;
                }
                NodeResult::Error(mut s, msg) => {
                    error!(run_id = %ctx.run_id, node_id = %current_id, error = %msg, "Graph node error");
                    s.set("_error", &msg);
                    return s;
                }
            };

            match self.find_next(&current_id, &state) {
                Some(next_id) => {
                    debug!(run_id = %ctx.run_id, from = %current_id, to = %next_id, "Graph routing");
                    current_id = next_id;
                }
                None => {
                    info!(
                        run_id = %ctx.run_id,
                        node_id = %current_id,
                        "Graph terminated — no outgoing edge"
                    );
                    break;
                }
            }
        }

        state
    }

    /// Find the next node ID from `from`, consulting edge conditions.
    fn find_next(&self, from: &str, state: &GraphState) -> Option<String> {
        for edge in &self.edges {
            match edge {
                GraphEdge::Direct { from: f, to } if f.as_str() == from => {
                    return Some(to.clone());
                }
                GraphEdge::Conditional {
                    from: f,
                    condition,
                } if f.as_str() == from => {
                    return Some(condition(state));
                }
                _ => {}
            }
        }
        None
    }
}

impl std::fmt::Debug for AgentGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentGraph")
            .field("entry_node", &self.entry_node)
            .field("node_count", &self.nodes.len())
            .field("edge_count", &self.edges.len())
            .finish()
    }
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Fluent builder for [`AgentGraph`].
#[allow(missing_debug_implementations)]
pub struct AgentGraphBuilder {
    entry_node: String,
    nodes: HashMap<String, Box<dyn GraphNode>>,
    edges: Vec<GraphEdge>,
}

impl AgentGraphBuilder {
    /// Add a node to the graph.
    ///
    /// The node's [`GraphNode::id`] is used as the key.
    #[must_use]
    pub fn add_node(mut self, node: impl GraphNode + 'static) -> Self {
        let id = node.id().to_string();
        self.nodes.insert(id, Box::new(node));
        self
    }

    /// Add a direct (unconditional) edge from `from` to `to`.
    #[must_use]
    pub fn add_edge(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.edges.push(GraphEdge::Direct {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    /// Add a conditional edge from `from`.
    ///
    /// The `condition` closure is called with the current [`GraphState`] and
    /// must return the target node ID.
    #[must_use]
    pub fn add_conditional_edge(
        mut self,
        from: impl Into<String>,
        condition: impl Fn(&GraphState) -> String + Send + Sync + 'static,
    ) -> Self {
        self.edges.push(GraphEdge::Conditional {
            from: from.into(),
            condition: Arc::new(condition),
        });
        self
    }

    /// Finalise and return the [`AgentGraph`].
    #[must_use]
    pub fn build(self) -> AgentGraph {
        AgentGraph {
            nodes: self.nodes,
            edges: self.edges,
            entry_node: self.entry_node,
        }
    }
}
