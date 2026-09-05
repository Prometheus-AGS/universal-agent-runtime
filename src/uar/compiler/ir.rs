//! Intermediate Representation (IR) for UAR-AGENT-MD documents.
//!
//! Two IR variants are provided:
//! - [`AgentDescriptorIR`]: All sections required — used after successful parse of a
//!   complete document.
//! - [`PartialAgentDescriptorIR`]: All sections optional — used during conversational
//!   mode to accumulate data incrementally.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────
// Top-level IR types
// ─────────────────────────────────────────────

/// Complete Intermediate Representation of a UAR-AGENT-MD document.
/// Every section is required. This is the canonical input to the compilation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDescriptorIR {
    /// The agent heading name (from `# Agent: <name>`)
    pub agent_name: String,

    /// §04 — Metadata
    pub metadata: MetadataSection,

    /// §05 — Identity
    pub identity: IdentitySection,

    /// §06 — UI (A2UI)
    pub ui: UiSection,

    /// §07 — Capabilities
    pub capabilities: CapabilitiesSection,

    /// §08 — Skills
    pub skills: SkillsSection,

    /// §09 — Tools
    pub tools: ToolsSection,

    /// §10 — MCP Servers
    pub mcp_servers: McpServersSection,

    /// §11 — Knowledge Base
    pub knowledge: KnowledgeSection,

    /// §12 — Memory Model
    pub memory: MemorySection,

    /// §13 — A2A Contracts
    pub a2a: A2ASection,

    /// §14 — Governance
    pub governance: GovernanceSection,

    /// §15 — Budgets & Constraints
    pub budgets: BudgetsSection,

    /// §16 — Execution Model
    pub execution: ExecutionSection,

    /// §17 — Observability
    pub observability: ObservabilitySection,

    /// §18 — Deployment Profiles
    pub deployment: DeploymentSection,

    /// §19 — Model Requirements (v2, CH-12 agent-spec-v2).
    /// `#[serde(default)]` so v1.1 documents (which predate this section)
    /// still deserialize — this and the four sections below are additive,
    /// not part of the original 15-section completeness contract.
    #[serde(default)]
    pub model_requirements: ModelRequirementsSection,

    /// §20 — Prompt Dialect (v2, CH-12 agent-spec-v2).
    #[serde(default)]
    pub prompt_dialect: PromptDialectSection,

    /// §21 — RAG Configuration (v2, CH-12 agent-spec-v2).
    #[serde(default)]
    pub rag_configuration: RagConfigurationSection,

    /// §22 — Context Strategy (v2, CH-12 agent-spec-v2).
    #[serde(default)]
    pub context_strategy: ContextStrategySection,

    /// §23 — API Harness (v2, CH-12 agent-spec-v2).
    #[serde(default)]
    pub api_harness: ApiHarnessSection,
}

/// Partial IR for conversational mode. Every section is `Option<T>` to allow
/// incremental population. Use [`CompletenessAnalyzer`] to check readiness.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialAgentDescriptorIR {
    pub agent_name: Option<String>,
    pub metadata: Option<MetadataSection>,
    pub identity: Option<IdentitySection>,
    pub ui: Option<UiSection>,
    pub capabilities: Option<CapabilitiesSection>,
    pub skills: Option<SkillsSection>,
    pub tools: Option<ToolsSection>,
    pub mcp_servers: Option<McpServersSection>,
    pub knowledge: Option<KnowledgeSection>,
    pub memory: Option<MemorySection>,
    pub a2a: Option<A2ASection>,
    pub governance: Option<GovernanceSection>,
    pub budgets: Option<BudgetsSection>,
    pub execution: Option<ExecutionSection>,
    pub observability: Option<ObservabilitySection>,
    pub deployment: Option<DeploymentSection>,
    /// v2 sections (CH-12) — always optional here, even after promotion to
    /// a complete `AgentDescriptorIR` (see `try_into_complete`), unlike the
    /// 15 sections above.
    pub model_requirements: Option<ModelRequirementsSection>,
    pub prompt_dialect: Option<PromptDialectSection>,
    pub rag_configuration: Option<RagConfigurationSection>,
    pub context_strategy: Option<ContextStrategySection>,
    pub api_harness: Option<ApiHarnessSection>,
}

