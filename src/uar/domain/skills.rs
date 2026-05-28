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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
