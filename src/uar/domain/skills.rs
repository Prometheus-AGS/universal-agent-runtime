use serde::{Deserialize, Serialize};

/// Execution model of a Skill. Determines which runtime is used to invoke it.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillKind {
    /// In-process Rust / native code path. Default for legacy rows.
    #[default]
    Native,
    /// SKILL.md manifest (YAML frontmatter + Markdown) used as prompt-context.
    Manifest,
    /// WebAssembly Component Model artifact loaded via wasmtime::component.
    Wasm,
}

/// Provenance flag — system-shipped skills are immutable; user-defined ones can be edited/deleted.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillOrigin {
    /// Loaded at startup from the embedded `prometheus-skill-system` submodule
    /// or another system-managed location. Cannot be deleted via the API.
    Builtin,
    /// Created by a user via the Skills admin UI or REST API.
    #[default]
    User,
}

/// Per-skill LLM execution configuration.
///
/// When a skill is matched and activated, these settings override the global
/// LLM configuration for the duration of the skill's execution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillExecutionConfig {
    /// Override the LLM provider (e.g. `"anthropic"`, `"openai"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_provider: Option<String>,
    /// Override the model ID (e.g. `"claude-opus-4-6"`, `"gpt-4o"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_model: Option<String>,
    /// Override the maximum output token budget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
}

/// Scope at which a skill enabled-state override applies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum SkillScope {
    /// Runtime-wide state. [`Skill::enabled`] remains the compatibility copy of
    /// this value for rows written before scoped configuration existed.
    Global,
    /// State for one agent.
    Agent(String),
    /// State for one conversation.
    Conversation(String),
}

/// Durable enabled-state override for one skill scope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopedSkillConfig {
    pub scope: SkillScope,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(clippy::struct_field_names)]
pub struct Skill {
    pub skill_id: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub triggers: SkillTriggers,
    pub prompt_overlay: String,
    #[serde(default)]
    pub preferred_tools: Vec<String>,
    #[serde(default, skip)]
    pub mcp_config: Option<crate::mcp::config::McpConfig>,
    #[serde(default)]
    pub constraints: SkillConstraints,
    /// Whether this skill is globally enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Durable global, agent, and conversation enabled-state records.
    ///
    /// Legacy rows without this field use [`Self::enabled`] as their global
    /// value. New global writes keep both representations synchronized.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scoped_config: Vec<ScopedSkillConfig>,
    /// Configuration-managed skill removed from its source but retained for
    /// audit and restoration. This is independent from operator enablement.
    #[serde(default)]
    pub tombstoned: bool,
    /// ID of the storage provider that loaded this skill.
    #[serde(default)]
    pub provider_id: String,
    /// Optional per-skill LLM execution overrides.
    #[serde(default)]
    pub execution_config: SkillExecutionConfig,
    /// Execution model — Native (default), Manifest (SKILL.md), or Wasm.
    #[serde(default)]
    pub kind: SkillKind,
    /// Provenance — Builtin (system-shipped, immutable) or User (default).
    #[serde(default)]
    pub origin: SkillOrigin,
    /// SPDX license identifier from agentskills.io frontmatter, when present.
    #[serde(default)]
    pub license: Option<String>,
    /// Author names from agentskills.io frontmatter.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Skill's declared implementation language (e.g. `"rust"`, `"typescript"`),
    /// when the frontmatter specifies one.
    #[serde(default)]
    pub language: Option<String>,
    /// Free-form compatibility note/object from frontmatter — shape varies
    /// per skill (plain string or a structured `{platforms, node, ...}`
    /// object), so this is passed through untyped rather than guessed at.
    #[serde(default)]
    pub compatibility: Option<serde_json::Value>,
    /// `metadata.tags` from frontmatter.
    #[serde(default)]
    pub metadata_tags: Vec<String>,
    /// `metadata.category` from frontmatter.
    #[serde(default)]
    pub metadata_category: Option<String>,
    /// Per-phase model-class routing hints from frontmatter's `model_routing`
    /// (agentskills.io convention — see e.g. `liter-llm-bridge`'s SKILL.md).
    #[serde(default)]
    pub model_routing: Option<SkillModelRouting>,
    /// `skill_id` of the parent skill when this one is nested
    /// (`skills/*/skills/*/SKILL.md`) — `None` for top-level skills.
    #[serde(default)]
    pub parent_skill_id: Option<String>,
}

