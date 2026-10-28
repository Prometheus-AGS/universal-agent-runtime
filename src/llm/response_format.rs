//! Request-scoped provider output constraints. These are never prompt instructions.

use serde::{Deserialize, Serialize};

/// OpenAI-compatible output format, matching the pinned Liter wire schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema { json_schema: JsonSchemaFormat },
}

/// Named schema passed unchanged to the model's constrained output API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonSchemaFormat {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub schema: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl ResponseFormat {
    pub(crate) fn is_structured(&self) -> bool { !matches!(self, Self::Text) }

    /// Reject invalid shapes and remote schema retrieval at the public boundary.
    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        if let Self::JsonSchema { json_schema } = self {
            anyhow::ensure!(!json_schema.name.is_empty() && json_schema.name.len() <= 64
                && json_schema.name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')),
                "response_format schema name must contain 1–64 ASCII letters, digits, underscores or hyphens");
            anyhow::ensure!(json_schema.schema.is_object(), "response_format schema must be an object");
            fn local_references(value: &serde_json::Value) -> bool {
                match value {
                    serde_json::Value::Object(object) => object.iter().all(|(key, value)| {
                        if matches!(key.as_str(), "$ref" | "$dynamicRef" | "$recursiveRef") {
                            value.as_str().is_some_and(|reference| reference.starts_with('#'))
                        } else { local_references(value) }
                    }),
                    serde_json::Value::Array(array) => array.iter().all(local_references),
                    _ => true,
                }
            }
            anyhow::ensure!(local_references(&json_schema.schema), "response_format external schema references are unsupported");
            jsonschema::validator_for(&json_schema.schema)
                .map_err(|_| anyhow::anyhow!("response_format schema is invalid or unsupported"))?;
        }
        Ok(())
    }

    /// Check the final projected payload before reporting successful completion.
    pub(crate) fn validate_output(&self, output: &str) -> anyhow::Result<()> {
        if !self.is_structured() { return Ok(()); }
        let value: serde_json::Value = serde_json::from_str(output)
            .map_err(|_| anyhow::anyhow!("Provider did not return valid structured JSON"))?;
        match self {
            Self::JsonObject => anyhow::ensure!(value.is_object(), "Provider did not return a JSON object"),
            Self::JsonSchema { json_schema } => {
                let validator = jsonschema::validator_for(&json_schema.schema)
                    .map_err(|_| anyhow::anyhow!("response_format schema is invalid or unsupported"))?;
                anyhow::ensure!(validator.is_valid(&value), "Provider output does not satisfy response_format schema");
            }
            Self::Text => {}
        }
        Ok(())
    }
}
