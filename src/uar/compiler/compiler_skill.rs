//! Single-shot compiler skill — a [`NativeSkill`] that compiles a complete
//! UAR-AGENT-MD document in one call.
//!
//! # Usage
//!
//! Tool name: `uar.compile`
//!
//! ```json
//! {
//!   "markdown": "# Agent: My Agent\n\n## Metadata\n```yaml\n..."
//! }
//! ```
//!
//! Returns the full [`CompileOutput`] as JSON on success.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::uar::compiler::error::CompileError;
use crate::uar::compiler::parser;
use crate::uar::compiler::pipeline;
use crate::uar::compiler::registries::{InMemoryEndpointRegistry, InMemorySchemaRegistry};
use crate::uar::compiler::signing::KeyProvider;
use crate::uar::runtime::native_skill::NativeSkill;

/// A [`NativeSkill`] that compiles a complete UAR-AGENT-MD Markdown document
/// through the 8-stage pipeline and returns a signed descriptor.
#[derive(Debug)]
pub struct CompilerAgentSkill {
    key_provider: Arc<dyn KeyProvider>,
}

impl CompilerAgentSkill {
    /// Create a new compiler skill with the given key provider.
    pub fn new(key_provider: Arc<dyn KeyProvider>) -> Self {
        Self { key_provider }
    }
}

#[async_trait::async_trait]
impl NativeSkill for CompilerAgentSkill {
    fn name(&self) -> &str {
        "uar.compile"
    }

    fn description(&self) -> &str {
        "Compile a complete UAR-AGENT-MD Markdown document into a signed agent descriptor. \
         Pass the full Markdown content in the 'markdown' field."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["markdown"],
            "properties": {
                "markdown": {
                    "type": "string",
                    "description": "The complete UAR-AGENT-MD Markdown document to compile"
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let markdown = args
            .get("markdown")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required 'markdown' parameter"))?;

        // Parse the document
        let ir = parser::parse(markdown)
            .map_err(|e: CompileError| anyhow::anyhow!("parse failed: {e}"))?;

        // Create default registries (in-memory, per-compilation)
        let schema_registry = Arc::new(InMemorySchemaRegistry::new());
        let endpoint_registry = Arc::new(InMemoryEndpointRegistry::new());

        // Run the 8-stage pipeline
        let output = pipeline::compile(
            ir,
            schema_registry,
            endpoint_registry,
            Arc::clone(&self.key_provider),
        )
        .await
        .map_err(|e: CompileError| anyhow::anyhow!("compile failed: {e}"))?;

        // Serialize the output
        let result = serde_json::to_value(&output)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::compiler::signing::LocalKeyProvider;

    fn sample_doc() -> String {
        r#"# Agent: TestBot

## Metadata
```yaml
version: "1.0.0"
```

## Identity
```yaml
name: TestBot
role: assistant
persona: A helpful test assistant
```

## UI
```yaml
forms: []
artifacts: []
actions: []
```

## Capabilities
```yaml
streaming: false
file_upload: false
image_generation: false
code_execution: false
web_browsing: false
```

## Skills
```yaml
skills: []
```

## Tools
```yaml
tools: []
allow: []
deny: []
```

## MCP Servers
```yaml
servers: []
```

## Knowledge Base
```yaml
sources: []
```

## Memory Model
```yaml
conversation:
  max_turns: 50
```

## A2A Contracts
```yaml
endpoints: []
dependencies: []
```

## Governance
```yaml
cedar_policies: []
audit: {}
```

## Budgets
```yaml
{}
```

## Execution Model
```yaml
{}
```

## Observability
```yaml
tracing: {}
metrics: {}
logging: {}
```

## Deployment Profiles
```yaml
profiles: []
```
"#
        .to_string()
    }

    #[tokio::test]
    async fn test_compiler_skill_end_to_end() {
        let kp = Arc::new(LocalKeyProvider::ephemeral());
        let skill = CompilerAgentSkill::new(kp);

        assert_eq!(skill.name(), "uar.compile");

        let result = skill
            .execute(json!({ "markdown": sample_doc() }))
            .await
            .unwrap();

        assert!(result.get("descriptor").is_some());
        assert!(result.get("signature").is_some());
        assert!(result.get("report").is_some());

        let descriptor = &result["descriptor"];
        assert_eq!(descriptor["agent_id"], "TestBot");
        assert_eq!(descriptor["version"], "1.0.0");
        assert_eq!(descriptor["schema"], "uar-agent-descriptor/v1");
    }

    #[tokio::test]
    async fn test_compiler_skill_rejects_incomplete() {
        let kp = Arc::new(LocalKeyProvider::ephemeral());
        let skill = CompilerAgentSkill::new(kp);

        let result = skill
            .execute(json!({ "markdown": "# Agent: Incomplete\n" }))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("incomplete") || err.contains("missing"));
    }

    #[tokio::test]
    async fn test_compiler_skill_missing_param() {
        let kp = Arc::new(LocalKeyProvider::ephemeral());
        let skill = CompilerAgentSkill::new(kp);

        let result = skill.execute(json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("markdown"));
    }
}