impl Skill {
    /// Resolve enabled state using conversation > agent > global precedence.
    #[must_use]
    pub fn enabled_for(&self, agent_id: Option<&str>, conversation_id: Option<&str>) -> bool {
        self.enabled_for_with_agent_fallback(agent_id, conversation_id, None)
    }

    /// Resolve scoped state with an optional compatibility agent fallback.
    pub(crate) fn enabled_for_with_agent_fallback(
        &self,
        agent_id: Option<&str>,
        conversation_id: Option<&str>,
        agent_fallback: Option<bool>,
    ) -> bool {
        if self.tombstoned {
            return false;
        }
        if let Some(conversation_id) = conversation_id
            && let Some(config) = self.scoped_config.iter().find(|config| {
                matches!(
                    &config.scope,
                    SkillScope::Conversation(id) if id == conversation_id
                )
            })
        {
            return config.enabled;
        }
        if let Some(agent_id) = agent_id
            && let Some(config) = self
                .scoped_config
                .iter()
                .find(|config| matches!(&config.scope, SkillScope::Agent(id) if id == agent_id))
        {
            return config.enabled;
        }
        if let Some(enabled) = agent_fallback {
            return enabled;
        }
        self.scoped_config
            .iter()
            .find(|config| config.scope == SkillScope::Global)
            .map_or(self.enabled, |config| config.enabled)
    }

    /// Insert or replace the enabled-state record for one scope.
    pub fn set_enabled_for(&mut self, scope: SkillScope, enabled: bool) {
        if scope == SkillScope::Global {
            self.enabled = enabled;
        }
        if let Some(config) = self
            .scoped_config
            .iter_mut()
            .find(|config| config.scope == scope)
        {
            config.enabled = enabled;
        } else {
            self.scoped_config
                .push(ScopedSkillConfig { scope, enabled });
        }
    }
}

/// Per-phase model-class routing hints, parsed from a SKILL.md's
/// `model_routing` frontmatter block (see e.g. `liter-llm-bridge`'s
/// SKILL.md). `phases` maps a phase name to a coarse model-class string
/// (`"small"`, `"medium"`, `"large"`, `"frontier"`, by convention in the
/// skill pack) — this is preserved data, not yet a resolved model
/// selection; see `crate::llm::router::route_requirements_for_model_class`
/// for the (intentionally approximate, documented-as-provisional)
/// conversion into a [`crate::llm::router::RouteRequirements`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillModelRouting {
    /// Where the skill author says the routing policy itself lives (e.g. a
    /// project config path) — informational, not resolved by UAR.
    #[serde(default)]
    pub policy_source: Option<String>,
    /// Phase name -> model-class string.
    #[serde(default)]
    pub phases: std::collections::HashMap<String, String>,
    /// Path to further routing documentation within the skill, if any.
    #[serde(default)]
    pub routing_reference: Option<String>,
}

/// Represents the YAML frontmatter of a SKILL.md file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub triggers: SkillTriggers,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scoped_config: Vec<ScopedSkillConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillTriggers {
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub semantic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillConstraints {
    #[serde(default)]
    pub deny_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatch {
    pub skill: Skill,
    pub score: f32,
}

fn default_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tombstone_overrides_scope_without_destroying_configuration() {
        let mut skill = Skill::default();
        skill.set_enabled_for(SkillScope::Global, true);
        skill.set_enabled_for(SkillScope::Agent("agent-a".to_string()), true);
        let scoped_config = skill.scoped_config.clone();

        skill.tombstoned = true;

        assert!(!skill.enabled_for(None, None));
        assert!(!skill.enabled_for(Some("agent-a"), None));
        assert_eq!(skill.scoped_config, scoped_config);
    }
}
