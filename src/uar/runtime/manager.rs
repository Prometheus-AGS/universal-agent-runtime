use crate::config::{LlmConfig, SkillEvolutionConfig};
use crate::llm::{LiterLlmDriver, Message, MessageRole, Orchestrator};
use crate::mcp::registry::McpRegistry;
use crate::session::SessionStore;
use crate::uar::domain::{
    artifact::AgentArtifact,
    context::ContextConfig,
    events::{CitationSource, MemoryItem, NormalizedEvent, StatePatchOp},
    runs::{Run, RunStatus},
};
use crate::uar::runtime::context::manager::ContextManager;
use crate::uar::runtime::matching::{ClassifierConfig, IntentClassifier, create_classifier};
use crate::uar::runtime::native_skill::NativeSkillRegistry;
use crate::uar::runtime::skills::SkillRegistry;
use crate::uar::runtime::skills::service::SkillService;
use futures::StreamExt;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Write,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{Mutex, RwLock, broadcast, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;
use tracing::instrument;
use uuid::Uuid;

const EVENT_HISTORY_LIMIT: usize = 512;

#[derive(Clone, Debug)]
pub struct StreamEvent {
    pub id: u64,
    pub event: NormalizedEvent,
}

#[derive(Debug)]
struct EventHistory {
    next_id: u64,
    buffer: VecDeque<StreamEvent>,
}

#[derive(Debug)]
struct RunStreamState {
    run: Run,
    sender: broadcast::Sender<StreamEvent>,
    history: Arc<Mutex<EventHistory>>,
}

#[derive(Clone, Debug)]
struct RunEventEmitter {
    sender: broadcast::Sender<StreamEvent>,
    history: Arc<Mutex<EventHistory>>,
}

impl RunEventEmitter {
    async fn emit(&self, event: NormalizedEvent) {
        let mut history = self.history.lock().await;
        let id = history.next_id;
        history.next_id = history.next_id.saturating_add(1);

        let stream_event = StreamEvent { id, event };
        history.buffer.push_back(stream_event.clone());
        if history.buffer.len() > EVENT_HISTORY_LIMIT {
            history.buffer.pop_front();
        }

        let _ = self.sender.send(stream_event);
    }
}

type ActiveRunMap = HashMap<String, RunStreamState>;

/// Extract `budgets.max_cost_per_session_usd` from an `AgentArtifact`'s
/// `extensions` map (CH-06). `budgets` has no typed home on `AgentPolicy` —
/// see `to_artifact.rs`'s module doc — so it's preserved losslessly as JSON
/// under `extensions["budgets"]`. Returns `None` for an absent key, a `null`
/// value (unset budgets section), a missing field, or a non-numeric field.
fn agent_cost_limit_from_extensions(
    extensions: &HashMap<String, serde_json::Value>,
) -> Option<f64> {
    extensions
        .get("budgets")
        .and_then(|v| v.get("max_cost_per_session_usd"))
        .and_then(serde_json::Value::as_f64)
}

/// Correlate matched-skill activation against actually-invoked tools (CH-08).
///
/// `skill_servers` maps each matched skill's id to the MCP server name(s) its
/// `mcp_config` introduced (captured before registries are merged, since the
/// merged registry no longer distinguishes which skill contributed which
/// server). `invoked_tool_servers` is the set of server names the run's
/// actually-invoked tools resolved back to. A skill is "used" if any of its
/// servers appears in that set. Returns one entry per key in `skill_servers`
/// — skills with no `mcp_config` are simply absent from that map and so never
/// appear here, which is the caller's signal to exclude them from outcome
/// tracking entirely rather than record a proxy `false`.
fn correlate_skill_activation_outcomes(
    skill_servers: &HashMap<String, Vec<String>>,
    invoked_tool_servers: &HashSet<String>,
) -> Vec<(String, bool)> {
    skill_servers
        .iter()
        .map(|(skill_id, servers)| {
            let used = servers.iter().any(|s| invoked_tool_servers.contains(s));
            (skill_id.clone(), used)
        })
        .collect()
}

#[derive(Clone)]
pub struct RunManager {
    // Map run_id -> (Run metadata, broadcast sender)
    active_runs: Arc<RwLock<ActiveRunMap>>,
    // Map session_id -> most recent run_id for deterministic session lookup.
    session_current_run: Arc<RwLock<HashMap<String, String>>>,
    llm_config: LlmConfig,
    global_mcp: Arc<McpRegistry>,
    sessions: SessionStore,
    skills: Arc<RwLock<SkillRegistry>>,
    vector_matcher: Arc<crate::uar::runtime::matching::VectorMatcher>,
    tag_matcher: Arc<crate::uar::runtime::matching::TagMatcher>,
    context_manager: Arc<ContextManager>,
    /// Intent classifier for skill matching
    intent_classifier: Arc<dyn IntentClassifier>,
    /// Classifier configuration
    classifier_config: ClassifierConfig,
    // Persistence layer (optional)
    pub persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
    /// Skill service for coordinated skill management
    skill_service: Option<Arc<SkillService>>,
    /// Configuration for the Hermes skill auto-creation/evolution post-run hook.
    skill_evolution_config: SkillEvolutionConfig,
    /// Provider registry for per-agent LLM provider resolution
    provider_registry: Option<Arc<crate::llm::ProviderRegistry>>,
    /// Multi-tenant credential service (per-user encrypted provider keys).
    /// `None` ⇒ single-tenant: the env/config key on `llm_config` is used as-is.
    provider_service: Option<Arc<crate::uar::security::credentials::ProviderService>>,
    /// Native skill registry for in-process tool execution
    native_skills: Arc<NativeSkillRegistry>,
    /// Pending tool-call approval channels: run_id -> oneshot sender.
    /// When a tool call requires approval, a oneshot channel is inserted here.
    /// The approval endpoint sends `true` (approved) or `false` (rejected) through it.
    pending_approvals: Arc<Mutex<HashMap<String, oneshot::Sender<bool>>>>,
    /// Root cancellation token. Every run derives a child token from this, so
    /// cancelling the root (e.g. on server shutdown) aborts all in-flight runs.
    root_cancellation: CancellationToken,
    /// Per-run cancellation tokens: `run_id` -> child token. Populated in
    /// `start_run` and removed when the run reaches a terminal state, so finished
    /// runs do not accumulate. `cancel_run` cancels the token found here.
    run_cancellations: Arc<RwLock<HashMap<String, CancellationToken>>>,
    /// Message-count based context strategy applied to session history before LLM calls.
    message_context_strategy: crate::uar::context::ContextStrategy,
    /// Optional agent graph for graph-based execution. When set, `start_run` uses
    /// graph-driven orchestration instead of the simple tool loop.
    agent_graph: Option<std::sync::Arc<crate::uar::runtime::graph::AgentGraph>>,
    /// Optional Cedar governance engine consulted at the tool-approval gate.
    /// When set, a tool that policy denies is routed to the HITL approval gate.
    /// `None` ⇒ tool approval relies solely on the keyword heuristic.
    governance_engine: Option<Arc<crate::uar::governance::engine::GovernanceEngine>>,
    /// Runtime model failover configuration (CH-03). `enabled: false` by
    /// default (opt-in) — when enabled, each run's `Orchestrator` is given a
    /// fallback driver built from `fallback_models.first()` plus the shared
    /// provider-health monitor.
    failover_config: crate::config::FailoverConfig,
    /// Per-run/task/session/agent/global spend aggregator (CH-06). Always
    /// present (unconfigured scopes simply have no limit, so `record` is a
    /// cheap no-op warning check).
    cost_budget: crate::uar::runtime::cost_budget::CostBudgetTracker,
    resilience_policy: crate::uar::settings::resilience_policy::ResiliencePolicy,
}

/// Memory mutation tool name sets — used to detect side effects in ToolEnd events.
const MEMORY_CREATE_TOOLS: &[&str] = &[
    "memory_add",
    "memory_save",
    "memory_extract_from_conversation",
];
const MEMORY_UPDATE_TOOLS: &[&str] = &["memory_update"];
const MEMORY_DELETE_TOOLS: &[&str] = &["memory_delete", "memory_delete_all"];

/// Apply the multi-tenant credential layer to a run's LLM config.
///
/// When `provider_service` is `Some` and a per-scope credential resolves for the
/// run's provider (chain: session → agent → user → system), override
/// `cfg.api_key` with the decrypted per-tenant key. When `provider_service` is
/// `None`, no credential resolves, or resolution errors, `cfg.api_key` is left
/// untouched — i.e. the single-tenant env/config key path is preserved.
///
/// Extracted from `start_run` so the resolution-and-override behavior is unit
/// testable without constructing an `Orchestrator` or making LLM calls.
async fn apply_credential_layer(
    mut cfg: LlmConfig,
    provider_service: Option<&Arc<crate::uar::security::credentials::ProviderService>>,
    user_id: Option<&str>,
    session_id: Option<&str>,
    agent_id: &str,
) -> LlmConfig {
    let Some(provider_service) = provider_service else {
        return cfg;
    };
    // Provider id is the segment before '/' in `provider/model`.
    let provider_id = cfg
        .model
        .split_once('/')
        .map_or(cfg.model.as_str(), |(p, _)| p)
        .to_string();
    match provider_service
        .resolver()
        .resolve_with_context(
            user_id.unwrap_or("anonymous"),
            &provider_id,
            session_id,
            Some(agent_id),
        )
        .await
    {
        Ok(Some(resolved)) => {
            use secrecy::ExposeSecret;
            cfg.api_key = Some(resolved.api_key.expose_secret().to_string());
            tracing::debug!(
                provider = %provider_id,
                scope = resolved.scope.as_str(),
                "Resolved per-tenant provider credential"
            );
        }
        Ok(None) => {
            tracing::trace!(
                provider = %provider_id,
                "No per-tenant credential; using env/config key"
            );
        }
        Err(e) => {
            // Do not leak the key; surface provider/scope only.
            tracing::warn!(
                provider = %provider_id,
                error = %e,
                "Credential resolution failed; falling back to env/config key"
            );
        }
    }
    cfg
}

/// Inspect a `ToolEnd` event and, if it represents a memory mutation, return a
/// corresponding `MemoryMutation` event. Returns `None` for non-memory tools.
fn memory_mutation_from_tool_end(evt: &NormalizedEvent, run_id: &str) -> Option<NormalizedEvent> {
    let NormalizedEvent::ToolEnd {
        tool, output, ok, ..
    } = evt
    else {
        return None;
    };

    if !ok {
        return None;
    }

    let operation = if MEMORY_CREATE_TOOLS.contains(&tool.as_str()) {
        "created"
    } else if MEMORY_UPDATE_TOOLS.contains(&tool.as_str()) {
        "updated"
    } else if MEMORY_DELETE_TOOLS.contains(&tool.as_str()) {
        "deleted"
    } else {
        return None;
    };

    // Try to extract memory_id and content from the tool output JSON.
    let memory_id = output
        .get("memory_id")
        .or_else(|| output.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let content = output
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let scope = output
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let memory_type = output
        .get("memory_type")
        .or_else(|| output.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("semantic")
        .to_string();

    Some(NormalizedEvent::MemoryMutation {
        run_id: run_id.to_string(),
        operation: operation.to_string(),
        memory_id,
        content,
        scope,
        memory_type,
    })
}

/// Simple heuristic to determine if a tool call requires user approval before execution.
/// Tools whose names contain destructive or write-oriented keywords are flagged.
/// This will be replaced by Cedar policy evaluation in a future milestone.
fn tool_requires_approval(tool_name: &str) -> bool {
    let lower = tool_name.to_lowercase();
    const RISKY_KEYWORDS: &[&str] = &["delete", "remove", "write", "drop", "truncate", "destroy"];
    RISKY_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

#[derive(Debug, PartialEq, Eq)]
enum ApprovalWaitOutcome {
    Approved,
    Rejected,
    ChannelClosed,
    TimedOut,
}

async fn await_approval(
    receiver: oneshot::Receiver<bool>,
    timeout_duration: Duration,
) -> ApprovalWaitOutcome {
    match tokio::time::timeout(timeout_duration, receiver).await {
        Ok(Ok(true)) => ApprovalWaitOutcome::Approved,
        Ok(Ok(false)) => ApprovalWaitOutcome::Rejected,
        Ok(Err(_)) => ApprovalWaitOutcome::ChannelClosed,
        Err(_) => ApprovalWaitOutcome::TimedOut,
    }
}

async fn resolve_pending_approval(
    approvals: &Mutex<HashMap<String, oneshot::Sender<bool>>>,
    run_id: &str,
    approved: bool,
) -> bool {
    let sender = approvals.lock().await.remove(run_id);
    sender.is_some_and(|sender| sender.send(approved).is_ok())
}

/// RAII guard tied to the lifetime of an SSE subscription.
///
/// When dropped (the SSE client disconnects and its stream is torn down), it
/// schedules a check after a short grace period: if the run then has no
/// remaining subscribers, it is cancelled. The grace period absorbs reconnect
/// races (a client resuming via history replay re-subscribes within the
/// window), and the no-subscriber check enforces last-subscriber-drop semantics
/// so a run watched by multiple clients survives any single disconnect.
#[derive(Debug)]
pub struct RunDisconnectGuard {
    manager: Arc<RunManager>,
    run_id: String,
}

impl RunDisconnectGuard {
    /// Grace period before deciding a disconnected run is abandoned.
    const GRACE: Duration = Duration::from_millis(250);

    #[must_use]
    pub fn new(manager: Arc<RunManager>, run_id: String) -> Self {
        Self { manager, run_id }
    }
}

impl Drop for RunDisconnectGuard {
    fn drop(&mut self) {
        let manager = Arc::clone(&self.manager);
        let run_id = std::mem::take(&mut self.run_id);
        if run_id.is_empty() {
            return;
        }
        tokio::spawn(async move {
            tokio::time::sleep(RunDisconnectGuard::GRACE).await;
            if manager.cancel_run_if_no_subscribers(&run_id).await {
                tracing::info!(
                    run_id = %run_id,
                    "Run cancelled: last SSE subscriber disconnected"
                );
            }
        });
    }
}

impl std::fmt::Debug for RunManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunManager")
            .field("active_runs_count", &"<locked>")
            .field("llm_config", &self.llm_config)
            .field("classifier_config", &self.classifier_config)
            .finish_non_exhaustive()
    }
}

impl RunManager {
    pub async fn new(
        llm_config: LlmConfig,
        global_mcp: Arc<McpRegistry>,
        sessions: SessionStore,
        skills: Arc<RwLock<SkillRegistry>>,
        vector_matcher: Arc<crate::uar::runtime::matching::VectorMatcher>,
        persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
    ) -> Self {
        Self::with_classifier_config(
            llm_config,
            global_mcp,
            sessions,
            skills,
            vector_matcher,
            persistence,
            ClassifierConfig::default(),
            Arc::new(NativeSkillRegistry::new()),
        )
        .await
    }

    /// Creates a new RunManager with a custom classifier configuration.
    pub async fn with_classifier_config(
        llm_config: LlmConfig,
        global_mcp: Arc<McpRegistry>,
        sessions: SessionStore,
        skills: Arc<RwLock<SkillRegistry>>,
        vector_matcher: Arc<crate::uar::runtime::matching::VectorMatcher>,
        persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
        classifier_config: ClassifierConfig,
        native_skills: Arc<NativeSkillRegistry>,
    ) -> Self {
        // Initialize vector matcher if not already (caller should ideally do this)
        if let Err(e) = vector_matcher.initialize().await {
            tracing::error!("Failed to initialize VectorMatcher: {:?}", e);
        }

        let tag_matcher = Arc::new(crate::uar::runtime::matching::TagMatcher::new());
        let context_manager = Arc::new(ContextManager::new(ContextConfig::default()));

        // Create intent classifier based on config
        let intent_classifier: Arc<dyn IntentClassifier> =
            Arc::from(create_classifier(&classifier_config));

        // Build the classifier index from existing skills
        {
            let skills_registry = skills.read().await;
            if let Err(e) = intent_classifier.rebuild_index(&skills_registry).await {
                tracing::error!("Failed to build intent classifier index: {:?}", e);
            }
        }

        tracing::info!(
            backend = ?classifier_config.backend,
            topk = classifier_config.topk,
            accept_threshold = classifier_config.accept_threshold,
            "Intent classifier initialized"
        );

        Self {
            active_runs: Arc::new(RwLock::new(HashMap::new())),
            session_current_run: Arc::new(RwLock::new(HashMap::new())),
            llm_config,
            global_mcp,
            sessions,
            skills,
            vector_matcher,
            tag_matcher,
            context_manager,
            intent_classifier,
            classifier_config,
            persistence,
            skill_service: None,
            skill_evolution_config: SkillEvolutionConfig::default(),
            provider_registry: None,
            provider_service: None,
            native_skills,
            pending_approvals: Arc::new(Mutex::new(HashMap::new())),
            root_cancellation: CancellationToken::new(),
            run_cancellations: Arc::new(RwLock::new(HashMap::new())),
            message_context_strategy: crate::uar::context::ContextStrategy::default(),
            agent_graph: None,
            governance_engine: None,
            failover_config: crate::config::FailoverConfig::default(),
            cost_budget: crate::uar::runtime::cost_budget::CostBudgetTracker::new(),
            resilience_policy: crate::uar::settings::resilience_policy::ResiliencePolicy::default(),
        }
    }

    /// Attach an agent graph for graph-driven execution.
    ///
    /// When set, `start_run` executes the graph instead of the simple tool loop.
    #[must_use]
    pub fn with_agent_graph(mut self, graph: crate::uar::runtime::graph::AgentGraph) -> Self {
        self.agent_graph = Some(std::sync::Arc::new(graph));
        self
    }

    /// Set the message-count based context strategy from global config.
    #[must_use]
    pub fn with_message_context_strategy(
        mut self,
        strategy: crate::uar::context::ContextStrategy,
    ) -> Self {
        self.message_context_strategy = strategy;
        self
    }

    /// Set the skill service for coordinated skill management.
    pub fn with_skill_service(mut self, service: Arc<SkillService>) -> Self {
        self.skill_service = Some(service);
        self
    }

    /// Configure the Hermes skill evolution post-run hook.
    pub fn with_skill_evolution_config(mut self, cfg: SkillEvolutionConfig) -> Self {
        self.skill_evolution_config = cfg;
        self
    }

    /// Set the provider registry for per-agent LLM provider resolution.
    pub fn with_provider_registry(mut self, registry: Arc<crate::llm::ProviderRegistry>) -> Self {
        self.provider_registry = Some(registry);
        self
    }

    /// Set the runtime model failover configuration (CH-03). When
    /// `enabled`, each run's `Orchestrator` gets a fallback driver built from
    /// `fallback_models.first()` plus the shared provider-health monitor.
    #[must_use]
    pub fn with_failover_config(mut self, config: crate::config::FailoverConfig) -> Self {
        self.failover_config = config;
        self
    }

    /// Configure bounded provider retry and stream-start behavior for runs.
    #[must_use]
    pub fn with_resilience_policy(
        mut self,
        policy: crate::uar::settings::resilience_policy::ResiliencePolicy,
    ) -> Self {
        self.resilience_policy = policy;
        self
    }

    /// Configure the global spend ceiling (CH-06) from `LlmConfig.budget`.
    /// A no-op when `budget` is `None` (global scope is left unlimited —
    /// `CostBudgetTracker`'s default `BudgetLimit` is `f64::INFINITY`).
    pub async fn with_global_cost_budget(
        self,
        budget: Option<&crate::config::LlmBudgetConfig>,
    ) -> Self {
        if let Some(budget) = budget {
            self.cost_budget
                .set_limit(
                    crate::uar::runtime::cost_budget::BudgetScope::Global,
                    "global",
                    crate::uar::runtime::cost_budget::BudgetLimit {
                        limit_usd: budget.global_limit,
                        warn_at: 0.8,
                    },
                )
                .await;
        }
        self
    }

    /// Attach the multi-tenant credential service. When set, `start_run`
    /// resolves a per-user/session/agent provider key and overrides the
    /// run's `api_key`. When unset, the env/config key is used (single-tenant).
    #[must_use]
    pub fn with_provider_service(
        mut self,
        service: Arc<crate::uar::security::credentials::ProviderService>,
    ) -> Self {
        self.provider_service = Some(service);
        self
    }

    /// Attach the Cedar governance engine consulted at the tool-approval gate.
    #[must_use]
    pub fn with_governance_engine(
        mut self,
        engine: Arc<crate::uar::governance::engine::GovernanceEngine>,
    ) -> Self {
        self.governance_engine = Some(engine);
        self
    }

    /// Set a shared native skill registry for in-process tool execution.
    pub fn with_native_skills(mut self, registry: Arc<NativeSkillRegistry>) -> Self {
        self.native_skills = registry;
        self
    }

    /// Resolve a pending tool-call approval for the given run.
    /// Returns `true` if an approval was pending and the decision was delivered,
    /// `false` if no pending approval was found for that run_id.
    pub async fn resolve_approval(&self, run_id: &str, approved: bool) -> bool {
        resolve_pending_approval(&self.pending_approvals, run_id, approved).await
    }

    /// Cancel an in-flight run.
    ///
    /// Cancels the run's cancellation token (which aborts the in-flight LLM
    /// stream and tool execution at the next await point) and resolves any
    /// pending tool approval as aborted so a run parked on the approval gate
    /// unblocks promptly. Returns `true` if a live run was found and cancelled,
    /// `false` for an unknown or already-terminal run (idempotent).
    pub async fn cancel_run(&self, run_id: &str) -> bool {
        let token = {
            let cancels = self.run_cancellations.read().await;
            cancels.get(run_id).cloned()
        };
        let Some(token) = token else {
            return false;
        };
        // Drop any lingering approval sender so the gate future resolves; the
        // token cancellation below also drops the orchestrator stream.
        {
            let mut approvals = self.pending_approvals.lock().await;
            if let Some(tx) = approvals.remove(run_id) {
                let _ = tx.send(false);
            }
        }
        token.cancel();
        tracing::info!(run_id = %run_id, "Run cancellation requested");
        true
    }

    /// A clone of the root cancellation token.
    ///
    /// Cancelling it aborts ALL in-flight runs at once; used to wire run
    /// cancellation into the server's graceful-shutdown path.
    #[must_use]
    pub fn root_cancellation_token(&self) -> CancellationToken {
        self.root_cancellation.clone()
    }

    #[instrument(
        skip(self, artifact, input, memory_hits),
        fields(
            agent_id = %artifact.id,
            session_id = ?session_id,
            user_id = ?user_id,
            memory_hits = memory_hits.len(),
            run_id = tracing::field::Empty
        )
    )]
    pub async fn start_run(
        &self,
        artifact: AgentArtifact,
        input: String,
        session_id: Option<String>,
        user_id: Option<String>,
        memory_hits: Vec<MemoryItem>,
    ) -> String {
        let run_id = Uuid::new_v4().to_string();
        tracing::Span::current().record("run_id", &run_id);
        tracing::info!("Starting new run");
        let (tx, _) = broadcast::channel(256); // Buffer size 256
        let history = Arc::new(Mutex::new(EventHistory {
            next_id: 1,
            buffer: VecDeque::with_capacity(EVENT_HISTORY_LIMIT),
        }));
        let emitter = RunEventEmitter {
            sender: tx.clone(),
            history: Arc::clone(&history),
        };

        // 1. Resolve Session
        let session = if let Some(id) = session_id {
            self.sessions.get_or_create(&id)
        } else {
            self.sessions.create()
        };

        // 2. Add User Message
        session.add_user_message(&input);

        // Capture identity for credential resolution before `user_id` is moved
        // into the Run record and the resolved session id is the source of truth.
        let user_id_for_creds = user_id.clone();
        let session_id_for_creds = Some(session.id().to_string());

        let run = Run {
            run_id: run_id.clone(),
            agent_id: artifact.id.clone(),
            conversation_id: Some(session.id().to_string()),
            user_id,
            status: RunStatus::Running,
            context: serde_json::json!({ "input": input }),
        };

        {
            let mut runs = self.active_runs.write().await;
            runs.insert(
                run_id.clone(),
                RunStreamState {
                    run,
                    sender: tx.clone(),
                    history: Arc::clone(&history),
                },
            );
        }
        {
            let mut session_runs = self.session_current_run.write().await;
            session_runs.insert(session.id().to_string(), run_id.clone());
        }

        // Per-run cancellation token, derived from the root so that a server
        // shutdown (which cancels the root) also aborts this run. `cancel_run`
        // and the client-disconnect guard cancel this token; the spawned task
        // selects on it and removes it from the map on any terminal state.
        let run_cancellation = self.root_cancellation.child_token();
        {
            let mut cancels = self.run_cancellations.write().await;
            cancels.insert(run_id.clone(), run_cancellation.clone());
        }

        // 3. Prepare Messages
        // We prioritize the Artifact's system prompt.
        let mut messages = Vec::new();
        let mut system_prompt = artifact.prompt.system.clone();

        // RAG Retrieval - scoped to agent's configured knowledge bases
        if artifact.memory.kb.enabled
            && let Some(db) = &self.persistence
        {
            match self.vector_matcher.embed_batch(vec![input.clone()]).await {
                Ok(embeddings) => {
                    if let Some(query_vec) = embeddings.first() {
                        // Get agent's configured KBs (or use all if empty)
                        let kb_names = &artifact.memory.kb.knowledge_bases;

                        let search_result = if kb_names.is_empty() {
                            // No specific KBs configured - search all
                            db.search_knowledge(query_vec, 3, 0.7).await
                        } else {
                            // Resolve KB names to IDs and search scoped
                            let mut kb_ids = Vec::new();
                            for name in kb_names {
                                if let Ok(Some(kb)) = db.get_knowledge_base_by_name(name).await {
                                    kb_ids.push(kb.id);
                                } else {
                                    tracing::warn!("Knowledge base not found: {}", name);
                                }
                            }

                            if kb_ids.is_empty() {
                                // All configured KBs were not found - fallback to all
                                tracing::warn!(
                                    "No configured knowledge bases found, searching all"
                                );
                                db.search_knowledge(query_vec, 3, 0.7).await
                            } else {
                                let kb_id_refs: Vec<&str> =
                                    kb_ids.iter().map(String::as_str).collect();
                                db.search_knowledge_scoped(&kb_id_refs, query_vec, 3, 0.7)
                                    .await
                            }
                        };

                        match search_result {
                            Ok(matches) => {
                                if !matches.is_empty() {
                                    system_prompt.push_str("\n\n[RELEVANT KNOWLEDGE]\n");
                                    for m in matches {
                                        let _ = writeln!(system_prompt, "- {}", m.chunk.content);
                                    }
                                }
                            }
                            Err(e) => tracing::error!("RAG search failed: {:?}", e),
                        }
                    }
                }
                Err(e) => tracing::error!("RAG embedding failed: {:?}", e),
            }
        }

        // SKILL INJECTION: Use SkillService if available, otherwise intent classifier
        let (matched_skills, skill_selection_method): (Vec<_>, String) = if let Some(
            ref skill_service,
        ) = self.skill_service
        {
            // Delegate to SkillService for coordinated matching.
            let agent_id = artifact.id.clone();
            let config = skill_service.get_matching_config().await;
            let selection_method = match config.algorithm {
                crate::uar::runtime::skills::service::SkillMatchingAlgorithm::Keyword => {
                    "skill_service.keyword"
                }
                crate::uar::runtime::skills::service::SkillMatchingAlgorithm::Embedding => {
                    "skill_service.embedding"
                }
                crate::uar::runtime::skills::service::SkillMatchingAlgorithm::Llm => {
                    "skill_service.llm"
                }
                crate::uar::runtime::skills::service::SkillMatchingAlgorithm::Hybrid => {
                    "skill_service.hybrid"
                }
                crate::uar::runtime::skills::service::SkillMatchingAlgorithm::LocalEmbedding => {
                    "skill_service.local_embedding"
                }
            }
            .to_string();
            (
                skill_service.match_skills(&input, Some(&agent_id)).await,
                selection_method,
            )
        } else {
            // Legacy path: use intent classifier directly.
            let skills_registry = self.skills.read().await;
            let backend_method = match self.classifier_config.backend {
                crate::uar::runtime::matching::ClassifierBackend::Rules => {
                    "legacy_classifier.rules"
                }
                crate::uar::runtime::matching::ClassifierBackend::Tfidf => {
                    "legacy_classifier.tfidf"
                }
                crate::uar::runtime::matching::ClassifierBackend::Wasm => "legacy_classifier.wasm",
                crate::uar::runtime::matching::ClassifierBackend::Hybrid => {
                    "legacy_classifier.hybrid"
                }
                crate::uar::runtime::matching::ClassifierBackend::LocalEmbedding => {
                    "legacy_classifier.local_embedding"
                }
                crate::uar::runtime::matching::ClassifierBackend::Llm => "legacy_classifier.llm",
            };

            let classification_result = self
                .intent_classifier
                .classify(&input, &[], &skills_registry)
                .await;

            match classification_result {
                Ok(result) => {
                    tracing::debug!(
                        scores = ?result.scores.iter().map(|s| (&s.label, s.score)).collect::<Vec<_>>(),
                        out_of_scope = result.out_of_scope,
                        "Intent classification complete"
                    );

                    let selected = if result.should_accept(
                        self.classifier_config.accept_threshold,
                        self.classifier_config.margin_threshold,
                    ) {
                        result
                            .scores
                            .into_iter()
                            .filter_map(|score| score.skill)
                            .collect()
                    } else if result.out_of_scope {
                        tracing::debug!("Query appears out-of-scope, no skills matched");
                        Vec::new()
                    } else {
                        tracing::debug!(
                            top_score = ?result.scores.first().map(|s| s.score),
                            threshold = self.classifier_config.accept_threshold,
                            "Classification below threshold, including top matches anyway"
                        );
                        result
                            .scores
                            .into_iter()
                            .filter_map(|score| score.skill)
                            .collect()
                    };

                    (selected, backend_method.to_string())
                }
                Err(e) => {
                    tracing::error!("Intent classification failed: {:?}", e);
                    let mut fallback_skills = HashMap::new();

                    if let Ok(matches) = crate::uar::domain::matching::SkillMatcher::match_skills(
                        self.tag_matcher.as_ref(),
                        &input,
                        &skills_registry,
                    )
                    .await
                    {
                        for m in matches {
                            fallback_skills.insert(m.skill_id.clone(), m.skill);
                        }
                    }

                    if let Ok(matches) = crate::uar::domain::matching::SkillMatcher::match_skills(
                        self.vector_matcher.as_ref(),
                        &input,
                        &skills_registry,
                    )
                    .await
                    {
                        for m in matches {
                            fallback_skills.entry(m.skill_id.clone()).or_insert(m.skill);
                        }
                    }

                    (
                        fallback_skills.into_values().collect(),
                        "legacy_fallback.tag_vector_hybrid".to_string(),
                    )
                }
            }
        };

        // Collect registries to merge (starting with global)
        let mut registries_to_merge = Vec::new();
        // CH-08: record which MCP server(s) each matched skill introduces,
        // captured before merge (the merged registry no longer distinguishes
        // which skill contributed which server). Skills with no `mcp_config`
        // (prompt-overlay-only) are deliberately absent from this map — they
        // have no distinguishable "used" signal at the tool-call layer, so
        // they're excluded from outcome tracking entirely rather than given a
        // proxy signal.
        let mut skill_servers: HashMap<String, Vec<String>> = HashMap::new();

        for skill in &matched_skills {
            // Append skill prompt overlay
            system_prompt.push_str("\n\n[SKILL: ");
            system_prompt.push_str(&skill.title);
            system_prompt.push_str("]\n");
            system_prompt.push_str(&skill.prompt_overlay);

            // Init Skill Tools
            if let Some(config) = &skill.mcp_config {
                match McpRegistry::from_config(config).await {
                    Ok(reg) => {
                        skill_servers
                            .entry(skill.skill_id.clone())
                            .or_default()
                            .extend(reg.server_names());
                        registries_to_merge.push(reg);
                    }
                    Err(e) => {
                        tracing::error!("Failed to init tools for skill {}: {:?}", skill.title, e);
                    }
                }
            }
        }

        messages.push(Message {
            role: MessageRole::System,
            content: crate::llm::MessageContent::text(system_prompt),
            tool_call_id: None,
            tool_calls: None,
        });
        messages.extend(session.messages());

        // Message-count context strategy (trim history before token-budget management).
        //
        // CH-05: `Auto` resolves against this deployment's default model's
        // cataloged context window (an approximation — the per-agent-resolved
        // model isn't known yet at this point in the pipeline). Real
        // Summarize/Hierarchical behavior needs an LLM call, so a lightweight
        // driver is built from the same default config only when the
        // resolved strategy actually needs one.
        let effective_strategy = {
            let (provider_id, model_id) =
                crate::llm::registry::split_model_string_pub(&self.llm_config.model);
            let effective_context_tokens = crate::llm::catalog::ModelCatalog::global()
                .model(&provider_id, &model_id)
                .map(|m| (m.limits.context_window as f64 * 0.7) as u32);
            crate::uar::context::resolve_effective_strategy(
                &self.message_context_strategy,
                effective_context_tokens,
            )
        };
        let summarization_driver: Option<crate::llm::LiterLlmDriver> = match &effective_strategy {
            crate::uar::context::ContextStrategy::Summarize { .. }
            | crate::uar::context::ContextStrategy::Hierarchical { .. } => {
                crate::llm::LiterLlmDriver::new(
                    crate::config::build_client_config(&self.llm_config),
                    self.llm_config.model.clone(),
                    self.llm_config.parallel_tool_calls,
                )
                .ok()
            }
            _ => None,
        };
        let messages = crate::uar::context::trim_with_summarization(
            messages,
            &effective_strategy,
            summarization_driver
                .as_ref()
                .map(|d| d as &dyn crate::llm::LlmDriver),
        )
        .await;

        // Token-budget context management (summarization, etc.)
        let (optimized_messages, context_action) =
            self.context_manager.apply(messages, 128_000).await;
        let messages = optimized_messages;
        if let Some(act) = context_action {
            emitter.emit(NormalizedEvent::ContextAction(act)).await;
        }
        for skill in &matched_skills {
            emitter
                .emit(NormalizedEvent::SkillActivated {
                    run_id: run_id.clone(),
                    skill_id: skill.skill_id.clone(),
                    title: skill.title.clone(),
                    selection_method: skill_selection_method.clone(),
                })
                .await;
        }

        // Spawn async execution task
        // Create per-run Orchestrator.

        // Merge registries
        let mut final_mcp = (*self.global_mcp).clone();
        for reg in registries_to_merge {
            final_mcp = final_mcp.merge(&reg);
        }
        let mcp = Arc::new(final_mcp);

        // Resolve per-agent LLM config via provider registry, falling back to global
        let run_llm_config = if let Some(ref registry) = self.provider_registry {
            match registry
                .resolve_llm_config_from_policy(&artifact.policy.provider)
                .await
            {
                Some(resolved) => {
                    tracing::info!(
                        provider = %artifact.policy.provider.default.provider,
                        model = %artifact.policy.provider.default.model,
                        "Using per-agent provider settings"
                    );
                    resolved
                }
                None => {
                    tracing::debug!("No provider match for agent policy, using global settings");
                    self.llm_config.clone()
                }
            }
        } else {
            self.llm_config.clone()
        };

        // Apply per-skill LLM execution overrides (first matched skill wins).
        let run_llm_config = {
            let mut cfg = run_llm_config;
            for skill in &matched_skills {
                let ec = &skill.execution_config;
                if let Some(ref model) = ec.preferred_model {
                    tracing::info!(
                        skill_id = %skill.skill_id,
                        model = %model,
                        "Skill overrides LLM model"
                    );
                    cfg.model = model.clone();
                    break;
                }
            }
            cfg
        };

        // Apply the multi-tenant credential layer (first match wins:
        // session → agent → user → system). When no ProviderService is
        // configured, or no per-scope credential exists, the env/config key on
        // `cfg.api_key` is left untouched — i.e. single-tenant behavior.
        let run_llm_config = apply_credential_layer(
            run_llm_config,
            self.provider_service.as_ref(),
            user_id_for_creds.as_deref(),
            session_id_for_creds.as_deref(),
            artifact.id.as_str(),
        )
        .await;

        // Clone for graph context before values are moved into Orchestrator
        let llm_config_for_graph = run_llm_config.clone();
        let mcp_for_graph = Arc::clone(&mcp);
        // CH-08: clone for activation-outcome correlation at run end (resolves
        // invoked tool names back to their owning MCP server).
        let mcp_for_outcome = Arc::clone(&mcp);

        // Capture the resolved model id so the final RunDoneWithUsage event can
        // report which model actually answered (moved into the spawned task below).
        let run_model = run_llm_config.model.clone();
        // Whether to compute per-request cost (captured before run_llm_config moves).
        let cost_tracking_enabled = run_llm_config.cost_tracking;

        let orchestrator = match Orchestrator::new(
            run_llm_config,
            mcp,
            Arc::clone(&self.native_skills),
        ) {
            Ok(o) => {
                // CH-03: attach the shared provider-health monitor (always,
                // when a registry is configured) and, if failover is enabled,
                // a fallback driver built from the first configured fallback
                // model (`FailoverStrategy::Priority`).
                let o = if self.failover_config.enabled {
                    match self.failover_config.fallback_models.first() {
                        Some(fallback_model) => {
                            match crate::llm::Orchestrator::build_fallback_driver(
                                &llm_config_for_graph,
                                fallback_model,
                            ) {
                                Ok(fallback_driver) => {
                                    o.with_failover(fallback_driver, self.failover_config.clone())
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        error = %e,
                                        "Failed to build failover fallback driver; continuing without failover"
                                    );
                                    o
                                }
                            }
                        }
                        None => o,
                    }
                } else {
                    o
                };
                let o = match &self.provider_registry {
                    Some(registry) => o.with_health_monitor(Arc::clone(registry.health())),
                    None => o,
                }
                .with_resilience_policy(self.resilience_policy.clone());

                // Wire up tool approval gate
                let approval_run_id = run_id.clone();
                let approval_emitter = emitter.clone();
                let approval_pending = Arc::clone(&self.pending_approvals);
                let approval_agent_id = artifact.id.clone();
                let approval_governance = self.governance_engine.clone();
                let gate: crate::llm::ToolApprovalGate = Arc::new(
                    move |tool_call_id, tool_name, arguments_json, call_index| {
                        let run_id = approval_run_id.clone();
                        let emitter = approval_emitter.clone();
                        let pending = Arc::clone(&approval_pending);
                        let agent_id = approval_agent_id.clone();
                        let governance = approval_governance.clone();
                        Box::pin(async move {
                            let heuristic_flag = tool_requires_approval(&tool_name);
                            let decision = match &governance {
                                Some(engine) => engine
                                    .tool_decision(&agent_id, &tool_name, heuristic_flag)
                                    .await,
                                None if heuristic_flag => crate::uar::governance::engine::ToolGovernanceDecision::RequireApproval,
                                None => crate::uar::governance::engine::ToolGovernanceDecision::Allow,
                            };
                            match decision {
                                crate::uar::governance::engine::ToolGovernanceDecision::Allow => {
                                    return crate::llm::ToolApprovalResult::Approved;
                                }
                                crate::uar::governance::engine::ToolGovernanceDecision::Deny => {
                                    let reason = format!("Tool '{tool_name}' is denied by governance policy");
                                    emitter.emit(NormalizedEvent::ToolCallDenied {
                                        run_id: run_id.clone(),
                                        call_index,
                                        tool_call_id: tool_call_id.clone(),
                                        name: tool_name.clone(),
                                        reason: reason.clone(),
                                    }).await;
                                    return crate::llm::ToolApprovalResult::Rejected { reason };
                                }
                                crate::uar::governance::engine::ToolGovernanceDecision::RequireApproval => {}
                            }
                            let risk_reason = format!(
                                "Tool '{tool_name}' may perform a destructive or write operation"
                            );
                            // Emit approval-required event to the client
                            emitter
                                .emit(NormalizedEvent::ToolCallApprovalRequired {
                                    run_id: run_id.clone(),
                                    call_index,
                                    tool_call_id: tool_call_id.clone(),
                                    name: tool_name.clone(),
                                    arguments_json: arguments_json.clone(),
                                    risk_reason: risk_reason.clone(),
                                })
                                .await;
                            // Create oneshot channel and wait for approval
                            let (tx, rx) = oneshot::channel();
                            {
                                let mut approvals = pending.lock().await;
                                approvals.insert(run_id.clone(), tx);
                            }
                            // Wait with 5-minute timeout; auto-reject on timeout or channel error
                            match await_approval(rx, Duration::from_secs(300)).await {
                                ApprovalWaitOutcome::Approved => {
                                    tracing::info!(run_id = %run_id, tool = %tool_name, "Tool call approved by user");
                                    crate::llm::ToolApprovalResult::Approved
                                }
                                ApprovalWaitOutcome::Rejected => {
                                    tracing::info!(run_id = %run_id, tool = %tool_name, "Tool call rejected by user");
                                    crate::llm::ToolApprovalResult::Rejected {
                                        reason: "Rejected by user".to_string(),
                                    }
                                }
                                ApprovalWaitOutcome::ChannelClosed => {
                                    tracing::warn!(run_id = %run_id, tool = %tool_name, "Approval channel dropped");
                                    crate::llm::ToolApprovalResult::Rejected {
                                        reason: "Approval channel closed".to_string(),
                                    }
                                }
                                ApprovalWaitOutcome::TimedOut => {
                                    // Timeout — clean up the pending entry
                                    let mut approvals = pending.lock().await;
                                    approvals.remove(&run_id);
                                    tracing::warn!(run_id = %run_id, tool = %tool_name, "Tool approval timed out after 5 minutes");
                                    crate::llm::ToolApprovalResult::Rejected {
                                        reason: "Approval timed out after 5 minutes".to_string(),
                                    }
                                }
                            }
                        })
                    },
                );
                Arc::new(o.with_tool_approval_gate(gate))
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to create orchestrator");
                return run_id;
            }
        };

        let execute_run_id = run_id.clone();
        let execute_agent_id = artifact.id.clone();
        let emitter = emitter.clone();
        let execution_session = session.clone();
        let skill_service_for_evolution = self.skill_service.clone();
        let skill_evolution_cfg = self.skill_evolution_config.clone();
        let cost_budget_for_run = self.cost_budget.clone();
        // `execute_agent_id` is moved into the `RunStart` event below; keep an
        // independent clone for the budget-scope keys used at run end (CH-06).
        let cost_scope_agent_id = execute_agent_id.clone();
        // CH-06: `budgets` has no typed home on `AgentPolicy` (see
        // `to_artifact.rs`'s module doc) — it's preserved losslessly under
        // `extensions["budgets"]` as JSON. Configure the agent-scoped limit
        // from there if the spec declared one. Re-set on every run rather
        // than caching "have we configured this agent" — `set_limit` is a
        // single `HashMap` insert behind a `RwLock`, cheap enough that a
        // cache would be premature complexity.
        if let Some(limit_usd) = agent_cost_limit_from_extensions(&artifact.extensions) {
            self.cost_budget
                .set_limit(
                    crate::uar::runtime::cost_budget::BudgetScope::Agent,
                    &cost_scope_agent_id,
                    crate::uar::runtime::cost_budget::BudgetLimit {
                        limit_usd,
                        warn_at: 0.8,
                    },
                )
                .await;
        }
        let graph_for_run = self.agent_graph.clone();
        let persistence_for_run = self.persistence.clone();
        // Cancellation: the run's child token (selected on in the consumption loop,
        // moved into the task below) and the registry used to deregister it on
        // terminal state.
        let cancellations_for_cleanup = Arc::clone(&self.run_cancellations);
        let cleanup_run_id = run_id.clone();

        // Run-level span: child LLM-call and tool-call spans created within the
        // task attach under this, producing a run → llm → tool trace tree.
        let run_span = tracing::info_span!(
            "run",
            run_id = %execute_run_id,
            agent_id = %execute_agent_id,
        );

        tokio::spawn(
            async move {
            // 1. Run Start
            emitter
                .emit(NormalizedEvent::RunStart {
                    run_id: execute_run_id.clone(),
                    agent_id: execute_agent_id,
                })
                .await;

            // Counter for skill evolution — tracks tool completions across the full run.
            let mut tool_call_count: usize = 0;

            // 2. Emit pre-call memory recall hits so the client knows what context was injected.
            if !memory_hits.is_empty() {
                tracing::debug!(
                    run_id = %execute_run_id,
                    count = memory_hits.len(),
                    "Emitting pre-call memory recall hits"
                );
                emitter
                    .emit(NormalizedEvent::MemoryRecall {
                        run_id: execute_run_id.clone(),
                        items: memory_hits,
                    })
                    .await;
            }

            emitter
                .emit(NormalizedEvent::StatePatch {
                    run_id: execute_run_id.clone(),
                    patch: vec![StatePatchOp {
                        op: "replace".to_string(),
                        path: "/run".to_string(),
                        value: Some(serde_json::json!({
                            "run_id": execute_run_id.clone(),
                            "conversation_id": execution_session.id(),
                            "status": "running"
                        })),
                    }],
                })
                .await;

            // Graph execution branch — runs instead of the simple tool loop when a
            // graph is attached. On completion we emit RunEnd and return early.
            if let Some(graph) = graph_for_run {
                let graph_driver: std::sync::Arc<dyn crate::llm::LlmDriver> = {
                    let client_cfg = crate::config::build_client_config(&llm_config_for_graph);
                    match LiterLlmDriver::new(
                        client_cfg,
                        llm_config_for_graph.model.clone(),
                        llm_config_for_graph.parallel_tool_calls,
                    ) {
                        Ok(d) => std::sync::Arc::new(d),
                        Err(e) => {
                            tracing::error!(
                                run_id = %execute_run_id,
                                error = %e,
                                "Failed to create LLM driver for graph execution"
                            );
                            emitter
                                .emit(NormalizedEvent::RunDone {
                                    run_id: execute_run_id.clone(),
                                })
                                .await;
                            return;
                        }
                    }
                };

                let graph_ctx = crate::uar::runtime::graph::GraphContext {
                    run_id: execute_run_id.clone(),
                    session_id: Some(execution_session.id().to_string()),
                    mcp: mcp_for_graph,
                    llm_config: llm_config_for_graph,
                    driver: graph_driver,
                    persistence: persistence_for_run.clone(),
                };

                let mut initial_state = crate::uar::runtime::graph::GraphState::default();
                // Seed state with the incoming messages so LlmNode can use them.
                for msg in &messages {
                    initial_state
                        .messages
                        .push(serde_json::to_value(msg).unwrap_or_default());
                }

                let final_state = tokio::select! {
                    biased;
                    () = run_cancellation.cancelled() => {
                        tracing::info!(run_id = %execute_run_id, "Run cancelled during graph execution");
                        emitter
                            .emit(NormalizedEvent::Cancelled {
                                run_id: execute_run_id.clone(),
                            })
                            .await;
                        cancellations_for_cleanup.write().await.remove(&cleanup_run_id);
                        return;
                    }
                    state = graph.execute(initial_state, &graph_ctx) => state,
                };

                if let Some(err) = final_state.get::<String>("_error") {
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: execute_run_id.clone(),
                            message: err,
                            code: String::new(),
                        })
                        .await;
                }
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: execute_run_id.clone(),
                    })
                    .await;
                return;
            }

            let mut accumulated_content = String::new();
            let mut run_cancelled = false;
            let mut accumulated_tool_calls: Vec<crate::llm::ToolCall> = Vec::new();
            let mut tool_call_indices: HashMap<String, usize> = HashMap::new();
            let mut tool_call_names: HashMap<String, String> = HashMap::new();

            // Token usage tracking — accumulated across all LLM calls in this run.
            let mut total_input_tokens: u32 = 0;
            let mut total_output_tokens: u32 = 0;
            let mut total_cache_read_tokens: u32 = 0;

            // 2. Execute Orchestrator
            match orchestrator.chat_with_history(messages).await {
                Ok(stream) => {
                    futures::pin_mut!(stream);
                    loop {
                        // Cancellation seam: selecting the run token against the
                        // orchestrator stream means a cancel drops the in-flight
                        // `next()` future — which drops the orchestrator's current
                        // await (LLM stream, tool call, or approval gate), aborting
                        // it cooperatively at the next suspension point.
                        let base_event = tokio::select! {
                            biased;
                            () = run_cancellation.cancelled() => {
                                run_cancelled = true;
                                break;
                            }
                            next = stream.next() => match next {
                                Some(ev) => ev,
                                None => break,
                            },
                        };
                        // Map base NormalizedEvent to domain NormalizedEvent with run_id
                        let uar_event = match base_event {
                            crate::normalized::NormalizedEvent::MessageDelta { text } => {
                                accumulated_content.push_str(&text);
                                Some(NormalizedEvent::ChatDelta {
                                    run_id: execute_run_id.clone(),
                                    text_delta: text,
                                })
                            }
                            crate::normalized::NormalizedEvent::ThinkingDelta { text } => {
                                Some(NormalizedEvent::ThinkingDelta {
                                    run_id: execute_run_id.clone(),
                                    text_delta: text,
                                })
                            }
                            crate::normalized::NormalizedEvent::ReasoningDelta { text } => {
                                Some(NormalizedEvent::ReasoningDelta {
                                    run_id: execute_run_id.clone(),
                                    text_delta: text,
                                })
                            }
                            crate::normalized::NormalizedEvent::RuntimeStep { step, kind } => {
                                let kind = match kind {
                                    crate::normalized::RuntimeStepKind::Started => "started",
                                    crate::normalized::RuntimeStepKind::Finished => "finished",
                                };
                                Some(NormalizedEvent::RuntimeStep {
                                    run_id: execute_run_id.clone(),
                                    step,
                                    kind: kind.to_string(),
                                })
                            }
                            crate::normalized::NormalizedEvent::CitationAdded(citation) => {
                                Some(NormalizedEvent::Citation {
                                    run_id: execute_run_id.clone(),
                                    sources: vec![CitationSource {
                                        title: citation
                                            .title
                                            .unwrap_or_else(|| citation.url.clone()),
                                        url: citation.url,
                                        snippet: citation.snippet,
                                    }],
                                })
                            }
                            crate::normalized::NormalizedEvent::MemoryUpdate {
                                key,
                                value,
                                operation,
                            } => Some(NormalizedEvent::MemoryRecall {
                                run_id: execute_run_id.clone(),
                                items: vec![MemoryItem {
                                    key,
                                    scope: None,
                                    memory_type: None,
                                    importance: None,
                                    value,
                                    source: operation,
                                }],
                            }),
                            crate::normalized::NormalizedEvent::ToolCallDelta {
                                call_index,
                                id,
                                name,
                                arguments_delta,
                            } => {
                                if let (Some(tid), Some(delta)) = (id, arguments_delta) {
                                    tool_call_indices.insert(tid.clone(), call_index);
                                    if let Some(tool_name) = name {
                                        tool_call_names.insert(tid.clone(), tool_name);
                                    }
                                    Some(NormalizedEvent::ToolDelta {
                                        run_id: execute_run_id.clone(),
                                        call_index,
                                        tool_call_id: tid,
                                        delta: serde_json::Value::String(delta),
                                    })
                                } else {
                                    None
                                }
                            }
                            crate::normalized::NormalizedEvent::ToolCallComplete {
                                call_index,
                                id,
                                name,
                                arguments_json,
                            } => {
                                tool_call_indices.insert(id.clone(), call_index);
                                tool_call_names.insert(id.clone(), name.clone());
                                accumulated_tool_calls.push(crate::llm::ToolCall {
                                    id: id.clone(),
                                    call_type: "function".to_string(),
                                    function: crate::llm::ToolCallFunction {
                                        name: name.clone(),
                                        arguments: arguments_json.clone(),
                                    },
                                });

                                Some(NormalizedEvent::ToolStart {
                                    run_id: execute_run_id.clone(),
                                    call_index,
                                    tool_call_id: id,
                                    tool: name,
                                    input: serde_json::from_str(&arguments_json)
                                        .unwrap_or(serde_json::Value::String(arguments_json)),
                                })
                            }
                            crate::normalized::NormalizedEvent::ToolResult {
                                id,
                                name: _,
                                content,
                                success,
                            } => {
                                if !accumulated_content.is_empty()
                                    || !accumulated_tool_calls.is_empty()
                                {
                                    execution_session.add_assistant_with_tool_calls(
                                        if accumulated_content.is_empty() {
                                            None
                                        } else {
                                            Some(accumulated_content.clone())
                                        },
                                        accumulated_tool_calls.clone(),
                                    );
                                    accumulated_content.clear();
                                    accumulated_tool_calls.clear();
                                }

                                execution_session.add_tool_result(id.clone(), content.clone());
                                let call_index = tool_call_indices.get(&id).copied().unwrap_or(0);
                                let tool = tool_call_names
                                    .get(&id)
                                    .cloned()
                                    .unwrap_or_else(|| "tool".to_string());

                                Some(NormalizedEvent::ToolEnd {
                                    run_id: execute_run_id.clone(),
                                    call_index,
                                    tool_call_id: id,
                                    tool,
                                    output: serde_json::from_str(&content)
                                        .unwrap_or(serde_json::Value::String(content)),
                                    ok: success,
                                })
                            }
                            crate::normalized::NormalizedEvent::Error { message, code } => {
                                Some(NormalizedEvent::Error {
                                    run_id: execute_run_id.clone(),
                                    message,
                                    code: code.unwrap_or_default(),
                                })
                            }
                            crate::normalized::NormalizedEvent::Usage {
                                prompt_tokens,
                                completion_tokens,
                                total_tokens: _,
                                cached_tokens,
                                ..
                            } => {
                                total_input_tokens =
                                    total_input_tokens.saturating_add(prompt_tokens);
                                total_output_tokens =
                                    total_output_tokens.saturating_add(completion_tokens);
                                // CH-06: cache-read tokens are billed at a discounted
                                // rate (`ModelCost::compute`) — accumulate them so the
                                // run-level cost estimate below isn't overcharged.
                                total_cache_read_tokens = total_cache_read_tokens
                                    .saturating_add(cached_tokens.unwrap_or(0));
                                None // Accumulate — emit on RunDone
                            }
                            _ => None, // Ignore other events for now
                        };

                        if let Some(evt) = uar_event {
                            // Track tool completions for the skill evolution hook.
                            if matches!(evt, NormalizedEvent::ToolEnd { .. }) {
                                tool_call_count += 1;

                                // Asynchronously persist a checkpoint after each tool call.
                                if let Some(db) = persistence_for_run.clone() {
                                    let cp = crate::uar::runtime::checkpoint::Checkpoint {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        run_id: execute_run_id.clone(),
                                        thread_id: execution_session.id().to_string(),
                                        node_id: format!("tool_loop_{tool_call_count}"),
                                        iteration: tool_call_count as u32,
                                        state: serde_json::Value::Null,
                                        messages: vec![],
                                        created_at: chrono::Utc::now().to_rfc3339(),
                                    };
                                    tokio::spawn(async move {
                                        if let Err(e) = db.save_checkpoint(&cp).await {
                                            tracing::warn!(
                                                error = %e,
                                                "Failed to save tool-loop checkpoint"
                                            );
                                        }
                                    });
                                }
                            }
                            // Derive a memory mutation event before consuming the ToolEnd.
                            let mutation_evt = memory_mutation_from_tool_end(&evt, &execute_run_id);
                            emitter.emit(evt).await;
                            if let Some(m) = mutation_evt {
                                emitter.emit(m).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: execute_run_id.clone(),
                            message: e.to_string(),
                            code: String::new(),
                        })
                        .await;
                }
            }

            if !accumulated_content.is_empty() {
                execution_session.add_assistant_message(accumulated_content);
            }

            let total_tokens = total_input_tokens.saturating_add(total_output_tokens);
            let has_usage = total_input_tokens > 0 || total_output_tokens > 0;

            // Preserve run_id before it is moved into the RunDone event below.
            let evolution_run_id = execute_run_id.clone();

            // CH-08: correlate matched-skill activation against actually-
            // invoked tools, once per run regardless of cancellation/usage/
            // cost-tracking status. Skills absent from `skill_servers` (no
            // `mcp_config`, i.e. prompt-overlay-only) have no distinguishable
            // "used" signal at this layer and are deliberately excluded from
            // outcome tracking — not given a proxy `false`.
            let invoked_tool_servers: HashSet<String> = tool_call_names
                .values()
                .filter_map(|tool_name| mcp_for_outcome.resolve_mcp_tool(tool_name))
                .map(|(server_name, _)| server_name)
                .collect();
            for (skill_id, used) in
                correlate_skill_activation_outcomes(&skill_servers, &invoked_tool_servers)
            {
                crate::uar::telemetry::metrics::record_skill_activation_outcome(&skill_id, used);
            }

            if run_cancelled {
                tracing::info!(run_id = %execute_run_id, "Run cancelled; emitting terminal Cancelled event");
                emitter
                    .emit(NormalizedEvent::Cancelled {
                        run_id: execute_run_id,
                    })
                    .await;
            } else if has_usage {
                // Compute estimated USD cost from the pricing catalog when cost
                // tracking is enabled; None when disabled or the model is unpriced.
                let cost_usd_estimate = if cost_tracking_enabled {
                    crate::llm::catalog::estimate_cost(
                        &run_model,
                        u64::from(total_input_tokens),
                        u64::from(total_output_tokens),
                        u64::from(total_cache_read_tokens),
                    )
                } else {
                    None
                };
                if let Some(cost) = cost_usd_estimate
                    && let Some((provider, model_id)) = run_model.split_once('/')
                {
                    crate::uar::telemetry::metrics::record_llm_cost(provider, model_id, cost);

                    // CH-06: aggregate spend across every scope and surface a
                    // `BudgetAlert` for the first scope (in priority order)
                    // that crosses its configured threshold. Unconfigured
                    // scopes have an unlimited `BudgetLimit::default()`, so
                    // `record` is a cheap no-op warning check for them.
                    // `BudgetScope::Task` is intentionally omitted — this
                    // runtime has no task entity distinct from a run.
                    use crate::uar::runtime::cost_budget::{BudgetScope, BudgetStatus};
                    let scopes: [(BudgetScope, &str); 3] = [
                        (BudgetScope::Run, execute_run_id.as_str()),
                        (BudgetScope::Session, execution_session.id()),
                        (BudgetScope::Agent, cost_scope_agent_id.as_str()),
                    ];
                    let mut alert: Option<(BudgetScope, String, f64, f64, bool)> = None;
                    for (scope, scope_id) in scopes {
                        let status = cost_budget_for_run.record(scope, scope_id, cost).await;
                        // CH-07: durable roll-up, fire-and-forget so the hot
                        // path never blocks on a DB write — mirrors the
                        // existing per-tool-call checkpoint persist pattern
                        // above.
                        if let Some(db) = persistence_for_run.clone() {
                            let scope_str = scope.as_str().to_string();
                            let scope_id_owned = scope_id.to_string();
                            tokio::spawn(async move {
                                if let Err(e) =
                                    db.record_cost_entry(&scope_str, &scope_id_owned, cost).await
                                {
                                    tracing::warn!(error = %e, scope = %scope_str, "Failed to persist cost ledger entry");
                                }
                            });
                        }
                        if alert.is_none()
                            && let BudgetStatus::Warning {
                                spent_usd,
                                limit_usd,
                            }
                            | BudgetStatus::Exceeded {
                                spent_usd,
                                limit_usd,
                            } = status
                        {
                            alert = Some((
                                scope,
                                scope_id.to_string(),
                                spent_usd,
                                limit_usd,
                                status.is_exceeded(),
                            ));
                        }
                    }
                    let global_status = cost_budget_for_run
                        .record(BudgetScope::Global, "global", cost)
                        .await;
                    if let Some(db) = persistence_for_run.clone() {
                        tokio::spawn(async move {
                            if let Err(e) = db.record_cost_entry("global", "global", cost).await {
                                tracing::warn!(error = %e, "Failed to persist cost ledger entry (global)");
                            }
                        });
                    }
                    if alert.is_none()
                        && let BudgetStatus::Warning {
                            spent_usd,
                            limit_usd,
                        }
                        | BudgetStatus::Exceeded {
                            spent_usd,
                            limit_usd,
                        } = global_status
                    {
                        alert = Some((
                            BudgetScope::Global,
                            "global".to_string(),
                            spent_usd,
                            limit_usd,
                            global_status.is_exceeded(),
                        ));
                    }
                    if let Some((scope, scope_id, spent_usd, limit_usd, exceeded)) = alert {
                        emitter
                            .emit(NormalizedEvent::BudgetAlert {
                                run_id: execute_run_id.clone(),
                                scope: scope.as_str().to_string(),
                                scope_id,
                                spent_usd,
                                limit_usd,
                                exceeded,
                            })
                            .await;
                    }
                }
                emitter
                    .emit(NormalizedEvent::RunDoneWithUsage {
                        run_id: execute_run_id,
                        input_tokens: Some(total_input_tokens),
                        output_tokens: Some(total_output_tokens),
                        total_tokens: Some(total_tokens),
                        cost_usd_estimate,
                        model: Some(run_model),
                    })
                    .await;
            } else {
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: execute_run_id,
                    })
                    .await;
            }

            // Deregister the run's cancellation token now that it is terminal, so
            // finished runs do not accumulate and a later cancel is a no-op.
            cancellations_for_cleanup
                .write()
                .await
                .remove(&cleanup_run_id);

            // ── Skill evolution (Hermes learning cycle) ──────────────────────────
            // Fire a background reflection task when evolution is enabled and the
            // run performed enough tool calls to be worth analysing.
            if skill_evolution_cfg.enabled && tool_call_count >= skill_evolution_cfg.min_tool_calls
            {
                if let Some(svc) = skill_service_for_evolution {
                    let run_id_ev = evolution_run_id.clone();
                    let cfg_ev = skill_evolution_cfg.clone();
                    tokio::spawn(async move {
                        tracing::info!(
                            run_id = %run_id_ev,
                            tool_calls = tool_call_count,
                            "Triggering skill evolution reflection"
                        );
                        if let Err(e) = svc
                            .evolve_from_run(&run_id_ev, tool_call_count, &cfg_ev)
                            .await
                        {
                            tracing::warn!(
                                run_id = %run_id_ev,
                                error = %e,
                                "Skill evolution reflection failed"
                            );
                        }
                    });
                } else {
                    tracing::debug!(
                        run_id = %evolution_run_id,
                        "Skill evolution enabled but no SkillService configured — skipping"
                    );
                }
            }
            }
            .instrument(run_span),
        );

        run_id
    }

    pub async fn subscribe(&self, run_id: &str) -> Option<broadcast::Receiver<StreamEvent>> {
        let runs = self.active_runs.read().await;
        runs.get(run_id).map(|state| state.sender.subscribe())
    }

    /// Number of active subscribers to a run's event stream (SSE clients).
    pub async fn subscriber_count(&self, run_id: &str) -> usize {
        let runs = self.active_runs.read().await;
        runs.get(run_id)
            .map_or(0, |state| state.sender.receiver_count())
    }

    /// Cancel a run only if it currently has no subscribers.
    ///
    /// Used by the SSE disconnect guard to implement last-subscriber-drop
    /// semantics: a run is abandoned only when the final viewer leaves, so a
    /// multi-viewer stream (or a client reconnecting via history replay) is not
    /// cancelled when one of several subscribers disconnects. Returns `true` if
    /// the run was cancelled.
    pub async fn cancel_run_if_no_subscribers(&self, run_id: &str) -> bool {
        if self.subscriber_count(run_id).await == 0 {
            self.cancel_run(run_id).await
        } else {
            false
        }
    }

    /// Emit an event into an existing run's broadcast channel.
    ///
    /// This is used by external callers (e.g. post-stream auto-capture) that need to
    /// inject events after `start_run` returns. The event is also appended to the
    /// run's history buffer so late-connecting SSE clients can replay it.
    /// Silently no-ops if the run has already been cleaned up.
    pub async fn emit_to_run(&self, run_id: &str, event: NormalizedEvent) {
        let runs = self.active_runs.read().await;
        if let Some(state) = runs.get(run_id) {
            let mut history = state.history.lock().await;
            let id = history.next_id;
            history.next_id = history.next_id.saturating_add(1);
            let stream_event = StreamEvent { id, event };
            history.buffer.push_back(stream_event.clone());
            if history.buffer.len() > EVENT_HISTORY_LIMIT {
                history.buffer.pop_front();
            }
            let _ = state.sender.send(stream_event);
        }
    }

    pub async fn history_since(
        &self,
        run_id: &str,
        last_event_id: Option<u64>,
    ) -> Option<Vec<StreamEvent>> {
        let runs = self.active_runs.read().await;
        let state = runs.get(run_id)?;
        let history = state.history.lock().await;
        let events = history
            .buffer
            .iter()
            .filter(|event| last_event_id.is_none_or(|id| event.id > id))
            .cloned()
            .collect();
        Some(events)
    }

    pub async fn get_run(&self, run_id: &str) -> Option<Run> {
        let runs = self.active_runs.read().await;
        runs.get(run_id).map(|state| state.run.clone())
    }

    pub async fn get_run_by_session_id(&self, session_id: &str) -> Option<Run> {
        let run_id = {
            let session_runs = self.session_current_run.read().await;
            session_runs.get(session_id).cloned()
        }?;
        self.get_run(&run_id).await
    }

    /// Return the resolved default model `(provider_id, model_id)` if one is available,
    /// or `None` if neither a provider registry entry nor a global `llm_config.model`
    /// is configured.
    pub async fn resolve_default_model(&self) -> Option<(String, String)> {
        // 1. Try the provider registry default.
        if let Some(registry) = &self.provider_registry {
            if let Some((provider_id, model_id)) = registry.default_model().await {
                return Some((provider_id, model_id));
            }
        }

        // 2. Fall back to the global llm_config.
        let model = self.llm_config.model.trim().to_string();
        if !model.is_empty() {
            let slash = model.find('/');
            return Some(match slash {
                Some(i) => (model[..i].to_string(), model[i + 1..].to_string()),
                None => ("default".to_string(), model),
            });
        }

        None
    }
}

