//! Built-in graph node implementations.
//!
//! All nodes implement [`crate::uar::runtime::graph::GraphNode`].
//!
//! | Node | Purpose |
//! |------|---------|
//! | [`LlmNode`] | Call the LLM driver and append the response |
//! | [`ToolNode`] | Execute a single MCP tool |
//! | [`RouterNode`] | Ask the LLM to choose a routing option |
//! | [`CheckpointNode`] | Record a named checkpoint in the execution flow |

pub mod agent_node;
pub mod checkpoint_node;
pub mod llm_node;
pub mod router_node;
pub mod tool_node;

pub use agent_node::AgentNode;
pub use checkpoint_node::CheckpointNode;
pub use llm_node::LlmNode;
pub use router_node::RouterNode;
pub use tool_node::ToolNode;