impl PartialAgentDescriptorIR {
    /// Attempt to promote this partial IR into a complete [`AgentDescriptorIR`].
    /// Returns `None` if any required section is missing.
    #[must_use]
    pub fn try_into_complete(self) -> Option<AgentDescriptorIR> {
        Some(AgentDescriptorIR {
            agent_name: self.agent_name?,
            metadata: self.metadata?,
            identity: self.identity?,
            ui: self.ui?,
            capabilities: self.capabilities?,
            skills: self.skills?,
            tools: self.tools?,
            mcp_servers: self.mcp_servers?,
            knowledge: self.knowledge?,
            memory: self.memory?,
            a2a: self.a2a?,
            governance: self.governance?,
            budgets: self.budgets?,
            execution: self.execution?,
            observability: self.observability?,
            deployment: self.deployment?,
            // v2 sections default rather than gate readiness (`?`) — a v1.1
            // document that never declares them still completes.
            model_requirements: self.model_requirements.unwrap_or_default(),
            prompt_dialect: self.prompt_dialect.unwrap_or_default(),
            rag_configuration: self.rag_configuration.unwrap_or_default(),
            context_strategy: self.context_strategy.unwrap_or_default(),
            api_harness: self.api_harness.unwrap_or_default(),
        })
    }
}

impl From<AgentDescriptorIR> for PartialAgentDescriptorIR {
    fn from(ir: AgentDescriptorIR) -> Self {
        Self {
            agent_name: Some(ir.agent_name),
            metadata: Some(ir.metadata),
            identity: Some(ir.identity),
            ui: Some(ir.ui),
            capabilities: Some(ir.capabilities),
            skills: Some(ir.skills),
            tools: Some(ir.tools),
            mcp_servers: Some(ir.mcp_servers),
            knowledge: Some(ir.knowledge),
            memory: Some(ir.memory),
            a2a: Some(ir.a2a),
            governance: Some(ir.governance),
            budgets: Some(ir.budgets),
            execution: Some(ir.execution),
            observability: Some(ir.observability),
            deployment: Some(ir.deployment),
            model_requirements: Some(ir.model_requirements),
            prompt_dialect: Some(ir.prompt_dialect),
            rag_configuration: Some(ir.rag_configuration),
            context_strategy: Some(ir.context_strategy),
            api_harness: Some(ir.api_harness),
        }
    }
}

// ─────────────────────────────────────────────
// Section names — for parsing and lookup
// ─────────────────────────────────────────────

/// Canonical section names as they appear in UAR-AGENT-MD `## <name>` headings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SectionName {
    Metadata,
    Identity,
    Ui,
    Capabilities,
    Skills,
    Tools,
    McpServers,
    Knowledge,
    Memory,
    A2A,
    Governance,
    Budgets,
    Execution,
    Observability,
    Deployment,
    /// v2 (CH-12) — recognized by the parser but intentionally excluded
    /// from `ALL`/completeness gating; see the module-level v2 note.
    ModelRequirements,
    PromptDialect,
    RagConfiguration,
    ContextStrategy,
    ApiHarness,
}

impl SectionName {
    /// All section names in specification order.
    pub const ALL: [Self; 15] = [
        Self::Metadata,
        Self::Identity,
        Self::Ui,
        Self::Capabilities,
        Self::Skills,
        Self::Tools,
        Self::McpServers,
        Self::Knowledge,
        Self::Memory,
        Self::A2A,
        Self::Governance,
        Self::Budgets,
        Self::Execution,
        Self::Observability,
        Self::Deployment,
    ];

