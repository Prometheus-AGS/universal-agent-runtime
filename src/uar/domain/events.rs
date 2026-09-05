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
    /// Numbered citation markers (`[1]`, `[2]`, ...) for a RAG-augmented run,
    /// each referencing one retrieved knowledge chunk. Emitted once retrieval
    /// completes and before the assistant's response streams, so the client
    /// can resolve `[n]` markers that appear in the response text to a
    /// hover-to-source panel. Built by [`crate::uar::rag::citation_stream::CitationStream`].
    /// Distinct from [`Self::Citation`], which carries LLM-native web
    /// citations (URL sources) rather than RAG chunk citations.
    RagCitations {
        run_id: String,
        citations: Vec<RagCitation>,
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
    /// Host-owned MCP binding state; a binding may outlive an individual run.
    McpServerStateChanged {
        run_id: Option<String>,
        lifecycle: McpServerLifecycle,
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

    /// A surface was rejected without terminating the readable-text run.
    /// This is a host diagnostic, not a successful publication receipt.
    PresentationDiagnostic {
        run_id: String,
        code: String,
        message: String,
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
    /// A sycophantic response was auto-corrected: carries the rewritten text,
    /// emitted as a follow-up after the original response. Opt-in (`auto_correct`).
    SycophancyCorrected {
        run_id: String,
        corrected_text: String,
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
    /// A persisted child turn started. Never contains its prompt or history.
    AgentThreadStarted {
        run_id: String,
        lifecycle: AgentLifecycle,
    },
    /// A child was persisted pending execution or changed nonterminal state.
    AgentThreadUpdated {
        run_id: String,
        lifecycle: AgentLifecycle,
    },
    /// A child turn completed successfully; its output is retrieved separately.
    AgentThreadFinished {
        run_id: String,
        lifecycle: AgentLifecycle,
    },
    /// A child failed or was cancelled, including before a child run started.
    AgentThreadError {
        run_id: String,
        lifecycle: AgentLifecycle,
    },
    StatePatch {
        run_id: String,
        patch: Vec<StatePatchOp>,
    },
    ContextAction(super::context::ContextAction),

    /// A cost-budget warning or hard-limit crossing (CH-06). Emitted when
    /// `CostBudgetTracker::record` returns `Warning`/`Exceeded` for any scope
    /// (run/task/session/agent/global) with a configured limit.
    BudgetAlert {
        run_id: String,
        /// `run` | `task` | `session` | `agent` | `global`.
        scope: String,
        scope_id: String,
        spent_usd: f64,
        limit_usd: f64,
        /// `true` for a hard-limit crossing, `false` for the warning threshold.
        exceeded: bool,
    },

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
        /// Host-issued request identity; required when resolving child approvals.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        approval_id: Option<String>,
        call_index: usize,
        tool_call_id: String,
        name: String,
        arguments_json: String,
        risk_reason: String,
    },
    /// A governance policy denied a tool call. This is terminal for the call
    /// and must never create a human approval request.
    ToolCallDenied {
        run_id: String,
        call_index: usize,
        tool_call_id: String,
        name: String,
        reason: String,
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

/// Content-free lifecycle projection. This is deliberately not a serialized
/// thread/result: arbitrary model output and backend error text stay private.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AgentLifecycle {
    pub root_thread_id: String,
    pub parent_thread_id: String,
    pub child_thread_id: String,
    pub parent_run_id: Option<String>,
    pub child_run_id: Option<String>,
    pub canonical_path: String,
    pub artifact_id: String,
    pub status: AgentLifecycleStatus,
    pub terminal_outcome: Option<AgentLifecycleOutcome>,
    /// Persisted storage revision, not a delivery counter or history revision.
    pub revision: u64,
    /// Stable source identity for deduplication across publication retries.
    pub lifecycle_id: String,
    /// Persisted transition time; replay must not replace it with wall-clock now.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Public state vocabulary independent of the execution implementation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentLifecycleStatus {
    Pending,
    Running,
    Waiting,
    Completed,
    Failed,
    Cancelled,
}

/// Terminal classification only; no response body, prompt, or raw error.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentLifecycleOutcome {
    Completed,
    Failed,
    Cancelled,
}

/// Trusted-host event publication boundary. Kernels supply typed event intents;
/// the host owns ordering, retention, and transport. No detached publication.
#[async_trait::async_trait]
pub trait RuntimeEventSink: Send + Sync {
    async fn emit(&self, event: NormalizedEvent);
}

/// Observable state of one exact MCP binding generation, not an access grant.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpServerState {
    /// This generation has no active connection (including dormant startup).
    Disabled,
    /// Its single-flight connection/discovery attempt is in progress.
    Connecting,
    /// A matching connection and complete catalog have been published.
    Ready,
    /// Authentication must be supplied or refreshed by the host.
    AuthRequired,
    /// Establishment or discovery failed, or its caller cancelled the attempt.
    Failed,
    /// Revocation has begun; this is not proof of completed resource cleanup.
    ShuttingDown,
}

/// Bounded reason labels; never serialize a raw connection error or token.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpStateReason {
    AuthenticationRequired,
    ConnectionFailed,
    IncompleteCatalog,
    InvalidBinding,
    Invalidated,
    Cancelled,
    Retired,
}

/// Secret-free lifecycle observation. IDs are random, not credential/config hashes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerLifecycle {
    pub event_id: uuid::Uuid,
    pub binding_id: uuid::Uuid,
    pub generation: uuid::Uuid,
    pub sequence: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub server: String,
    pub state: McpServerState,
    pub reason: Option<McpStateReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CitationSource {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
}

/// A single numbered citation marker referencing one retrieved RAG chunk.
///
/// `marker` is the 1-based number that appears in the assistant's response
/// text as `[marker]` (e.g. `marker: 1` for `[1]`). Carried on the wire by
/// [`NormalizedEvent::RagCitations`] and constructed by
/// [`crate::uar::rag::citation_stream::CitationStream`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RagCitation {
    /// 1-based marker number, matching the `[n]` shown in the response text.
    pub marker: usize,
    /// ID of the retrieved knowledge chunk this marker attributes to.
    pub chunk_id: String,
    /// ID of the source document (if the chunk was ingested from one).
    pub document_id: Option<String>,
    /// ID of the knowledge base that produced the retrieved chunk.
    pub knowledge_base_id: Option<String>,
    /// Human-readable document name (filename, or a fallback label).
    pub document_name: String,
    /// Retrieval relevance score (0.0-1.0-ish, retriever-dependent).
    pub relevance_score: f32,
    /// Short snippet of the cited chunk's content, for the hover panel.
    pub snippet: String,
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
