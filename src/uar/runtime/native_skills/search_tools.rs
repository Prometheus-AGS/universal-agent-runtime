//! Model-only discovery over the current host-authorized MCP descriptor set.

use serde_json::{Value, json};

use crate::mcp::exposure::{MCP_SEARCH_QUERY_LIMIT, McpToolExposure};
use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::tools::descriptor::{Exposure, ToolEffect, ToolSource};

/// Reserved host control, installed only in a chat-local native registry.
pub const SEARCH_TOOLS_NAME: &str = "search_tools";

/// Search does not connect servers, mutate external systems or bypass policy.
#[derive(Debug)]
pub struct SearchToolsTool {
    exposure: McpToolExposure,
    thread_policy:
        Option<std::sync::Arc<crate::uar::runtime::thread::policy_intersection::ThreadPolicy>>,
}

impl SearchToolsTool {
    /// Bind the handler to this stream's discovery state, not a global catalog.
    pub fn new(exposure: McpToolExposure) -> Self {
        Self {
            exposure,
            thread_policy: None,
        }
    }

    /// Capture this stream's host policy along with its discovery state.
    pub(crate) fn with_thread_policy(
        mut self,
        policy: Option<
            std::sync::Arc<crate::uar::runtime::thread::policy_intersection::ThreadPolicy>,
        >,
    ) -> Self {
        self.thread_policy = policy;
        self
    }
}

#[async_trait::async_trait]
impl NativeSkill for SearchToolsTool {
    fn name(&self) -> &str {
        SEARCH_TOOLS_NAME
    }

    fn description(&self) -> &str {
        "Search eligible deferred MCP tools by name or description. Matching tools become callable on the next model step, not in this batch. Search does not start servers or grant permissions."
    }

    fn parameters_schema(&self) -> Value {
        json!({"type": "object", "properties": {
            "query": {"type": "string", "minLength": 1, "maxLength": MCP_SEARCH_QUERY_LIMIT}
        }, "required": ["query"], "additionalProperties": false})
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }
    fn concurrency_key(&self) -> Option<&str> {
        Some("mcp_tool_discovery")
    }
    fn exposure(&self) -> Exposure {
        Exposure::ModelOnly
    }
    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }

    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.thread_policy
                .as_ref()
                .is_some_and(|bound| std::ptr::eq(bound.as_ref(), policy)),
            "Tool discovery is not bound to this delegated turn"
        );
        Ok(())
    }

    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let query = args
            .get("query")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("query must be a string"))?;
        let matches = self.exposure.search(query)?;
        Ok(
            json!({"status": "selected_for_next_step", "tools": matches.iter()
            .map(|tool| tool.openai_tool_json()).collect::<Vec<_>>() }),
        )
    }
}
