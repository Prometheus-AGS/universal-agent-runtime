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