    /// Parse a heading string into a section name (case-insensitive, flexible matching).
    #[must_use]
    pub fn from_heading(heading: &str) -> Option<Self> {
        let normalized = heading.trim().to_lowercase();
        match normalized.as_str() {
            "metadata" => Some(Self::Metadata),
            "identity" => Some(Self::Identity),
            "ui" | "ui (a2ui)" | "a2ui" => Some(Self::Ui),
            "capabilities" => Some(Self::Capabilities),
            "skills" => Some(Self::Skills),
            "tools" => Some(Self::Tools),
            "mcp servers" | "mcp_servers" | "mcpservers" => Some(Self::McpServers),
            "knowledge base" | "knowledge" | "knowledgebase" => Some(Self::Knowledge),
            "memory model" | "memory" | "memorymodel" => Some(Self::Memory),
            "a2a contracts" | "a2a" | "a2acontracts" => Some(Self::A2A),
            "governance" => Some(Self::Governance),
            "budgets & constraints" | "budgets" | "budgets and constraints" => Some(Self::Budgets),
            "execution model" | "execution" | "executionmodel" => Some(Self::Execution),
            "observability" => Some(Self::Observability),
            "deployment profiles" | "deployment" | "deploymentprofiles" => Some(Self::Deployment),
            "model requirements" | "model_requirements" | "modelrequirements" => {
                Some(Self::ModelRequirements)
            }
            "prompt dialect" | "prompt_dialect" | "promptdialect" => Some(Self::PromptDialect),
            "rag configuration" | "rag_configuration" | "ragconfiguration" | "rag" => {
                Some(Self::RagConfiguration)
            }
            "context strategy" | "context_strategy" | "contextstrategy" => {
                Some(Self::ContextStrategy)
            }
            "api harness" | "api_harness" | "apiharness" => Some(Self::ApiHarness),
            _ => None,
        }
    }

    /// Human-readable display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Metadata => "Metadata",
            Self::Identity => "Identity",
            Self::Ui => "UI (A2UI)",
            Self::Capabilities => "Capabilities",
            Self::Skills => "Skills",
            Self::Tools => "Tools",
            Self::McpServers => "MCP Servers",
            Self::Knowledge => "Knowledge Base",
            Self::Memory => "Memory Model",
            Self::A2A => "A2A Contracts",
            Self::Governance => "Governance",
            Self::Budgets => "Budgets & Constraints",
            Self::Execution => "Execution Model",
            Self::Observability => "Observability",
            Self::Deployment => "Deployment Profiles",
            Self::ModelRequirements => "Model Requirements",
            Self::PromptDialect => "Prompt Dialect",
            Self::RagConfiguration => "RAG Configuration",
            Self::ContextStrategy => "Context Strategy",
            Self::ApiHarness => "API Harness",
        }
    }
}

impl std::fmt::Display for SectionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

// ─────────────────────────────────────────────
// §04 — Metadata
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSection {
    pub version: String,
    #[serde(default)]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
}

// ─────────────────────────────────────────────
// §05 — Identity
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySection {
    pub name: String,
    pub role: String,
    pub persona: String,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instructions: Vec<String>,
    #[serde(default)]
    pub greeting: Option<String>,
}

