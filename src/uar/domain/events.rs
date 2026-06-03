use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum NormalizedEvent {
    RunStart {
        run_id: String,
        agent_id: String,
    },
    ChatDelta {
        run_id: String,
        text_delta: String,
    },
    ThinkingDelta {
        run_id: String,
        text_delta: String,
    },
    ReasoningDelta {
        run_id: String,
        text_delta: String,
    },

    Citation {
        run_id: String,
        sources: Vec<CitationSource>,
    },
    MemoryRecall {
        run_id: String,
        items: Vec<MemoryItem>,
    },
    SkillActivated {
        run_id: String,
        skill_id: String,
        title: String,
        selection_method: String,
    },

    ToolStart {
        run_id: String,
        call_index: usize,
        tool_call_id: String,
        tool: String,
        input: serde_json::Value,
    },
    ToolDelta {
        run_id: String,
        call_index: usize,
        tool_call_id: String,
        delta: serde_json::Value,
    },
    ToolEnd {
        run_id: String,
        call_index: usize,
        tool_call_id: String,
        tool: String,
        output: serde_json::Value,
        ok: bool,
    },

    Artifact {
        run_id: String,
        artifact: ArtifactPayload,
    },

    Error {
        run_id: String,
        code: String,
        message: String,
    },
    RunDone {
        run_id: String,
    },
    /// A run was cancelled — explicitly via the cancel endpoint, by the last SSE
    /// subscriber disconnecting, or by server shutdown. Terminal, distinct from
    /// `RunDone` (normal completion) and `Error` (failure).
    Cancelled {
        run_id: String,
    },
    /// The completed assistant response was flagged by sycophancy detection.
    /// Quality signal (not an error); emitted only when the score meets the
    /// configured threshold or a critical pattern was found. Carries the score
    /// and pattern classifications, never the full response text.
    SycophancyFlagged {
        run_id: String,
        /// 0.0 (clean) – 1.0 (fully sycophantic).
        sycophancy_score: f32,
        has_critical: bool,
        correction_mandatory: bool,
        classifications: Vec<SycophancyClassification>,
    },
    /// Chat input was flagged by an input guardrail (prompt-injection or PII).
    /// `run_id` is absent when the input was blocked before a run started.
    /// Carries only the category and a short reason — never the raw input or the
    /// matched secret value.
    GuardrailFlagged {
        run_id: Option<String>,
        /// `injection` | `pii`.
        category: String,
        /// Short, content-free reason label.
        reason: String,
    },
    /// A tool-loop iteration boundary — per-step run progress for the Runtime
    /// Console. Emitted at the start and end of each orchestrator iteration.
    RuntimeStep {
        run_id: String,
        /// Monotonic per-run step index (the orchestrator iteration number).
        step: u32,
        /// `started` | `finished`.
        kind: String,
    },
    StatePatch {
        run_id: String,
        patch: Vec<StatePatchOp>,
    },
    ContextAction(super::context::ContextAction),

    /// A memory was created, updated, or deleted — either by an LLM tool call or by auto-capture.
    MemoryMutation {
        run_id: String,
        /// "created" | "updated" | "deleted"
        operation: String,
        /// The record ID of the affected memory (e.g. "memory:abc123"). Empty if unavailable.
        memory_id: String,
        /// Content of the memory at time of mutation (empty for deletions).
        content: String,
        /// Scope label: "session" | "user" | "agent" | "global" | "task"
        scope: String,
        /// Cognitive type: "semantic" | "episodic" | "procedural" | "associative"
        memory_type: String,
    },

    /// An agent produced a displayable artifact (code block, document, chart, etc.).
    /// This is also used for `artifact_type = "display"` UI surfaces.
    /// Emitted as `agui.artifact` on the SSE stream.
    ArtifactDisplay {
        run_id: String,
        artifact: ArtifactPayload,
    },

    /// An agent needs structured input from the user before it can continue.
    /// The `artifact` field carries the JSON Schema of the input form.
    /// The agent run is paused until the user submits a response via
    /// `POST /api/uar/runs/{run_id}/artifact-response`.
    /// Emitted as `agui.artifact_input_request` on the SSE stream.
    ArtifactInputRequest {
        run_id: String,
        artifact: ArtifactPayload,
    },

    /// A tool call requires user approval before execution.
    /// The run is paused until the user responds via the approval endpoint.
    ToolCallApprovalRequired {
        run_id: String,
        call_index: usize,
        tool_call_id: String,
        name: String,
        arguments_json: String,
        risk_reason: String,
    },

    /// A run completed. Optionally carries token usage and cost estimates
    /// when the persistence layer or LLM provider reports them.
    RunDoneWithUsage {
        run_id: String,
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
        total_tokens: Option<u32>,
        /// Estimated cost in USD based on model pricing.
        cost_usd_estimate: Option<f64>,
        /// Model used for this run.
        model: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CitationSource {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
}

/// A single sycophancy pattern match (a compact, serializable summary of a
/// detector `HeuristicMatch`). Excludes the response text; the rationale is the
/// detector's short explanation of why the pattern triggered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SycophancyClassification {
    /// Pattern id, e.g. `S-01` … `S-08`.
    pub pattern_id: String,
    /// Severity: `low` | `medium` | `high` | `critical`.
    pub severity: String,
    /// Short explanation of why the pattern triggered.
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    /// For pre-call retrieval: "memory_context". For model-provided: the operation type.
    pub source: String,
    /// Memory scope (session, user, agent, global, task). Present for pre-call retrieval hits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Memory type (episodic, semantic, procedural, associative). Present for pre-call retrieval hits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<String>,
    /// Importance score 0.0–1.0. Present for pre-call retrieval hits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactPayload {
    pub artifact_id: String,
    pub artifact_type: String,
    pub title: String,
    pub content: String,
    pub language: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatePatchOp {
    pub op: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}
