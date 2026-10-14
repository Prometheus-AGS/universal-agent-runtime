//! Host-observed outcomes. Losing a response after dispatch never proves no effect.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Execution certainty, independent of transport response delivery or receipt retention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolOutcome {
    Succeeded,
    ApplicationFailed,
    TransportFailed,
    OutcomeUnknown,
}

impl ToolOutcome {
    /// Only definitive success may satisfy a success dependency.
    #[must_use]
    pub const fn succeeded(self) -> bool {
        matches!(self, Self::Succeeded)
    }

    /// Classify the registry/preflight result at the trusted host boundary.
    /// Untyped errors arise before dispatch; dispatched loss is explicitly typed.
    #[must_use]
    pub fn from_mcp_result(result: &anyhow::Result<Value>) -> Self {
        match result {
            Ok(value) if value.get("isError").and_then(Value::as_bool) == Some(true) => {
                Self::ApplicationFailed
            }
            Ok(_) => Self::Succeeded,
            Err(error) => {
                McpToolFailure::find(error).map_or(Self::TransportFailed, |failure| failure.outcome)
            }
        }
    }
}

/// A failed MCP invocation with its complete SDK-represented response, when received.
/// Formatting deliberately excludes server content and transport diagnostics.
#[derive(Clone, PartialEq, Eq)]
pub struct McpToolFailure {
    pub outcome: ToolOutcome,
    pub result: Option<Value>,
}

impl std::fmt::Debug for McpToolFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpToolFailure")
            .field("outcome", &self.outcome)
            .field(
                "result",
                &self.result.as_ref().map(|_| "[host-held content]"),
            )
            .finish()
    }
}

impl std::fmt::Display for McpToolFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.outcome {
            ToolOutcome::ApplicationFailed => "MCP tool reported an application failure",
            ToolOutcome::OutcomeUnknown => "MCP response lost after dispatch; outcome unknown",
            ToolOutcome::TransportFailed => "MCP tool failed before dispatch",
            ToolOutcome::Succeeded => "MCP tool succeeded",
        })
    }
}

impl std::error::Error for McpToolFailure {}

impl McpToolFailure {
    /// Recover typed evidence through scoped runtime and anyhow source wrappers.
    #[must_use]
    pub fn find(error: &anyhow::Error) -> Option<&Self> {
        error
            .chain()
            .find_map(|source| source.downcast_ref::<Self>())
    }
}