#[cfg(test)]
mod approval_gate_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::{ApprovalWaitOutcome, await_approval, resolve_pending_approval};
    use std::{collections::HashMap, time::Duration};
    use tokio::sync::{Mutex, oneshot};

    #[tokio::test]
    async fn approval_wait_covers_approve_reject_close_and_timeout() {
        let (approved_tx, approved_rx) = oneshot::channel();
        approved_tx.send(true).expect("receiver remains open");
        assert_eq!(
            await_approval(approved_rx, Duration::from_secs(1)).await,
            ApprovalWaitOutcome::Approved
        );

        let (rejected_tx, rejected_rx) = oneshot::channel();
        rejected_tx.send(false).expect("receiver remains open");
        assert_eq!(
            await_approval(rejected_rx, Duration::from_secs(1)).await,
            ApprovalWaitOutcome::Rejected
        );

        let (closed_tx, closed_rx) = oneshot::channel::<bool>();
        drop(closed_tx);
        assert_eq!(
            await_approval(closed_rx, Duration::from_secs(1)).await,
            ApprovalWaitOutcome::ChannelClosed
        );

        let (_timeout_tx, timeout_rx) = oneshot::channel::<bool>();
        assert_eq!(
            await_approval(timeout_rx, Duration::from_millis(1)).await,
            ApprovalWaitOutcome::TimedOut
        );
    }

    #[tokio::test]
    async fn approval_resolution_is_single_use() {
        let approvals = Mutex::new(HashMap::new());
        let (sender, receiver) = oneshot::channel();
        approvals.lock().await.insert("run-1".to_string(), sender);

        assert!(resolve_pending_approval(&approvals, "run-1", true).await);
        assert!(!resolve_pending_approval(&approvals, "run-1", false).await);
        assert_eq!(receiver.await, Ok(true));
    }
}

