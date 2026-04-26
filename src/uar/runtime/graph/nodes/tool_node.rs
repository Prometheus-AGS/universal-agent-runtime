//! Tool graph node — executes a single MCP tool and stores the result.

use async_trait::async_trait;
use tracing::debug;

use crate::uar::runtime::graph::types::{GraphContext, GraphNode, GraphState, NodeResult};

/// A graph node that calls a named MCP tool and stores the result in state.
///
/// # State keys
/// - **input**: `args_key` (default `"_tool_args"`) — JSON object with tool arguments.
/// - **output**: `output_key` (default `"_tool_result"`) — raw JSON string result.
pub struct ToolNode {
    id: String,
    /// Fully-qualified MCP tool name (e.g. `"tavily::search"`).
    tool_name: String,
    /// State key holding the tool arguments (a JSON object).
    args_key: String,
    /// State key where the result is written.
    output_key: String,
}

impl ToolNode {
    /// Create a tool node with the given ID and tool name.
    #[must_use]
    pub fn new(id: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            tool_name: tool_name.into(),
            args_key: "_tool_args".to_string(),
            output_key: "_tool_result".to_string(),
        }
    }

    /// Override the state key used to read arguments from.
    #[must_use]
    pub fn with_args_key(mut self, key: impl Into<String>) -> Self {
        self.args_key = key.into();
        self
    }

    /// Override the state key used to write the result to.
    #[must_use]
    pub fn with_output_key(mut self, key: impl Into<String>) -> Self {
        self.output_key = key.into();
        self
    }
}

impl std::fmt::Debug for ToolNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolNode")
            .field("id", &self.id)
            .field("tool_name", &self.tool_name)
            .finish()
    }
}

#[async_trait]
impl GraphNode for ToolNode {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&self, mut state: GraphState, ctx: &GraphContext) -> NodeResult {
        debug!(
            node_id = %self.id,
            tool = %self.tool_name,
            run_id = %ctx.run_id,
            "ToolNode executing"
        );

        // Read arguments from state (default to empty object if absent)
        let args: serde_json::Value = state
            .get::<serde_json::Value>(&self.args_key)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        match ctx.mcp.call_namespaced_tool(&self.tool_name, args).await {
            Ok(result) => {
                let content = serde_json::to_string(&result).unwrap_or_default();
                state.set(&self.output_key, &content);
                NodeResult::Continue(state)
            }
            Err(e) => {
                NodeResult::Error(state, format!("ToolNode '{}' failed: {e}", self.tool_name))
            }
        }
    }
}
