use crate::config::{LlmConfig, SkillEvolutionConfig};
use crate::llm::{LlmDriver, Message, MessageRole};
use crate::mcp::binding_cache::McpBindingEnvironment;
use crate::mcp::catalog::{McpCatalog, ServerAuthentication, ServerDefinition, ServerSource};
use crate::mcp::registry::McpRegistry;
use crate::mcp::runtime::{McpRunResources, McpRuntimeManager};
use crate::session::SessionStore;
use crate::uar::a2ui::realtime::A2uiReplayBackbone;
use crate::uar::a2ui::{policy_surface::effective_policy_surface, protocol};
use crate::uar::domain::{
    artifact::AgentArtifact,
    events::{ArtifactPayload, CitationSource, MemoryItem, NormalizedEvent, StatePatchOp},
    policy::{
        ChatMode, ConversationPolicyRecord, EffectiveRunPolicy, ModelRoute,
        PolicyResolutionContext, PolicyResolutionInput, PolicyUniverse, RunPolicy, SelectionMode,
        ToolApprovalPolicy, policy_from_agent_artifact, resolve_effective_run_policy_core,
        resolve_run_policy,
    },
    runs::{Run, RunStatus},
};
use crate::uar::rag::{
    citation_stream::CitationStream,
    pipeline::{RagRetrievalPipeline, RetrievalBackend},
};
use crate::uar::runtime::matching::{ClassifierConfig, IntentClassifier, create_classifier};
use crate::uar::runtime::native_skill::NativeSkillRegistry;
use crate::uar::runtime::prompt::{
    Authority, PromptBudgets, PromptFragment, PromptRole, PromptSection, RenderOptions, Retention,
    TurnInterrupted, TurnInterruptionReason, TurnManifest, render_with_options,
};
use crate::uar::runtime::skills::SkillRegistry;
use crate::uar::runtime::skills::service::SkillService;
use crate::uar::runtime::thread::approvals::{ApprovalBroker, ApprovalOutcome};
use futures::StreamExt;
use std::{
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{Mutex, RwLock, broadcast, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;
use tracing::instrument;
use uuid::Uuid;

const EVENT_HISTORY_LIMIT: usize = 512;

#[cfg(test)]
#[path = "presentation_history_tests.rs"]
mod presentation_history_tests;

#[derive(Clone, Debug)]
pub struct StreamEvent {
    pub id: u64,
    pub event: NormalizedEvent,
}

#[derive(Debug)]
struct EventHistory {
    next_id: u64,
    buffer: VecDeque<StreamEvent>,
    presentation: Option<super::presentations::PresentationObservation>,
    latest_presentation: Option<StreamEvent>,
}

impl EventHistory {
    fn publish(
        &mut self,
        event: NormalizedEvent,
        sender: &broadcast::Sender<StreamEvent>,
        completion: Option<
            &std::sync::Mutex<crate::uar::runtime::thread::execution::RunCompletionCapture>,
        >,
    ) -> StreamEvent {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        if let Some(completion) = completion {
            completion
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .record(&event);
        }
        let stream_event = StreamEvent { id, event };
        self.buffer.push_back(stream_event.clone());
        if self.buffer.len() > EVENT_HISTORY_LIMIT {
            self.buffer.pop_front();
        }
        let _ = sender.send(stream_event.clone());
        stream_event
    }

    fn record(
        &mut self,
        run_id: &str,
        event: NormalizedEvent,
        snapshot: Option<&super::presentations::RunPresentationSnapshot>,
        sender: &broadcast::Sender<StreamEvent>,
        completion: Option<
            &std::sync::Mutex<crate::uar::runtime::thread::execution::RunCompletionCapture>,
        >,
    ) {
        let terminal = matches!(
            &event,
            NormalizedEvent::RunDone { .. }
                | NormalizedEvent::RunDoneWithUsage { .. }
                | NormalizedEvent::Cancelled { .. }
                | NormalizedEvent::Error { .. }
        );
        // Publication observations follow their actual event. Terminal state
        // precedes the terminal frame, which closes transport subscribers.
        if !terminal {
            self.publish(event.clone(), sender, completion);
        }
        let observation_changed = self.presentation.is_none()
            || terminal
            || matches!(
                &event,
                NormalizedEvent::StatePatch { .. }
                    | NormalizedEvent::ArtifactDisplay { .. }
                    | NormalizedEvent::PresentationDiagnostic { .. }
                    | NormalizedEvent::ToolEnd { .. }
            );
        if let Some(snapshot) = snapshot.filter(|_| observation_changed) {
            let observation = self.presentation.get_or_insert_with(|| {
                super::presentations::PresentationObservation::new(snapshot)
            });
            observation.observe(&event, snapshot);
            let value = serde_json::json!(observation);
            let unchanged = self.latest_presentation.as_ref().is_some_and(|previous| {
                matches!(&previous.event, NormalizedEvent::StatePatch { patch, .. }
                    if patch.first().and_then(|op| op.value.as_ref()) == Some(&value))
            });
            let root_changed = matches!(&event, NormalizedEvent::StatePatch { patch, .. }
                if patch.iter().any(|op| matches!(op.path.as_str(), "" | "/")));
            if !unchanged || root_changed {
                let projection = NormalizedEvent::StatePatch {
                    run_id: run_id.to_owned(),
                    patch: vec![crate::uar::domain::events::StatePatchOp {
                        op: "add".into(),
                        path: "/presentation".into(),
                        value: Some(value),
                    }],
                };
                self.latest_presentation = Some(self.publish(projection, sender, completion));
            }
        }
        if terminal {
            self.publish(event, sender, completion);
        }
    }
}

/// Dialogue owned by a single kernel producer. Shared conversation sessions
/// can advance independently; a child's history lookup must not follow them.
#[derive(Clone)]
struct RunDialogue(crate::session::Session);

impl std::fmt::Debug for RunDialogue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunDialogue")
            .field("message_count", &self.0.message_count())
            .finish()
    }
}

impl RunDialogue {
    fn record(
        &self,
        conversation: &crate::session::Session,
        update: impl Fn(&crate::session::Session),
    ) {
        update(&self.0);
        update(conversation);
    }
}

#[derive(Debug)]
struct RunStreamState {
    run: Run,
    /// Full middleware-verified identity retained only by the host. The public
    /// Run record keeps its stable subject-only wire schema.
    verified_owner: Option<crate::uar::runtime::actor::messages::ActorOwner>,
    presentations: Option<Arc<super::presentations::RunPresentationSnapshot>>,
    dialogue: RunDialogue,
    sender: broadcast::Sender<StreamEvent>,
    history: Arc<Mutex<EventHistory>>,
    // Observation must not keep a vanished producer alive: only emitters own
    // the completion sender. A weak link lets disconnect guards see a mailbox.
    completion: Option<
        std::sync::Weak<
            std::sync::Mutex<crate::uar::runtime::thread::execution::RunCompletionCapture>,
        >,
    >,
    delegation: Option<std::sync::Weak<crate::uar::runtime::turn::bindings::RunDelegationBindings>>,
}

#[derive(Clone, Debug)]
struct RunEventEmitter {
    run_id: String,
    presentations: Option<Arc<super::presentations::RunPresentationSnapshot>>,
    sender: broadcast::Sender<StreamEvent>,
    history: Arc<Mutex<EventHistory>>,
    completion:
        Option<Arc<std::sync::Mutex<crate::uar::runtime::thread::execution::RunCompletionCapture>>>,
}

impl RunEventEmitter {
    async fn emit(&self, event: NormalizedEvent) {
        let event =
            super::a2ui_output::enforce_output_ceiling(event, self.presentations.as_deref());
        let mut history = self.history.lock().await;
        history.record(
            &self.run_id,
            event,
            self.presentations.as_deref(),
            &self.sender,
            self.completion.as_deref(),
        );
    }
}

#[async_trait::async_trait]
impl crate::uar::domain::events::RuntimeEventSink for RunEventEmitter {
    async fn emit(&self, event: NormalizedEvent) {
        RunEventEmitter::emit(self, event).await;
    }
}

struct ChatRagSearchBackend<'a> {
    persistence: &'a dyn crate::uar::persistence::PersistenceLayer,
    vector_matcher: &'a crate::uar::runtime::matching::VectorMatcher,
    owner_id: &'a str,
    kb_ids: &'a [String],
}

#[async_trait::async_trait]
impl RetrievalBackend for ChatRagSearchBackend<'_> {
    async fn search_one(
        &self,
        sub_query: &str,
        limit: usize,
        min_score: f32,
    ) -> anyhow::Result<Vec<crate::uar::domain::knowledge::KnowledgeMatch>> {
        let embeddings = self
            .vector_matcher
            .embed_batch(vec![sub_query.to_string()])
            .await?;
        let query_vec = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no embedding generated for chat sub-query"))?;
        let kb_id_refs = self.kb_ids.iter().map(String::as_str).collect::<Vec<_>>();
        self.persistence
            .search_knowledge_scoped(self.owner_id, &kb_id_refs, &query_vec, limit, min_score)
            .await
    }
}

type ActiveRunMap = HashMap<String, RunStreamState>;