#[cfg(test)]
mod activation_outcome_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::correlate_skill_activation_outcomes;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn skill_used_when_its_server_was_invoked() {
        let mut skill_servers = HashMap::new();
        skill_servers.insert("skill-a".to_string(), vec!["weather_mcp".to_string()]);
        let invoked: HashSet<String> = ["weather_mcp".to_string()].into_iter().collect();

        let result = correlate_skill_activation_outcomes(&skill_servers, &invoked);
        assert_eq!(result, vec![("skill-a".to_string(), true)]);
    }

    #[test]
    fn skill_not_used_when_its_server_was_never_invoked() {
        let mut skill_servers = HashMap::new();
        skill_servers.insert("skill-a".to_string(), vec!["weather_mcp".to_string()]);
        let invoked: HashSet<String> = ["other_mcp".to_string()].into_iter().collect();

        let result = correlate_skill_activation_outcomes(&skill_servers, &invoked);
        assert_eq!(result, vec![("skill-a".to_string(), false)]);
    }

    #[test]
    fn skill_used_if_any_of_its_multiple_servers_was_invoked() {
        let mut skill_servers = HashMap::new();
        skill_servers.insert(
            "skill-a".to_string(),
            vec!["server_one".to_string(), "server_two".to_string()],
        );
        let invoked: HashSet<String> = ["server_two".to_string()].into_iter().collect();

        let result = correlate_skill_activation_outcomes(&skill_servers, &invoked);
        assert_eq!(result, vec![("skill-a".to_string(), true)]);
    }

    #[test]
    fn no_matched_skills_with_mcp_config_yields_empty_result() {
        // Prompt-overlay-only skills never appear in `skill_servers` at the
        // call site — an empty map here models "no skill this run introduced
        // any MCP tools," which must yield no outcome calls at all, not a
        // false one.
        let skill_servers = HashMap::new();
        let invoked: HashSet<String> = ["some_mcp".to_string()].into_iter().collect();

        let result = correlate_skill_activation_outcomes(&skill_servers, &invoked);
        assert!(result.is_empty());
    }
}

