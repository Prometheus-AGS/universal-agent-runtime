//! Echo native skill — returns input arguments unchanged.
//!
//! Primarily useful for testing the native skill pipeline and
//! verifying that arguments are correctly round-tripped.

use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::tools::descriptor::{ToolEffect, ToolSource};

/// A simple echo skill that returns its input unchanged.
#[derive(Debug)]
pub struct EchoSkill;

#[async_trait::async_trait]
impl NativeSkill for EchoSkill {
    fn name(&self) -> &str {
        "native_echo"
    }

    fn description(&self) -> &str {
        "Returns the input arguments unchanged. Useful for testing."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo back"
                }
            },
            "required": ["message"]
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        tracing::debug!(args = %args, "Executing native echo skill");
        Ok(serde_json::json!({
            "echoed": args
        }))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[tokio::test]
    async fn test_echo_returns_input() {
        let skill = EchoSkill;
        let input = serde_json::json!({"message": "hello world"});
        let result = skill.execute(input.clone()).await.unwrap();
        assert_eq!(result["echoed"], input);
    }

    #[test]
    fn test_echo_metadata() {
        let skill = EchoSkill;
        assert_eq!(skill.name(), "native_echo");
        assert!(!skill.description().is_empty());
        assert!(skill.parameters_schema()["properties"]["message"].is_object());
    }
}
