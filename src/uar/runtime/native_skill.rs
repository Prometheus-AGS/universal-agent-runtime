//! Native Skill Trait System.
//!
//! Provides a mechanism for embedding high-performance Rust tool implementations
//! directly into the runtime binary, bypassing MCP serialization overhead for
//! critical hot-path operations.
//!
//! # Architecture
//!
//! Native skills implement the [`NativeSkill`] trait and are registered in the
//! [`NativeSkillRegistry`]. When the orchestrator receives a tool call, it first
//! checks whether a native skill is registered for that tool name. If found, the
//! native skill is executed directly; otherwise, the call falls through to MCP.
//!
//! # Example
//!
//! ```rust,ignore
//! use universal_agent_runtime::uar::runtime::native_skill::{NativeSkill, NativeSkillRegistry};
//!
//! struct MyTool;
//!
//! #[async_trait::async_trait]
//! impl NativeSkill for MyTool {
//!     fn name(&self) -> &str { "my_tool" }
//!     fn description(&self) -> &str { "Does something useful" }
//!     fn parameters_schema(&self) -> serde_json::Value {
//!         serde_json::json!({ "type": "object", "properties": {} })
//!     }
//!     async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
//!         Ok(serde_json::json!({ "result": "done" }))
//!     }
//! }
//!
//! let registry = NativeSkillRegistry::new();
//! registry.register(MyTool);
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::RwLock;

/// Trait for embedding high-performance Rust tool implementations directly
/// into the runtime binary.
///
/// Native skills bypass MCP serialization and execute in-process, making them
/// ideal for latency-sensitive operations like system introspection, caching
/// lookups, or internal state queries.
#[async_trait::async_trait]
pub trait NativeSkill: Send + Sync {
    /// Unique tool name. Must be globally unique across both native and MCP tools.
    fn name(&self) -> &str;

    /// Human-readable description of what the tool does.
    fn description(&self) -> &str;

    /// JSON Schema describing the expected input parameters.
    fn parameters_schema(&self) -> serde_json::Value;

    /// Execute the tool with the given arguments and return the result.
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value>;

    /// Format one successful result for model-visible history.
    ///
    /// The default preserves the existing JSON representation and applies the
    /// run's output policy at the single history-ingest boundary. Native tools
    /// whose raw output needs format-aware truncation may override this method.
    fn format_result(
        &self,
        result: &serde_json::Value,
        policy: crate::uar::runtime::context::truncate::TruncationPolicy,
        model: &str,
    ) -> String {
        let content = serde_json::to_string(result).unwrap_or_default();
        crate::uar::runtime::context::truncate::formatted_truncate_for_model(
            &content, policy, model,
        )
    }
}

/// Registry holding all registered native skills, keyed by their name.
///
/// Thread-safe for concurrent reads and writes via [`tokio::sync::RwLock`].
pub struct NativeSkillRegistry {
    skills: RwLock<HashMap<String, Arc<dyn NativeSkill>>>,
}

impl std::fmt::Debug for NativeSkillRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSkillRegistry")
            .field("skills", &"<locked>")
            .finish()
    }
}