/// Extract `budgets.max_cost_per_session_usd` from an `AgentArtifact`'s
/// `extensions` map (CH-06). `budgets` has no typed home on `AgentPolicy` —
/// see `to_artifact.rs`'s module doc — so it's preserved losslessly as JSON
/// under `extensions["budgets"]`. Returns `None` for an absent key, a `null`
/// value (unset budgets section), a missing field, or a non-numeric field.
/// Legacy parser tests only; execution uses strict `ThreadBudgets` decoding.
#[cfg(test)]
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
/// including prompt-only skills (an empty server list), whose outcome follows
/// the terminal run status rather than a nonexistent tool-use signal.
fn correlate_skill_activation_outcomes(
    skill_servers: &HashMap<String, Vec<String>>,
    invoked_tool_servers: &HashSet<String>,
) -> Vec<(String, bool)> {
    skill_servers
        .iter()
        .map(|(skill_id, servers)| {
            let used =
                servers.is_empty() || servers.iter().any(|s| invoked_tool_servers.contains(s));
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
    /// Shared projected runtime for verified root turns. Definitions remain in
    /// `global_mcp`/the skill catalog and are frozen into each root request.
    mcp_runtime: Option<McpRuntimeManager>,
    mcp_environment: Option<Arc<McpBindingEnvironment>>,
    /// Opaque host binding revision shared by definitions captured this boot.
    /// Environment/config hashes remain separate parts of the exact cache key.
    mcp_auth_revision: Uuid,
    sessions: SessionStore,
    skills: Arc<RwLock<SkillRegistry>>,
    vector_matcher: Arc<crate::uar::runtime::matching::VectorMatcher>,
    tag_matcher: Arc<crate::uar::runtime::matching::TagMatcher>,
    /// Intent classifier for skill matching
    intent_classifier: Arc<dyn IntentClassifier>,
    /// Classifier configuration
    classifier_config: ClassifierConfig,
    harness_config: crate::config::HarnessConfig,
    project_instructions_config:
        crate::uar::runtime::project_instructions::ProjectInstructionsConfig,
    world_state_config: crate::uar::runtime::world_state::sections::WorldStateConfig,
    world_state_clock: Arc<dyn crate::uar::runtime::world_state::sections::Clock>,
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
    /// Backend selected by the trusted host, never by model arguments.
    sandbox_runner: Option<Arc<dyn crate::sandbox::SandboxRunner>>,
    sandbox_operations: Arc<crate::sandbox::execution::SandboxSupervisor>,
    terminal_operations: Arc<crate::uar::tools::terminal_process::TerminalSupervisor>,
    graph_roots: Arc<crate::uar::runtime::thread::graph_host::GraphRootSupervisor>,
    /// Shared A2UI replay stream. Agent tool output and REST message ingress
    /// publish through the same backbone so every client sees one surface.
    a2ui_backbone: Arc<crate::uar::a2ui::realtime::InMemoryReplayBackbone>,
    /// Optional host-supplied primary LLM driver. Used by embedded/library
    /// deployments that keep provider credentials and local model runtimes
    /// outside UAR while letting UAR own the agent/tool/skill loop.
    primary_driver: Option<Arc<dyn LlmDriver>>,
    /// Serialized root approval channels, shared with hosted descendants.
    /// Only the authenticated host resolver can deliver a decision.
    approvals: ApprovalBroker,
    /// Root cancellation token. Every run derives a child token from this, so
    /// cancelling the root (e.g. on server shutdown) aborts all in-flight runs.
    root_cancellation: CancellationToken,
    /// Per-run cancellation tokens: `run_id` -> child token. Populated in
    /// `start_run` and removed when the run reaches a terminal state, so finished
    /// runs do not accumulate. `cancel_run` cancels the token found here.
    run_cancellations: Arc<RwLock<HashMap<String, CancellationToken>>>,
    /// Message-count based context strategy applied to session history before LLM calls.
    message_context_strategy: crate::uar::context::ContextStrategy,
    /// Optional agent graph for graph-based execution. When set, orchestrator-agent
    /// runs use graph-driven delegation instead of the simple tool loop.
    agent_graph: Option<std::sync::Arc<crate::uar::runtime::graph::AgentGraph>>,
    /// Optional Cedar governance engine consulted at the tool-approval gate.
    /// When set, a tool that policy denies is routed to the HITL approval gate.
    /// `None` ⇒ tool approval relies solely on the keyword heuristic.
    governance_engine: Option<Arc<crate::uar::governance::engine::GovernanceEngine>>,
    /// Coherent boot-effective governance gate; Initializing/unavailable gates On.
    governance_gate: Option<crate::uar::governance::runtime_control::GovernanceGateHandle>,
    /// Runtime model failover configuration (CH-03). `enabled: false` by
    /// default (opt-in) — when enabled, each run's `Orchestrator` receives the
    /// ordered healthy fallback chain plus the shared provider-health monitor.
    failover_config: crate::config::FailoverConfig,
    /// Per-run/task/session/agent/global spend aggregator (CH-06). Always
    /// present (unconfigured scopes simply have no limit, so `record` is a
    /// cheap no-op warning check).
    cost_budget: crate::uar::runtime::cost_budget::CostBudgetTracker,
    resilience_policy: crate::uar::settings::resilience_policy::ResiliencePolicy,
    /// Optional runtime settings source for the Global policy scope
    /// (`run_policy.global`). Built from `persistence` at construction when
    /// persistence is present. When `None`, policy resolution falls back to the
    /// legacy agent+conversation path, so callers without settings storage keep
    /// their existing behavior.
    settings_manager: Option<Arc<crate::uar::settings::manager::SettingsManager>>,
    /// Immutable authenticated UAR-peer bindings for governed remote children.
    a2a_peers: Arc<crate::uar::api::a2a::peer::TrustedA2APeers>,
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
    let provider_id = provider_id_for_config(&cfg);
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

fn provider_id_for_config(config: &LlmConfig) -> String {
    config
        .resolved_provider_id
        .clone()
        .unwrap_or_else(|| crate::llm::registry::split_model_string_pub(&config.model).0)
}

fn qualified_model_name(config: &LlmConfig) -> String {
    let (_, model_id) = crate::llm::registry::split_model_string_pub(&config.model);
    let provider_id = provider_id_for_config(config);
    format!("{provider_id}/{model_id}")
}

fn apply_routed_connection(mut base: LlmConfig, routed: LlmConfig) -> LlmConfig {
    let provider_changed = provider_id_for_config(&base) != provider_id_for_config(&routed);
    base.model = routed.model;
    base.resolved_provider_id = routed.resolved_provider_id;
    if provider_changed {
        base.api_key = routed.api_key;
        base.api_key_env = None;
    } else if routed.api_key.is_some() {
        base.api_key = routed.api_key;
    }
    base.base_url = routed.base_url;
    base
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

fn governance_bypass_decision(
    gate: Option<&crate::uar::governance::runtime_control::GovernanceGateHandle>,
) -> Option<crate::llm::ToolApprovalResult> {
    gate.filter(|gate| !gate.effective_enabled())
        .map(|_| crate::llm::ToolApprovalResult::GovernanceBypassed)
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
enum ApprovalWaitOutcome {
    Approved,
    Rejected,
    ChannelClosed,
    TimedOut,
}

#[cfg(test)]
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

#[cfg(test)]
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
    pub(crate) fn run_usage(
        &self,
        run_id: &str,
    ) -> crate::uar::runtime::cost_budget::RunUsageSnapshot {
        self.cost_budget.run_usage(run_id)
    }

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
        let tag_matcher = Arc::new(crate::uar::runtime::matching::TagMatcher::new());

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

        // Build a settings manager from the same persistence the run manager
        // owns so the embedded path can read `run_policy.global` (the Global
        // policy scope). Absent persistence ⇒ no settings manager ⇒ legacy
        // agent+conversation resolution.
        let settings_manager = persistence.as_ref().map(|p| {
            Arc::new(crate::uar::settings::manager::SettingsManager::new(
                Arc::clone(p),
            ))
        });

        Self {
            graph_roots: Arc::new(
                crate::uar::runtime::thread::graph_host::GraphRootSupervisor::default(),
            ),
            active_runs: Arc::new(RwLock::new(HashMap::new())),
            session_current_run: Arc::new(RwLock::new(HashMap::new())),
            llm_config,
            global_mcp,
            mcp_runtime: None,
            mcp_environment: None,
            mcp_auth_revision: Uuid::new_v4(),
            sessions,
            skills,
            vector_matcher,
            tag_matcher,
            intent_classifier,
            classifier_config,
            harness_config: crate::config::HarnessConfig::default(),
            project_instructions_config: Default::default(),
            world_state_config: Default::default(),
            world_state_clock: Arc::new(crate::uar::runtime::world_state::sections::SystemClock),
            persistence,
            settings_manager,
            skill_service: None,
            skill_evolution_config: SkillEvolutionConfig::default(),
            provider_registry: None,
            provider_service: None,
            native_skills,
            sandbox_runner: None,
            sandbox_operations: Arc::new(crate::sandbox::execution::SandboxSupervisor::default()),
            terminal_operations: Arc::new(
                crate::uar::tools::terminal_process::TerminalSupervisor::default(),
            ),
            a2ui_backbone: crate::uar::a2ui::realtime::InMemoryReplayBackbone::new(),
            primary_driver: None,
            approvals: ApprovalBroker::default(),
            root_cancellation: CancellationToken::new(),
            run_cancellations: Arc::new(RwLock::new(HashMap::new())),
            message_context_strategy: crate::uar::context::ContextStrategy::default(),
            agent_graph: None,
            governance_engine: None,
            governance_gate: None,
            failover_config: crate::config::FailoverConfig::default(),
            cost_budget: crate::uar::runtime::cost_budget::CostBudgetTracker::new(),
            resilience_policy: crate::uar::settings::resilience_policy::ResiliencePolicy::default(),
            a2a_peers: Arc::new(crate::uar::api::a2a::peer::TrustedA2APeers::default()),
        }
    }

    #[must_use]
    pub(crate) fn with_a2a_config(mut self, config: &crate::config::A2aConfig) -> Self {
        self.a2a_peers = Arc::new(crate::uar::api::a2a::peer::TrustedA2APeers::from_config(
            config,
        ));
        self
    }

    pub(crate) fn trusted_a2a_peer(
        &self,
        endpoint: &str,
        agent_id: &str,
    ) -> anyhow::Result<crate::uar::api::a2a::peer::TrustedA2APeer> {
        self.a2a_peers.resolve(endpoint, agent_id)
    }

    pub(crate) fn a2a_instance_id(&self) -> &str {
        self.a2a_peers.source_instance_id()
    }

    pub(crate) async fn resolve_remote_policy_constraint(
        &self,
        artifact: &AgentArtifact,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
        session_id: &str,
        constraint: RunPolicy,
    ) -> anyhow::Result<EffectiveRunPolicy> {
        let mut local = self
            .resolve_effective_policy(
                artifact,
                owner.user_id(),
                session_id,
                true,
                None,
                Some(owner),
            )
            .await;
        let mut constrained = self
            .resolve_effective_policy(
                artifact,
                owner.user_id(),
                session_id,
                true,
                Some(constraint),
                Some(owner),
            )
            .await;
        self.backfill_effective_model(&mut local).await;
        self.backfill_effective_model(&mut constrained).await;
        anyhow::ensure!(
            constrained.chat_mode == ChatMode::Agent
                && constrained.agent_id.as_deref() == Some(artifact.id.as_str())
                && constrained.model == local.model
                && (!constrained.memory_enabled || local.memory_enabled)
                && (!constrained.prompt_caching_enabled || local.prompt_caching_enabled),
            "remote UAR policy is incompatible with the target artifact"
        );
        for (remote, target) in [
            (&constrained.skills.ids, &local.skills.ids),
            (&constrained.tools.ids, &local.tools.ids),
            (&constrained.mcp_servers.ids, &local.mcp_servers.ids),
            (&constrained.knowledge_bases.ids, &local.knowledge_bases.ids),
            (&constrained.presentations.ids, &local.presentations.ids),
        ] {
            anyhow::ensure!(
                remote.iter().all(|id| target.contains(id)),
                "remote UAR policy exceeds the target artifact"
            );
        }
        Ok(constrained)
    }

    /// Attach an agent graph for graph-driven execution.
    ///
    /// When set, orchestrator-agent runs execute the graph instead of the simple
    /// tool loop. Other agents retain their existing execution path.
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
    pub fn with_harness_config(mut self, config: crate::config::HarnessConfig) -> Self {
        self.harness_config = config;
        self
    }

    async fn resolved_harness_config(&self) -> crate::config::HarnessConfig {
        let mut config = self.harness_config.clone();
        if let Some(settings) = &self.settings_manager {
            if let Ok(Some(mode)) = settings.get_typed("harness.mode").await {
                config.mode = mode;
            }
            if let Ok(Some(mode)) = settings.get_typed("harness.skill_activation_mode").await {
                config.skill_activation_mode = mode;
            }
            if let Ok(Some(budget)) = settings.get_typed("harness.skill_reattachment").await {
                config.skill_reattachment = budget;
            }
        }
        config
    }

    /// Configure host-owned workspace trust and world-state precision.
    pub fn with_world_state_config(
        mut self,
        instructions: crate::uar::runtime::project_instructions::ProjectInstructionsConfig,
        world_state: crate::uar::runtime::world_state::sections::WorldStateConfig,
    ) -> Self {
        self.project_instructions_config = instructions;
        self.world_state_config = world_state;
        self
    }

    /// Substitute the host clock without changing assembly or request behavior.
    pub fn with_world_state_clock(
        mut self,
        clock: Arc<dyn crate::uar::runtime::world_state::sections::Clock>,
    ) -> Self {
        self.world_state_clock = clock;
        self
    }

    pub fn with_skill_service(mut self, service: Arc<SkillService>) -> Self {
        self.skill_service = Some(service);
        self
    }

    /// Install the application host's one shared MCP cache/connector and the
    /// environment snapshot captured before request admission.
    #[must_use]
    pub(crate) fn with_mcp_runtime(
        mut self,
        runtime: McpRuntimeManager,
        environment: Arc<McpBindingEnvironment>,
    ) -> Self {
        self.mcp_runtime = Some(runtime);
        self.mcp_environment = Some(environment);
        self
    }

    async fn catalog_skills(&self) -> Vec<crate::uar::domain::skills::Skill> {
        match &self.skill_service {
            Some(service) => service.get_skills().await,
            None => self.skills.read().await.list_enabled(),
        }
    }

    async fn root_mcp_catalog(&self) -> anyhow::Result<Arc<McpCatalog>> {
        let mut definitions = Vec::new();
        for (name, configuration) in self.global_mcp.server_entries() {
            let binding_id = format!("{}:global:{name}", self.mcp_auth_revision);
            definitions.push(ServerDefinition::new(
                name,
                ServerSource::Global,
                configuration,
                false,
                ServerAuthentication::Authenticated { binding_id },
            )?);
        }
        for skill in self.catalog_skills().await {
            let Some(config) = skill.mcp_config else {
                continue;
            };
            for (name, configuration) in config.mcp_servers {
                let binding_id =
                    format!("{}:skill:{}:{name}", self.mcp_auth_revision, skill.skill_id,);
                definitions.push(ServerDefinition::new(
                    name,
                    ServerSource::Skill {
                        skill_id: skill.skill_id.clone(),
                    },
                    configuration,
                    true,
                    ServerAuthentication::Authenticated { binding_id },
                )?);
            }
        }
        Ok(Arc::new(McpCatalog::from_definitions(definitions)?))
    }

    async fn capture_root_mcp_resources(
        &self,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
    ) -> anyhow::Result<Option<McpRunResources>> {
        let (Some(runtime), Some(environment)) = (&self.mcp_runtime, &self.mcp_environment) else {
            return Ok(None);
        };
        Ok(Some(McpRunResources::new(
            owner.clone(),
            runtime.clone(),
            self.root_mcp_catalog().await?,
            Arc::clone(environment),
        )))
    }

    /// Server and tool identities known without granting a connection. Skills
    /// contribute preferred tools only when they declare an MCP dependency
    /// capable of providing them; host-editor tool names in ordinary skill
    /// manifests are not UAR execution authority. Connected global discovery
    /// contributes exact compiled provider names.
    pub(crate) async fn mcp_policy_inventory(
        &self,
        catalog: Option<&McpCatalog>,
    ) -> (BTreeSet<String>, BTreeSet<String>) {
        let servers = match catalog {
            Some(catalog) => catalog.server_names().map(str::to_owned).collect(),
            None => match self.root_mcp_catalog().await {
                Ok(catalog) => catalog.server_names().map(str::to_owned).collect(),
                Err(error) => {
                    tracing::warn!(%error, "MCP catalog unavailable while resolving policy");
                    BTreeSet::new()
                }
            },
        };
        let mut tools = self
            .global_mcp
            .tools()
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<BTreeSet<String>>();
        for skill in self.catalog_skills().await {
            if skill
                .mcp_config
                .as_ref()
                .is_some_and(|config| !config.mcp_servers.is_empty())
            {
                tools.extend(skill.preferred_tools);
            }
        }
        (servers, tools)
    }

    /// Revoke projected bindings after a trusted administrator changes or
    /// removes a server definition. Legacy registry mutation happens at the
    /// API boundary before this call.
    pub(crate) async fn invalidate_mcp_server(&self, server: &str) {
        if let Some(runtime) = &self.mcp_runtime {
            runtime.invalidate_server(server).await;
        }
    }

    /// Override the settings manager used to read the Global policy scope.
    ///
    /// [`RunManager::with_classifier_config`] builds a settings manager from the
    /// supplied persistence by default. Embedded hosts that build and seed a
    /// single shared [`SettingsManager`](crate::uar::settings::manager::SettingsManager)
    /// (so runtime resolution and the admin surface observe the same cache) pass
    /// it here to replace the auto-built instance.
    #[must_use]
    pub fn with_settings_manager(
        mut self,
        settings_manager: Arc<crate::uar::settings::manager::SettingsManager>,
    ) -> Self {
        self.settings_manager = Some(settings_manager);
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

    /// Set a host-supplied primary LLM driver for embedded deployments.
    ///
    /// The default service/runtime path still constructs `LiterLlmDriver` from
    /// `LlmConfig`. This override is intentionally explicit so mobile hosts can
    /// register a local provider via `ExternalLlmDriver` without changing UAR's
    /// default cloud/provider behavior.
    #[must_use]
    pub fn with_llm_driver(mut self, driver: Arc<dyn LlmDriver>) -> Self {
        self.primary_driver = Some(driver);
        self
    }

    /// Set the runtime model failover configuration (CH-03). When
    /// `enabled`, each run's `Orchestrator` receives every configured healthy
    /// fallback in declared order plus the shared provider-health monitor.
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

    /// Attach the coherent boot-effective governance master gate.
    #[must_use]
    pub fn with_governance_gate(
        mut self,
        gate: crate::uar::governance::runtime_control::GovernanceGateHandle,
    ) -> Self {
        self.governance_gate = Some(gate);
        self
    }

    /// Set a shared native skill registry for in-process tool execution.
    pub fn with_native_skills(mut self, registry: Arc<NativeSkillRegistry>) -> Self {
        self.native_skills = registry;
        self
    }

    /// Bind an isolation backend resolved by the trusted embedding/server host.
    /// Required tools still check the backend's isolation contract at execution.
    #[must_use]
    pub fn with_sandbox_runner(mut self, runner: Arc<dyn crate::sandbox::SandboxRunner>) -> Self {
        self.sandbox_runner = Some(runner);
        self
    }

    /// Join sandbox operations, including scopes whose run future unwound.
    ///
    /// # Errors
    /// Retains and reports unconfirmed backend outcomes instead of replaying them.
    pub async fn shutdown_sandboxes(&self) -> anyhow::Result<()> {
        self.sandbox_operations
            .shutdown()
            .await
            .map_err(anyhow::Error::from)
    }

    /// Cancel and join directly launched terminal processes retained by runs.
    ///
    /// # Errors
    /// Reports unconfirmed process cleanup without forgetting its owned handle.
    pub async fn shutdown_terminals(&self) -> anyhow::Result<()> {
        self.terminal_operations
            .shutdown()
            .await
            .map_err(anyhow::Error::from)
    }

    /// Cancel and join graph root workers before closing shared transports.
    ///
    /// # Errors
    /// Retains failed workers and unresolved persistence/cleanup receipts.
    pub async fn shutdown_graph_roots(&self) -> anyhow::Result<()> {
        self.graph_roots.shutdown().await
    }

    /// Whether this artifact enters the configured host-owned graph path.
    pub(crate) fn uses_agent_graph(&self, artifact: &AgentArtifact) -> bool {
        artifact.id == "orchestrator-agent" && self.agent_graph.is_some()
    }

    /// Content-free host diagnostics for retained sandbox operations.
    pub async fn sandbox_operations(
        &self,
    ) -> Vec<crate::sandbox::execution::SandboxOperationSnapshot> {
        self.sandbox_operations.operations().await
    }

    #[must_use]
    pub fn with_a2ui_backbone(
        mut self,
        backbone: Arc<crate::uar::a2ui::realtime::InMemoryReplayBackbone>,
    ) -> Self {
        self.a2ui_backbone = backbone;
        self
    }

    /// Resolve a pending tool-call approval for the given run.
    /// Returns `true` if an approval was pending and the decision was delivered,
    /// `false` if no pending approval was found for that run_id.
    pub async fn resolve_approval(&self, run_id: &str, approved: bool) -> bool {
        self.resolve_approval_request(run_id, None, approved).await
    }

    /// Resolve an exact approval request after authenticating the root owner.
    /// Child requests require an ID; run-only decisions serve legacy roots.
    pub async fn resolve_approval_request(
        &self,
        run_id: &str,
        approval_id: Option<&str>,
        approved: bool,
    ) -> bool {
        self.approvals.resolve(run_id, approval_id, approved)
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
        // The approval queue observes the same token and drops only its own
        // pending request. A child cancellation cannot clear a sibling's slot.
        token.cancel();
        tracing::info!(run_id = %run_id, "Run cancellation requested");
        true
    }

    /// Cancel an in-flight run only when it belongs to the authenticated owner.
    pub async fn cancel_run_for_user(&self, owner_id: &str, run_id: &str) -> bool {
        if self.get_run_for_user(owner_id, run_id).await.is_none() {
            return false;
        }
        self.cancel_run(run_id).await
    }

    /// Cancel the current in-flight run associated with a conversation session.
    ///
    /// Service clients receive a stable session identifier before the first
    /// streamed event reveals UAR's internal run id. Resolving the mapping in
    /// the runtime keeps cancellation reliable without asking clients to parse
    /// transport-specific event payloads.
    pub async fn cancel_session_run(&self, session_id: &str) -> bool {
        self.cancel_session_run_for_user(crate::session::ANONYMOUS_SESSION_OWNER, session_id)
            .await
    }

    /// Cancel the current run for an owner-scoped conversation session.
    pub async fn cancel_session_run_for_user(&self, owner_id: &str, session_id: &str) -> bool {
        let session_key = crate::uar::persistence::tenant_storage_key(owner_id, session_id);
        let run_id = {
            let session_runs = self.session_current_run.read().await;
            session_runs.get(&session_key).cloned()
        };
        match run_id {
            Some(run_id) => self.cancel_run_for_user(owner_id, &run_id).await,
            None => false,
        }
    }

    /// A clone of the root cancellation token.
    ///
    /// Cancelling it aborts ALL in-flight runs at once; used to wire run
    /// cancellation into the server's graceful-shutdown path.
    #[must_use]
    pub fn root_cancellation_token(&self) -> CancellationToken {
        self.root_cancellation.clone()
    }

    /// Build the runtime resource universe and the persisted conversation scope
    /// for policy resolution.
    ///
    /// Shared by [`Self::resolve_legacy_run_policy`] and
    /// [`Self::resolve_effective_policy`] so both the legacy fallback and the
    /// global-aware path see an identical universe + conversation scope.
    async fn build_universe_and_conversation(
        &self,
        owner_id: &str,
        conversation_id: &str,
        thread_controls: bool,
        mcp_catalog: Option<&McpCatalog>,
        verified_owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
    ) -> (PolicyUniverse, Option<RunPolicy>) {
        let skills = match &self.skill_service {
            Some(service) => service
                .get_skills()
                .await
                .into_iter()
                .map(|skill| skill.skill_id)
                .collect(),
            None => self
                .skills
                .read()
                .await
                .list_enabled()
                .into_iter()
                .map(|skill| skill.skill_id)
                .collect(),
        };
        let mut tools = self
            .global_mcp
            .tools()
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let (mcp_servers, catalog_tools) = self.mcp_policy_inventory(mcp_catalog).await;
        tools.extend(catalog_tools);
        for tool in self.native_skills.openai_tools_json().await {
            if let Some(name) = tool
                .get("function")
                .and_then(|function| function.get("name"))
                .and_then(serde_json::Value::as_str)
            {
                tools.insert(name.to_string());
            }
        }
        if thread_controls {
            tools.extend(
                crate::uar::runtime::thread::control::AGENT_TOOL_NAMES
                    .into_iter()
                    .map(str::to_owned),
            );
        }
        let mut knowledge_bases = std::collections::BTreeSet::new();
        let (mut presentations, mut presentation_warnings) =
            crate::uar::persistence::presentations::eligible_presentations(
                self.persistence.as_ref(),
                verified_owner.filter(|owner| owner.user_id() == owner_id),
            )
            .await;
        let conversation = if let Some(persistence) = &self.persistence {
            if let Ok(records) = persistence.list_knowledge_bases(owner_id).await {
                for knowledge_base in records {
                    knowledge_bases.insert(knowledge_base.id);
                    knowledge_bases.insert(knowledge_base.name);
                }
            }
            match crate::uar::domain::policy::load_owner_scoped_conversation_policy(
                persistence.as_ref(),
                owner_id,
                conversation_id,
                verified_owner,
            )
            .await
            {
                Ok((policy, _)) => policy,
                Err(error) => {
                    tracing::warn!(%error, "Conversation policy admission failed");
                    presentations.clear();
                    presentation_warnings.push(
                        "Conversation policy could not be loaded; Presentation access is closed"
                            .into(),
                    );
                    None
                }
            }
        } else {
            None
        };
        let universe = PolicyUniverse {
            skills,
            tools,
            mcp_servers,
            knowledge_bases,
            presentations,
            presentation_warnings,
        };
        (universe, conversation)
    }

    /// Resolve the effective run policy for an agent + conversation.
    ///
    /// When a settings manager is available (the embedded path builds one from
    /// its persistence), resolution runs through the shared transport-free core
    /// [`resolve_effective_run_policy_core`], which includes the Global scope
    /// (`run_policy.global`) — matching the HTTP service path exactly. Without a
    /// settings manager it falls back to [`Self::resolve_legacy_run_policy`]
    /// (agent + conversation only), preserving the prior behavior.
    async fn resolve_effective_policy(
        &self,
        artifact: &AgentArtifact,
        owner_id: &str,
        conversation_id: &str,
        thread_controls: bool,
        turn: Option<RunPolicy>,
        verified_owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
    ) -> EffectiveRunPolicy {
        self.resolve_effective_policy_with_catalog(
            artifact,
            owner_id,
            conversation_id,
            thread_controls,
            turn,
            None,
            verified_owner,
        )
        .await
    }

    async fn resolve_effective_policy_with_catalog(
        &self,
        artifact: &AgentArtifact,
        owner_id: &str,
        conversation_id: &str,
        thread_controls: bool,
        turn: Option<RunPolicy>,
        mcp_catalog: Option<&McpCatalog>,
        verified_owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
    ) -> EffectiveRunPolicy {
        let Some(settings_manager) = self.settings_manager.as_ref() else {
            return self
                .resolve_legacy_run_policy(
                    artifact,
                    owner_id,
                    conversation_id,
                    thread_controls,
                    turn,
                    mcp_catalog,
                    verified_owner,
                )
                .await;
        };
        let (universe, conversation) = self
            .build_universe_and_conversation(
                owner_id,
                conversation_id,
                thread_controls,
                mcp_catalog,
                verified_owner,
            )
            .await;
        let ctx = PolicyResolutionContext {
            settings_manager: Some(settings_manager.as_ref()),
            universe,
            default_context_strategy: self.message_context_strategy.clone(),
        };
        resolve_effective_run_policy_core(ctx, artifact, conversation, turn).await
    }

    /// Compute the effective configuration for a conversation, mirroring the
    /// service path's `GET /conversations/{id}/effective-config`: it resolves the
    /// agent named by the stored conversation policy (or the default agent),
    /// resolves the effective run policy for that agent + conversation (Global →
    /// Agent → Conversation → Turn), and backfills the model route from the
    /// registry default so the reported model is the one that will execute.
    ///
    /// Returns the resolved agent, the stored requested policy (if any), and the
    /// effective policy — the pieces an embedded admin surface needs without a
    /// service.
    pub async fn effective_config(&self, conversation_id: &str) -> EffectiveConfig {
        let requested = if let Some(persistence) = &self.persistence {
            persistence
                .load_conversation_policy(crate::session::ANONYMOUS_SESSION_OWNER, conversation_id)
                .await
                .ok()
                .flatten()
        } else {
            None
        };
        let agent_id = requested
            .as_ref()
            .and_then(|record| record.policy.agent_id.clone())
            .unwrap_or_else(|| "default-agent".to_string());
        let agent = self.resolve_agent_or_default(&agent_id).await;
        let mut effective = self
            .resolve_effective_policy(
                &agent,
                crate::uar::domain::knowledge::ANONYMOUS_KNOWLEDGE_OWNER,
                conversation_id,
                false,
                None,
                None,
            )
            .await;
        self.backfill_effective_model(&mut effective).await;
        EffectiveConfig {
            agent,
            requested_policy: requested,
            effective_policy: effective,
        }
    }

    /// Resolve an agent artifact by id: persisted definition first, then the
    /// two built-ins, then the default agent as a last resort. Mirrors the
    /// service path's `resolve_agent_for_run`.
    async fn resolve_agent_or_default(&self, agent_id: &str) -> AgentArtifact {
        if let Some(persistence) = &self.persistence
            && let Ok(Some(agent)) = persistence.load_agent(agent_id).await
        {
            return agent;
        }
        match agent_id {
            "orchestrator-agent" => crate::uar::defaults::orchestrator_agent(),
            _ => crate::uar::defaults::default_agent(),
        }
    }

    /// Resolve an explicitly selected actor artifact without silently replacing
    /// an unknown ID or a failed storage read with the default agent.
    pub(crate) async fn resolve_registered_agent(
        &self,
        agent_id: &str,
    ) -> anyhow::Result<AgentArtifact> {
        if let Some(persistence) = &self.persistence
            && let Some(agent) = persistence.load_agent(agent_id).await?
        {
            return Ok(agent);
        }
        match agent_id {
            "default-agent" => Ok(crate::uar::defaults::default_agent()),
            "orchestrator-agent" => Ok(crate::uar::defaults::orchestrator_agent()),
            "general-purpose" => Ok(crate::uar::defaults::general_purpose_agent()),
            "rust-reviewer" => Ok(crate::uar::defaults::rust_reviewer_agent()),
            "compiler-agent" => Ok(crate::uar::defaults::compiler_agent()),
            _ => anyhow::bail!("Requested agent artifact is not registered"),
        }
    }

    /// Backward-compatible agent + conversation resolution (no Global scope).
    ///
    /// Retained as the fallback used when no settings manager is available so
    /// callers that never opted into global policy keep identical behavior.
    async fn resolve_legacy_run_policy(
        &self,
        artifact: &AgentArtifact,
        owner_id: &str,
        conversation_id: &str,
        thread_controls: bool,
        turn: Option<RunPolicy>,
        mcp_catalog: Option<&McpCatalog>,
        verified_owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
    ) -> EffectiveRunPolicy {
        let (universe, conversation) = self
            .build_universe_and_conversation(
                owner_id,
                conversation_id,
                thread_controls,
                mcp_catalog,
                verified_owner,
            )
            .await;
        let default_model = ModelRoute {
            provider_id: artifact.policy.provider.default.provider.clone(),
            model_id: artifact.policy.provider.default.model.clone(),
        };

        resolve_run_policy(PolicyResolutionInput {
            agent: Some(policy_from_agent_artifact(artifact)),
            conversation,
            turn,
            universe,
            default_chat_mode: ChatMode::Agent,
            default_context_strategy: self.message_context_strategy.clone(),
            default_agent_id: Some(artifact.id.clone()),
            default_model: Some(default_model),
            ..PolicyResolutionInput::default()
        })
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
        self.start_run_with_policy(artifact, input, session_id, user_id, memory_hits, None)
            .await
    }

    /// Start a run with explicit skill attachments admitted before matching.
    #[allow(clippy::too_many_arguments)]
    pub async fn start_run_with_skill_attachments(
        &self,
        artifact: AgentArtifact,
        input: String,
        session_id: Option<String>,
        user_id: Option<String>,
        memory_hits: Vec<MemoryItem>,
        skill_attachments: Vec<String>,
    ) -> String {
        let mut request = crate::uar::runtime::turn::RunExecutionRequest::new(artifact, input);
        request.session_id = session_id;
        request.user_id = user_id;
        request.memory_hits = memory_hits;
        request.skill_attachments = skill_attachments;
        self.execute_request(request).await
    }

    /// [`Self::start_run`] plus a `seed_history` of prior turns used to
    /// repopulate an empty (cold-started) session. See
    /// [`Self::start_run_with_policy_and_history`].
    pub async fn start_run_with_history(
        &self,
        artifact: AgentArtifact,
        input: String,
        session_id: Option<String>,
        user_id: Option<String>,
        memory_hits: Vec<MemoryItem>,
        seed_history: Vec<SeedMessage>,
    ) -> String {
        self.start_run_with_policy_and_history(
            artifact,
            input,
            session_id,
            user_id,
            memory_hits,
            None,
            seed_history,
        )
        .await
    }

    /// Continue an existing run after a user responds to an interactive A2UI
    /// surface. A continuation is a real agent run in the same conversation,
    /// not a synthetic event injected into a completed stream.
    pub async fn continue_with_interaction(
        &self,
        run_id: &str,
        interaction: serde_json::Value,
        user: &crate::uar::security::claims::UserContext,
    ) -> Result<String, String> {
        let run = self
            .get_run(run_id)
            .await
            .ok_or_else(|| format!("run '{run_id}' not found"))?;
        if run.user_id.as_deref() != Some(user.user_id.as_str()) {
            return Err("interaction principal does not own the source run".to_string());
        }
        let persistence = self
            .persistence
            .as_ref()
            .ok_or_else(|| "agent persistence is unavailable".to_string())?;
        let artifact = persistence
            .load_agent(&run.agent_id)
            .await
            .map_err(|error| format!("failed to load agent '{}': {error}", run.agent_id))?
            .ok_or_else(|| format!("agent '{}' not found", run.agent_id))?;
        let input = serde_json::json!({
            "type": "a2ui.user_action",
            "sourceRunId": run_id,
            "interaction": interaction,
        })
        .to_string();
        let effective_policy = run
            .context
            .get("effective_run_policy")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok());
        let mut request = crate::uar::runtime::turn::RunExecutionRequest::new(artifact, input)
            .with_user_context(user)
            .map_err(|_| "invalid interaction principal".to_string())?;
        request.session_id = run.conversation_id;
        request.resolved_policy = effective_policy;
        request.presentation_negotiation = match run.context.get("presentation_negotiation") {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|_| "Stored Presentation negotiation is invalid".to_string())?,
            None => Default::default(),
        };
        Ok(self.execute_request(request).await)
    }

    /// Start a run using an immutable policy already resolved by UAR's control
    /// plane. When omitted, a backward-compatible policy is resolved from the
    /// artifact, persisted conversation policy, and currently available
    /// resources.
    pub async fn start_run_with_policy(
        &self,
        artifact: AgentArtifact,
        input: String,
        session_id: Option<String>,
        user_id: Option<String>,
        memory_hits: Vec<MemoryItem>,
        resolved_policy: Option<EffectiveRunPolicy>,
    ) -> String {
        self.start_run_with_policy_and_history(
            artifact,
            input,
            session_id,
            user_id,
            memory_hits,
            resolved_policy,
            Vec::new(),
        )
        .await
    }

    /// [`Self::start_run_with_policy`] plus a `seed_history` of prior turns used
    /// to repopulate an **empty** session (a cold-started conversation whose
    /// durable history lives in the host, not the in-process store). A session
    /// that already holds messages is never re-seeded, so this is idempotent
    /// across warm turns.
    #[allow(clippy::too_many_arguments)]
    pub async fn start_run_with_policy_and_history(
        &self,
        artifact: AgentArtifact,
        input: String,
        session_id: Option<String>,
        user_id: Option<String>,
        memory_hits: Vec<MemoryItem>,
        resolved_policy: Option<EffectiveRunPolicy>,
        seed_history: Vec<SeedMessage>,
    ) -> String {
        let mut request = crate::uar::runtime::turn::RunExecutionRequest::new(artifact, input);
        request.session_id = session_id;
        request.user_id = user_id;
        request.memory_hits = memory_hits;
        request.resolved_policy = resolved_policy;
        request.seed_history = seed_history;
        self.execute_request(request).await
    }

    /// Start a run from a checkpoint's exact graph state and conversation
    /// history. Unlike an ordinary run, an absent input does not append an
    /// empty user turn to the restored history.
    #[allow(clippy::too_many_arguments)]
    pub async fn start_run_from_checkpoint(
        &self,
        artifact: AgentArtifact,
        input: Option<String>,
        session_id: Option<String>,
        user_id: Option<String>,
        memory_hits: Vec<MemoryItem>,
        restored_history: Vec<Message>,
        restored_state: crate::uar::runtime::graph::GraphState,
    ) -> String {
        self.execute_request(crate::uar::runtime::turn::RunExecutionRequest {
            artifact,
            input,
            session_id,
            user_id,
            memory_hits,
            verified_owner: None,
            mcp_resources: None,
            resolved_policy: None,
            presentation_negotiation: Default::default(),
            host_policy_constraint: None,
            host_budget_constraint: None,
            host_usage_grant: None,
            host_sandbox_constraint: None,
            seed_history: Vec::new(),
            restored_state: Some(restored_state),
            checkpoint_history: Some(restored_history),
            skill_attachments: Vec::new(),
            working_directory: None,
        })
        .await
    }

    /// Execute the shared request type; compatibility entry points adapt here.
    pub async fn execute_request(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
    ) -> String {
        let run_id = Uuid::new_v4().to_string();
        if self.uses_agent_graph(&request.artifact) {
            let result = self
                .graph_roots
                .start(
                    self.clone(),
                    request.clone(),
                    run_id.clone(),
                    self.root_cancellation.child_token(),
                )
                .await;
            if let Err(error) = &result {
                tracing::error!(%run_id, %error, "Graph root could not prepare");
            }
            if result.is_err() || self.get_run(&run_id).await.is_none() {
                self.record_graph_root_failure(&request, &run_id).await;
            }
            return run_id;
        }
        self.execute_request_inner(request, run_id, None, None, None, None)
            .await
    }

    /// Preserve an observable failure even when root persistence fails before
    /// kernel assembly creates its ordinary event stream. Never rerun the input.
    pub(crate) async fn record_graph_root_failure(
        &self,
        request: &crate::uar::runtime::turn::RunExecutionRequest,
        run_id: &str,
    ) {
        let emitter = {
            let mut runs = self.active_runs.write().await;
            let state = runs.entry(run_id.to_owned()).or_insert_with(|| {
                let (sender, _) = broadcast::channel(256);
                let history = Arc::new(Mutex::new(EventHistory {
                    next_id: 1,
                    buffer: VecDeque::with_capacity(EVENT_HISTORY_LIMIT),
                    presentation: None,
                    latest_presentation: None,
                }));
                RunStreamState {
                    run: Run {
                        run_id: run_id.to_owned(),
                        agent_id: request.artifact.id.clone(),
                        conversation_id: request.session_id.clone(),
                        user_id: request.user_id.clone(),
                        status: RunStatus::Error,
                        context: serde_json::json!({}),
                    },
                    verified_owner: request.verified_owner.clone(),
                    presentations: None,
                    dialogue: RunDialogue(
                        SessionStore::new().get_or_create_for_user(
                            request.session_id.as_deref().unwrap_or(run_id),
                            request
                                .user_id
                                .as_deref()
                                .unwrap_or(crate::session::ANONYMOUS_SESSION_OWNER),
                        ),
                    ),
                    sender,
                    history,
                    completion: None,
                    delegation: None,
                }
            });
            // Worker and preparation waiter can observe the same failed start.
            // Emit its receipt once without converting an error into a retry.
            if state
                .run
                .context
                .get("graph_root_failed")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
            {
                return;
            }
            state.run.status = RunStatus::Error;
            state.run.context["graph_root_failed"] = serde_json::Value::Bool(true);
            RunEventEmitter {
                run_id: run_id.to_owned(),
                presentations: state.presentations.clone(),
                sender: state.sender.clone(),
                history: Arc::clone(&state.history),
                completion: None,
            }
        };
        emitter
            .emit(NormalizedEvent::Error {
                run_id: run_id.to_owned(),
                code: "graph_root_failed".into(),
                message: "Graph root could not complete its owned execution and persistence".into(),
            })
            .await;
        emitter
            .emit(NormalizedEvent::RunDone {
                run_id: run_id.to_owned(),
            })
            .await;
    }

    /// Enter the same kernel for a host-owned actor turn. The supplied identity
    /// is allocated by the host, not decoded from model/client tool arguments.
    pub(crate) async fn start_hosted_root_turn(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
        root: crate::uar::runtime::thread::actor_host::ActorRootBinding,
        cancellation: CancellationToken,
        keeps_run_alive: bool,
    ) -> oneshot::Receiver<crate::uar::runtime::thread::AgentThreadResult> {
        let (capture, receiver) = if keeps_run_alive {
            crate::uar::runtime::thread::execution::RunCompletionCapture::channel()
        } else {
            crate::uar::runtime::thread::execution::RunCompletionCapture::observer_channel()
        };
        let run_id = root.record.thread.root_run_id.clone();
        self.execute_request_inner(
            request,
            run_id,
            Some(capture),
            Some(cancellation),
            None,
            Some(root),
        )
        .await;
        receiver
    }

    /// Host-only authenticated actor adapter. The explicit request confirms
    /// this spawn, but cannot override a Cedar deny or grant child tool approval.
    pub(crate) async fn collaborate_actor_root(
        &self,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
        root: &crate::uar::runtime::thread::actor_host::ActorRootBinding,
        request: crate::uar::runtime::thread::spawn::AgentSpawnRequest,
    ) -> anyhow::Result<crate::uar::runtime::thread::AgentThread> {
        request.validate()?;
        root.record.validate(owner.user_id())?;
        anyhow::ensure!(
            root.ready.load(std::sync::atomic::Ordering::Acquire),
            "Source actor must have a live, prepared root turn"
        );
        let service = root
            .service
            .get()
            .ok_or_else(|| anyhow::anyhow!("Actor thread service is unavailable"))?;
        if let Some(governance) = &self.governance_engine {
            anyhow::ensure!(
                governance
                    .is_tool_allowed(&root.record.thread.artifact_id, "spawn_agent")
                    .await,
                "Actor delegation is denied by governance policy"
            );
        }
        let result = service.collaborate_from_user(owner, request).await;
        tracing::info!(root_run_id = %root.record.thread.root_run_id, success = result.is_ok(),
            "Authenticated actor delegation settled");
        result
    }

    /// Capture a live root's executable resources for its trusted thread host.
    /// This neither admits a child nor creates a second thread scheduler.
    ///
    /// # Errors
    /// Rejects unverified owners, completed roots and unavailable MCP bindings.
    pub async fn capture_thread_kernel(
        &self,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
        root: &crate::uar::persistence::agent_threads::PersistedAgentThread,
        persistence: Arc<dyn crate::uar::persistence::PersistenceLayer>,
    ) -> anyhow::Result<crate::uar::runtime::thread::kernel::CapturedThreadKernel> {
        let resources = self
            .active_runs
            .read()
            .await
            .get(&root.thread.root_run_id)
            .and_then(|state| state.delegation.as_ref().and_then(std::sync::Weak::upgrade))
            .ok_or_else(|| anyhow::anyhow!("Root executable capture is unavailable"))?;
        crate::uar::runtime::thread::kernel::CapturedThreadKernel::capture(
            self.clone(),
            owner,
            root,
            persistence,
            resources,
        )
        .await
    }

    pub(crate) async fn execute_captured_thread(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
        run_id: String,
        cancellation: CancellationToken,
        bindings: crate::uar::runtime::turn::bindings::InheritedRunBindings,
    ) -> anyhow::Result<crate::uar::runtime::thread::AgentThreadResult> {
        let (capture, receiver) =
            crate::uar::runtime::thread::execution::RunCompletionCapture::channel();
        Box::pin(self.execute_request_inner(
            request,
            run_id,
            Some(capture),
            Some(cancellation),
            Some(bindings),
            None,
        ))
        .await;
        Ok(receiver.await.unwrap_or_else(|_| {
            crate::uar::runtime::thread::AgentThreadResult::Failed {
                code: "child_kernel_completion_closed".into(),
                message: "Child kernel ended without a terminal completion record".into(),
            }
        }))
    }

    pub(crate) async fn canonical_thread_history(
        &self,
        owner_id: &str,
        run_id: &str,
        child_session_id: Option<&str>,
    ) -> anyhow::Result<Vec<Message>> {
        let runs = self.active_runs.read().await;
        let state = runs
            .get(run_id)
            .filter(|state| state.run.user_id.as_deref() == Some(owner_id))
            .ok_or_else(|| anyhow::anyhow!("Thread kernel history is unavailable"))?;
        let session_id = state
            .run
            .conversation_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Thread kernel has no conversation"))?;
        anyhow::ensure!(
            child_session_id.is_none_or(|expected| expected == session_id),
            "Child kernel history belongs to another session"
        );
        Ok(state.dialogue.0.messages())
    }

    async fn execute_request_inner(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
        run_id: String,
        completion: Option<crate::uar::runtime::thread::execution::RunCompletionCapture>,
        host_cancellation: Option<CancellationToken>,
        inherited: Option<crate::uar::runtime::turn::bindings::InheritedRunBindings>,
        actor_root: Option<crate::uar::runtime::thread::actor_host::ActorRootBinding>,
    ) -> String {
        let execution_started_at = std::time::Instant::now();
        let plan = crate::uar::runtime::turn::TurnAssemblyPlan::for_request(&request);
        let harness_config = match &inherited {
            Some(bindings) => bindings.harness.clone(),
            None => self.resolved_harness_config().await,
        };
        let crate::uar::runtime::turn::RunExecutionRequest {
            mut artifact,
            input,
            session_id,
            user_id,
            memory_hits,
            resolved_policy,
            presentation_negotiation,
            seed_history,
            restored_state,
            checkpoint_history,
            skill_attachments: _,
            working_directory,
            verified_owner,
            mut mcp_resources,
            host_policy_constraint,
            host_budget_constraint,
            host_usage_grant,
            host_sandbox_constraint,
        } = request;
        let append_input = plan.append_input;
        let skill_attachments = plan.requested_skill_ids;
        let input = input.unwrap_or_default();
        tracing::Span::current().record("run_id", &run_id);
        tracing::info!("Starting new run");
        let (tx, _) = broadcast::channel(256); // Buffer size 256
        let history = Arc::new(Mutex::new(EventHistory {
            next_id: 1,
            buffer: VecDeque::with_capacity(EVENT_HISTORY_LIMIT),
            presentation: None,
            latest_presentation: None,
        }));
        let mut emitter = RunEventEmitter {
            run_id: run_id.clone(),
            presentations: None,
            sender: tx.clone(),
            history: Arc::clone(&history),
            completion: completion.map(|capture| Arc::new(std::sync::Mutex::new(capture))),
        };
        let completion_guard = crate::uar::runtime::thread::execution::RunCompletionGuard::new(
            emitter.completion.clone(),
        );

        if let Some(remote) = host_budget_constraint {
            let narrowed = crate::uar::runtime::thread::policy_intersection::ThreadBudgets::from_artifact(&artifact)
                .map(|local| local.intersect(&remote))
                .and_then(|limits| serde_json::to_value(limits).map_err(|_| {
                    crate::uar::runtime::thread::policy_intersection::PolicyIntersectionError::UnsupportedShape {
                        section: "budgets",
                    }
                }));
            match narrowed {
                Ok(narrowed) => {
                    artifact.extensions.insert("budgets".into(), narrowed);
                }
                Err(error) => {
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "remote_budget_invalid".into(),
                            message: error.to_string(),
                        })
                        .await;
                    emitter
                        .emit(NormalizedEvent::RunDone {
                            run_id: run_id.clone(),
                        })
                        .await;
                    return run_id;
                }
            }
        }

        if let Some(root) = &actor_root {
            let valid = verified_owner.as_ref().is_some_and(|owner| {
                root.record.validate(owner.user_id()).is_ok()
                    && root.record.thread.parent_thread_id.is_none()
                    && root.record.thread.root_run_id == run_id
                    && root.record.thread.run_id.as_ref() == Some(&run_id)
                    && root.record.thread.artifact_id == artifact.id
                    && !root.record.thread.status.is_terminal()
            });
            let committed = if valid {
                root.persistence
                    .load_agent_thread(&root.record.thread.owner_id, &root.record.thread.thread_id)
                    .await
                    .is_ok_and(|stored| stored.as_ref() == Some(&root.record))
            } else {
                false
            };
            if !committed || inherited.is_some() {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "actor_root_mismatch".into(),
                        message: "Actor root does not match the verified run request".into(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                return run_id;
            }
        }

        // A host stamp cannot be reused with an independently edited user ID.
        // Reject before session/history mutation or executable resource lookup.
        if verified_owner
            .as_ref()
            .is_some_and(|owner| user_id.as_deref() != Some(owner.user_id()))
        {
            emitter
                .emit(NormalizedEvent::Error {
                    run_id: run_id.clone(),
                    code: "run_owner_mismatch".into(),
                    message: "Run owner does not match the verified host identity".into(),
                })
                .await;
            emitter
                .emit(NormalizedEvent::RunDone {
                    run_id: run_id.clone(),
                })
                .await;
            return run_id;
        }

        if mcp_resources.is_none()
            && inherited.is_none()
            && let Some(owner) = &verified_owner
        {
            match self.capture_root_mcp_resources(owner).await {
                Ok(resources) => mcp_resources = resources,
                Err(error) => {
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "mcp_catalog_unavailable".into(),
                            message: error.to_string(),
                        })
                        .await;
                    emitter
                        .emit(NormalizedEvent::RunDone {
                            run_id: run_id.clone(),
                        })
                        .await;
                    return run_id;
                }
            }
        }

        // Captured credentials cannot be attached to another principal, cwd,
        // or descendant. Policy resolution below uses this exact frozen catalog
        // when the ingress did not already supply an effective policy.
        if let Some(resources) = &mcp_resources {
            let invalid_capture = verified_owner.as_ref() != Some(resources.owner())
                || inherited.is_some()
                || working_directory
                    .as_ref()
                    .is_some_and(|cwd| cwd != resources.environment().directory());
            if invalid_capture {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "mcp_capture_mismatch".into(),
                        message:
                            "Root MCP capture requires its verified owner and working directory"
                                .into(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                return run_id;
            }
        }

        // Validate host identity before mutating a session or consulting any
        // global resource. A child always supplies its canonical history, even
        // when empty, rather than inheriting an unrelated warm session.
        if let Some(bindings) = &inherited {
            let thread = &bindings.thread;
            let valid = thread.validate().is_ok()
                && thread.parent_thread_id.is_some()
                && !thread.status.is_terminal()
                && thread.run_id.as_deref() == Some(run_id.as_str())
                && thread.owner_id == bindings.policy.owner_id()
                && bindings.presentations.owner() == verified_owner.as_ref()
                && thread.artifact_id == bindings.policy.artifact().id
                && thread.artifact_id == artifact.id
                && user_id.as_deref() == Some(thread.owner_id.as_str())
                && session_id.as_deref() == Some(thread.thread_id.as_str())
                && checkpoint_history.is_some()
                && host_cancellation.is_some()
                && bindings.approvals.root_run_id() == thread.root_run_id
                && bindings.policy.approval_root_run_id() == thread.root_run_id
                && bindings.controls.scope().caller() == thread
                && std::ptr::eq(bindings.controls.scope().policy(), bindings.policy.as_ref());
            let bound = bindings.mcp.require_bound_servers(
                bindings
                    .policy
                    .effective()
                    .mcp_servers
                    .ids
                    .iter()
                    .map(String::as_str),
            );
            if !valid || bound.is_err() {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "child_bindings_unavailable".into(),
                        message: "Child identity or inherited execution bindings are unavailable"
                            .into(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                return run_id;
            }
        }
        let artifact = inherited
            .as_ref()
            .map_or(artifact, |bindings| bindings.policy.artifact().clone());
        let sandbox = match &inherited {
            Some(bindings) => Ok(bindings.sandbox.clone()),
            None => self
                .sandbox_runner
                .as_ref()
                .map(|runner| {
                    crate::sandbox::bindings::SandboxBinding::capture(
                        Arc::clone(runner),
                        crate::sandbox::SandboxConfig::default(),
                    )
                    .map(Arc::new)
                })
                .transpose(),
        };
        let sandbox = match sandbox {
            Ok(binding) => binding,
            Err(error) => {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "sandbox_binding_unavailable".into(),
                        message: error.to_string(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                return run_id;
            }
        };
        let sandbox = match (sandbox, host_sandbox_constraint.as_ref()) {
            (Some(binding), Some(constraint)) => match binding.for_permissions(constraint) {
                Ok(binding) => Some(Arc::new(binding)),
                Err(error) => {
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "remote_sandbox_incompatible".into(),
                            message: error.to_string(),
                        })
                        .await;
                    emitter
                        .emit(NormalizedEvent::RunDone {
                            run_id: run_id.clone(),
                        })
                        .await;
                    return run_id;
                }
            },
            (None, Some(constraint))
                if constraint.network_enabled
                    || !constraint.filesystem.is_empty()
                    || !constraint.environment.is_empty() =>
            {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "remote_sandbox_unavailable".into(),
                        message: "Target UAR cannot enforce the inherited sandbox bindings".into(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                return run_id;
            }
            (binding, _) => binding,
        };

        // 1. Resolve Session
        let owner_id = user_id
            .clone()
            .unwrap_or_else(|| crate::session::ANONYMOUS_SESSION_OWNER.to_string());
        let session = if let Some(id) = session_id {
            self.sessions.get_or_create_for_user(&id, &owner_id)
        } else {
            self.sessions.create_for_user(&owner_id)
        };

        // 1b. Seed prior turns into an empty session. The in-process session
        // store is not durable, so a cold-started conversation resolves to an
        // empty session even though the host still holds the full thread. Replay
        // the host-supplied history so the model receives prior context rather
        // than only the current message. Only seed when empty so warm sessions
        // (which already accumulated their turns) are never duplicated.
        if let Some(history) = &checkpoint_history {
            // Resuming creates a branch from the named checkpoint. Replace a
            // warm in-process session rather than silently keeping messages
            // that occurred after that checkpoint.
            session.clear();
            for message in history {
                if message.role != MessageRole::System {
                    session.add_message(message.clone());
                }
            }
        } else if session.message_count() == 0 {
            for message in &seed_history {
                match message.role.as_str() {
                    "assistant" => session.add_assistant_message(&message.content),
                    "tool" => session.add_tool_result(
                        message.tool_call_id.clone().unwrap_or_default(),
                        &message.content,
                    ),
                    "system" => {} // system prompt is owned by the agent artifact
                    _ => session.add_user_message(&message.content),
                }
            }
        }

        let mut effective_policy = match inherited
            .as_ref()
            .map(|bindings| bindings.policy.effective().clone())
            .or(resolved_policy)
        {
            Some(policy) => policy,
            None => {
                self.resolve_effective_policy_with_catalog(
                    &artifact,
                    &owner_id,
                    session.id(),
                    actor_root.is_some(),
                    host_policy_constraint,
                    mcp_resources
                        .as_ref()
                        .map(|resources| resources.catalog().as_ref()),
                    verified_owner.as_ref(),
                )
                .await
            }
        };

        let (presentation_snapshot, presentation_warnings) = match &inherited {
            Some(bindings) => (bindings.presentations.narrow(&effective_policy), Vec::new()),
            None => {
                super::presentations::RunPresentationSnapshot::capture(
                    self.persistence.as_ref(),
                    verified_owner.clone(),
                    &effective_policy,
                    presentation_negotiation,
                )
                .await
            }
        };
        let presentation_snapshot = Arc::new(presentation_snapshot);
        emitter.presentations = Some(Arc::clone(&presentation_snapshot));
        effective_policy.warnings.extend(presentation_warnings);
        effective_policy.presentations.ids.retain(|id| {
            if presentation_snapshot.contains(id) {
                return true;
            }
            effective_policy.warnings.push(format!(
                "Presentation '{id}' is unavailable at run admission"
            ));
            false
        });
        if effective_policy.presentations.ids.is_empty() {
            effective_policy.presentations.mode = SelectionMode::None;
        }
        let tool_count_before_presentation_ceiling = effective_policy.tools.ids.len();
        effective_policy.tools.ids.retain(|name| match name.as_str() {
            "a2ui_render" => presentation_snapshot.selection().allows_surfaces(),
            crate::uar::runtime::native_skills::presentation_render::PRESENTATION_RENDER_NAME => {
                presentation_snapshot.selection().allows_surfaces() && presentation_snapshot.has_templates()
            }
            _ => true,
        });
        if effective_policy.tools.ids.is_empty() {
            effective_policy.tools.mode = SelectionMode::None;
        } else if effective_policy.tools.ids.len() != tool_count_before_presentation_ceiling {
            effective_policy.tools.mode = SelectionMode::Selected;
        }

        if inherited.is_none() {
            self.backfill_effective_model(&mut effective_policy).await;
        }

        // 2. Add User Message
        if append_input {
            session.add_user_message(&input);
        }

        // Capture identity for credential resolution before `user_id` is moved
        // into the Run record and the resolved session id is the source of truth.
        let user_id_for_creds = user_id.clone();
        let session_id_for_creds = Some(session.id().to_string());

        let dialogue = RunDialogue(crate::session::Session::from_state(session.to_state()));
        let run = Run {
            run_id: run_id.clone(),
            agent_id: artifact.id.clone(),
            conversation_id: Some(session.id().to_string()),
            user_id,
            status: RunStatus::Running,
            context: serde_json::json!({
                "input": input,
                "effective_run_policy": effective_policy,
                "presentation_negotiation": presentation_snapshot.negotiation(),
                "presentation_selection": presentation_snapshot.selection(),
                "presentation_templates": presentation_snapshot.identities(),
            }),
        };

        {
            let mut runs = self.active_runs.write().await;
            runs.insert(
                run_id.clone(),
                RunStreamState {
                    run,
                    verified_owner: verified_owner.clone(),
                    presentations: Some(Arc::clone(&presentation_snapshot)),
                    dialogue: dialogue.clone(),
                    sender: tx.clone(),
                    history: Arc::clone(&history),
                    completion: emitter.completion.as_ref().map(Arc::downgrade),
                    delegation: None,
                },
            );
        }

        emitter
            .emit(NormalizedEvent::Artifact {
                run_id: run_id.clone(),
                artifact: ArtifactPayload {
                    artifact_id: format!("run-policy-{run_id}"),
                    artifact_type: "effective_run_policy".to_string(),
                    title: "Effective run policy".to_string(),
                    content: if presentation_snapshot.selection().allows_surfaces() {
                        effective_policy_surface(&run_id, &effective_policy).to_string()
                    } else {
                        serde_json::json!(&effective_policy).to_string()
                    },
                    language: Some(
                        if presentation_snapshot.selection().allows_surfaces() {
                            "a2ui"
                        } else {
                            "json"
                        }
                        .to_string(),
                    ),
                    metadata: if presentation_snapshot.selection().allows_surfaces() {
                        serde_json::json!({
                            "profile": protocol::PROFILE,
                            "protocol_version": protocol::VERSION,
                            "catalog_id": protocol::CATALOG_ID,
                            "version": effective_policy.version,
                            "warnings": effective_policy.warnings,
                        })
                    } else {
                        serde_json::json!({
                            "version": effective_policy.version,
                            "warnings": effective_policy.warnings,
                        })
                    },
                },
            })
            .await;
        {
            let mut session_runs = self.session_current_run.write().await;
            session_runs.insert(
                crate::uar::persistence::tenant_storage_key(&owner_id, session.id()),
                run_id.clone(),
            );
        }

        // Per-run cancellation token, derived from the root so that a server
        // shutdown (which cancels the root) also aborts this run. `cancel_run`
        // and the client-disconnect guard cancel this token; the spawned task
        // selects on it and removes it from the map on any terminal state.
        let run_cancellation =
            host_cancellation.unwrap_or_else(|| self.root_cancellation.child_token());
        {
            let mut cancels = self.run_cancellations.write().await;
            cancels.insert(run_id.clone(), run_cancellation.clone());
        }
        if run_cancellation.is_cancelled() {
            if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                state.run.status = RunStatus::Cancelled;
            }
            emitter
                .emit(NormalizedEvent::Cancelled {
                    run_id: run_id.clone(),
                })
                .await;
            self.run_cancellations.write().await.remove(&run_id);
            return run_id;
        }

        // Register before assembly so the root host can capture this channel
        // alongside its resolved resources. Children inherit it, never the
        // broker's human-resolution capability.
        let child_run = inherited.is_some();
        let approval_channel = match &inherited {
            Some(bindings) => bindings.approvals.for_child(),
            None => match self.approvals.register(
                run_id.clone(),
                Arc::new(emitter.clone()),
                run_cancellation.clone(),
            ) {
                Ok(channel) => channel,
                Err(error) => {
                    tracing::error!(%error, "Root approval channel registration failed");
                    if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "approval_channel_unavailable".into(),
                            message: "Run approval channel is unavailable".into(),
                        })
                        .await;
                    emitter
                        .emit(NormalizedEvent::RunDone {
                            run_id: run_id.clone(),
                        })
                        .await;
                    self.run_cancellations.write().await.remove(&run_id);
                    return run_id;
                }
            },
        };

        let working_directory = inherited
            .as_ref()
            .map(|bindings| bindings.working_directory.clone())
            .or(working_directory)
            .or_else(|| {
                mcp_resources
                    .as_ref()
                    .map(|resources| resources.environment().directory().to_path_buf())
            });
        let world_state = match working_directory
            .map(Ok)
            .unwrap_or_else(std::env::current_dir)
            .and_then(|cwd| {
                crate::uar::runtime::world_state::runtime::WorldStateRuntime::new(
                    session.clone(),
                    cwd,
                    self.project_instructions_config.clone(),
                    self.world_state_config,
                    effective_policy.clone(),
                    Arc::clone(&self.world_state_clock),
                )
            }) {
            Ok(world_state) => Arc::new(world_state),
            Err(error) => {
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "world_state_load_failed".into(),
                        message: error.to_string(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                self.run_cancellations.write().await.remove(&run_id);
                return run_id;
            }
        };

        // 3. Prepare Messages
        let mut messages = Vec::new();
        let mut prompt_fragments =
            crate::uar::runtime::turn::builtin::artifact_fragments(&artifact);
        prompt_fragments.push(crate::uar::runtime::turn::builtin::policy_fragment(
            &effective_policy,
        ));
        if let Some(guidance) = presentation_snapshot.selection().output_guidance() {
            prompt_fragments.push(PromptFragment::new(
                "presentation.output",
                PromptSection::HostInstructions,
                "host.presentation_selection",
                Authority::Host,
                PromptRole::System,
                Retention::Turn,
                guidance,
            ));
        }
        if effective_policy.tools.ids.iter().any(|name| name == crate::uar::runtime::native_skills::presentation_render::PRESENTATION_RENDER_NAME) {
            prompt_fragments.push(PromptFragment::new(
                "presentation.catalog", PromptSection::MemoryAndRetrieval, "host.presentation_snapshot",
                Authority::Retrieved, PromptRole::System, Retention::Turn,
                format!("[ELIGIBLE PRESENTATION DATA]\n{}", presentation_snapshot.catalog()),
            ));
        }
        prompt_fragments.extend(crate::uar::runtime::turn::builtin::memory_fragment(
            &effective_policy,
            &memory_hits,
        ));

        // RAG Retrieval - scoped to agent's configured knowledge bases
        if !effective_policy.knowledge_bases.ids.is_empty()
            && let Some(db) = &self.persistence
        {
            let mut kb_ids = Vec::new();
            for id_or_name in &effective_policy.knowledge_bases.ids {
                let resolved = db
                    .get_knowledge_base(&owner_id, id_or_name)
                    .await
                    .ok()
                    .flatten();
                let resolved = match resolved {
                    Some(kb) => Some(kb),
                    None => db
                        .get_knowledge_base_by_name(&owner_id, id_or_name)
                        .await
                        .ok()
                        .flatten(),
                };
                if let Some(kb) = resolved
                    && !kb_ids.contains(&kb.id)
                {
                    kb_ids.push(kb.id);
                }
            }

            // A configured selection that resolves to nothing is a safe empty
            // result, never an implicit search-all.
            let search_result = if kb_ids.is_empty() {
                Ok(Vec::new())
            } else {
                let backend = ChatRagSearchBackend {
                    persistence: db.as_ref(),
                    vector_matcher: self.vector_matcher.as_ref(),
                    owner_id: &owner_id,
                    kb_ids: &kb_ids,
                };
                RagRetrievalPipeline::new()
                    .retrieve(&backend, &kb_ids.join(","), &input, 3, 0.7)
                    .await
            };

            match search_result {
                Ok(matches) => {
                    if !matches.is_empty() {
                        // Resolve document names for the citation panel
                        // (best-effort; falls back to the document/chunk id
                        // when a document record can't be found or has none).
                        let mut document_names: HashMap<String, String> = HashMap::new();
                        let mut seen_document_ids: HashSet<String> = HashSet::new();
                        for m in &matches {
                            if let Some(did) = &m.chunk.document_id
                                && seen_document_ids.insert(did.clone())
                                && let Ok(Some(doc)) = db.get_document(&owner_id, did).await
                            {
                                document_names.insert(did.clone(), doc.filename);
                            }
                        }

                        // Assign [1], [2], ... markers matching retrieval
                        // order, inject the numbered block, and emit the same
                        // provenance on the SSE stream for the chat UI.
                        let citation_stream =
                            CitationStream::from_matches(&matches, &document_names);
                        prompt_fragments.push(PromptFragment::new(
                            "retrieved.rag",
                            PromptSection::MemoryAndRetrieval,
                            format!(
                                "knowledge_bases:{}",
                                effective_policy.knowledge_bases.ids.join(",")
                            ),
                            Authority::Retrieved,
                            PromptRole::System,
                            Retention::Turn,
                            citation_stream.prompt_block().trim().to_string(),
                        ));
                        if let Some(event) = citation_stream.to_normalized_event(run_id.clone()) {
                            emitter.emit(event).await;
                        }
                    }
                }
                Err(e) => tracing::error!("RAG retrieval pipeline failed: {:?}", e),
            }
        }

        use crate::uar::domain::skills::{SkillCandidate, SkillMatchResult};
        let skill_bindings = match &inherited {
            Some(bindings) => Arc::clone(&bindings.skills),
            None => Arc::new(
                crate::uar::runtime::turn::bindings::RunSkillBindings::capture(
                    &self.skills,
                    self.skill_service.as_deref(),
                )
                .await,
            ),
        };
        let (candidates, skill_selection_method, threshold, margin, top_k) = if let Some(matching) =
            &skill_bindings.matching
        {
            let config = &matching.config;
            let result = matching
                .match_skills_scoped(&input, Some(&artifact.id), Some(session.id()))
                .await;
            (
                result.candidates,
                format!("skill_service.{:?}", config.algorithm).to_lowercase(),
                config.threshold,
                config.margin_threshold,
                config.top_k,
            )
        } else {
            let registry = skill_bindings.registry.read().await;
            let (candidates, method) = match self
                .intent_classifier
                .classify(&input, &[], &registry)
                .await
            {
                Ok(result) => (
                    result
                        .scores
                        .into_iter()
                        .filter_map(|score| {
                            score.skill.map(|skill| SkillCandidate {
                                skill,
                                score: score.score,
                            })
                        })
                        .collect::<Vec<_>>(),
                    format!("legacy_classifier.{:?}", self.classifier_config.backend)
                        .to_lowercase(),
                ),
                Err(error) => {
                    tracing::warn!(%error, "Intent classification failed; scoring fallback candidates");
                    let mut candidates = HashMap::<String, SkillCandidate>::new();
                    for matcher in [
                        self.tag_matcher.as_ref()
                            as &dyn crate::uar::domain::matching::SkillMatcher,
                        self.vector_matcher.as_ref()
                            as &dyn crate::uar::domain::matching::SkillMatcher,
                    ] {
                        if let Ok(matches) = matcher.match_skills(&input, &registry).await {
                            for candidate in matches {
                                let entry = candidates.entry(candidate.skill_id).or_insert(
                                    SkillCandidate {
                                        skill: candidate.skill,
                                        score: candidate.score,
                                    },
                                );
                                entry.score = entry.score.max(candidate.score);
                            }
                        }
                    }
                    (
                        candidates.into_values().collect(),
                        "legacy_fallback.tag_vector_hybrid".to_string(),
                    )
                }
            };
            (
                candidates,
                method,
                self.classifier_config.accept_threshold,
                self.classifier_config.margin_threshold,
                self.classifier_config.topk,
            )
        };
        let allowed_skill_ids = effective_policy.skills.ids.iter().collect::<HashSet<_>>();
        let candidates = {
            let registry = skill_bindings.registry.read().await;
            candidates
                .into_iter()
                .filter_map(|candidate| {
                    let skill = registry.get(&candidate.skill.skill_id)?;
                    (allowed_skill_ids.contains(&skill.skill_id)
                        && skill.enabled_for(Some(&artifact.id), Some(session.id())))
                    .then(|| SkillCandidate {
                        skill: skill.clone(),
                        score: candidate.score,
                    })
                })
                .collect()
        };
        let skill_match_result = SkillMatchResult::resolve_with_prefer(
            candidates,
            threshold,
            margin,
            top_k,
            &artifact.policy.skills.prefer,
        );
        let suggested_skill_ids = skill_match_result
            .accepted
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        if let Some(state) = self.active_runs.write().await.get_mut(&run_id)
            && let Some(context) = state.run.context.as_object_mut()
        {
            context.insert(
                "skill_candidates".to_string(),
                serde_json::json!(
                    skill_match_result
                        .candidates
                        .iter()
                        .map(|candidate| serde_json::json!({
                            "skill_id": candidate.skill.skill_id,
                            "score": candidate.score,
                            "accepted": suggested_skill_ids.contains(&candidate.skill.skill_id),
                        }))
                        .collect::<Vec<_>>()
                ),
            );
        }
        let mut matched_skills = match harness_config.skill_activation_mode {
            crate::config::SkillActivationMode::LegacyOverlay => {
                skill_match_result.accepted_skills()
            }
            crate::config::SkillActivationMode::Catalog => Vec::new(),
        };
        matched_skills.truncate(artifact.policy.skills.max_active as usize);

        let selected_servers = effective_policy
            .mcp_servers
            .ids
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let selected_tools = effective_policy
            .tools
            .ids
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let server_filter = matches!(
            effective_policy.mcp_servers.mode,
            SelectionMode::None | SelectionMode::Selected
        )
        .then_some(&selected_servers);
        let tool_filter = matches!(
            effective_policy.tools.mode,
            SelectionMode::None | SelectionMode::Selected
        )
        .then_some(&selected_tools);
        let native_source = inherited
            .as_ref()
            .map_or(&self.native_skills, |bindings| &bindings.native);
        // Parent-bound activation/agent handlers must never survive an
        // equivalent-descriptor dedup into the child's registry.
        let child_native_names = selected_tools
            .iter()
            .filter(|name| {
                name.as_str() != "activate_skill"
                    && name.as_str()
                        != crate::uar::runtime::native_skills::search_tools::SEARCH_TOOLS_NAME
                    && !crate::uar::runtime::thread::control::AGENT_TOOL_NAMES
                        .contains(&name.as_str())
            })
            .cloned()
            .collect::<HashSet<_>>();
        let native_skills = Arc::new(
            native_source
                .filtered(if child_run {
                    Some(&child_native_names)
                } else {
                    tool_filter
                })
                .await,
        );
        let mcp_source = inherited
            .as_ref()
            .map_or(&self.global_mcp, |bindings| &bindings.mcp);
        let native_descriptors = native_skills.descriptors().await;
        let activation_result = if let Some(resources) = &mcp_resources {
            let host = crate::uar::runtime::skills::activation::ProjectedActivationHost::new(
                resources.runtime().clone(),
                Arc::clone(resources.catalog()),
                effective_policy.clone(),
                resources.owner().clone(),
                Arc::clone(resources.environment()),
                run_cancellation.clone(),
            )
            .with_events(run_id.clone(), Arc::new(emitter.clone()));
            crate::uar::runtime::skills::activation::ActivationContext::new_projected(
                Arc::clone(&skill_bindings.registry),
                artifact.id.clone(),
                session.id().to_owned(),
                artifact.policy.skills.max_active,
                (**mcp_source).clone(),
                native_descriptors,
                host,
            )
            .await
        } else {
            Ok(
                crate::uar::runtime::skills::activation::ActivationContext::new(
                    Arc::clone(&skill_bindings.registry),
                    effective_policy.skills.ids.iter().cloned().collect(),
                    artifact.id.clone(),
                    session.id().to_string(),
                    artifact.policy.skills.max_active,
                    (**mcp_source).clone(),
                    server_filter.cloned(),
                    tool_filter.cloned(),
                    native_descriptors,
                ),
            )
        };
        let activation_context = match activation_result {
            Ok(context) => Arc::new(Mutex::new(context)),
            Err(error) => {
                let cancelled = run_cancellation.is_cancelled();
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = if cancelled {
                        RunStatus::Cancelled
                    } else {
                        RunStatus::Error
                    };
                }
                if cancelled {
                    emitter
                        .emit(NormalizedEvent::Cancelled {
                            run_id: run_id.clone(),
                        })
                        .await;
                } else {
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "mcp_preflight_failed".into(),
                            message: error.to_string(),
                        })
                        .await;
                    emitter
                        .emit(NormalizedEvent::RunDone {
                            run_id: run_id.clone(),
                        })
                        .await;
                }
                self.run_cancellations.write().await.remove(&run_id);
                return run_id;
            }
        };
        let register_turn_tools = async {
            native_skills
                .register(
                    crate::uar::runtime::native_skills::activate_skill::ActivateSkillTool::new(
                        Arc::clone(&activation_context),
                    )
                    .with_thread_policy(
                        inherited
                            .as_ref()
                            .map(|bindings| Arc::clone(&bindings.policy)),
                    ),
                )
                .await?;
            if let Some(bindings) = &inherited {
                let controls = crate::uar::runtime::native_skills::agents::registry_for_turn(
                    Arc::clone(&bindings.controls),
                )
                .await?;
                for name in controls.names().await {
                    if let Some(handler) = controls.get(&name).await {
                        native_skills.register_arc(handler).await?;
                    }
                }
            }
            Ok::<(), crate::uar::tools::descriptor::ToolAssemblyError>(())
        };
        if let Err(error) = register_turn_tools.await {
            if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                state.run.status = RunStatus::Error;
            }
            emitter
                .emit(NormalizedEvent::Error {
                    run_id: run_id.clone(),
                    code: "tool_collision".to_string(),
                    message: error.to_string(),
                })
                .await;
            emitter
                .emit(NormalizedEvent::RunDone {
                    run_id: run_id.clone(),
                })
                .await;
            self.run_cancellations.write().await.remove(&run_id);
            return run_id;
        }
        activation_context
            .lock()
            .await
            .set_native_descriptors(native_skills.descriptors().await);
        activation_context.lock().await.set_shadow_candidates(
            skill_selection_method.clone(),
            skill_match_result
                .candidates
                .iter()
                .map(|candidate| candidate.skill.skill_id.clone()),
        );
        let mut attachment_failures = Vec::new();
        for skill_id in &skill_attachments {
            let mut context = activation_context.lock().await;
            if let Err(failure) = crate::uar::runtime::skills::activation::activate(
                skill_id,
                &mut context,
                crate::uar::runtime::skills::activation::InvokeType::Attachment,
            )
            .await
            {
                attachment_failures.push(failure);
            }
        }
        if let Some(state) = self.active_runs.write().await.get_mut(&run_id)
            && let Some(context) = state.run.context.as_object_mut()
        {
            context.insert(
                "skill_attachments".to_string(),
                serde_json::json!(&skill_attachments),
            );
            context.insert(
                "activation_failures".to_string(),
                serde_json::json!(&attachment_failures),
            );
        }
        for skill in &matched_skills {
            if skill_attachments.contains(&skill.skill_id) {
                continue;
            }
            let mut context = activation_context.lock().await;
            match crate::uar::runtime::skills::activation::activate(
                &skill.skill_id,
                &mut context,
                crate::uar::runtime::skills::activation::InvokeType::Implicit,
            )
            .await
            {
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(%error, failure = ?error, "Implicit skill activation refused")
                }
            }
        }
        matched_skills = activation_context
            .lock()
            .await
            .active()
            .into_iter()
            .map(|entry| entry.skill)
            .collect();
        // Resolve the model that will actually receive this run before applying
        // any model-keyed context budget. This includes provider-registry and
        // first-matched-skill overrides.
        let model_bindings_result = if let Some(bindings) = &inherited {
            bindings.models.for_policy(&bindings.policy)
        } else {
            let preferred_llm_config = if let Some(ref registry) = self.provider_registry {
                let mut provider_policy = artifact.policy.provider.clone();
                if let Some(route) = &effective_policy.model {
                    provider_policy.default.provider = route.provider_id.clone();
                    provider_policy.default.model = route.model_id.clone();
                }
                match registry
                    .resolve_llm_config_from_policy(&provider_policy)
                    .await
                {
                    Some(resolved) => {
                        tracing::info!(
                            provider = %provider_policy.default.provider,
                            model = %provider_policy.default.model,
                            "Using per-agent provider settings"
                        );
                        resolved
                    }
                    None => {
                        tracing::debug!(
                            "No provider match for agent policy, using global settings"
                        );
                        self.llm_config.clone()
                    }
                }
            } else {
                self.llm_config.clone()
            };
            let skill_preferred_model = matched_skills.iter().find_map(|skill| {
                skill
                    .execution_config
                    .preferred_model
                    .as_ref()
                    .map(|model| (skill.skill_id.as_str(), model.as_str()))
            });
            let run_llm_config = if let Some(ref registry) = self.provider_registry {
                let policy_preferred_model = qualified_model_name(&preferred_llm_config);
                let (policy_provider, _) =
                    crate::llm::registry::split_model_string_pub(&policy_preferred_model);
                let preferred_model = match skill_preferred_model {
                    Some((skill_id, model)) => {
                        let qualified = if model.contains('/') {
                            model.to_string()
                        } else {
                            format!("{policy_provider}/{model}")
                        };
                        tracing::info!(
                            skill_id,
                            model = %qualified,
                            "Skill supplies the preferred model candidate"
                        );
                        qualified
                    }
                    None => policy_preferred_model,
                };
                let (preferred_provider, _) =
                    crate::llm::registry::split_model_string_pub(&preferred_model);
                let router = crate::llm::ModelRouter::new(Arc::clone(registry));
                let requirements = crate::llm::router::RouteRequirements {
                    preferred_provider: Some(preferred_provider),
                    ..crate::llm::router::RouteRequirements::default()
                };
                match router
                    .route_with_preferred_model(&requirements, Some(&preferred_model))
                    .await
                {
                    Some(selected_model) => {
                        let (provider_id, model_id) =
                            crate::llm::registry::split_model_string_pub(&selected_model);
                        match registry
                            .resolve_to_llm_config(&provider_id, &model_id)
                            .await
                        {
                            Some(routed) => {
                                tracing::info!(
                                    preferred_model = %preferred_model,
                                    selected_model = %selected_model,
                                    "Selected healthy run model"
                                );
                                apply_routed_connection(preferred_llm_config, routed)
                            }
                            None => {
                                tracing::warn!(
                                    selected_model = %selected_model,
                                    "Routed model became unavailable during resolution; using policy-resolved configuration"
                                );
                                preferred_llm_config
                            }
                        }
                    }
                    None => {
                        tracing::warn!(
                            preferred_model = %preferred_model,
                            "No healthy registered model matched the run; using policy-resolved configuration"
                        );
                        preferred_llm_config
                    }
                }
            } else {
                let mut config = preferred_llm_config;
                if let Some((skill_id, model)) = skill_preferred_model {
                    tracing::info!(skill_id, model, "Skill overrides LLM model");
                    config.model = model.to_string();
                }
                config
            };
            let run_llm_config = apply_credential_layer(
                run_llm_config,
                self.provider_service.as_ref(),
                user_id_for_creds.as_deref(),
                session_id_for_creds.as_deref(),
                artifact.id.as_str(),
            )
            .await;

            // This artifact's session ceiling belongs to the captured root session,
            // not the aggregate spend of every session using the same agent.
            let model_budget =
                crate::uar::runtime::thread::policy_intersection::ThreadBudgets::from_artifact(
                    &artifact,
                )
                .map_err(anyhow::Error::from)
                .and_then(|limits| {
                    crate::uar::runtime::cost_budget::ModelCallBudget::for_run(
                        self.cost_budget.clone(),
                        run_id.clone(),
                        session.id().to_string(),
                        artifact.id.clone(),
                        run_cancellation.clone(),
                        limits,
                        host_usage_grant.clone(),
                        execution_started_at,
                    )
                });

            // Capture once before any summarization or model execution. Rebuilding
            // a client from the same config can resolve different environment or
            // provider credentials, and cannot serve as an inherited binding.
            match model_budget {
                Ok(model_budget) => {
                    crate::uar::runtime::turn::bindings::RunModelBindings::capture(
                        run_llm_config.clone(),
                        self.primary_driver.clone(),
                        self.failover_config.clone(),
                        self.provider_registry
                            .as_ref()
                            .map(|registry| Arc::clone(registry.health())),
                        model_budget,
                    )
                    .await
                }
                Err(error) => Err(error),
            }
        };
        let model_bindings = match model_bindings_result {
            Ok(bindings) => bindings,
            Err(error) => {
                tracing::error!(%error, "Failed to capture run model bindings");
                activation_context.lock().await.record_outcomes(false);
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "orchestrator_start_failed".into(),
                        message: "Failed to create the run orchestrator".into(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                self.run_cancellations.write().await.remove(&run_id);
                return run_id;
            }
        };

        let run_llm_config = model_bindings.config().clone();

        // Capture before adding root-local handlers: retaining those handlers
        // here would form service -> kernel -> registry -> service ownership.
        let captured_native = Arc::new(native_skills.filtered(None).await);
        let delegation_lifetime = crate::uar::runtime::turn::bindings::RunDelegationLifetime(
            verified_owner.clone().filter(|_| !child_run).map(|owner| {
                Arc::new(crate::uar::runtime::turn::bindings::RunDelegationBindings {
                    owner,
                    run_id: run_id.clone(),
                    policy: effective_policy.clone(),
                    presentations: Arc::clone(&presentation_snapshot),
                    artifact: artifact.clone(),
                    thread_controls: actor_root.is_some(),
                    thread_attachment_claimed: std::sync::atomic::AtomicBool::new(false),
                    models: model_bindings.clone(),
                    skills: Arc::clone(&skill_bindings),
                    native: captured_native,
                    activation: Arc::clone(&activation_context),
                    sandbox: sandbox.clone(),
                    harness: harness_config.clone(),
                    working_directory: world_state.directory().to_path_buf(),
                    approvals: approval_channel.clone(),
                    cancellation: run_cancellation.child_token(),
                })
            }),
        );
        if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
            state.delegation = delegation_lifetime.0.as_ref().map(Arc::downgrade);
        }
        let mut graph_controls = inherited
            .as_ref()
            .map(|bindings| Arc::clone(&bindings.controls));
        if let Some(root) = &actor_root {
            let attach = async {
                let owner = verified_owner
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Actor owner missing"))?;
                let kernel = self
                    .capture_thread_kernel(owner, &root.record, Arc::clone(&root.persistence))
                    .await?;
                let service = Arc::new(
                    crate::uar::runtime::thread::service::ThreadService::attach(
                        Arc::new(kernel),
                        Arc::new(emitter.clone()),
                        None,
                    )
                    .await?,
                );
                // Retain before handler setup so every subsequent error path
                // still belongs to the actor's joined cleanup lifetime.
                root.service
                    .set(Arc::clone(&service))
                    .map_err(|_| anyhow::anyhow!("Actor root already has a thread service"))?;
                let root_controls = service.root_controls().await?;
                graph_controls = Some(Arc::clone(&root_controls));
                let controls =
                    crate::uar::runtime::native_skills::agents::registry_for_turn(root_controls)
                        .await?;
                for name in controls.names().await {
                    if let Some(handler) = controls.get(&name).await {
                        native_skills.register_arc(handler).await?;
                    }
                }
                activation_context
                    .lock()
                    .await
                    .set_native_descriptors(native_skills.descriptors().await);
                Ok::<(), anyhow::Error>(())
            }
            .await;
            if let Err(error) = attach {
                tracing::error!(%error, "Actor root thread attachment failed");
                if let Err(cleanup) = root.shutdown().await {
                    tracing::error!(%cleanup, "Actor attachment cleanup remains unconfirmed");
                }
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "thread_attachment_failed".into(),
                        message: "Actor root thread service could not attach".into(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                self.run_cancellations.write().await.remove(&run_id);
                return run_id;
            }
        }

        let eligible_skills = {
            skill_bindings
                .registry
                .read()
                .await
                .list()
                .into_iter()
                .filter(|skill| {
                    allowed_skill_ids.contains(&skill.skill_id)
                        && skill.enabled_for(Some(artifact.id.as_str()), Some(session.id()))
                })
                .collect::<Vec<_>>()
        };
        let catalog_model = qualified_model_name(&run_llm_config);
        let (catalog_provider, catalog_model_id) =
            crate::llm::registry::split_model_string_pub(&catalog_model);
        let catalog_window = crate::llm::catalog::ModelCatalog::global()
            .model(&catalog_provider, &catalog_model_id)
            .map(|model| model.limits.context_window as usize)
            .filter(|window| *window > 0);
        let catalog_entries = eligible_skills
            .iter()
            .map(|skill| {
                let mut entry = crate::uar::runtime::skills::catalog::CatalogEntry::from(skill);
                entry.suggested = suggested_skill_ids.contains(&skill.skill_id);
                entry
            })
            .collect::<Vec<_>>();
        match crate::uar::runtime::skills::catalog::render_catalog(
            &catalog_entries,
            &catalog_model,
            catalog_window,
        ) {
            Ok(catalog) => {
                tracing::debug!(
                    included = catalog.included,
                    omitted = catalog.omitted,
                    used_units = catalog.used_units,
                    budget = ?catalog.budget,
                    "Rendered eligible skill catalog"
                );
                if !catalog.content.is_empty() {
                    prompt_fragments.push(catalog.into_fragment());
                }
            }
            Err(error) => tracing::warn!(%error, "Skill catalog cannot fit the model budget"),
        }

        let prompt_dialect =
            crate::llm::prompt_dialect::PromptDialect::detect(&run_llm_config.model);
        let render_options = RenderOptions {
            prefers_xml_envelope: prompt_dialect.prefers_xml_envelope(),
            markdown_averse: prompt_dialect.markdown_averse(),
        };
        if let Some(restored_history) = checkpoint_history {
            messages.extend(
                restored_history
                    .into_iter()
                    .filter(|message| message.role != MessageRole::System),
            );
            if append_input {
                messages.push(Message {
                    role: MessageRole::User,
                    content: crate::llm::MessageContent::text(input.clone()),
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
        } else {
            messages.extend(session.messages());
        }
        let unrendered_history = messages.clone();
        if harness_config.mode != crate::config::HarnessMode::Typed {
            messages.insert(
                0,
                Message {
                    role: MessageRole::System,
                    content: crate::llm::MessageContent::text(render_with_options(
                        &prompt_fragments,
                        render_options,
                    )),
                    tool_call_id: None,
                    tool_calls: None,
                },
            );
        }

        // Message-count context strategy followed by model-token budgeting.
        let (effective_strategy, context_model) = {
            let (provider_id, model_id) =
                crate::llm::registry::split_model_string_pub(&run_llm_config.model);
            let effective_context_tokens = crate::llm::catalog::ModelCatalog::global()
                .model(&provider_id, &model_id)
                .map(|m| (m.limits.context_window as f64 * 0.7) as u32);
            (
                crate::uar::context::resolve_effective_strategy(
                    &effective_policy.context_strategy,
                    effective_context_tokens,
                ),
                format!("{provider_id}/{model_id}"),
            )
        };
        let summarization_driver: Option<Arc<dyn crate::llm::LlmDriver>> = match &effective_strategy
        {
            crate::uar::context::ContextStrategy::Summarize { .. }
            | crate::uar::context::ContextStrategy::Hierarchical { .. } => {
                Some(model_bindings.primary())
            }
            _ => None,
        };
        // One reduction path: structural trimming, then the token budget, then
        // tool-call normalization, with the system message pinned throughout
        // (`uar::runtime::context::reduce`). The operator-declared strategy
        // drives both stages, so a run reduces once against one tokenizer.
        let (context_provider, context_model_id) =
            crate::llm::registry::split_model_string_pub(&run_llm_config.model);
        let context_limit = crate::llm::catalog::ModelCatalog::global()
            .model(&context_provider, &context_model_id)
            .map(|model| model.limits.context_window as usize)
            .unwrap_or(8_192);
        let mut world_contributor = world_state.contributor(plan.restore_checkpoint).await;
        let world_reserved_tokens = match world_contributor
            .reserved_tokens(&messages, &context_model)
        {
            Ok(tokens) if tokens.saturating_add(1_000) < context_limit => tokens,
            result => {
                let message = match result {
                    Ok(tokens) => format!(
                        "World state requires {tokens} tokens, exceeding the {context_limit}-token model budget including response reserve"
                    ),
                    Err(error) => error.to_string(),
                };
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                activation_context.lock().await.record_outcomes(false);
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "world_state_budget_exceeded".into(),
                        message,
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                self.run_cancellations.write().await.remove(&run_id);
                return run_id;
            }
        };
        let legacy_reduction = if harness_config.mode != crate::config::HarnessMode::Typed {
            Some(
                crate::uar::runtime::context::reduce::reduce_history(
                    messages,
                    &effective_strategy,
                    &context_model,
                    context_limit - world_reserved_tokens,
                    summarization_driver.as_deref(),
                )
                .await,
            )
        } else {
            None
        };
        let (mcp, mcp_preflight, mcp_descriptors, active_for_manifest) = {
            let context = activation_context.lock().await;
            (
                Arc::new(context.mcp().clone()),
                context.mcp_preflight().cloned(),
                context.mcp_descriptors(),
                context.active(),
            )
        };
        let authorized_tools =
            match crate::uar::runtime::turn::contributors::collect_authorized_tools(
                mcp_descriptors
                    .into_iter()
                    .chain(native_skills.descriptors().await),
            ) {
                Ok(tools) => tools,
                Err(error) => {
                    if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    activation_context.lock().await.record_outcomes(false);
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "turn_assembly_rejected".into(),
                            message: error.to_string(),
                        })
                        .await;
                    emitter
                        .emit(NormalizedEvent::RunDone {
                            run_id: run_id.clone(),
                        })
                        .await;
                    self.run_cancellations.write().await.remove(&run_id);
                    return run_id;
                }
            };
        // Both assembly paths compare the same host snapshot, including clock bucket.
        world_contributor.history_rewritten |= legacy_reduction
            .as_ref()
            .is_some_and(|(_, report)| report.history_rewritten);
        let typed_assembly = if harness_config.mode != crate::config::HarnessMode::Legacy {
            let mut prepared_fragments = prompt_fragments.clone();
            prepared_fragments.extend(
                active_for_manifest
                    .iter()
                    .map(|activation| activation.fragment()),
            );
            let inputs = crate::uar::runtime::turn::contributors::AssemblyInputs {
                artifact: artifact.clone(),
                policy: effective_policy.clone(),
                memory_hits: memory_hits.clone(),
                prepared_fragments,
                history: unrendered_history,
                prepared_history: legacy_reduction
                    .as_ref()
                    .map(|(history, _)| history.clone()),
                authorized_tools: authorized_tools.clone(),
                active_skills: active_for_manifest
                    .iter()
                    .map(|activation| activation.skill.skill_id.clone())
                    .collect(),
                budgets: PromptBudgets {
                    context_window_tokens: Some(context_limit),
                    ..PromptBudgets::default()
                },
            };
            let mut registry = crate::uar::runtime::turn::builtin::registry(
                crate::uar::runtime::turn::builtin::ContextStage {
                    model: context_model.clone(),
                    context_limit,
                    strategy: effective_strategy.clone(),
                    reserved_tokens: world_reserved_tokens,
                    options: render_options,
                    skill_budget: harness_config.skill_reattachment,
                    driver: summarization_driver.clone(),
                },
            );
            // Reducer -> world state -> bounded active bodies, in one context stage.
            registry
                .context
                .insert(1, Arc::new(world_contributor.clone()));
            match registry.assemble(&inputs).await {
                Ok(assembled) => Some(assembled),
                Err(error) => {
                    if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    activation_context.lock().await.record_outcomes(false);
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "turn_assembly_rejected".into(),
                            message: error.to_string(),
                        })
                        .await;
                    emitter
                        .emit(NormalizedEvent::RunDone {
                            run_id: run_id.clone(),
                        })
                        .await;
                    self.run_cancellations.write().await.remove(&run_id);
                    return run_id;
                }
            }
        } else {
            None
        };
        let (messages, reduce_report, world_update) =
            if let Some((mut history, report)) = legacy_reduction {
                let update = match world_contributor.baseline.prepare(
                    &world_contributor.snapshot,
                    &history,
                    world_contributor.history_rewritten,
                ) {
                    Ok(update) => update,
                    Err(error) => {
                        if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                            state.run.status = RunStatus::Error;
                        }
                        activation_context.lock().await.record_outcomes(false);
                        emitter
                            .emit(NormalizedEvent::Error {
                                run_id: run_id.clone(),
                                code: "world_state_assembly_failed".into(),
                                message: error.to_string(),
                            })
                            .await;
                        emitter
                            .emit(NormalizedEvent::RunDone {
                                run_id: run_id.clone(),
                            })
                            .await;
                        self.run_cancellations.write().await.remove(&run_id);
                        return run_id;
                    }
                };
                history.extend(update.messages.iter().cloned());
                prompt_fragments.extend(update.fragments.iter().cloned());
                (history, report, Some(update))
            } else if let Some(assembled) = &typed_assembly {
                prompt_fragments = assembled.fragments.clone();
                effective_policy = assembled.policy.clone();
                (
                    assembled.history.clone(),
                    assembled.reduce_report.clone().unwrap_or_default(),
                    assembled.world_state.clone(),
                )
            } else {
                unreachable!("harness mode selects a legacy or typed assembly")
            };
        if let Some(update) = &world_update {
            world_state.commit(update).await;
        }
        if !reduce_report.normalize.is_clean() {
            tracing::warn!(
                run_id = %run_id,
                synthesized = reduce_report.normalize.synthesized.len(),
                removed = reduce_report.normalize.removed.len(),
                "Repaired tool-call pairs before dispatch"
            );
        }
        if let Some(act) = reduce_report.context_action {
            emitter.emit(NormalizedEvent::ContextAction(act)).await;
        }
        for activation in activation_context.lock().await.active() {
            emitter
                .emit(NormalizedEvent::SkillActivated {
                    run_id: run_id.clone(),
                    skill_id: activation.skill.skill_id,
                    title: activation.skill.title,
                    selection_method: activation.invoke_type.as_str().to_string(),
                })
                .await;
        }

        // Spawn async execution task
        // Create per-run Orchestrator.

        if harness_config.mode != crate::config::HarnessMode::Typed {
            let (_, bounded_skill_fragments) =
                crate::uar::runtime::skills::retention::reattach_skills(
                    &messages,
                    &active_for_manifest,
                    &context_model,
                    context_limit,
                    harness_config.skill_reattachment,
                    RenderOptions {
                        prefers_xml_envelope: prompt_dialect.prefers_xml_envelope(),
                        markdown_averse: prompt_dialect.markdown_averse(),
                    },
                );
            prompt_fragments.extend(bounded_skill_fragments);
        }
        let mut manifest_budgets = PromptBudgets::for_rendered(&render_with_options(
            &prompt_fragments,
            RenderOptions {
                prefers_xml_envelope: prompt_dialect.prefers_xml_envelope(),
                markdown_averse: prompt_dialect.markdown_averse(),
            },
        ));
        manifest_budgets.context_window_tokens = Some(context_limit);
        manifest_budgets.max_output_tokens = matched_skills
            .iter()
            .find_map(|skill| skill.execution_config.max_tokens);
        let initial_exposure =
            crate::mcp::exposure::McpToolExposure::default().project(&authorized_tools);
        let mut selected_tool_names = initial_exposure
            .visible()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        if initial_exposure.has_deferred() {
            selected_tool_names.insert(
                crate::uar::runtime::native_skills::search_tools::SEARCH_TOOLS_NAME.to_owned(),
            );
        }
        let turn_manifest = TurnManifest::from_fragments(
            &prompt_fragments,
            manifest_budgets,
            matched_skills.iter().map(|skill| skill.skill_id.clone()),
            selected_tool_names,
            effective_policy.warnings.clone(),
        );
        let turn_manifest_value = serde_json::json!(&turn_manifest);
        if let Some(state) = self.active_runs.write().await.get_mut(&run_id)
            && let Some(context) = state.run.context.as_object_mut()
        {
            context.insert("turn_manifest".to_string(), turn_manifest_value.clone());
        }
        emitter
            .emit(NormalizedEvent::Artifact {
                run_id: run_id.clone(),
                artifact: ArtifactPayload {
                    artifact_id: format!("turn-manifest-{run_id}"),
                    artifact_type: "turn_manifest".to_string(),
                    title: "Turn manifest".to_string(),
                    content: turn_manifest_value.to_string(),
                    language: Some("json".to_string()),
                    metadata: serde_json::json!({
                        "schema_version": turn_manifest.schema_version,
                        "manifest_hash": turn_manifest.manifest_hash,
                        "fragment_count": turn_manifest.counts.total,
                    }),
                },
            })
            .await;

        // Clone for graph context before values are moved into Orchestrator
        let llm_config_for_graph = run_llm_config.clone();
        let resolved_turn = Arc::new(
            crate::uar::runtime::turn::ResolvedTurn::new(
                artifact.clone(),
                effective_policy.clone(),
                crate::uar::runtime::turn::TurnEnvironment {
                    run_id: run_id.clone(),
                    owner_id: owner_id.clone(),
                    session_id: session.id().to_string(),
                },
                run_llm_config.clone(),
                prompt_fragments.clone(),
            )
            .with_verified_owner(verified_owner.clone())
            .with_presentations(Arc::clone(&presentation_snapshot)),
        );
        let shadow_turn = if harness_config.mode == crate::config::HarnessMode::Shadow {
            typed_assembly.as_ref().map(|assembled| {
                (
                    Arc::new(
                        crate::uar::runtime::turn::ResolvedTurn::new(
                            artifact.clone(),
                            assembled.policy.clone(),
                            resolved_turn.environment().clone(),
                            run_llm_config.clone(),
                            assembled.fragments.clone(),
                        )
                        .with_verified_owner(verified_owner.clone())
                        .with_presentations(Arc::clone(&presentation_snapshot)),
                    ),
                    assembled.history.clone(),
                )
            })
        } else {
            None
        };
        let primary_driver_for_graph = model_bindings.primary();

        // Capture the resolved model id so the final RunDoneWithUsage event can
        // report which model actually answered (moved into the spawned task below).
        let run_model = run_llm_config.model.clone();
        // Whether to compute per-request cost (captured before run_llm_config moves).
        let cost_tracking_enabled = run_llm_config.cost_tracking;

        let sandbox_lease = match self
            .sandbox_operations
            .open_run(
                run_id.clone(),
                &run_cancellation,
                model_bindings.budget().execution_deadline(),
                sandbox.clone(),
            )
            .await
        {
            Ok(lease) => lease,
            Err(error) => {
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "sandbox_scope_unavailable".into(),
                        message: error.to_string(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                self.run_cancellations.write().await.remove(&run_id);
                return run_id;
            }
        };
        let sandbox_scope = sandbox_lease.scope();
        let sandbox_operations = Arc::clone(&self.sandbox_operations);
        if let Some(root) = &actor_root
            && root
                .sandbox
                .set((Arc::clone(&sandbox_operations), sandbox_scope.clone()))
                .is_err()
        {
            // No tools have started. Never replace the root's existing receipt.
            if let Err(error) = sandbox_operations.finish_run(&sandbox_scope).await {
                tracing::error!(%error, "Conflicting actor sandbox scope did not close");
            }
            if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                state.run.status = RunStatus::Error;
            }
            emitter
                .emit(NormalizedEvent::Error {
                    run_id: run_id.clone(),
                    code: "root_resource_binding_conflict".into(),
                    message: "Actor sandbox scope is already bound".into(),
                })
                .await;
            emitter
                .emit(NormalizedEvent::RunDone {
                    run_id: run_id.clone(),
                })
                .await;
            self.run_cancellations.write().await.remove(&run_id);
            return run_id;
        }
        let terminal_lease = match self
            .terminal_operations
            .open_run(
                run_id.clone(),
                &run_cancellation,
                model_bindings.budget().execution_deadline(),
            )
            .await
        {
            Ok(lease) => lease,
            Err(error) => {
                if let Err(cleanup) = sandbox_operations.finish_run(&sandbox_scope).await {
                    tracing::error!(%cleanup, "Prepared sandbox scope did not close");
                }
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "terminal_scope_unavailable".into(),
                        message: error.to_string(),
                    })
                    .await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                self.run_cancellations.write().await.remove(&run_id);
                return run_id;
            }
        };
        let terminal_scope = terminal_lease.scope();
        let terminal_operations = Arc::clone(&self.terminal_operations);
        if let Some(root) = &actor_root
            && root
                .terminal
                .set((Arc::clone(&terminal_operations), terminal_scope.clone()))
                .is_err()
        {
            if let Err(error) = terminal_operations.finish_run(&terminal_scope).await {
                tracing::error!(%error, "Conflicting actor terminal scope did not close");
            }
            if let Err(error) = root.shutdown().await {
                tracing::error!(%error, "Conflicting actor resources did not close");
            }
            if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                state.run.status = RunStatus::Error;
            }
            emitter
                .emit(NormalizedEvent::Error {
                    run_id: run_id.clone(),
                    code: "root_resource_binding_conflict".into(),
                    message: "Actor terminal scope is already bound".into(),
                })
                .await;
            emitter
                .emit(NormalizedEvent::RunDone {
                    run_id: run_id.clone(),
                })
                .await;
            self.run_cancellations.write().await.remove(&run_id);
            return run_id;
        }

        let (orchestrator, graph_thread_delegate) = {
            let o = model_bindings
                .orchestrator(mcp, Arc::clone(&native_skills))
                .with_sandbox_scope(sandbox_scope.clone())
                .with_terminal_scope(terminal_scope.clone())
                .with_artifact_collector(
                    actor_root.as_ref().and_then(|root| root.artifacts.clone()),
                )
                .with_tool_execution_mode(artifact.policy.tools.execution_mode.clone())
                .with_resilience_policy(self.resilience_policy.clone())
                .with_resolved_turn(Arc::clone(&resolved_turn))
                .with_world_state(Arc::clone(&world_state))
                .with_skill_activation(
                    Arc::clone(&activation_context),
                    effective_strategy.clone(),
                    context_model.clone(),
                    context_limit,
                    harness_config.skill_reattachment,
                )
                .with_cache_strategy(
                    effective_policy
                        .prompt_caching_enabled
                        .then(crate::llm::anthropic_cache::CacheStrategy::default),
                );
            let o = match sandbox {
                Some(binding) => o.with_sandbox(
                    binding.runner(),
                    artifact.policy.tools.execution_mode.clone(),
                ),
                None => o,
            };
            let o = match &inherited {
                Some(bindings) => o.with_thread_policy(Arc::clone(&bindings.policy)),
                None => o,
            };
            let o = match &mcp_preflight {
                Some(preflight) => o.with_mcp_preflight(Arc::clone(preflight)),
                None => o,
            };
            let o = match &shadow_turn {
                Some((turn, history)) => o.with_shadow_turn(Arc::clone(turn), history.clone()),
                None => o,
            };

            // Wire up tool approval gate
            let approval_run_id = run_id.clone();
            let approval_emitter = emitter.clone();
            let approval_cancellation = run_cancellation.clone();
            let approval_agent_id = artifact.id.clone();
            let approval_governance = self.governance_engine.clone();
            let approval_governance_gate = self.governance_gate.clone();
            let effective_tool_approval = effective_policy.tool_approval;
            let gate: crate::llm::ToolApprovalGate = Arc::new(
                move |tool_call_id, tool_name, approval_class, arguments_json, call_index| {
                    let run_id = approval_run_id.clone();
                    let emitter = approval_emitter.clone();
                    let channel = approval_channel.clone();
                    let cancellation = approval_cancellation.clone();
                    let agent_id = approval_agent_id.clone();
                    let governance = approval_governance.clone();
                    let governance_gate = approval_governance_gate.clone();
                    Box::pin(async move {
                        if cancellation.is_cancelled() {
                            return crate::llm::ToolApprovalResult::Rejected {
                                reason: "Tool call cancelled with its run".to_string(),
                            };
                        }
                        // A root's local governance toggle must not erase
                        // a child's independently narrowed Ask/Deny policy.
                        if !child_run
                            && let Some(decision) =
                                governance_bypass_decision(governance_gate.as_ref())
                        {
                            tracing::debug!(
                                run_id = %run_id,
                                tool = %tool_name,
                                decision_source = "governance_disabled",
                                "Tool governance bypassed for verified local mode"
                            );
                            return decision;
                        }
                        if effective_tool_approval == ToolApprovalPolicy::Deny {
                            let reason =
                                format!("Tool '{tool_name}' is denied by the effective run policy");
                            emitter
                                .emit(NormalizedEvent::ToolCallDenied {
                                    run_id: run_id.clone(),
                                    call_index,
                                    tool_call_id: tool_call_id.clone(),
                                    name: tool_name.clone(),
                                    reason: reason.clone(),
                                })
                                .await;
                            return crate::llm::ToolApprovalResult::Rejected { reason };
                        }
                        let approval_required = effective_tool_approval == ToolApprovalPolicy::Ask
                            || approval_class
                                == crate::uar::tools::descriptor::ApprovalClass::Required;
                        let decision = match &governance {
                                Some(engine) => engine
                                    .tool_decision(&agent_id, &tool_name, approval_required)
                                    .await,
                                None if approval_required => crate::uar::governance::engine::ToolGovernanceDecision::RequireApproval,
                                None => crate::uar::governance::engine::ToolGovernanceDecision::Allow,
                            };
                        match decision {
                                crate::uar::governance::engine::ToolGovernanceDecision::Allow => {
                                    return crate::llm::ToolApprovalResult::Allowed;
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
                        let risk_reason = if effective_tool_approval == ToolApprovalPolicy::Ask {
                            format!(
                                "Tool '{tool_name}' requires approval under the effective run policy"
                            )
                        } else {
                            format!("Tool '{tool_name}' requires approval under its descriptor")
                        };
                        match channel
                            .request(
                                call_index,
                                tool_call_id,
                                tool_name.clone(),
                                arguments_json,
                                risk_reason,
                                &cancellation,
                            )
                            .await
                        {
                            ApprovalOutcome::Approved => {
                                tracing::info!(run_id = %run_id, tool = %tool_name, "Tool call approved by user");
                                crate::llm::ToolApprovalResult::Approved
                            }
                            ApprovalOutcome::Rejected => {
                                tracing::info!(run_id = %run_id, tool = %tool_name, "Tool call rejected by user");
                                crate::llm::ToolApprovalResult::Rejected {
                                    reason: "Rejected by user".to_string(),
                                }
                            }
                            ApprovalOutcome::ChannelClosed => {
                                tracing::warn!(run_id = %run_id, tool = %tool_name, "Approval channel dropped");
                                crate::llm::ToolApprovalResult::Rejected {
                                    reason: "Approval channel closed".to_string(),
                                }
                            }
                            ApprovalOutcome::TimedOut => {
                                tracing::warn!(run_id = %run_id, tool = %tool_name, "Tool approval timed out after 5 minutes");
                                crate::llm::ToolApprovalResult::Rejected {
                                    reason: "Approval timed out after 5 minutes".to_string(),
                                }
                            }
                            ApprovalOutcome::Cancelled => {
                                crate::llm::ToolApprovalResult::Rejected {
                                    reason: "Approval cancelled with its run".to_string(),
                                }
                            }
                        }
                    })
                },
            );
            let tool_budget = model_bindings.budget().clone();
            let budget_emitter = emitter.clone();
            let budget_run_id = run_id.clone();
            let budget_gate: crate::llm::ToolApprovalGate = Arc::new(
                move |tool_call_id, tool_name, approval_class, arguments_json, call_index| {
                    let gate = Arc::clone(&gate);
                    let budget = tool_budget.clone();
                    let emitter = budget_emitter.clone();
                    let run_id = budget_run_id.clone();
                    Box::pin(async move {
                        let decision = gate(
                            tool_call_id.clone(),
                            tool_name.clone(),
                            approval_class,
                            arguments_json,
                            call_index,
                        )
                        .await;
                        // Both Approved and GovernanceBypassed remain
                        // subject to the same host-owned root allowance.
                        if !matches!(&decision, crate::llm::ToolApprovalResult::Rejected { .. })
                            && let Err(error) = budget.admit_tool()
                        {
                            let reason = error.to_string();
                            emitter
                                .emit(NormalizedEvent::ToolCallDenied {
                                    run_id,
                                    call_index,
                                    tool_call_id,
                                    name: tool_name,
                                    reason: reason.clone(),
                                })
                                .await;
                            return crate::llm::ToolApprovalResult::Rejected { reason };
                        }
                        decision
                    })
                },
            );
            let graph_thread_delegate = graph_controls.map(|controls| {
                Arc::new(
                    crate::uar::runtime::graph::delegation::GraphThreadDelegate::new(
                        run_id.clone(),
                        controls,
                        Arc::clone(&budget_gate),
                    ),
                )
            });
            (
                Arc::new(o.with_tool_approval_gate(budget_gate)),
                graph_thread_delegate,
            )
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
        let graph_for_run = if artifact.id == "orchestrator-agent" {
            self.agent_graph.clone()
        } else {
            None
        };
        let graph_a2ui: Arc<dyn A2uiReplayBackbone> = self.a2ui_backbone.clone();
        let graph_tool_host = graph_for_run.as_ref().map(|_| {
            Arc::new(crate::uar::runtime::graph::tools::GraphToolHost::new(
                run_id.clone(),
                Arc::clone(&orchestrator),
                Arc::new(emitter.clone()),
                &run_cancellation,
                dialogue.0.clone(),
                execution_session.clone(),
                self.persistence.clone(),
                Arc::clone(&graph_a2ui),
                Arc::clone(&presentation_snapshot),
            ))
        });
        if let (Some(root), Some(host)) = (&actor_root, &graph_tool_host)
            && root.graph_tools.set(Arc::clone(host)).is_err()
        {
            if let Err(error) = host.shutdown().await {
                tracing::error!(%error, "Conflicting graph tool host did not settle");
            }
            if let Err(error) = root.shutdown().await {
                tracing::error!(%error, "Conflicting graph tool host did not close");
            }
            if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                state.run.status = RunStatus::Error;
            }
            emitter
                .emit(NormalizedEvent::Error {
                    run_id: run_id.clone(),
                    code: "root_resource_binding_conflict".into(),
                    message: "Actor graph tool host is already bound".into(),
                })
                .await;
            emitter
                .emit(NormalizedEvent::RunDone {
                    run_id: run_id.clone(),
                })
                .await;
            self.run_cancellations.write().await.remove(&run_id);
            return run_id;
        }
        let cache_strategy_for_graph = effective_policy
            .prompt_caching_enabled
            .then(crate::llm::anthropic_cache::CacheStrategy::default);
        let persistence_for_run = self.persistence.clone();
        let a2ui_backbone_for_run = Arc::clone(&self.a2ui_backbone);
        // Cancellation: the run's child token (selected on in the consumption loop,
        // moved into the task below) and the registry used to deregister it on
        // terminal state.
        let cancellations_for_cleanup = Arc::clone(&self.run_cancellations);
        let runs_for_completion = Arc::clone(&self.active_runs);
        let cleanup_run_id = run_id.clone();
        let skill_reattachment_budget = harness_config.skill_reattachment;

        // Run-level span: child LLM-call and tool-call spans created within the
        // task attach under this, producing a run → llm → tool trace tree.
        let run_span = tracing::info_span!(
            "run",
            run_id = %execute_run_id,
            agent_id = %execute_agent_id,
        );

        // Actor shutdown may arrive during async assembly. Do not launch a
        // prepared model/graph turn after its host lifetime has been cancelled.
        if run_cancellation.is_cancelled() {
            // No model/tool work has started, but retire the prepared scope.
            if let Err(error) = terminal_operations.finish_run(&terminal_scope).await {
                tracing::error!(%error, "Prepared terminal scope did not close");
            }
            if let Some(root) = &actor_root
                && let Err(error) = root.shutdown().await
            {
                tracing::error!(%error, "Prepared actor tree did not close");
            }
            if let Err(error) = sandbox_operations.finish_run(&sandbox_scope).await {
                tracing::error!(%error, "Prepared sandbox scope did not close");
            }
            if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                state.run.status = RunStatus::Cancelled;
            }
            emitter
                .emit(NormalizedEvent::Cancelled {
                    run_id: run_id.clone(),
                })
                .await;
            self.run_cancellations.write().await.remove(&run_id);
            return run_id;
        }

        let finalizer_emitter = emitter.clone();
        let finalizer_run_id = execute_run_id.clone();
        let finalizer_runs = Arc::clone(&runs_for_completion);
        let finalizer_cancellations = Arc::clone(&cancellations_for_cleanup);
        let finalizer_sandboxes = Arc::clone(&sandbox_operations);
        let finalizer_scope = sandbox_scope.clone();
        let finalizer_actor_root = actor_root.clone();
        let finalizer_graph_tools = graph_tool_host.clone();
        let finalizer_terminals = Arc::clone(&terminal_operations);
        let finalizer_terminal_scope = terminal_scope.clone();
        let actor_producer = actor_root.as_ref().map(|root| Arc::clone(&root.producer));
        if let Some(root) = &actor_root {
            root.ready.store(true, std::sync::atomic::Ordering::Release);
        }
        let execution = async move {
            let _delegation_lifetime = delegation_lifetime;
            let _sandbox_lease = sandbox_lease;
            let _terminal_lease = terminal_lease;
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
                let graph_driver =
                    Arc::new(crate::uar::runtime::skills::usage::SkillRequestDriver {
                        inner: primary_driver_for_graph,
                        context: Arc::clone(&activation_context),
                        model: context_model.clone(),
                        context_limit,
                        budget: skill_reattachment_budget,
                        cost_tracking: cost_tracking_enabled,
                    });
                let graph_ctx = crate::uar::runtime::graph::GraphContext {
                    run_id: execute_run_id.clone(),
                    session_id: Some(execution_session.id().to_string()),
                    llm_config: llm_config_for_graph,
                    driver: graph_driver,
                    cache_strategy: cache_strategy_for_graph,
                    persistence: persistence_for_run.clone(),
                    thread_delegate: graph_thread_delegate,
                    tool_host: graph_tool_host.clone(),
                };

                let mut initial_state = restored_state.unwrap_or_default();
                // Preserve the checkpoint's data bag and iteration while using
                // the fully assembled, normalized history for the resumed call.
                initial_state.messages = messages
                    .iter()
                    .map(|msg| serde_json::to_value(msg).unwrap_or_default())
                    .collect();

                let final_state = tokio::select! {
                    biased;
                    () = run_cancellation.cancelled() => {
                        tracing::info!(run_id = %execute_run_id, "Run cancelled during graph execution");
                        let mut cleanup_failed = false;
                        if let Some(host) = &graph_tool_host && let Err(error) = host.shutdown().await {
                            cleanup_failed = true;
                            emitter.emit(NormalizedEvent::Error {
                                run_id: execute_run_id.clone(), code: "thread_cleanup_unconfirmed".into(), message: error.to_string(),
                            }).await;
                        }
                        // Activation may hold this lock inside retained model
                        // work. Drain that work before acquiring the lock.
                        activation_context.lock().await.record_outcomes(false);
                        if let Err(error) = terminal_operations.finish_run(&terminal_scope).await {
                            cleanup_failed = true;
                            emitter.emit(NormalizedEvent::Error {
                                run_id: execute_run_id.clone(), code: "terminal_cleanup_unconfirmed".into(), message: error.to_string(),
                            }).await;
                        }
                        if let Some(root) = &actor_root && let Err(error) = root.shutdown().await {
                            cleanup_failed = true;
                            emitter.emit(NormalizedEvent::Error {
                                run_id: execute_run_id.clone(), code: "thread_cleanup_unconfirmed".into(), message: error.to_string(),
                            }).await;
                        }
                        if let Err(error) = sandbox_operations.finish_run(&sandbox_scope).await {
                            cleanup_failed = true;
                            emitter.emit(NormalizedEvent::Error {
                                run_id: execute_run_id.clone(), code: "sandbox_cleanup_unconfirmed".into(), message: error.to_string(),
                            }).await;
                        }
                        if let Some(state) = runs_for_completion.write().await.get_mut(&execute_run_id) {
                            state.run.status = if cleanup_failed { RunStatus::Error } else { RunStatus::Cancelled };
                        }
                        if cleanup_failed {
                            emitter.emit(NormalizedEvent::RunDone { run_id: execute_run_id.clone() }).await;
                        } else {
                            emitter.emit(NormalizedEvent::Cancelled { run_id: execute_run_id.clone() }).await;
                        }
                        cancellations_for_cleanup.write().await.remove(&cleanup_run_id);
                        return;
                    }
                    state = graph.execute_with_events(initial_state, &graph_ctx, &emitter) => state,
                };

                let mut graph_succeeded = final_state.get::<String>("_error").is_none();
                if let Some(host) = &graph_tool_host
                    && let Err(error) = host.shutdown().await
                {
                    graph_succeeded = false;
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: execute_run_id.clone(),
                            code: "thread_cleanup_unconfirmed".into(),
                            message: error.to_string(),
                        })
                        .await;
                }
                if let Some(err) = final_state.get::<String>("_error") {
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: execute_run_id.clone(),
                            message: err,
                            code: String::new(),
                        })
                        .await;
                } else if let Some(route) = final_state.get::<String>("_route") {
                    let output_key = format!("_agent_output_{route}");
                    match final_state.get::<String>(&output_key) {
                        Some(output) => {
                            let attributed_output = format!("[{route}]\n\n{output}");
                            dialogue.record(&execution_session, |history| {
                                history.add_assistant_message(attributed_output.clone());
                            });
                            emitter
                                .emit(NormalizedEvent::ChatDelta {
                                    run_id: execute_run_id.clone(),
                                    text_delta: attributed_output,
                                })
                                .await;
                        }
                        None => {
                            graph_succeeded = false;
                            emitter
                                .emit(NormalizedEvent::Error {
                                    run_id: execute_run_id.clone(),
                                    message: format!(
                                        "Delegated sub-agent '{route}' returned no text output"
                                    ),
                                    code: "delegation_output_missing".to_string(),
                                })
                                .await;
                        }
                    }
                }
                activation_context
                    .lock()
                    .await
                    .record_outcomes(graph_succeeded);
                if let Err(error) = terminal_operations.finish_run(&terminal_scope).await {
                    graph_succeeded = false;
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: execute_run_id.clone(),
                            code: "terminal_cleanup_unconfirmed".into(),
                            message: error.to_string(),
                        })
                        .await;
                }
                if let Some(root) = &actor_root
                    && let Err(error) = root.shutdown().await
                {
                    graph_succeeded = false;
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: execute_run_id.clone(),
                            code: "thread_cleanup_unconfirmed".into(),
                            message: error.to_string(),
                        })
                        .await;
                }
                if let Err(error) = sandbox_operations.finish_run(&sandbox_scope).await {
                    graph_succeeded = false;
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: execute_run_id.clone(),
                            code: "sandbox_cleanup_unconfirmed".into(),
                            message: error.to_string(),
                        })
                        .await;
                }
                if let Some(state) = runs_for_completion.write().await.get_mut(&execute_run_id) {
                    state.run.status = if graph_succeeded {
                        RunStatus::Done
                    } else {
                        RunStatus::Error
                    };
                }
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: execute_run_id.clone(),
                    })
                    .await;
                cancellations_for_cleanup
                    .write()
                    .await
                    .remove(&cleanup_run_id);
                return;
            }

            let mut accumulated_content = String::new();
            let mut run_cancelled = false;
            let mut run_failed = false;
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
                        // `next()` future. Sandbox operations remain owned by
                        // the supervisor and are joined before terminal events.
                        // Other tool transports retain their own cancellation
                        // contracts; dropping a stream alone is not proof of stop.
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
                            crate::normalized::NormalizedEvent::SkillActivation {
                                name,
                                status,
                            } => Some(NormalizedEvent::SkillActivated {
                                run_id: execute_run_id.clone(),
                                skill_id: name.clone(),
                                title: name,
                                selection_method: status,
                            }),
                            crate::normalized::NormalizedEvent::Custom {
                                source,
                                event_name,
                                payload,
                            } => Some(NormalizedEvent::Artifact {
                                run_id: execute_run_id.clone(),
                                artifact: ArtifactPayload {
                                    artifact_id: format!("{source}:{event_name}"),
                                    artifact_type: "provider_event".to_string(),
                                    title: event_name,
                                    content: serde_json::json!({
                                        "source": source,
                                        "payload": payload,
                                    })
                                    .to_string(),
                                    language: Some("json".to_string()),
                                    metadata: serde_json::json!({
                                        "source": "external_llm_driver",
                                    }),
                                },
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
                                    dialogue.record(&execution_session, |history| {
                                        history.add_assistant_with_tool_calls(
                                            if accumulated_content.is_empty() {
                                                None
                                            } else {
                                                Some(accumulated_content.clone())
                                            },
                                            accumulated_tool_calls.clone(),
                                        )
                                    });
                                    accumulated_content.clear();
                                    accumulated_tool_calls.clear();
                                }

                                dialogue.record(&execution_session, |history| {
                                    history.add_tool_result(id.clone(), content.clone());
                                });
                                let call_index = tool_call_indices.get(&id).copied().unwrap_or(0);
                                let tool = tool_call_names
                                    .get(&id)
                                    .cloned()
                                    .unwrap_or_else(|| "tool".to_string());
                                let output = serde_json::from_str(&content)
                                    .unwrap_or_else(|_| serde_json::Value::String(content));

                                crate::uar::runtime::a2ui_output::publish_tool_output(
                                    &execute_run_id,
                                    &tool,
                                    success,
                                    &id,
                                    &presentation_snapshot,
                                    &run_cancellation,
                                    a2ui_backbone_for_run.as_ref(),
                                    &emitter,
                                )
                                .await;

                                Some(NormalizedEvent::ToolEnd {
                                    run_id: execute_run_id.clone(),
                                    call_index,
                                    tool_call_id: id,
                                    tool,
                                    output,
                                    ok: success,
                                })
                            }
                            crate::normalized::NormalizedEvent::Error { message, code } => {
                                run_failed = true;
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
                    run_failed = true;
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: execute_run_id.clone(),
                            message: e.to_string(),
                            code: String::new(),
                        })
                        .await;
                }
            }

            if let Err(error) = terminal_operations.finish_run(&terminal_scope).await {
                run_cancelled = false;
                run_failed = true;
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: execute_run_id.clone(),
                        code: "terminal_cleanup_unconfirmed".into(),
                        message: error.to_string(),
                    })
                    .await;
            }
            if let Some(root) = &actor_root
                && let Err(error) = root.shutdown().await
            {
                run_cancelled = false;
                run_failed = true;
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: execute_run_id.clone(),
                        code: "thread_cleanup_unconfirmed".into(),
                        message: error.to_string(),
                    })
                    .await;
            }
            if let Err(error) = sandbox_operations.finish_run(&sandbox_scope).await {
                // Cancellation is not an acknowledgment that unknown remote
                // work stopped. Publish failure, not a clean Cancelled outcome.
                run_cancelled = false;
                run_failed = true;
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: execute_run_id.clone(),
                        code: "sandbox_cleanup_unconfirmed".into(),
                        message: error.to_string(),
                    })
                    .await;
            }

            let mut interrupted_fragment = None;
            if !accumulated_content.is_empty() {
                if run_cancelled || run_failed {
                    let fragment = TurnInterrupted {
                        run_id: execute_run_id.clone(),
                        reason: if run_cancelled {
                            TurnInterruptionReason::Cancelled
                        } else {
                            TurnInterruptionReason::ProviderError
                        },
                    }
                    .into_fragment();
                    dialogue.record(&execution_session, |history| {
                        history.add_message(Message {
                            role: MessageRole::Assistant,
                            content: crate::llm::MessageContent::text(format!(
                                "{accumulated_content}\n\n{}",
                                fragment.marked_content()
                            )),
                            tool_call_id: None,
                            tool_calls: None,
                        })
                    });
                    interrupted_fragment = Some(fragment);
                    if let Some(db) = &persistence_for_run
                        && let Err(error) = db.save_session(&execution_session).await
                    {
                        tracing::warn!(
                            run_id = %execute_run_id,
                            error = %error,
                            "Failed to persist interrupted assistant turn"
                        );
                    }
                } else {
                    dialogue.record(&execution_session, |history| {
                        history.add_assistant_message(accumulated_content.clone());
                    });
                }
            }

            if let Some(state) = runs_for_completion.write().await.get_mut(&execute_run_id) {
                state.run.status = if run_cancelled {
                    RunStatus::Cancelled
                } else if run_failed {
                    RunStatus::Error
                } else {
                    RunStatus::Done
                };
                if let Some(fragment) = interrupted_fragment
                    && let Some(context) = state.run.context.as_object_mut()
                {
                    context.insert("turn_interrupted".to_string(), serde_json::json!(fragment));
                }
            }

            let total_tokens = total_input_tokens.saturating_add(total_output_tokens);
            let has_usage = total_input_tokens > 0 || total_output_tokens > 0;

            // Preserve run_id before it is moved into the RunDone event below.
            let evolution_run_id = execute_run_id.clone();

            // Snapshot at completion includes skills activated by model calls,
            // not just those active before the first request.
            let (final_activations, mcp_tools_for_outcome) = {
                let context = activation_context.lock().await;
                let tools = context
                    .mcp_descriptors()
                    .into_iter()
                    .filter(|descriptor| {
                        descriptor.source == crate::uar::tools::descriptor::ToolSource::Mcp
                    })
                    .filter_map(|descriptor| {
                        descriptor
                            .server
                            .as_ref()
                            .map(|server| (descriptor.provider_name.clone(), server.clone()))
                    })
                    .collect::<HashMap<_, _>>();
                (context.active(), tools)
            };
            let skill_servers = final_activations
                .iter()
                .map(|activation| {
                    let servers = activation
                        .skill
                        .mcp_config
                        .as_ref()
                        .map(|config| config.mcp_servers.keys().cloned().collect())
                        .unwrap_or_default();
                    (activation.skill.skill_id.clone(), servers)
                })
                .collect::<HashMap<_, Vec<String>>>();
            let invoked_tool_servers: HashSet<String> = tool_call_names
                .values()
                .filter_map(|tool_name| mcp_tools_for_outcome.get(tool_name).cloned())
                .collect();
            for (skill_id, used) in
                correlate_skill_activation_outcomes(&skill_servers, &invoked_tool_servers)
            {
                crate::uar::telemetry::metrics::record_skill_activation_outcome(
                    &skill_id,
                    used && !run_failed && !run_cancelled,
                );
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

                    // Driver wrappers already charged every model call. Surface a
                    // `BudgetAlert` for the first scope (in priority order)
                    // that crosses its configured threshold. Unconfigured
                    // scopes have an unlimited `BudgetLimit::default()`, so
                    // status read does not charge the final request again.
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
                        let status = cost_budget_for_run.status(scope, scope_id).await;
                        // CH-07: durable roll-up, fire-and-forget so the hot
                        // path never blocks on a DB write — mirrors the
                        // existing per-tool-call checkpoint persist pattern
                        // above.
                        if let Some(db) = persistence_for_run.clone() {
                            let scope_str = scope.as_str().to_string();
                            let scope_id_owned = scope_id.to_string();
                            tokio::spawn(async move {
                                if let Err(e) = db
                                    .record_cost_entry(&scope_str, &scope_id_owned, cost)
                                    .await
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
                        .status(BudgetScope::Global, "global")
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
        };
        // Acquire the actor's empty producer slot before launch. Publishing the
        // join handle then has no await/cancellation gap after tokio::spawn.
        let mut actor_producer_slot = match &actor_producer {
            Some(producer) => Some(producer.lock().await),
            None => None,
        };
        let producer = tokio::spawn(
            async move {
                use futures::FutureExt;
                // Keep completion outside the fallible execution body so a
                // caught unwind cannot release its actor before sandbox drain.
                let completion_guard = completion_guard;
                let result = std::panic::AssertUnwindSafe(execution).catch_unwind().await;
                // Child graphs can have no actor root of their own. Retain
                // their host independently of the fallible execution body.
                let graph_cleanup = match &finalizer_graph_tools {
                    Some(host) => host.shutdown().await,
                    None => Ok(()),
                };
                let terminal_cleanup = finalizer_terminals.finish_run(&finalizer_terminal_scope).await;
                let thread_cleanup = match &finalizer_actor_root {
                    Some(root) => root.shutdown().await,
                    None => Ok(()),
                };
                let cleanup = finalizer_sandboxes.finish_run(&finalizer_scope).await;
                if result.is_err() {
                    completion_guard.fail("kernel_panicked", "Run kernel failed while unwinding");
                    if let Some(state) = finalizer_runs.write().await.get_mut(&finalizer_run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    finalizer_emitter.emit(NormalizedEvent::Error {
                        run_id: finalizer_run_id.clone(), code: "kernel_panicked".into(),
                        message: "Run kernel failed while unwinding".into(),
                    }).await;
                    finalizer_emitter.emit(NormalizedEvent::RunDone { run_id: finalizer_run_id.clone() }).await;
                    finalizer_cancellations.write().await.remove(&finalizer_run_id);
                }
                if let Err(error) = cleanup {
                    completion_guard.fail("sandbox_cleanup_unconfirmed", "Sandbox cleanup remains unconfirmed");
                    if let Some(state) = finalizer_runs.write().await.get_mut(&finalizer_run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    tracing::error!(run_id = %finalizer_run_id, %error, "Run retains unconfirmed sandbox operations");
                }
                if let Err(error) = graph_cleanup {
                    completion_guard.fail("thread_cleanup_unconfirmed", "Graph history settlement remains unconfirmed");
                    if let Some(state) = finalizer_runs.write().await.get_mut(&finalizer_run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    tracing::error!(run_id = %finalizer_run_id, %error, "Run retains unconfirmed graph history");
                }
                if let Err(error) = thread_cleanup {
                    completion_guard.fail("thread_cleanup_unconfirmed", "Child thread cleanup remains unconfirmed");
                    if let Some(state) = finalizer_runs.write().await.get_mut(&finalizer_run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    tracing::error!(run_id = %finalizer_run_id, %error, "Run retains unconfirmed child threads");
                }
                if let Err(error) = terminal_cleanup {
                    completion_guard.fail("terminal_cleanup_unconfirmed", "Terminal cleanup remains unconfirmed");
                    if let Some(state) = finalizer_runs.write().await.get_mut(&finalizer_run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    tracing::error!(run_id = %finalizer_run_id, %error, "Run retains unconfirmed terminal processes");
                }
            }.instrument(run_span),
        );
        if let Some(slot) = actor_producer_slot.as_mut() {
            slot.handle = Some(producer);
        }

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
        let completion = {
            let runs = self.active_runs.read().await;
            let Some(state) = runs.get(run_id) else {
                return false;
            };
            if state.sender.receiver_count() != 0 {
                return false;
            }
            state.completion.as_ref().and_then(std::sync::Weak::upgrade)
        };
        if let Some(completion) = completion {
            if completion
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .has_waiter()
            {
                return false;
            }
        }
        // A viewer may have reconnected while the completion lock was held.
        if self.subscriber_count(run_id).await != 0 {
            return false;
        }
        self.cancel_run(run_id).await
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
            let event =
                super::a2ui_output::enforce_output_ceiling(event, state.presentations.as_deref());
            let mut history = state.history.lock().await;
            history.record(
                run_id,
                event,
                state.presentations.as_deref(),
                &state.sender,
                None,
            );
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
        let mut events: Vec<_> = history
            .buffer
            .iter()
            .filter(|event| last_event_id.is_none_or(|id| event.id > id))
            .cloned()
            .collect();
        if let Some(projection) = &history.latest_presentation
            && last_event_id.is_none_or(|id| projection.id > id)
            && !events.iter().any(|event| event.id == projection.id)
        {
            // Retain exactly one full projection beyond the bounded event ring.
            // Replay snapshot construction still filters by its requested cursor.
            let index = events.partition_point(|event| event.id < projection.id);
            events.insert(index, projection.clone());
        }
        Some(events)
    }

    pub async fn get_run(&self, run_id: &str) -> Option<Run> {
        let runs = self.active_runs.read().await;
        runs.get(run_id).map(|state| state.run.clone())
    }

    /// Direct A2UI admission uses the original host capture and exact tenant
    /// identity. Anonymous legacy runs retain only their anonymous namespace.
    pub(crate) async fn presentation_run_for_user(
        &self,
        user: &crate::uar::security::claims::UserContext,
        run_id: &str,
    ) -> Option<(Run, Arc<super::presentations::RunPresentationSnapshot>)> {
        let owner = if user.user_id == crate::session::ANONYMOUS_SESSION_OWNER {
            if user.claims.sub != user.user_id || user.tenant_id.is_some() {
                return None;
            }
            None
        } else {
            Some(
                crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)
                    .ok()?,
            )
        };
        let runs = self.active_runs.read().await;
        let state = runs.get(run_id)?;
        if state.verified_owner != owner
            || state
                .run
                .user_id
                .as_deref()
                .unwrap_or(crate::session::ANONYMOUS_SESSION_OWNER)
                != user.user_id
        {
            return None;
        }
        Some((state.run.clone(), Arc::clone(state.presentations.as_ref()?)))
    }

    /// Return a run only when it belongs to the authenticated owner.
    /// Includes the verified tenant without depending on successful admission.
    pub(crate) async fn get_run_for_context(
        &self,
        user: &crate::uar::security::claims::UserContext,
        run_id: &str,
    ) -> Option<Run> {
        let owner = if user.user_id == crate::session::ANONYMOUS_SESSION_OWNER {
            if user.claims.sub != user.user_id || user.tenant_id.is_some() {
                return None;
            }
            None
        } else {
            Some(
                crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)
                    .ok()?,
            )
        };
        let runs = self.active_runs.read().await;
        let state = runs.get(run_id)?;
        (state.verified_owner == owner
            && state
                .run
                .user_id
                .as_deref()
                .unwrap_or(crate::session::ANONYMOUS_SESSION_OWNER)
                == user.user_id)
            .then(|| state.run.clone())
    }

    /// Return a run only when it belongs to the authenticated subject.
    pub async fn get_run_for_user(&self, owner_id: &str, run_id: &str) -> Option<Run> {
        self.get_run(run_id).await.filter(|run| match &run.user_id {
            Some(run_owner) => run_owner == owner_id,
            None => owner_id == crate::session::ANONYMOUS_SESSION_OWNER,
        })
    }

    /// Return a run only for the exact verified subject and tenant identity.
    pub async fn get_run_for_owner(
        &self,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
        run_id: &str,
    ) -> Option<Run> {
        let runs = self.active_runs.read().await;
        runs.get(run_id)
            .filter(|state| state.verified_owner.as_ref() == Some(owner))
            .map(|state| state.run.clone())
    }

    pub async fn get_run_by_session_id(&self, session_id: &str) -> Option<Run> {
        self.get_run_by_session_id_for_user(crate::session::ANONYMOUS_SESSION_OWNER, session_id)
            .await
    }

    /// Return the current run for an owner-scoped conversation session.
    pub async fn get_run_by_session_id_for_user(
        &self,
        owner_id: &str,
        session_id: &str,
    ) -> Option<Run> {
        let session_key = crate::uar::persistence::tenant_storage_key(owner_id, session_id);
        let run_id = {
            let session_runs = self.session_current_run.read().await;
            session_runs.get(&session_key).cloned()
        }?;
        self.get_run_for_user(owner_id, &run_id).await
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

    /// Backfill the model route so a resolved [`EffectiveRunPolicy`] reports the
    /// model that will actually execute. Built-in agents seed an empty
    /// `provider.default` (they defer to the registry/global default, see
    /// `defaults::default_agent`), which otherwise leaves the resolved route
    /// empty — and downstream provenance surfaces (the assistant-bubble
    /// agent/provider/model chip) then show a blank model. The registry default
    /// is the on-device provider on embedded builds and the configured
    /// `llm_config.model` on service builds, so this yields the true executing
    /// route on every deployment mode. A route that already names a non-empty
    /// provider and model is left untouched, preserving precedence.
    pub(crate) async fn backfill_effective_model(&self, policy: &mut EffectiveRunPolicy) {
        let needs_backfill = policy.model.as_ref().is_none_or(|route| {
            route.provider_id.trim().is_empty() || route.model_id.trim().is_empty()
        });
        if needs_backfill && let Some((provider_id, model_id)) = self.resolve_default_model().await
        {
            policy.model = Some(ModelRoute {
                provider_id,
                model_id,
            });
        }
    }
}

/// A prior conversation turn used to seed an empty embedded session.
///
/// Embedded hosts keep their durable conversation history in their own store
/// (e.g. the mobile UI's local database). The in-process [`SessionStore`] is not
/// durable, so after a cold start the session for a conversation is empty even
/// though the host still holds the full thread. Passing the host's history as
/// `SeedMessage`s lets [`RunManager::start_run_with_history`] repopulate an empty
/// session so the model receives prior turns instead of only the current one.
#[derive(Debug, Clone)]
pub struct SeedMessage {
    /// `"user"`, `"assistant"`, `"tool"`, or `"system"` (unknown roles are
    /// treated as `user`).
    pub role: String,
    /// The message text.
    pub content: String,
    /// The tool-call id when `role == "tool"`.
    pub tool_call_id: Option<String>,
}

/// The effective configuration for a conversation: the resolved agent, the
/// stored requested policy (if any), and the effective run policy after
/// Global → Agent → Conversation → Turn resolution + model backfill. Mirrors the
/// JSON the service path returns from `GET /conversations/{id}/effective-config`.
#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    /// The agent definition the conversation resolves to.
    pub agent: AgentArtifact,
    /// The stored conversation-scoped policy, if one was saved.
    pub requested_policy: Option<ConversationPolicyRecord>,
    /// The resolved effective policy (model route backfilled).
    pub effective_policy: EffectiveRunPolicy,
}

#[cfg(test)]
mod approval_gate_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::{
        ApprovalWaitOutcome, await_approval, governance_bypass_decision, resolve_pending_approval,
    };
    use crate::llm::ToolApprovalResult;
    use crate::uar::governance::runtime_control::governance_runtime_handles;
    use std::{
        collections::HashMap,
        sync::{Arc, Barrier},
        time::Duration,
    };
    use tokio::sync::{Mutex, oneshot};

    fn sealed_local_runtime() -> (
        crate::uar::governance::runtime_control::GovernanceMutationHandle,
        crate::uar::governance::runtime_control::GovernanceGateHandle,
    ) {
        let (mutation, gate, _) = governance_runtime_handles("localhost");
        mutation.record_installed_authentication(false);
        mutation.declare_ingress("primary-http").expect("declare");
        let proof = mutation
            .register_bound_ingress("primary-http", "127.0.0.1:1906".parse().expect("address"))
            .expect("register");
        mutation.seal_ingress_inventory(&[proof]).expect("seal");
        (mutation, gate)
    }

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

    #[test]
    fn governance_precheck_is_fail_closed_until_off_and_observes_toggle_boundary() {
        let (initializing_mutation, initializing_gate, _) = governance_runtime_handles("localhost");
        assert!(governance_bypass_decision(Some(&initializing_gate)).is_none());
        initializing_mutation
            .finalize_mutation_unavailable()
            .expect("fail-closed finalization");
        assert!(governance_bypass_decision(Some(&initializing_gate)).is_none());

        let (mutation, gate) = sealed_local_runtime();
        let plan = mutation.preference_plan(Some(false)).expect("plan");
        mutation.finalize_preference(&plan).expect("finalize Off");

        let before_publication = Arc::new(Barrier::new(2));
        let after_publication = Arc::new(Barrier::new(2));
        let reader_gate = gate.clone();
        let reader_before = Arc::clone(&before_publication);
        let reader_after = Arc::clone(&after_publication);
        let reader = std::thread::spawn(move || {
            let before = governance_bypass_decision(Some(&reader_gate));
            reader_before.wait();
            reader_after.wait();
            let after = governance_bypass_decision(Some(&reader_gate));
            (before, after)
        });

        before_publication.wait();
        mutation
            .publish_committed_preference(true)
            .expect("publish On");
        after_publication.wait();
        let (before, after) = reader.join().expect("reader thread");
        assert!(matches!(
            before,
            Some(ToolApprovalResult::GovernanceBypassed)
        ));
        assert!(after.is_none());
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
