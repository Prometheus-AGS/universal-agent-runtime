//! Private A2A extension for caller-owned function declarations and results.
//!
//! The extension carries data only. It never registers a host executable or
//! changes the peer's delegation, approval, or tool policy.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Versioned extension identifier, also used as the metadata member name.
pub const CLIENT_TOOLS_EXTENSION: &str =
    "https://prometheus.dev/a2a/extensions/client-tools/v1";

/// Wire payload shared by JSON-RPC and gRPC after A2A normalization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientToolsExtension {
    #[serde(default)]
    pub tools: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<crate::llm::ToolChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Native assistant tool calls and tool results, correlated by call ID.
    #[serde(default)]
    pub history: Vec<Value>,
}

impl ClientToolsExtension {
    pub fn parse(
        metadata: &std::collections::HashMap<String, Value>,
    ) -> Result<Option<Self>, &'static str> {
        metadata.get(CLIENT_TOOLS_EXTENSION).map(|value| {
            serde_json::from_value(value.clone())
                .map_err(|_| "Caller-tools extension is malformed")
        }).transpose()
    }
}

impl ClientToolsExtension {
    pub fn into_input(self) -> crate::uar::runtime::turn::client_tools::ClientToolInput {
        crate::uar::runtime::turn::client_tools::ClientToolInput {
            tools: self.tools, tool_choice: self.tool_choice,
            parallel_tool_calls: self.parallel_tool_calls, history: self.history,
            visible_history: None, graph_state: None, budget_snapshot: None, delegation: None, output_token_cap: None, protocol: "a2a".to_owned(),
        }
    }
}