impl Default for NativeSkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeSkillRegistry {
    /// Convert an internal native-skill identifier into the function-name
    /// alphabet accepted by OpenAI-compatible providers.
    fn provider_tool_name(name: &str) -> String {
        name.chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '_' | '-') {
                    character
                } else {
                    '_'
                }
            })
            .collect()
    }

    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            skills: RwLock::new(HashMap::new()),
        }
    }

    /// Register a native skill. Replaces any existing skill with the same name.
    pub async fn register<S: NativeSkill + 'static>(&self, skill: S) {
        let name = skill.name().to_owned();
        tracing::info!(skill_name = %name, "Registering native skill");
        self.skills.write().await.insert(name, Arc::new(skill));
    }

    /// Register a native skill from an `Arc`. Replaces any existing skill with the same name.
    pub async fn register_arc(&self, skill: Arc<dyn NativeSkill>) {
        let name = skill.name().to_owned();
        tracing::info!(skill_name = %name, "Registering native skill (Arc)");
        self.skills.write().await.insert(name, skill);
    }

    /// Look up a native skill by name.
    pub async fn get(&self, name: &str) -> Option<Arc<dyn NativeSkill>> {
        let skills = self.skills.read().await;
        skills.get(name).cloned().or_else(|| {
            skills
                .iter()
                .find(|(registered_name, _)| Self::provider_tool_name(registered_name) == name)
                .map(|(_, skill)| Arc::clone(skill))
        })
    }

    /// Check whether a native skill is registered for the given name.
    pub async fn contains(&self, name: &str) -> bool {
        let skills = self.skills.read().await;
        skills.contains_key(name)
            || skills
                .keys()
                .any(|registered_name| Self::provider_tool_name(registered_name) == name)
    }

    /// Return a snapshot of all registered skill names.
    pub async fn names(&self) -> Vec<String> {
        self.skills.read().await.keys().cloned().collect()
    }

    /// Return the number of registered native skills.
    pub async fn len(&self) -> usize {
        self.skills.read().await.len()
    }

    /// Return whether the registry is empty.
    pub async fn is_empty(&self) -> bool {
        self.skills.read().await.is_empty()
    }

    /// Clone a policy-filtered registry snapshot.
    ///
    /// `None` preserves every registered native skill. `Some` keeps only the
    /// explicitly eligible names, matching the MCP registry's filtering
    /// contract so scoped run policies govern both tool backends identically.
    pub async fn filtered(&self, allowed: Option<&HashSet<String>>) -> Self {
        let skills = self.skills.read().await;
        let filtered = skills
            .iter()
            .filter(|(name, _)| allowed.is_none_or(|names| names.contains(*name)))
            .map(|(name, skill)| (name.clone(), Arc::clone(skill)))
            .collect();
        Self {
            skills: RwLock::new(filtered),
        }
    }

    /// Generate OpenAI-compatible tool definitions for all registered native skills.
    ///
    /// This allows native skills to be announced to the LLM alongside MCP tools.
    pub async fn openai_tools_json(&self) -> Vec<serde_json::Value> {
        let skills = self.skills.read().await;
        let mut tools = skills
            .values()
            .map(|skill| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": Self::provider_tool_name(skill.name()),
                        "description": skill.description(),
                        "parameters": skill.parameters_schema()
                    }
                })
            })
            .collect::<Vec<_>>();
        tools.sort_by(|left, right| {
            left["function"]["name"]
                .as_str()
                .cmp(&right["function"]["name"].as_str())
        });
        tools
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    struct EchoSkill;

    #[async_trait::async_trait]
    impl NativeSkill for EchoSkill {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Returns the input unchanged"
        }
        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                }
            })
        }
        async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
            Ok(args)
        }
    }

    #[tokio::test]
    async fn test_register_and_get() {
        let registry = NativeSkillRegistry::new();
        registry.register(EchoSkill).await;

        assert!(registry.contains("echo").await);
        assert!(!registry.contains("nonexistent").await);
        assert_eq!(registry.len().await, 1);

        let skill = registry.get("echo").await.unwrap();
        assert_eq!(skill.name(), "echo");

        let result = skill
            .execute(serde_json::json!({"message": "hello"}))
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!({"message": "hello"}));
    }

    #[tokio::test]
    async fn test_names() {
        let registry = NativeSkillRegistry::new();
        registry.register(EchoSkill).await;

        let names = registry.names().await;
        assert_eq!(names, vec!["echo".to_string()]);
    }

    #[tokio::test]
    async fn test_openai_tools_json() {
        let registry = NativeSkillRegistry::new();
        registry.register(EchoSkill).await;

        let tools = registry.openai_tools_json().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "echo");
    }
}
