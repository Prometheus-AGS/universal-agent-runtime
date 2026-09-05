//! Graph-based agent orchestration.
//!
//! Provides the types and engine for composing LLM calls, tool executions,
//! routing nodes, and checkpoints into a directed processing graph.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use universal_agent_runtime::uar::runtime::graph::{AgentGraph, GraphState};
//! use universal_agent_runtime::uar::runtime::graph::nodes::LlmNode;
//!
//! let graph = AgentGraph::builder("llm")
//!     .add_node(LlmNode::new("llm"))
//!     .build();
//!
//! let state = graph.execute(GraphState::default(), &ctx).await;
//! ```

pub mod delegation;
pub mod engine;
pub mod nodes;
pub mod tools;
pub(crate) mod turn;
pub mod types;

pub use engine::{AgentGraph, AgentGraphBuilder};
pub use nodes::{AgentNode, CheckpointNode, LlmNode, RouterNode, ToolNode};
pub use types::{GraphContext, GraphEdge, GraphNode, GraphState, NodeResult};