#[cfg(test)]
mod cost_budget_wiring_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::agent_cost_limit_from_extensions;
    use std::collections::HashMap;

    #[test]
    fn extracts_declared_limit() {
        let mut ext = HashMap::new();
        ext.insert(
            "budgets".to_string(),
            serde_json::json!({ "max_cost_per_session_usd": 2.5 }),
        );
        assert_eq!(agent_cost_limit_from_extensions(&ext), Some(2.5));
    }

    #[test]
    fn returns_none_when_budgets_key_absent() {
        let ext = HashMap::new();
        assert_eq!(agent_cost_limit_from_extensions(&ext), None);
    }

    #[test]
    fn returns_none_when_budgets_is_null() {
        // `stash()` in to_artifact.rs serializes `ir.budgets: Option<BudgetsSection>`
        // unconditionally, so an agent with no declared budgets section still
        // gets a `"budgets": null` entry, not a missing key.
        let mut ext = HashMap::new();
        ext.insert("budgets".to_string(), serde_json::Value::Null);
        assert_eq!(agent_cost_limit_from_extensions(&ext), None);
    }

    #[test]
    fn returns_none_when_field_absent_or_null() {
        let mut ext = HashMap::new();
        ext.insert(
            "budgets".to_string(),
            serde_json::json!({ "max_tokens_per_turn": 1000 }),
        );
        assert_eq!(agent_cost_limit_from_extensions(&ext), None);

        let mut ext2 = HashMap::new();
        ext2.insert(
            "budgets".to_string(),
            serde_json::json!({ "max_cost_per_session_usd": null }),
        );
        assert_eq!(agent_cost_limit_from_extensions(&ext2), None);
    }
}

