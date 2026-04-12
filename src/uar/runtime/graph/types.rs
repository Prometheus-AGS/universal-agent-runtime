//! Core types for graph-based agent orchestration.
//!
//! The graph model lets you compose LLM calls, tool invocations, routing
//! decisions, and checkpoints into a directed acyclic (or cyclic) processing
//! graph.  Each node receives the shared [`GraphState`] and a read-only
//! [`GraphContext`] and may transform the state before signalling how
//! execution should continue.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ── State ────────────────────────────────────────────────────────────────────

/// Shared mutable state that flows through every node in the graph.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphState {
    /// Typed data bag for arbitrary JSON values.
    pub data: HashMap<String, serde_json::Value>,
    /// Conversation messages accumulated during execution.
    pub messages: Vec<serde_json::Value>,
    /// Number of node executions completed so far (incremented by the engine).
    pub iteration: u32,
}

impl GraphState {
    /// Read a typed value from the data bag.
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Write a typed value to the data bag.
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) {
        if let Ok(v) = serde_json::to_value(value) {
            self.data.insert(key.to_string(), v);
        }
    }
}

// ── Context ───────────────────────────────────────────────────────────────────

/// Immutable execution context passed to every node.
///
/// Provides access to the LLM driver, MCP registry, and per-run metadata.
#[allow(missing_debug_implementations)]
pub struct GraphContext {
    /// Unique ID for this run.
    pub run_id: String,
    /// Optional session / thread ID.
    pub session_id: Option<String>,
    /// MCP tool registry.
    pub mcp: Arc<crate::mcp::registry::McpRegistry>,
    /// LLM configuration (model, provider, etc.).
    pub llm_config: crate::config::LlmConfig,
    /// Shared LLM driver for making inference calls.
    pub driver: Arc<dyn crate::llm::LlmDriver>,
}

// ── Node result ────────────────────────────────────────────────────────────────

/// Result returned by a [`GraphNode`] after execution.
#[derive(Debug)]
pub enum NodeResult {
    /// Continue graph execution with the (possibly modified) state.
    Continue(GraphState),
    /// Execution is complete; return the final state immediately.
    Finished(GraphState),
    /// An error occurred; state is preserved for inspection.
    Error(GraphState, String),
}

// ── Node trait ────────────────────────────────────────────────────────────────

/// A processing node in the agent graph.
///
/// Nodes receive the current [`GraphState`] by value, transform it, and
/// return a [`NodeResult`] indicating how the engine should proceed.
#[async_trait]
pub trait GraphNode: Send + Sync {
    /// Return the unique node identifier used for routing.
    fn id(&self) -> &str;
    /// Execute the node with the given state and context.
    async fn execute(&self, state: GraphState, ctx: &GraphContext) -> NodeResult;
}

// ── Edge ──────────────────────────────────────────────────────────────────────

/// A directed edge between two nodes in the graph.
pub enum GraphEdge {
    /// Always route from `from` to `to`.
    Direct { from: String, to: String },
    /// Route conditionally based on the current graph state.
    ///
    /// The condition closure receives the current state and returns the
    /// target node ID.
    Conditional {
        from: String,
        condition: Arc<dyn Fn(&GraphState) -> String + Send + Sync>,
    },
}

impl std::fmt::Debug for GraphEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct { from, to } => f
                .debug_struct("Direct")
                .field("from", from)
                .field("to", to)
                .finish(),
            Self::Conditional { from, .. } => f
                .debug_struct("Conditional")
                .field("from", from)
                .field("condition", &"<fn>")
                .finish(),
        }
    }
}