// ─────────────────────────────────────────────
// §06 — UI (A2UI)
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSection {
    #[serde(default)]
    pub forms: Vec<UiForm>,
    #[serde(default)]
    pub artifacts: Vec<UiArtifact>,
    #[serde(default)]
    pub actions: Vec<UiAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiForm {
    pub id: String,
    #[serde(default)]
    pub schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiArtifact {
    pub id: String,
    #[serde(rename = "type")]
    pub artifact_type: String,
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiAction {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
}

// ─────────────────────────────────────────────
// §07 — Capabilities
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesSection {
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub file_upload: bool,
    #[serde(default)]
    pub image_generation: bool,
    #[serde(default)]
    pub code_execution: bool,
    #[serde(default)]
    pub web_browsing: bool,
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

// ─────────────────────────────────────────────
// §08 — Skills
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsSection {
    #[serde(default)]
    pub skills: Vec<SkillRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRef {
    pub id: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

// ─────────────────────────────────────────────
// §09 — Tools
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsSection {
    #[serde(default)]
    pub tools: Vec<ToolRef>,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRef {
    pub name: String,
    #[serde(default)]
    pub server: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

// ─────────────────────────────────────────────
// §10 — MCP Servers
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServersSection {
    #[serde(default)]
    pub servers: Vec<McpServerRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerRef {
    pub id: String,
    pub url: String,
    #[serde(default)]
    pub transport: Option<String>,
    #[serde(default)]
    pub auth: Option<McpAuth>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(default)]
    pub token_env: Option<String>,
}

// ─────────────────────────────────────────────
// §11 — Knowledge Base
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSection {
    #[serde(default)]
    pub sources: Vec<KnowledgeSource>,
    #[serde(default)]
    pub retrieval: Option<RetrievalConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub threshold: Option<f64>,
}

// ─────────────────────────────────────────────
// §12 — Memory Model
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySection {
    #[serde(default)]
    pub conversation: ConversationMemoryConfig,
    #[serde(default)]
    pub persistent: Option<PersistentMemoryConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationMemoryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub max_turns: Option<u32>,
    #[serde(default)]
    pub summarization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentMemoryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub backend: Option<String>,
}

fn default_true() -> bool {
    true
}

// ─────────────────────────────────────────────
// §13 — A2A Contracts
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ASection {
    #[serde(default)]
    pub endpoints: Vec<A2AEndpoint>,
    #[serde(default)]
    pub dependencies: Vec<A2ADependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AEndpoint {
    pub id: String,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ADependency {
    pub agent_id: String,
    pub endpoint: String,
    #[serde(default)]
    pub required: bool,
}

// ─────────────────────────────────────────────
// §14 — Governance
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSection {
    #[serde(default)]
    pub cedar_policies: Vec<String>,
    #[serde(default)]
    pub cedar_inline: Option<String>,
    #[serde(default)]
    pub data_classification: Option<String>,
    #[serde(default)]
    pub audit: AuditConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub log_level: Option<String>,
}

// ─────────────────────────────────────────────
// §15 — Budgets & Constraints
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetsSection {
    #[serde(default)]
    pub max_tokens_per_turn: Option<u64>,
    #[serde(default)]
    pub max_tokens_per_session: Option<u64>,
    #[serde(default)]
    pub max_tool_calls_per_turn: Option<u32>,
    #[serde(default)]
    pub max_cost_per_session_usd: Option<f64>,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default)]
    pub requests_per_minute: Option<u32>,
    #[serde(default)]
    pub tokens_per_minute: Option<u64>,
}

// ─────────────────────────────────────────────
// §16 — Execution Model
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSection {
    /// e.g. "sequential", "parallel", "reactive"
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub max_iterations: Option<u32>,
    #[serde(default)]
    pub stop_conditions: Vec<String>,
    #[serde(default)]
    pub fallback_behavior: Option<String>,
}

// ─────────────────────────────────────────────
// §17 — Observability
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilitySection {
    #[serde(default)]
    pub tracing: TracingConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TracingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub exporter: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub exporter: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
}

// ─────────────────────────────────────────────
// §18 — Deployment Profiles
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSection {
    #[serde(default)]
    pub profiles: Vec<DeploymentProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentProfile {
    pub id: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub provider: Option<ProviderConfig>,
    #[serde(default)]
    pub resources: Option<ResourceConfig>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub model: String,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub max_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    #[serde(default)]
    pub memory_mb: Option<u64>,
    #[serde(default)]
    pub cpu_cores: Option<f64>,
    #[serde(default)]
    pub gpu: Option<bool>,
}

// ─────────────────────────────────────────────
// §19 — Model Requirements (v2, CH-12 agent-spec-v2)
// ─────────────────────────────────────────────

/// Declares the capabilities a compiled agent's resolved model must have.
/// Mirrors [`crate::llm::router::RouteRequirements`] — the compiler-time
/// declaration and the runtime router's selection criteria are the same
/// shape by design, so a conformance check (CH-14) can compare them
/// directly.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelRequirementsSection {
    #[serde(default)]
    pub needs_tools: bool,
    #[serde(default)]
    pub needs_reasoning: bool,
    #[serde(default)]
    pub needs_vision: bool,
    #[serde(default)]
    pub needs_structured_output: bool,
    #[serde(default)]
    pub min_context: Option<u64>,
    #[serde(default)]
    pub max_cost_per_1m_input: Option<f64>,
    #[serde(default)]
    pub preferred_provider: Option<String>,
}

// ─────────────────────────────────────────────
// §20 — Prompt Dialect (v2, CH-12 agent-spec-v2)
// ─────────────────────────────────────────────

/// Declares prompt-dialect hints for the resolved model. Mirrors
/// [`crate::llm::prompt_dialect::DialectRequest`] — `dialect` is an
/// optional override; when absent, the dialect is detected from the
/// resolved model id at request-assembly time (unchanged from CH-04).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptDialectSection {
    /// Explicit dialect override. One of `anthropic_xml`, `openai_json`,
    /// `kimi_markdown`, `glm_thinking`, `qwen_hybrid`, `minimax_structured`,
    /// `generic`. Absent = auto-detect from the resolved model id.
    #[serde(default)]
    pub dialect: Option<String>,
    #[serde(default)]
    pub wants_reasoning: bool,
    #[serde(default)]
    pub hard: bool,
}

// ─────────────────────────────────────────────
// §21 — RAG Configuration (v2, CH-12 agent-spec-v2)
// ─────────────────────────────────────────────

/// Declares this agent's retrieval-augmented-generation posture. Mirrors
/// the in-process hardening knobs shipped in CH-11 (query decomposition,
/// retrieval verification, audit events) — deliberately in-process per
/// `uar-next-harness`'s D-A decision; Knowledge Service extraction remains
/// out of scope.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RagConfigurationSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub decomposition: bool,
    #[serde(default)]
    pub verification: bool,
    #[serde(default)]
    pub audit: bool,
    #[serde(default)]
    pub knowledge_base_ids: Vec<String>,
}

// ─────────────────────────────────────────────
// §22 — Context Strategy (v2, CH-12 agent-spec-v2)
// ─────────────────────────────────────────────

/// Declares this agent's message-history trimming strategy. Mirrors
/// [`crate::uar::context::ContextStrategy`] variant-for-variant (including
/// `Auto`, CH-05's model-aware selection) so a compiled descriptor can
/// express exactly what the runtime will do — the conformance harness
/// (CH-14) checks this against `RunManager.message_context_strategy` at
/// load and run time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextStrategySection {
    #[default]
    Auto,
    None,
    SlidingWindow {
        #[serde(default)]
        max_messages: Option<usize>,
    },
    Summarize {
        #[serde(default)]
        threshold: Option<usize>,
        #[serde(default)]
        summary_max_tokens: Option<usize>,
    },
    TruncateMiddle {
        #[serde(default)]
        keep_first: Option<usize>,
        #[serde(default)]
        keep_last: Option<usize>,
    },
    Hierarchical {
        #[serde(default)]
        short_term_turns: Option<usize>,
        #[serde(default)]
        mid_term_summary_tokens: Option<usize>,
        #[serde(default)]
        long_term_facts_tokens: Option<usize>,
    },
}

// ─────────────────────────────────────────────
// §23 — API Harness (v2, CH-12 agent-spec-v2)
// ─────────────────────────────────────────────

/// Declares which transport protocols this agent supports and, for the
/// SSE surface, which `stream_mode` it emits by default. Mirrors the
/// `stream_mode` selector CH-21 (agui-spec-alignment) added to
/// `src/server.rs` (`openai` | `agui` | `dual` | `agui_spec`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiHarnessSection {
    /// Supported transport protocols, e.g. `["a2a", "agui", "openai", "rest"]`.
    #[serde(default)]
    pub protocols: Vec<String>,
    /// Default SSE stream mode: `openai` | `agui` | `dual` | `agui_spec`.
    #[serde(default)]
    pub stream_mode: Option<String>,
}