#[cfg(test)]
mod credential_layer_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::apply_credential_layer;
    use crate::config::LlmConfig;
    use crate::uar::security::credentials::{
        CredentialEncryption, CredentialRecord, CredentialScope, CredentialStore,
        InMemoryCredentialStore, ProviderService,
    };
    use std::sync::Arc;

    const KEY: &[u8; 32] = b"0123456789abcdef0123456789abcdef";

    fn cfg_with_env_key() -> LlmConfig {
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            api_key: Some("env-key".to_string()),
            ..Default::default()
        }
    }

    async fn service_with_user_key(user: &str, provider: &str, key: &str) -> Arc<ProviderService> {
        let store = Arc::new(InMemoryCredentialStore::new());
        let enc = CredentialEncryption::from_key(KEY);
        let now = chrono::Utc::now();
        store
            .put(CredentialRecord {
                scope: CredentialScope::User,
                scope_id: user.to_string(),
                provider_id: provider.to_string(),
                api_key_encrypted: enc.encrypt(key).expect("encrypt"),
                api_key_hint: CredentialEncryption::key_hint(key, 4),
                created_at: now,
                updated_at: now,
            })
            .await
            .expect("put");
        Arc::new(ProviderService::new(store, Arc::new(enc)))
    }

    // Single-tenant: no ProviderService ⇒ env/config key is left untouched.
    #[tokio::test]
    async fn single_tenant_no_service_keeps_env_key() {
        let out =
            apply_credential_layer(cfg_with_env_key(), None, Some("alice"), None, "agent-1").await;
        assert_eq!(out.api_key.as_deref(), Some("env-key"));
    }

    // Multi-tenant: a stored user key for this provider overrides the env key.
    #[tokio::test]
    async fn multi_tenant_user_key_overrides_env_key() {
        let svc = service_with_user_key("alice", "openai", "sk-alice-USER").await;
        let out = apply_credential_layer(
            cfg_with_env_key(),
            Some(&svc),
            Some("alice"),
            None,
            "agent-1",
        )
        .await;
        assert_eq!(out.api_key.as_deref(), Some("sk-alice-USER"));
    }

    // Multi-tenant but no credential for this user ⇒ falls back to env key.
    #[tokio::test]
    async fn multi_tenant_no_user_credential_falls_back_to_env() {
        let svc = service_with_user_key("alice", "openai", "sk-alice-USER").await;
        let out = apply_credential_layer(
            cfg_with_env_key(),
            Some(&svc),
            Some("bob"), // bob has no stored key
            None,
            "agent-1",
        )
        .await;
        assert_eq!(out.api_key.as_deref(), Some("env-key"));
    }

    // A stored key for a *different* provider must not be used.
    #[tokio::test]
    async fn provider_isolation_keeps_env_key() {
        let svc = service_with_user_key("alice", "anthropic", "sk-anthropic").await;
        let out = apply_credential_layer(
            cfg_with_env_key(), // model is openai/*
            Some(&svc),
            Some("alice"),
            None,
            "agent-1",
        )
        .await;
        assert_eq!(out.api_key.as_deref(), Some("env-key"));
    }
}
