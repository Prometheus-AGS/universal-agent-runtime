use crate::config::{LlmConfig, RunsConfig, SessionsConfig, SkillEvolutionConfig};
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
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, Notify, RwLock, broadcast, oneshot, watch};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;
use tracing::instrument;
use uuid::Uuid;

const EVENT_HISTORY_LIMIT: usize = 512;

#[cfg(test)]
#[path = "presentation_history_tests.rs"]
mod presentation_history_tests;

/// Graph nodes own their message bodies, so retain those bodies while binding
/// the same host-reviewed final-wire budget used by ordinary model attempts.
struct DestinationBudgetDriver {
    inner: Arc<dyn LlmDriver>,
    contract: Option<crate::uar::runtime::context::budget::RequestBudgetContract>,
    output_token_cap: u32,
}

#[async_trait::async_trait]
impl LlmDriver for DestinationBudgetDriver {
    async fn stream(
        &self,
        mut request: crate::llm::LlmRequest,
    ) -> anyhow::Result<crate::llm::ExternalDriverStream> {
        if let Some(contract) = &self.contract {
            if let Some(existing) = &request.budget_contract {
                anyhow::ensure!(
                    existing == contract,
                    "Graph request conflicts with its captured destination budget"
                );
            }
            request.budget_contract = Some(contract.clone());
        }
        request.output_token_cap = Some(
            request
                .output_token_cap
                .map_or(self.output_token_cap, |cap| cap.min(self.output_token_cap)),
        );
        self.inner.stream(request).await
    }
}

#[derive(Clone, Debug)]
pub struct StreamEvent {
    pub id: u64,
    pub event: NormalizedEvent,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingApprovalView {
    pub version: u32,
    pub event_id: String,
    pub cursor: Option<u64>,
    #[serde(flatten)]
    pub approval: crate::uar::runtime::thread::approvals::PendingApprovalSnapshot,
}

#[derive(Debug)]
struct EventHistory {
    next_id: u64,
    buffer: VecDeque<StreamEvent>,
    presentation: Option<super::presentations::PresentationObservation>,
    latest_presentation: Option<StreamEvent>,
    pending_client_handoff: Option<NormalizedEvent>,
    client_handoff_failed: bool,
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
        presentation_enabled: bool,
        sender: &broadcast::Sender<StreamEvent>,
        completion: Option<
            &std::sync::Mutex<crate::uar::runtime::thread::execution::RunCompletionCapture>,
        >,
    ) -> Vec<StreamEvent> {
        // A durable pending row is not proof that producer cleanup succeeded.
        // Keep caller handoff private until the host publishes its settled terminal.
        if matches!(&event, NormalizedEvent::ClientToolsRequested { .. }) {
            if !self.client_handoff_failed {
                self.pending_client_handoff = Some(event);
            }
            return Vec::new();
        }
        if matches!(
            &event,
            NormalizedEvent::Error { .. } | NormalizedEvent::Cancelled { .. }
        ) {
            self.client_handoff_failed = true;
            self.pending_client_handoff = None;
        }
        let terminal = matches!(
            &event,
            NormalizedEvent::RunDone { .. }
                | NormalizedEvent::RunDoneWithUsage { .. }
                | NormalizedEvent::Cancelled { .. }
                | NormalizedEvent::Error { .. }
        );
        // Publication observations follow their actual event. Terminal state
        // precedes the terminal frame, which closes transport subscribers.
        let mut published = Vec::new();
        if matches!(
            &event,
            NormalizedEvent::RunDone { .. } | NormalizedEvent::RunDoneWithUsage { .. }
        ) && !self.client_handoff_failed
            && let Some(handoff) = self.pending_client_handoff.take()
        {
            published.push(self.publish(handoff, sender, completion));
        }

        if !terminal {
            published.push(self.publish(event.clone(), sender, completion));
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
        if let Some(snapshot) = snapshot.filter(|_| presentation_enabled && observation_changed) {
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
                let projected = self.publish(projection, sender, completion);
                self.latest_presentation = Some(projected.clone());
                published.push(projected);
            }
        }
        if terminal {
            published.push(self.publish(event, sender, completion));
        }
        published
    }
}

/// Write-through queue for the restart-durable public run history. The
/// streaming path must not wait on the store; ordering within a run is carried
/// by the monotonically assigned event ids, and each run's terminal record
/// upsert is queued after every event already published for that run.
#[derive(Clone, Debug)]
struct RunHistoryWriter {
    inner: Arc<RunHistoryWriterInner>,
}

#[derive(Debug)]
struct RunHistoryWriterInner {
    // Closing this sender under the same lock used by producers establishes the
    // queue's admission boundary. A successful send is therefore always
    // drained before `shutdown` returns.
    sender: std::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<RunHistoryWrite>>>,
    scope: crate::uar::runtime::lifecycle::ResourceScope,
    failures: Arc<std::sync::Mutex<Vec<String>>>,
}

#[derive(Debug)]
enum RunHistoryWrite {
    Record(crate::uar::persistence::run_history::RunHistoryRecord),
    Event(crate::uar::persistence::run_history::RunHistoryEvent),
}

impl RunHistoryWriter {
    fn start(
        persistence: &Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
    ) -> Option<Self> {
        let persistence = persistence
            .clone()
            .filter(|layer| layer.supports_durable_run_history())?;
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let failures = Arc::new(std::sync::Mutex::new(Vec::new()));
        let task_failures = Arc::clone(&failures);
        let scope = crate::uar::runtime::lifecycle::ResourceScope::standalone(
            "run-history-writer",
            "run-manager",
        );
        let task_scope = scope.clone();
        if let Err(error) = task_scope.spawn("run-history-writer", async move {
            while let Some(write) = receiver.recv().await {
                let result = match &write {
                    RunHistoryWrite::Record(record) => {
                        persistence.upsert_run_history_record(record).await
                    }
                    RunHistoryWrite::Event(event) => {
                        persistence.append_run_history_event(event).await
                    }
                };
                // Durable history must not fail the live stream; the read path
                // still serves the in-process history for this boot.
                if let Err(error) = result {
                    tracing::warn!(?error, "durable run history write failed");
                    task_failures
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(format!("{error:#}"));
                }
            }
            Ok(())
        }) {
            failures
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(format!("history writer could not start: {error}"));
        }
        Some(Self {
            inner: Arc::new(RunHistoryWriterInner {
                sender: std::sync::Mutex::new(Some(sender)),
                scope,
                failures,
            }),
        })
    }

    fn write_record(
        &self,
        run: &Run,
        verified_owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
    ) {
        let Ok(serialized) = serde_json::to_value(run) else {
            tracing::warn!("durable run history record serialization failed");
            return;
        };
        let record = crate::uar::persistence::run_history::RunHistoryRecord {
            owner_key: verified_owner.map(|owner| owner.presentation_owner_key()),
            run_id: run.run_id.clone(),
            run: serialized,
            revision: 1,
        };
        self.send(RunHistoryWrite::Record(record));
    }

    fn write_events(&self, run_id: &str, events: Vec<StreamEvent>) {
        for event in events {
            let Ok(serialized) = serde_json::to_value(&event.event) else {
                tracing::warn!("durable run history event serialization failed");
                continue;
            };
            self.send(RunHistoryWrite::Event(
                crate::uar::persistence::run_history::RunHistoryEvent {
                    run_id: run_id.to_owned(),
                    event_id: event.id,
                    event: serialized,
                },
            ));
        }
    }

    fn send(&self, write: RunHistoryWrite) {
        let sender = self
            .inner
            .sender
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(sender) = sender.as_ref() else {
            tracing::warn!("durable run history writer is closed");
            return;
        };
        if sender.send(write).is_err() {
            self.inner
                .failures
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("history writer task ended before queue drain".into());
        }
    }

    /// Close writer admission, drain every accepted write, and join the
    /// writer while its persistence dependency remains usable.
    async fn shutdown(&self) -> anyhow::Result<()> {
        self.inner
            .sender
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let report = self.inner.scope.close().await;
        if !report.released {
            self.inner
                .failures
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .extend(report.failures);
        }
        let failures = std::mem::take(
            &mut *self
                .inner
                .failures
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        if failures.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "durable run history drain failed: {}",
                failures.join("; ")
            ))
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

fn redacted_checkpoint_dialogue(session: &crate::session::Session) -> RunDialogue {
    let mut state = session.to_state();
    state.messages.clear();
    state.system_prompt = None;
    RunDialogue(crate::session::Session::from_state(state))
}

fn pre_resolved_policy_for_run(
    is_checkpoint_resume: bool,
    inherited_policy: Option<EffectiveRunPolicy>,
    requested_policy: Option<EffectiveRunPolicy>,
) -> Option<EffectiveRunPolicy> {
    if is_checkpoint_resume {
        None
    } else {
        inherited_policy.or(requested_policy)
    }
}

#[cfg(test)]
mod checkpoint_dialogue_tests {
    use super::{pre_resolved_policy_for_run, redacted_checkpoint_dialogue};

    #[test]
    fn revoked_checkpoint_dialogue_excludes_warm_session_content() {
        let session = crate::session::SessionStore::new().create_for_user("alice");
        session.add_user_message("previously-authorized-sentinel");
        session.set_system_prompt("private-system-sentinel");

        let dialogue = redacted_checkpoint_dialogue(&session);

        assert!(dialogue.0.messages().is_empty());
        assert!(dialogue.0.system_prompt().is_none());
    }

    #[test]
    fn checkpoint_resume_ignores_a_stale_pre_resolved_policy() {
        let stale = crate::uar::domain::policy::resolve_run_policy(Default::default());

        assert!(pre_resolved_policy_for_run(true, None, Some(stale)).is_none());
    }
}

#[derive(Clone)]
struct ActorToolApprovalGate(crate::llm::ToolApprovalGate);

impl std::fmt::Debug for ActorToolApprovalGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ActorToolApprovalGate(..)")
    }
}

#[derive(Debug)]
struct RunStreamState {
    run: Run,
    caller_handoff_confirmation: Arc<Mutex<()>>,
    approval_root_run_id: Option<String>,
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
    tool_admission: Option<Arc<crate::uar::runtime::tool_admission::ToolAdmissionRuntime>>,
    actor_spawn_descriptor: Option<Arc<crate::uar::tools::descriptor::ToolDescriptor>>,
    actor_tool_approval: Option<ActorToolApprovalGate>,
    started_at: Instant,
    terminal_at: Option<Instant>,
    last_detached_at: Option<Instant>,
}

#[derive(Clone, Debug)]
struct RunEventEmitter {
    run_id: String,
    presentations: Option<Arc<super::presentations::RunPresentationSnapshot>>,
    sender: broadcast::Sender<StreamEvent>,
    history: Arc<Mutex<EventHistory>>,
    completion:
        Option<Arc<std::sync::Mutex<crate::uar::runtime::thread::execution::RunCompletionCapture>>>,
    runs: std::sync::Weak<RwLock<ActiveRunMap>>,
    secret_scrubber: crate::uar::runtime::turn::host::RunSecretScrubber,
    admission_projection: Arc<Mutex<crate::uar::runtime::turn::host::AdmissionProjection>>,
    run_history: Option<RunHistoryWriter>,
    retention_work: Arc<Notify>,
}

impl RunEventEmitter {
    /// A preparation refusal is not a serialized final request or exact fit proof.
    async fn record_budget_refusal(&self, code: &str, candidate: &serde_json::Value) {
        if !matches!(code, "context_overflow" | "protected_history_overflow") {
            return;
        }
        let mut receipt = candidate.clone();
        receipt["outcome"] = serde_json::json!("protected_overflow");
        receipt["failure_code"] = serde_json::json!(code);
        if let Some(runs) = self.runs.upgrade()
            && let Some(state) = runs.write().await.get_mut(&self.run_id)
            && let Some(context) = state.run.context.as_object_mut()
        {
            context.insert("request_budget".into(), receipt);
        }
    }

    async fn emit(&self, mut event: NormalizedEvent) {
        let mut projection = self.admission_projection.lock().await;
        let Some(projected) = projection.project_or_stage(event) else {
            return;
        };
        let presentation_enabled = !projection.suppresses_presentation();
        drop(projection);
        event = projected;
        let terminal = matches!(
            &event,
            NormalizedEvent::RunDone { .. }
                | NormalizedEvent::RunDoneWithUsage { .. }
                | NormalizedEvent::Cancelled { .. }
                | NormalizedEvent::Error { .. }
        );
        // Terminal run metadata is final before the terminal frame publishes.
        let mut terminal_record = None;
        if terminal
            && let Some(runs) = self.runs.upgrade()
            && let Some(state) = runs.write().await.get_mut(&self.run_id)
        {
            state.terminal_at.get_or_insert_with(Instant::now);
            terminal_record = Some((state.run.clone(), state.verified_owner.clone()));
            self.retention_work.notify_one();
        }
        let event =
            super::a2ui_output::enforce_output_ceiling(event, self.presentations.as_deref());
        let published = {
            let mut history = self.history.lock().await;
            history.record(
                &self.run_id,
                event,
                self.presentations.as_deref(),
                presentation_enabled,
                &self.sender,
                self.completion.as_deref(),
            )
        };
        if let Some(writer) = &self.run_history {
            if let Some((run, owner)) = terminal_record {
                writer.write_record(&run, owner.as_ref());
            }
            writer.write_events(&self.run_id, published);
        }
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
    embedding: Arc<dyn crate::uar::rag::embeddings::EmbeddingBackend>,
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
        // Same backend as ingestion (`llm.embedding`); see the KB search route.
        // Pinned once so a concurrent backend swap cannot interleave.
        let query_vec = self.embedding.embed_one(sub_query).await?;
        let kb_id_refs = self.kb_ids.iter().map(String::as_str).collect::<Vec<_>>();
        self.persistence
            .search_knowledge_scoped_in_space(
                self.owner_id,
                &kb_id_refs,
                &query_vec,
                limit,
                min_score,
                &self.embedding.fingerprint(),
            )
            .await
    }
}

type ActiveRunMap = HashMap<String, RunStreamState>;

/// Existing actors keep their handles; ordinary producers gain a host owner.
enum CheckpointProducer {
    Ordinary(Option<tokio::task::JoinHandle<()>>),
    Actor(crate::uar::runtime::thread::actor_host::ActorRootBinding),
}

/// Private authority minted only after the exact durable CAS acknowledgment.
struct CheckpointResumePermit {
    claim: crate::uar::persistence::checkpoint_resume::CheckpointContinuationClaim,
    envelope: crate::uar::persistence::checkpoint_resume::CheckpointResumeEnvelope,
    owner: crate::uar::runtime::actor::messages::ActorOwner,
    request_identity: serde_json::Value,
    approval: Option<crate::uar::persistence::approval_decisions::ApprovalRecord>,
}

fn checkpoint_request_identity(
    request: &crate::uar::runtime::turn::RunExecutionRequest,
) -> anyhow::Result<serde_json::Value> {
    let resume = request
        .checkpoint_resume
        .as_ref()
        .ok_or(crate::uar::persistence::checkpoint_resume::CheckpointResumeError::Invalid)?;
    Ok(serde_json::json!({
        "artifact_revision": request.artifact.definition_revision(),
        "user_id": request.user_id,
        "session_id": request.session_id,
        "input": request.input,
        "response_format": request.response_format,
        "state": serde_json::to_value(&resume.state)?,
        "history": serde_json::to_value(&resume.history)?,
        "authorization": resume.authorization_digest,
        "host_policy": request.host_policy_constraint,
        "a2a_delegation": request.a2a_delegation,
        "resolved_policy": request.resolved_policy,
        "host_budget": request.host_budget_constraint,
        "host_sandbox": request.host_sandbox_constraint,
        "host_usage": request.host_usage_grant.as_ref().map(|binding| serde_json::json!({
            "accounting_id": binding.accounting_id, "grant": binding.grant,
        })),
        "target": request.target_selection,
        "resources": request.host_resources_marker,
        "service_binding": request.service_binding,
        "collaboration": request.collaboration_binding.as_ref().map(|binding| serde_json::json!({
            "owner": binding.owner_id, "workspace": binding.workspace_id, "receipt": binding.receipt,
        })),
        "working_directory": request.working_directory,
        "reasoning_effort": request.reasoning_effort,
        "presentation": request.presentation_negotiation,
    }))
}

impl CheckpointResumePermit {
    fn accepts(
        &self,
        request: &crate::uar::runtime::turn::RunExecutionRequest,
        run_id: &str,
        epoch: &str,
    ) -> bool {
        self.claim.continuation_run_id == run_id
            && self.claim.executor_runtime_epoch == epoch
            && request.verified_owner.as_ref() == Some(&self.owner)
            && self
                .owner
                .credential_principal()
                .is_some_and(|principal| principal.storage_key() == self.claim.owner_key)
            && self.claim.validate(&self.envelope).is_ok()
            && checkpoint_request_identity(request)
                .is_ok_and(|identity| identity == self.request_identity)
    }
}

/// Only these existing public ingress shapes can renew a host policy on resume.
/// Other host constraints must not disappear behind an artifact-only restore.
fn checkpoint_host_policy_renewable(
    request: &crate::uar::runtime::turn::RunExecutionRequest,
) -> bool {
    let Some(policy) = request.host_policy_constraint.as_ref() else {
        return true;
    };
    if request
        .a2a_delegation
        .as_ref()
        .is_some_and(|contract| contract.execution_policy() == *policy)
    {
        return true;
    }
    let expected = if let Some(target) = request.target_selection.as_ref() {
        let Some(provider_id) = target
            .get("provider_id")
            .and_then(serde_json::Value::as_str)
        else {
            return false;
        };
        let Some(model_id) = target.get("model_id").and_then(serde_json::Value::as_str) else {
            return false;
        };
        Some(crate::uar::domain::policy::RunPolicy {
            model: Some(crate::uar::domain::policy::ModelRoute {
                provider_id: provider_id.to_owned(),
                model_id: model_id.to_owned(),
            }),
            ..Default::default()
        })
    } else {
        request
            .collaboration_binding
            .as_ref()
            .and_then(|binding| binding.receipt.effective.get("contextStrategy"))
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
            .map(|context_strategy| crate::uar::domain::policy::RunPolicy {
                context_strategy: Some(context_strategy),
                ..Default::default()
            })
    };
    expected.as_ref() == Some(policy)
}

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
    skill_required_roles: BTreeMap<String, BTreeSet<String>>,
    global_mcp: Arc<McpRegistry>,
    /// Shared projected runtime for verified root turns. Definitions remain in
    /// `global_mcp`/the skill catalog and are frozen into each root request.
    mcp_runtime: Option<McpRuntimeManager>,
    mcp_environment: Option<Arc<McpBindingEnvironment>>,
    /// Opaque host binding revision shared by definitions captured this boot.
    /// Environment/config hashes remain separate parts of the exact cache key.
    mcp_auth_revision: Uuid,
    /// Runtime boot identity bound into every prepared tool invocation.
    tool_runtime_epoch: String,
    /// Host admission implementation. Standalone UAR installs a local adapter;
    /// a paired host replaces it before serving runs.
    host_tool_admission: Arc<dyn crate::uar::runtime::tool_admission::HostToolAdmissionPort>,
    sessions: SessionStore,
    skills: Arc<RwLock<SkillRegistry>>,
    vector_matcher: Arc<crate::uar::runtime::matching::VectorMatcher>,
    local_skill_matcher: Option<Arc<crate::uar::runtime::matching::VectorMatcher>>,
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
    /// Restart-durable public run history write-through; `None` when no
    /// configured store supports it (behavior stays process-local there).
    run_history: Option<RunHistoryWriter>,
    /// Trusted-host task scope for background work started by this manager.
    /// It is attached before serving; standalone construction remains valid
    /// without a host scope for embedding compatibility.
    lifecycle_scope: crate::uar::runtime::lifecycle::ResourceScope,
    /// Skill service for coordinated skill management
    skill_service: Option<Arc<SkillService>>,
    /// Configuration for the Hermes skill auto-creation/evolution post-run hook.
    skill_evolution_config: SkillEvolutionConfig,
    /// Provider registry for per-agent LLM provider resolution
    provider_registry: Option<Arc<crate::llm::ProviderRegistry>>,
    /// Multi-tenant credential service (per-user encrypted provider keys).
    /// `None` ⇒ single-tenant: the env/config key on `llm_config` is used as-is.
    provider_service: Option<Arc<crate::uar::security::credentials::ProviderService>>,
    /// Trusted host policy that authorizes every provider source before client
    /// construction. It remains present when encrypted credential storage is
    /// intentionally unavailable.
    credential_policy: Arc<crate::uar::security::credentials::CredentialTenancyPolicy>,
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
    /// Exact host-reviewed per-destination preparation contracts. Configured
    /// runtimes fail closed when a selected primary or fallback has no profile.
    destination_preparations: Option<Arc<crate::llm::DestinationRequestPreparations>>,
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
    /// Retained registration generation for lifecycle commands that arrive
    /// before a run has installed its cancellation capability.
    run_admission_generation: Arc<watch::Sender<u64>>,
    /// Explicit lifecycle activity wakes the one-shot retention owner.
    retention_work: Arc<Notify>,
    /// Monotonic durable-work generation. Persistence and realtime adapters
    /// advance it only after a committed change; observers subscribe before
    /// their authoritative snapshot to avoid a poll/snapshot race.
    committed_work_generation: Arc<watch::Sender<u64>>,
    /// Shutdown waits for admission/assembly before joining every producer.
    checkpoint_assemblies: Arc<RwLock<()>>,
    checkpoint_producers: Arc<Mutex<HashMap<String, Arc<Mutex<CheckpointProducer>>>>>,
    /// Canonical receipt partitions and source trees owned by this incarnation.
    checkpoint_effect_roots: Arc<RwLock<BTreeSet<(String, String)>>>,
    checkpoint_evidence_owners: Arc<RwLock<BTreeSet<String>>>,
    /// Minted after durable CAS and consumed once at shared kernel entry.
    checkpoint_resume_admitted: Arc<Mutex<HashMap<String, CheckpointResumePermit>>>,
    checkpoint_cleanup_failed: Arc<std::sync::atomic::AtomicBool>,
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
    run_retention: RunsConfig,
    session_retention: SessionsConfig,
}

/// Memory mutation tool name sets — used to detect side effects in ToolEnd events.
const MEMORY_CREATE_TOOLS: &[&str] = &[
    "memory_add",
    "memory_save",
    "memory_extract_from_conversation",
];
const MEMORY_UPDATE_TOOLS: &[&str] = &["memory_update"];
const MEMORY_DELETE_TOOLS: &[&str] = &["memory_delete", "memory_delete_all"];

/// Capture one policy-authorized provider source before a client constructor can
/// inspect configuration or an ambient credential. The returned decision remains
/// in the run binding; descendants can only select that captured client.
async fn resolve_credential_binding(
    cfg: LlmConfig,
    policy: &crate::uar::security::credentials::CredentialTenancyPolicy,
    provider_service: Option<&Arc<crate::uar::security::credentials::ProviderService>>,
    principal: Option<&crate::uar::security::claims::CredentialPrincipal>,
    session_id: Option<&str>,
    agent_id: Option<&str>,
    explicit_request_authorized: bool,
) -> anyhow::Result<(
    LlmConfig,
    crate::uar::security::credentials::CredentialSourceDecision,
)> {
    let provider_id = provider_id_for_config(&cfg);
    let endpoint = cfg.base_url.as_deref().unwrap_or("configured-provider");
    let decision = if explicit_request_authorized {
        policy.explicit_request(principal, &provider_id, endpoint, cfg.api_key.as_deref())?
    } else {
        policy
            .resolve_source(
                provider_service.map(Arc::as_ref),
                principal,
                &provider_id,
                endpoint,
                session_id,
                agent_id,
                cfg.api_key.as_deref(),
            )
            .await?
    };
    let cfg = crate::uar::runtime::turn::bindings::apply_credential_decision(cfg, &decision)?;
    Ok((cfg, decision))
}

/// Classify host binding failures without projecting provider, storage, or
/// constructor details into the run stream or trusted logs.
fn run_binding_failure(error: &anyhow::Error) -> (&'static str, &'static str) {
    if let Some(origin) =
        error.downcast_ref::<crate::uar::runtime::cost_budget::BudgetFailureOrigin>()
    {
        return ("budget_admission", origin.category());
    }
    if matches!(
        error.downcast_ref::<crate::uar::runtime::thread::policy_intersection::PolicyIntersectionError>(),
        Some(crate::uar::runtime::thread::policy_intersection::PolicyIntersectionError::UnsupportedShape { section: "budgets" })
    ) {
        return ("budget_admission", "budget_admission_failed");
    }
    match error.to_string().as_str() {
        "remote_surreal_durability_not_attested" => (
            "durable_accounting",
            "remote_surreal_durability_not_attested",
        ),
        "durable_storage_unsupported" | "durable_accounting_unsupported" => {
            ("durable_accounting", "durable_storage_unsupported")
        }
        "accounting_verified_principal_required" => {
            ("durable_accounting", "verified_principal_required")
        }
        "budget_admission_failed" => ("budget_admission", "budget_admission_failed"),
        "shared credential policy cannot verify a supplied primary driver" => {
            ("client_construction", "opaque_driver_provenance_missing")
        }
        "Fallback model lacks credential decision" | "credential binding unavailable" => {
            ("credential_binding", "credential_binding_missing")
        }
        "credential_source_denied"
        | "credential_identity_required"
        | "credential_installation_id_required"
        | "credential_migration_required" => ("credential_binding", "credential_source_denied"),
        _ => ("client_construction", "client_capture_failed"),
    }
}

fn provider_id_for_config(config: &LlmConfig) -> String {
    config
        .resolved_provider_id
        .clone()
        .unwrap_or_else(|| crate::llm::registry::split_model_string_pub(&config.model).0)
}

fn qualified_model_name(config: &LlmConfig) -> String {
    let provider_id = provider_id_for_config(config);
    let (model_provider, split_model_id) =
        crate::llm::registry::split_model_string_pub(&config.model);
    let model_id = if config.model.contains('/') && model_provider == provider_id {
        split_model_id
    } else {
        config.model.clone()
    };
    format!("{provider_id}/{model_id}")
}

fn apply_routed_connection(mut base: LlmConfig, routed: LlmConfig) -> LlmConfig {
    let provider_changed = provider_id_for_config(&base) != provider_id_for_config(&routed);
    base.model = routed.model;
    base.resolved_provider_id = routed.resolved_provider_id;
    base.catalog_pricing_model = routed.catalog_pricing_model;
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

/// RAII guard tied to one SSE subscription attachment.
///
/// A broadcast receiver count cannot prove which authenticated attachment
/// disappeared or whether a replay is already being attached. Dropping this
/// guard therefore records a retention transition only. A trusted host must
/// issue an explicit, owner-scoped cancellation command when its attachment
/// lifecycle establishes that the run itself is abandoned.
#[derive(Debug)]
pub struct RunDisconnectGuard {
    manager: Arc<RunManager>,
    run_id: String,
}

impl RunDisconnectGuard {
    #[must_use]
    pub fn new(manager: Arc<RunManager>, run_id: String) -> Self {
        Self { manager, run_id }
    }
}

impl Drop for RunDisconnectGuard {
    fn drop(&mut self) {
        if !self.run_id.is_empty() {
            self.manager.notify_retention_work();
        }
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
    /// Commit the conversation before a durable logical turn is reported terminal.
    pub(crate) async fn persist_instance_session(
        &self,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
        session_id: &str,
    ) -> anyhow::Result<()> {
        let owner_id = owner.session_owner_key()?;
        let session = self
            .sessions
            .get_for_user(session_id, &owner_id)
            .ok_or_else(|| anyhow::anyhow!("Instance conversation is unavailable"))?;
        let persistence = self
            .persistence
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Instance conversation has no durable store"))?;
        persistence.save_session(&session).await
    }

    /// Execute an authenticated direct host call through the same descriptor,
    /// admission, claim, dispatch, and terminal lifecycle as model tool calls.
    pub async fn execute_direct_tool(
        &self,
        owner_id: String,
        principal_id: String,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let _assembly = tokio::select! {
            biased;
            () = self.root_cancellation.cancelled() => anyhow::bail!("runtime is shutting down"),
            guard = self.checkpoint_assemblies.read() => guard,
        };
        anyhow::ensure!(
            !self.root_cancellation.is_cancelled(),
            "runtime is shutting down"
        );
        self.checkpoint_evidence_owners
            .write()
            .await
            .insert(owner_id.clone());
        use crate::uar::runtime::tool_admission::{
            LocalAdmissionDisposition, ToolAdmissionContext, ToolAdmissionRuntime,
        };
        use crate::uar::tools::descriptor::{ApprovalClass, Exposure};

        anyhow::ensure!(!owner_id.trim().is_empty(), "Direct tool owner is missing");
        anyhow::ensure!(
            !principal_id.trim().is_empty(),
            "Direct tool principal is missing"
        );
        let descriptor = self
            .global_mcp
            .descriptor(tool_name)
            .ok_or_else(|| anyhow::anyhow!("unknown tool: {tool_name}"))?;
        anyhow::ensure!(
            descriptor.exposure != Exposure::ModelOnly,
            "Tool is restricted to model-controlled execution"
        );
        let argument_errors = descriptor
            .validator
            .iter_errors(&arguments)
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        anyhow::ensure!(
            argument_errors.is_empty(),
            "invalid tool arguments: {}",
            argument_errors.join("; ")
        );
        anyhow::ensure!(
            descriptor.approval_class != ApprovalClass::Required,
            "Direct tool execution requires an interactive approval channel"
        );

        let governance_policy_revision = self
            .governance_engine
            .as_ref()
            .map(|engine| engine.policy_revision());
        let governance_policy_revision = match governance_policy_revision {
            Some(revision) => revision.await,
            None => "cedar:unavailable".to_string(),
        };
        let host = Arc::clone(&self.host_tool_admission);
        let context = ToolAdmissionContext::direct(
            owner_id,
            principal_id,
            std::env::current_dir()?.display().to_string(),
            self.tool_runtime_epoch.clone(),
            format!("direct:{tool_name}"),
            governance_policy_revision,
            host.binding(),
        );
        let runtime = ToolAdmissionRuntime::new_in_scope(
            context,
            host,
            self.persistence.clone(),
            self.root_cancellation.child_token(),
            self.governance_engine.clone(),
            self.governance_gate.clone(),
            self.lifecycle_scope.clone(),
        )?;
        let prepared = runtime.prepare(
            uuid::Uuid::new_v4().to_string(),
            descriptor.as_ref(),
            arguments.clone(),
            0,
        );
        let host_preparation = runtime.prepare_host(Arc::clone(&prepared)).await?;
        anyhow::ensure!(
            host_preparation.host_disposition
                != crate::uar::runtime::tool_admission::HostAdmissionDisposition::Deny,
            "Direct tool execution is denied by the host"
        );
        anyhow::ensure!(
            host_preparation.host_disposition
                != crate::uar::runtime::tool_admission::HostAdmissionDisposition::Ask,
            "Direct tool execution requires an interactive host approval channel"
        );
        let admitted = runtime
            .resolve(
                prepared,
                host_preparation,
                LocalAdmissionDisposition::Allowed,
                true,
            )
            .await?
            .ok_or_else(|| anyhow::anyhow!("Direct tool execution was not admitted"))?;
        runtime.claim(&admitted).await?;
        let result = self
            .global_mcp
            .call_namespaced_tool_with_meta(
                tool_name,
                arguments,
                admitted.host_receipt.mcp_request_meta(),
            )
            .await;
        runtime.finish(&admitted, result.is_ok()).await?;
        result
    }

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

        let tool_runtime_epoch = Uuid::new_v4().to_string();
        let host_tool_admission: Arc<
            dyn crate::uar::runtime::tool_admission::HostToolAdmissionPort,
        > = Arc::new(
            crate::uar::runtime::tool_admission::StandaloneToolAdmissionPort::new(
                &tool_runtime_epoch,
            ),
        );
        let approvals = ApprovalBroker::new(persistence.clone(), tool_runtime_epoch.clone());
        let run_history = RunHistoryWriter::start(&persistence);
        let (run_admission_generation, _) = watch::channel(0);
        let (committed_work_generation, _) = watch::channel(0);
        let retention_work = Arc::new(Notify::new());
        let lifecycle_scope = crate::uar::runtime::lifecycle::ResourceScope::standalone(
            "run-manager",
            "embedded-run-manager",
        );
        let root_cancellation = lifecycle_scope.cancellation_token();
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
            tool_runtime_epoch,
            host_tool_admission,
            sessions,
            skills,
            vector_matcher,
            local_skill_matcher: None,
            tag_matcher,
            intent_classifier,
            classifier_config,
            skill_required_roles: BTreeMap::new(),
            harness_config: crate::config::HarnessConfig::default(),
            project_instructions_config: Default::default(),
            world_state_config: Default::default(),
            world_state_clock: Arc::new(crate::uar::runtime::world_state::sections::SystemClock),
            persistence,
            run_history,
            lifecycle_scope,
            settings_manager,
            skill_service: None,
            skill_evolution_config: SkillEvolutionConfig::default(),
            provider_registry: None,
            provider_service: None,
            credential_policy: Arc::new(Default::default()),
            native_skills,
            sandbox_runner: None,
            sandbox_operations: Arc::new(crate::sandbox::execution::SandboxSupervisor::default()),
            terminal_operations: Arc::new(
                crate::uar::tools::terminal_process::TerminalSupervisor::default(),
            ),
            a2ui_backbone: crate::uar::a2ui::realtime::InMemoryReplayBackbone::new(),
            primary_driver: None,
            destination_preparations: None,
            approvals,
            root_cancellation,
            run_cancellations: Arc::new(RwLock::new(HashMap::new())),
            run_admission_generation: Arc::new(run_admission_generation),
            retention_work,
            committed_work_generation: Arc::new(committed_work_generation),
            checkpoint_assemblies: Arc::new(RwLock::new(())),
            checkpoint_producers: Arc::new(Mutex::new(HashMap::new())),
            checkpoint_effect_roots: Arc::new(RwLock::new(BTreeSet::new())),
            checkpoint_evidence_owners: Arc::new(RwLock::new(BTreeSet::new())),
            checkpoint_resume_admitted: Arc::new(Mutex::new(HashMap::new())),
            checkpoint_cleanup_failed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            message_context_strategy: crate::uar::context::ContextStrategy::default(),
            agent_graph: None,
            governance_engine: None,
            governance_gate: None,
            failover_config: crate::config::FailoverConfig::default(),
            cost_budget: crate::uar::runtime::cost_budget::CostBudgetTracker::new(),
            resilience_policy: crate::uar::settings::resilience_policy::ResiliencePolicy::default(),
            a2a_peers: Arc::new(crate::uar::api::a2a::peer::TrustedA2APeers::default()),
            run_retention: RunsConfig::default(),
            session_retention: SessionsConfig::default(),
        }
    }

    #[must_use]
    pub(crate) fn with_retention_config(
        mut self,
        runs: RunsConfig,
        sessions: SessionsConfig,
    ) -> Self {
        self.run_retention = runs;
        self.session_retention = sessions;
        self
    }

    /// Start the one retention owner for run replay state and conversation sessions.
    ///
    /// It wakes on a retained terminal/detach/activity signal or the earliest
    /// actual expiry. The expiry sleep models business retention time; it never
    /// certifies that work or shutdown has completed.
    pub(crate) fn spawn_retention_sweeper(self: &Arc<Self>, shutdown: CancellationToken) {
        let manager = Arc::clone(self);
        let work = Arc::clone(&manager.retention_work);
        let scope = manager.lifecycle_scope.clone();
        let scope_cancellation = scope.cancellation_token();
        let mut session_activity = manager.sessions.subscribe_activity();
        let mut persisted_session_activity = manager
            .persistence
            .as_ref()
            .and_then(|persistence| persistence.subscribe_persisted_session_changes());
        if let Err(error) = scope.spawn("run-retention", async move {
            loop {
                manager.sweep_retained_state().await;
                match manager.next_retention_due().await {
                    Some(due) => tokio::select! {
                        _ = shutdown.cancelled() => break,
                        _ = scope_cancellation.cancelled() => break,
                        _ = work.notified() => {},
                        changed = session_activity.changed() => {
                            if changed.is_err() {
                                break;
                            }
                        },
                        changed = async {
                            match &mut persisted_session_activity {
                                Some(activity) => activity.changed().await,
                                None => std::future::pending().await,
                            }
                        } => {
                            if changed.is_err() { break; }
                        },
                        _ = tokio::time::sleep_until(due) => {},
                    },
                    None => tokio::select! {
                        _ = shutdown.cancelled() => break,
                        _ = scope_cancellation.cancelled() => break,
                        _ = work.notified() => {},
                        changed = session_activity.changed() => {
                            if changed.is_err() {
                                break;
                            }
                        },
                        changed = async {
                            match &mut persisted_session_activity {
                                Some(activity) => activity.changed().await,
                                None => std::future::pending().await,
                            }
                        } => {
                            if changed.is_err() { break; }
                        },
                    },
                }
            }
            Ok(())
        }) {
            tracing::warn!(%error, "Run retention owner was rejected during lifecycle shutdown");
        }
    }

    async fn next_retention_due(&self) -> Option<tokio::time::Instant> {
        let retention = Duration::from_secs(self.run_retention.retention_after_terminal_secs);
        let runs = self.active_runs.read().await;
        let run_due = runs
            .values()
            .filter_map(|state| {
                let terminal = state.terminal_at?;
                let detached = state.last_detached_at?;
                let has_live_descendant = state
                    .delegation
                    .as_ref()
                    .is_some_and(|delegation| delegation.upgrade().is_some());
                let deadline = tokio::time::Instant::from_std(terminal.max(detached) + retention);
                // The loop has already made one authoritative sweep for an
                // expired record. Do not reschedule unchanged overdue state
                // at `Instant::now()`: a failed cleanup must wait for a
                // committed-work or activity event rather than poll.
                (!has_live_descendant && deadline > tokio::time::Instant::now()).then_some(deadline)
            })
            .min();
        drop(runs);
        let idle_timeout = (self.session_retention.idle_timeout_secs > 0)
            .then(|| Duration::from_secs(self.session_retention.idle_timeout_secs));
        let protected = self
            .active_runs
            .read()
            .await
            .values()
            .filter(|state| {
                !matches!(
                    state.run.status,
                    RunStatus::Done | RunStatus::Error | RunStatus::Cancelled
                )
            })
            .filter_map(|state| {
                state
                    .run
                    .conversation_id
                    .clone()
                    .map(|session_id| (state.dialogue.0.owner_id().to_owned(), session_id))
            })
            .collect::<HashSet<_>>();
        let session_due = self.sessions.next_idle_expiry(idle_timeout, &protected);
        let persisted_due = if let (Some(ttl_secs), Some(persistence)) = (
            self.session_retention
                .persisted_ttl_secs
                .filter(|ttl| *ttl > 0),
            self.persistence.as_ref(),
        ) {
            match persistence
                .next_persisted_session_expiry(chrono::Duration::seconds(
                    i64::try_from(ttl_secs).unwrap_or(i64::MAX),
                ))
                .await
            {
                Ok(Some(deadline)) => deadline
                    .signed_duration_since(chrono::Utc::now())
                    .to_std()
                    .ok()
                    .map(|duration| tokio::time::Instant::now() + duration),
                Ok(None) => None,
                Err(error) => {
                    tracing::warn!(%error, "failed to read persisted session expiry schedule");
                    None
                }
            }
        } else {
            None
        };
        [run_due, session_due, persisted_due]
            .into_iter()
            .flatten()
            .min()
    }

    fn notify_retention_work(&self) {
        self.retention_work.notify_one();
    }

    async fn sweep_retained_state(&self) {
        let now = Instant::now();
        let retention = Duration::from_secs(self.run_retention.retention_after_terminal_secs);
        let mut evicted = Vec::new();
        {
            let mut runs = self.active_runs.write().await;
            for state in runs.values_mut() {
                let terminal = matches!(
                    state.run.status,
                    RunStatus::Done | RunStatus::Error | RunStatus::Cancelled
                );
                if terminal {
                    state.terminal_at.get_or_insert(now);
                    if state.sender.receiver_count() == 0 {
                        state.last_detached_at.get_or_insert(now);
                    } else {
                        state.last_detached_at = None;
                    }
                }
            }

            let expired = runs
                .iter()
                .filter(|(_, state)| {
                    let Some(terminal_at) = state.terminal_at else {
                        return false;
                    };
                    let Some(detached_at) = state.last_detached_at else {
                        return false;
                    };
                    let has_live_descendant = state
                        .delegation
                        .as_ref()
                        .is_some_and(|delegation| delegation.upgrade().is_some());
                    !has_live_descendant
                        && now.duration_since(terminal_at.max(detached_at)) >= retention
                })
                .map(|(run_id, _)| run_id.clone())
                .collect::<Vec<_>>();
            for run_id in expired {
                if runs.remove(&run_id).is_some() {
                    evicted.push(run_id);
                }
            }

            let cap = self.run_retention.max_retained_terminal;
            if cap > 0 {
                let terminal_count = runs
                    .values()
                    .filter(|state| state.terminal_at.is_some())
                    .count();
                if terminal_count > cap {
                    let mut eligible = runs
                        .iter()
                        .filter(|(_, state)| {
                            state.terminal_at.is_some()
                                && state.sender.receiver_count() == 0
                                && state
                                    .delegation
                                    .as_ref()
                                    .is_none_or(|delegation| delegation.upgrade().is_none())
                        })
                        .map(|(run_id, state)| {
                            (
                                run_id.clone(),
                                state.terminal_at.unwrap_or(state.started_at),
                            )
                        })
                        .collect::<Vec<_>>();
                    eligible.sort_by_key(|(_, terminal_at)| *terminal_at);
                    for (run_id, _) in eligible.into_iter().take(terminal_count - cap) {
                        if runs.remove(&run_id).is_some() {
                            evicted.push(run_id);
                        }
                    }
                }
            }
            #[allow(clippy::cast_precision_loss)]
            crate::uar::telemetry::metrics::set_runs_retained(runs.len() as f64);
        }

        if !evicted.is_empty() {
            for run_id in &evicted {
                self.a2ui_backbone.remove(run_id);
            }
            let evicted = evicted.into_iter().collect::<HashSet<_>>();
            self.session_current_run
                .write()
                .await
                .retain(|_, run_id| !evicted.contains(run_id));
        }

        let idle_timeout = (self.session_retention.idle_timeout_secs > 0)
            .then(|| Duration::from_secs(self.session_retention.idle_timeout_secs));
        if idle_timeout.is_none()
            && self.session_retention.max_retained == 0
            && self
                .session_retention
                .persisted_ttl_secs
                .is_none_or(|ttl| ttl == 0)
        {
            return;
        }
        let protected = {
            let runs = self.active_runs.read().await;
            runs.values()
                .filter(|state| {
                    !matches!(
                        state.run.status,
                        RunStatus::Done | RunStatus::Error | RunStatus::Cancelled
                    )
                })
                .filter_map(|state| {
                    let session_id = state.run.conversation_id.clone()?;
                    let owner = state.dialogue.0.owner_id().to_owned();
                    Some((owner, session_id))
                })
                .collect::<HashSet<_>>()
        };
        let removed_sessions = self.sessions.cleanup_bounded(
            idle_timeout,
            self.session_retention.max_retained,
            &protected,
        );
        if !removed_sessions.is_empty() {
            let removed = removed_sessions
                .into_iter()
                .map(|(owner, session_id)| {
                    crate::uar::persistence::tenant_storage_key(&owner, &session_id)
                })
                .collect::<HashSet<_>>();
            self.session_current_run
                .write()
                .await
                .retain(|session_key, _| !removed.contains(session_key));
        }

        // Persisted-session TTL sweep (opt-in via sessions.persisted_ttl_secs).
        if let Some(ttl_secs) = self.session_retention.persisted_ttl_secs {
            if ttl_secs > 0 {
                if let Some(persistence) = &self.persistence {
                    let cutoff = chrono::Utc::now()
                        - chrono::Duration::seconds(i64::try_from(ttl_secs).unwrap_or(i64::MAX));
                    match persistence.list_expired_sessions(cutoff).await {
                        Ok(expired) => {
                            for (owner_id, session_id) in &expired {
                                match persistence.delete_session(owner_id, session_id).await {
                                    Err(error) => tracing::warn!(
                                        %error, %owner_id, %session_id,
                                        "failed to delete expired persisted session"
                                    ),
                                    Ok(deleted) => {
                                        self.sessions.remove_for_user(session_id, owner_id);
                                        if deleted {
                                            tracing::info!(
                                                %owner_id, %session_id,
                                                "persisted session purged by TTL sweep"
                                            );
                                        }
                                    }
                                }
                            }
                            if !expired.is_empty() {
                                tracing::info!(
                                    count = expired.len(),
                                    "persisted sessions purged by TTL sweep"
                                );
                            }
                        }
                        Err(error) => {
                            tracing::warn!(%error, "failed to list expired sessions for TTL sweep");
                        }
                    }
                }
            }
        }
    }

    /// Install operator-owned skill policy before accepting requests.
    #[must_use]
    pub fn with_skill_required_roles(mut self, roles: BTreeMap<String, BTreeSet<String>>) -> Self {
        self.skill_required_roles = roles;
        self
    }

    fn skill_role_allowed(
        &self,
        skill_id: &str,
        owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
    ) -> bool {
        self.skill_required_roles
            .get(skill_id)
            .is_none_or(|required| {
                required.is_empty()
                    || owner.is_some_and(|owner| required.is_subset(owner.verified_roles()))
            })
    }

    /// Warm continuation cannot retain protected eligibility after credential-role removal.
    pub(crate) async fn authorize_run_skill_roles(
        &self,
        user: &crate::uar::security::claims::UserContext,
        run_id: &str,
    ) -> anyhow::Result<()> {
        if self.skill_required_roles.is_empty() {
            return Ok(());
        }
        let owner = crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)?;
        let run = self
            .get_run_for_context(user, run_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Protected continuation run is unavailable"))?;
        let policy: EffectiveRunPolicy = serde_json::from_value(
            run.context
                .get("effective_run_policy")
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Protected continuation policy is unavailable"))?,
        )?;
        anyhow::ensure!(
            policy
                .skills
                .ids
                .iter()
                .all(|id| self.skill_role_allowed(id, Some(&owner))),
            "Insufficient skill roles for protected continuation"
        );
        Ok(())
    }

    /// Check explicit protected selections before a transport reserves a task.
    pub(crate) async fn authorize_preferred_skills(
        &self,
        user: &crate::uar::security::claims::UserContext,
        artifact: &AgentArtifact,
    ) -> Result<(), crate::uar::security::skill_roles::SkillAdmissionError> {
        use crate::uar::security::skill_roles::SkillAdmissionError;
        let owner = crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)
            .map_err(|_| SkillAdmissionError::Forbidden)?;
        let protected = artifact
            .policy
            .skills
            .prefer
            .iter()
            .filter(|id| self.skill_required_roles.contains_key(*id))
            .collect::<Vec<_>>();
        if protected
            .iter()
            .any(|id| !self.skill_role_allowed(id, Some(&owner)))
        {
            return Err(SkillAdmissionError::Forbidden);
        }
        let skills = self.catalog_skills().await;
        if protected.iter().any(|id| {
            !skills
                .iter()
                .any(|skill| &skill.skill_id == *id && skill.enabled_for(Some(&artifact.id), None))
        }) {
            return Err(SkillAdmissionError::Unavailable);
        }
        Ok(())
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
            let reads = async {
                if let Ok(Some(mode)) = settings.get_typed("harness.mode").await {
                    config.mode = mode;
                }
                if let Ok(Some(mode)) = settings.get_typed("harness.skill_activation_mode").await {
                    config.skill_activation_mode = mode;
                }
                if let Ok(Some(budget)) = settings.get_typed("harness.skill_reattachment").await {
                    config.skill_reattachment = budget;
                }
            };
            reads.await;
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

    /// Replace standalone admission with a host-owned paired adapter before
    /// this manager accepts runs.
    #[must_use]
    pub fn with_host_tool_admission(
        mut self,
        admission: Arc<dyn crate::uar::runtime::tool_admission::HostToolAdmissionPort>,
    ) -> Self {
        self.host_tool_admission = admission;
        self
    }

    /// Supply a host-owned local-only matcher without changing hosted RAG routing.
    pub fn with_local_skill_matcher(
        mut self,
        matcher: Option<Arc<crate::uar::runtime::matching::VectorMatcher>>,
    ) -> Self {
        self.local_skill_matcher = matcher;
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
        if !self.global_mcp.shared_transports_enabled() {
            return Ok(None);
        }
        let (Some(runtime), Some(environment)) = (&self.mcp_runtime, &self.mcp_environment) else {
            return Ok(None);
        };
        Ok(Some(McpRunResources::new(
            owner.clone(),
            runtime.clone(),
            Arc::new(McpCatalog::from_definitions(
                self.root_mcp_catalog()
                    .await?
                    .definitions()
                    .filter(|definition| !definition.configuration().requires_agent_grant())
                    .map(|definition| definition.as_ref().clone()),
            )?),
            Arc::clone(environment),
        )))
    }

    pub(crate) async fn admit_run_mcp_servers(
        &self,
        inputs: Vec<crate::uar::runtime::turn::host::RunMcpServerInput>,
        user: &crate::uar::security::claims::UserContext,
        owner: crate::uar::runtime::actor::messages::ActorOwner,
        working_directory: std::path::PathBuf,
        agent: Option<&AgentArtifact>,
    ) -> Result<
        (
            crate::uar::runtime::turn::host::RunMcpServers,
            McpRunResources,
        ),
        crate::uar::runtime::turn::host::HostInputError,
    > {
        let catalog = self.root_mcp_catalog().await.map_err(|_| {
            crate::uar::runtime::turn::host::HostInputError::new(
                "run_mcp_destination_unavailable",
                "registered MCP destinations are unavailable",
            )
        })?;
        let registered = crate::uar::runtime::turn::host::RegisteredMcpAgent::capture(
            self.persistence.clone(),
            Arc::clone(&self.global_mcp),
            agent,
        )
        .await?;
        let environment = match &self.mcp_environment {
            Some(environment) => Arc::clone(environment),
            None => Arc::new(
                McpBindingEnvironment::new(working_directory.clone(), BTreeMap::new()).map_err(
                    |_| {
                        crate::uar::runtime::turn::host::HostInputError::new(
                            "working_directory_invalid",
                            "working directory is invalid",
                        )
                    },
                )?,
            ),
        };
        let servers = crate::uar::runtime::turn::host::RunMcpServers::from_inputs_for_agent(
            inputs,
            user,
            &catalog,
            &environment,
            registered.as_ref(),
        )?
        .with_registry(Arc::clone(&self.global_mcp));
        let resources = servers.resources(owner, working_directory, environment.as_ref())?;
        Ok((servers, resources))
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
    #[cfg(feature = "server")]
    pub(crate) fn target_sources(&self) -> crate::uar::api::targets::ModelSources<'_> {
        crate::uar::api::targets::ModelSources {
            registry: self.provider_registry.as_deref(),
            settings: self.settings_manager.as_deref(),
            credentials: self.provider_service.as_deref(),
        }
    }

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

    /// Install exact model/template/settings contracts resolved by the trusted
    /// host. The manager supplies unreduced canonical history to this registry
    /// for ordinary, graph and checkpoint-resume execution.
    #[must_use]
    pub fn with_destination_preparations(
        mut self,
        preparations: Arc<crate::llm::DestinationRequestPreparations>,
    ) -> Self {
        self.destination_preparations = Some(preparations);
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

    /// Compatibility configuration retained for callers from before lifecycle
    /// admission was explicit. Store availability and scope cancellation now
    /// determine admission outcomes, not elapsed time.
    pub fn with_admission_timeout(self, _timeout: std::time::Duration) -> Self {
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

    /// Install the startup-captured provider credential policy independently of
    /// encrypted credential storage. The policy authorizes source selection; it
    /// does not grant kernels a resolver or mutation capability.
    #[must_use]
    pub fn with_credential_policy(
        mut self,
        policy: Arc<crate::uar::security::credentials::CredentialTenancyPolicy>,
    ) -> Self {
        self.credential_policy = policy;
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

    /// Attach the trusted-host resource scope before background runtime work
    /// begins. The scope owns completion accounting; it does not grant kernel
    /// mutation authority.
    #[must_use]
    pub fn attach_lifecycle(
        mut self,
        scope: crate::uar::runtime::lifecycle::ResourceScope,
    ) -> Self {
        // Every run derives from the scope cancellation capability. Closing a
        // host scope therefore reaches producer roots before it waits for their
        // acknowledged completion.
        self.root_cancellation = scope.cancellation_token();
        self.lifecycle_scope = scope;
        self
    }

    #[must_use]
    pub fn lifecycle_scope(&self) -> crate::uar::runtime::lifecycle::ResourceScope {
        self.lifecycle_scope.clone()
    }

    /// Close manager-owned background admission and retain the resulting task
    /// report. The host calls this after run producers settle and before its
    /// persistence finalizer.
    pub async fn shutdown_lifecycle(&self) -> crate::uar::runtime::lifecycle::CloseReport {
        self.root_cancellation.cancel();
        self.lifecycle_scope.close().await
    }

    /// Close durable run-history admission and join its writer before the
    /// trusted host releases persistence.
    ///
    /// # Errors
    /// Returns retained durable write failures instead of claiming a drained
    /// history queue.
    pub async fn shutdown_run_history(&self) -> anyhow::Result<()> {
        if let Some(writer) = &self.run_history {
            writer.shutdown().await
        } else {
            Ok(())
        }
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
        self.root_cancellation.cancel();
        // Losing an HTTP waiter cannot detach a producer from this join set.
        self.graph_roots.shutdown().await?;
        let _assembly = self.checkpoint_assemblies.write().await;
        let producers = self
            .checkpoint_producers
            .lock()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for producer in producers {
            let mut producer = producer.lock().await;
            match &mut *producer {
                CheckpointProducer::Ordinary(handle) => {
                    if let Some(join) = handle.as_mut() {
                        join.await
                            .map_err(|_| anyhow::anyhow!("checkpoint producer join failed"))?;
                        *handle = None;
                    }
                }
                CheckpointProducer::Actor(root) => root.finish().await?,
            }
        }
        self.terminal_operations
            .shutdown()
            .await
            .map_err(anyhow::Error::from)?;
        self.sandbox_operations
            .shutdown()
            .await
            .map_err(anyhow::Error::from)?;
        anyhow::ensure!(
            !self
                .checkpoint_cleanup_failed
                .load(std::sync::atomic::Ordering::Acquire),
            "checkpoint predecessor cleanup remains unconfirmed"
        );
        if let Some(db) = &self.persistence
            && db.supports_durable_checkpoint_resume()
        {
            use crate::uar::persistence::tool_admission::ToolAdmissionEvidenceState;
            // Unknown outcomes are deliberately sticky; time never releases them.
            for owner in self.checkpoint_evidence_owners.read().await.iter() {
                let mut latest = BTreeMap::new();
                for evidence in db.list_tool_admission_evidence(owner).await? {
                    if evidence.runtime_epoch == self.tool_runtime_epoch {
                        anyhow::ensure!(
                            evidence.state != ToolAdmissionEvidenceState::OutcomeUnknown,
                            "checkpoint predecessor has an unreconciled unknown effect"
                        );
                        latest.insert(evidence.invocation_id.clone(), evidence.state);
                    }
                }
                anyhow::ensure!(
                    latest.values().all(|state| !matches!(
                        state,
                        ToolAdmissionEvidenceState::ClaimIntent
                            | ToolAdmissionEvidenceState::OutcomeUnknown
                            | ToolAdmissionEvidenceState::AwaitingApproval
                    )),
                    "checkpoint predecessor effects remain uncertain"
                );
            }
            for (owner, run) in self.checkpoint_effect_roots.read().await.iter() {
                let mut run_ids = BTreeSet::from([run.clone()]);
                for thread in db.list_agent_threads(owner, run).await? {
                    if let Some(run_id) = thread.thread.run_id {
                        run_ids.insert(run_id);
                    }
                }
                for run_id in run_ids {
                    for receipt in db.list_canonical_tool_receipts(owner, &run_id).await? {
                        anyhow::ensure!(
                            receipt.outcome.is_some_and(|outcome| outcome
                                != crate::mcp::outcome::ToolOutcome::OutcomeUnknown),
                            "checkpoint canonical effect outcome remains uncertain"
                        );
                    }
                }
            }
            db.exclude_checkpoint_runtime_epoch(&self.tool_runtime_epoch)
                .await?;
        }
        Ok(())
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

    /// Resolve an exact approval request after authenticating the root owner.
    /// Missing identity fails closed for both root and descendant requests.
    pub async fn resolve_approval_request(
        &self,
        run_id: &str,
        approval_id: Option<&str>,
        approved: bool,
    ) -> bool {
        self.approvals
            .resolve_record(run_id, approval_id, approved)
            .await
            .ok()
            .flatten()
            .is_some_and(|(_, delivered)| delivered)
    }

    pub(crate) async fn resolve_approval_record(
        &self,
        run_id: &str,
        approval_id: Option<&str>,
        approved: bool,
    ) -> anyhow::Result<
        Option<(
            crate::uar::persistence::approval_decisions::ApprovalRecord,
            bool,
        )>,
    > {
        self.approvals
            .resolve_record(run_id, approval_id, approved)
            .await
    }

    pub(crate) async fn approval_records_for_context(
        &self,
        user: &crate::uar::security::claims::UserContext,
        run_id: &str,
        workspace: Option<&str>,
    ) -> anyhow::Result<(
        bool,
        Vec<crate::uar::persistence::approval_decisions::ApprovalRecordView>,
    )> {
        let owner = crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)?;
        let principal = owner
            .credential_principal()
            .ok_or_else(|| anyhow::anyhow!("Verified approval principal required"))?;
        self.approvals
            .records(&principal.storage_key(), run_id, workspace)
            .await
    }

    pub(crate) async fn approval_records_for_owner(
        &self,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
        run_id: &str,
        workspace: Option<&str>,
    ) -> anyhow::Result<(
        bool,
        Vec<crate::uar::persistence::approval_decisions::ApprovalRecordView>,
    )> {
        let principal = owner
            .credential_principal()
            .ok_or_else(|| anyhow::anyhow!("Verified approval principal required"))?;
        self.approvals
            .records(&principal.storage_key(), run_id, workspace)
            .await
    }

    pub(crate) async fn approval_recovery_record(
        &self,
        user: &crate::uar::security::claims::UserContext,
        run: &str,
        id: &str,
        workspace: Option<&str>,
    ) -> anyhow::Result<Option<crate::uar::persistence::approval_decisions::ApprovalRecord>> {
        let owner = crate::uar::security::claims::CredentialPrincipal::from_verified_context(user)?;
        self.approvals
            .recovery_record(&owner.storage_key(), run, id, workspace)
            .await
    }

    pub(crate) async fn finish_recovered_approval(
        &self,
        record: &crate::uar::persistence::approval_decisions::ApprovalRecord,
        state: crate::uar::persistence::approval_decisions::ApprovalState,
    ) -> anyhow::Result<bool> {
        self.approvals.finish_recovered(record, state).await
    }

    pub(crate) async fn deny_recovered_approval(
        &self,
        record: &crate::uar::persistence::approval_decisions::ApprovalRecord,
    ) -> anyhow::Result<bool> {
        self.approvals.deny_recovered(record).await
    }

    /// Return the existing live waiter for an owner. This snapshot is a view of
    /// the current continuation and never creates a replacement approval.
    pub(crate) async fn pending_approval_for_user(
        &self,
        owner_id: &str,
        run_id: &str,
    ) -> Option<PendingApprovalView> {
        let approval = self.approvals.pending(owner_id, run_id)?;
        let history = self
            .active_runs
            .read()
            .await
            .get(run_id)
            .map(|state| Arc::clone(&state.history));
        let cursor = match history {
            Some(history) => history.lock().await.buffer.iter().find_map(|event| {
                matches!(
                    &event.event,
                    NormalizedEvent::ToolCallApprovalRequired {
                        approval_id: Some(approval_id),
                        ..
                    } if approval_id == &approval.approval_id
                )
                .then_some(event.id)
            }),
            None => None,
        };
        Some(PendingApprovalView {
            version: 1,
            event_id: approval.approval_id.clone(),
            cursor,
            approval,
        })
    }

    /// Read the identity captured from the assembled root approval channel.
    pub(crate) async fn approval_root_run_id(&self, run_id: &str) -> Option<String> {
        self.active_runs
            .read()
            .await
            .get(run_id)
            .and_then(|state| state.approval_root_run_id.clone())
    }

    /// Read sanitized durable lifecycle evidence for one owner/run tree.
    pub(crate) async fn tool_admission_evidence_for_user(
        &self,
        owner_id: &str,
        run_id: &str,
    ) -> anyhow::Result<Vec<crate::uar::persistence::tool_admission::ToolAdmissionEvidence>> {
        let Some(persistence) = &self.persistence else {
            return Ok(Vec::new());
        };
        let mut records = persistence.list_tool_admission_evidence(owner_id).await?;
        records.retain(|record| record.root_run_id == run_id || record.run_id == run_id);
        let mut latest =
            HashMap::<String, crate::uar::persistence::tool_admission::ToolAdmissionEvidence>::new(
            );
        for record in &records {
            latest.insert(record.invocation_id.clone(), record.clone());
        }
        for stale in latest.into_values().filter(|record| {
            record.runtime_epoch != self.tool_runtime_epoch && !record.state.is_terminal()
        }) {
            if let Some(terminal) = stale.after_runtime_restart() {
                records.push(persistence.save_tool_admission_evidence(&terminal).await?);
            }
        }
        records.sort_by(|left, right| {
            left.occurred_at
                .cmp(&right.occurred_at)
                .then_with(|| left.evidence_id.cmp(&right.evidence_id))
        });
        Ok(records
            .into_iter()
            .filter(|record| record.root_run_id == run_id || record.run_id == run_id)
            .collect())
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

    /// Subscribe before inspecting an admitted attempt's local run binding.
    pub(crate) fn subscribe_run_registrations(&self) -> watch::Receiver<u64> {
        self.run_admission_generation.subscribe()
    }

    /// Wait for an already-admitted instance attempt to install its run
    /// cancellation capability, without inferring completion from elapsed
    /// time. Scope cancellation is the only local escape from an unresolved
    /// host admission outcome.
    pub async fn cancel_run_when_registered(
        &self,
        user_id: &str,
        run_id: &str,
        cancellation: CancellationToken,
    ) -> bool {
        let mut registrations = self.run_admission_generation.subscribe();
        loop {
            if cancellation.is_cancelled() {
                return false;
            }
            if self.cancel_run_for_user(user_id, run_id).await {
                return true;
            }
            tokio::select! {
                _ = cancellation.cancelled() => return false,
                changed = registrations.changed() => {
                    if changed.is_err() {
                        return false;
                    }
                }
            }
        }
    }

    /// Host grant installation invalidates only runs pinned to an older grant revision.
    pub(crate) async fn invalidate_representation_grant(
        &self,
        owner: &str,
        workspace: &str,
        grant: &crate::uar::domain::collaboration::RepresentationGrant,
    ) {
        let runs = self.active_runs.read().await.values().filter_map(|state| {
            let context = &state.run.context;
            let binding = context.get("effective_collaboration_binding")?;
            if binding["ownerId"].as_str() != Some(owner)
                || binding["workspaceId"].as_str() != Some(workspace) {
                return None;
            }
            let invalidated = context.get("representation_grants")?.as_array()?.iter().any(|reference| {
                reference["grantId"].as_str() == Some(grant.grant_id.as_str())
                    && (reference["revision"].as_u64() != Some(grant.revision)
                        || grant.status != crate::uar::domain::collaboration::RepresentationGrantStatus::Active
                        || grant.revocation.is_some())
            });
            invalidated.then(|| state.run.run_id.clone())
        }).collect::<Vec<_>>();
        for run in runs {
            self.cancel_run(&run).await;
        }
    }

    /// Cancel an in-flight run only when it belongs to the authenticated owner.
    pub async fn cancel_run_for_user(&self, owner_id: &str, run_id: &str) -> bool {
        if self.get_run_for_user(owner_id, run_id).await.is_none() {
            return false;
        }
        self.cancel_run(run_id).await
    }

    /// Cancel only when the complete middleware-verified identity owns the run.
    pub async fn cancel_run_for_context(
        &self,
        user: &crate::uar::security::claims::UserContext,
        run_id: &str,
    ) -> bool {
        if self.get_run_for_context(user, run_id).await.is_none() {
            return false;
        }
        self.cancel_run(run_id).await
    }

    /// Cancel only when the exact verified actor owner owns the run.
    pub async fn cancel_run_for_owner(
        &self,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
        run_id: &str,
    ) -> bool {
        if self.get_run_for_owner(owner, run_id).await.is_none() {
            return false;
        }
        self.cancel_run(run_id).await
    }

    /// Revoke one run-owned remote MCP credential for its exact verified
    /// owner. Revocation cannot install a replacement; the host must resume at
    /// a safe boundary with a compatible renewed grant.
    pub(crate) async fn revoke_run_mcp_grant_for_context(
        &self,
        user: &crate::uar::security::claims::UserContext,
        run_id: &str,
        server: &str,
    ) -> bool {
        if self.get_run_for_context(user, run_id).await.is_none() {
            return false;
        }
        let resources = self
            .active_runs
            .read()
            .await
            .get(run_id)
            .and_then(|state| state.delegation.as_ref())
            .and_then(std::sync::Weak::upgrade);
        let Some(resources) = resources else {
            return false;
        };
        let Some(grants) = &resources.run_mcp_grants else {
            return false;
        };
        if !grants.revoke(server) {
            return false;
        }
        if let Some(runtime) = &resources.run_scoped_mcp {
            runtime.require_authentication(server);
        }
        resources.cancellation.cancel();
        let _ = self.cancel_run(run_id).await;
        tracing::info!(run_id = %run_id, server = %server, "Run MCP credential revoked");
        true
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

    /// Cancel a session's current run only for the complete verified identity.
    pub async fn cancel_session_run_for_context(
        &self,
        user: &crate::uar::security::claims::UserContext,
        session_id: &str,
    ) -> bool {
        let Ok(owner_key) = user.session_owner_key() else {
            return false;
        };
        let session_key = crate::uar::persistence::tenant_storage_key(&owner_key, session_id);
        let run_id = self
            .session_current_run
            .read()
            .await
            .get(&session_key)
            .cloned();
        match run_id {
            Some(run_id) => self.cancel_run_for_context(user, &run_id).await,
            None => false,
        }
    }

    /// Exact host process identity registered by the external supervisor.
    pub(crate) fn runtime_epoch(&self) -> &str {
        &self.tool_runtime_epoch
    }

    /// A clone of the root cancellation token.
    ///
    /// Cancelling it aborts ALL in-flight runs at once; used to wire run
    /// cancellation into the server's graceful-shutdown path.
    #[must_use]
    pub fn root_cancellation_token(&self) -> CancellationToken {
        self.root_cancellation.clone()
    }

    /// Publish a committed persistence or realtime transition to lifecycle
    /// observers. This is a wakeup only: consumers reread their authoritative
    /// durable row and do not infer the mutation from the generation value.
    pub fn notify_committed_work(&self) {
        let next = (*self.committed_work_generation.borrow()).saturating_add(1);
        self.committed_work_generation.send_replace(next);
    }

    /// Subscribe before an authoritative durable snapshot. The retained
    /// generation closes the event/snapshot race and replaces polling.
    #[must_use]
    pub fn subscribe_committed_work(&self) -> watch::Receiver<u64> {
        self.committed_work_generation.subscribe()
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
        additional_tools: Option<&BTreeSet<String>>,
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
        if let Some(additional_tools) = additional_tools {
            tools.extend(additional_tools.iter().cloned());
        }
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
            let knowledge_owner = verified_owner
                .and_then(|owner| owner.session_owner_key().ok())
                .or_else(|| {
                    (owner_id == crate::session::ANONYMOUS_SESSION_OWNER)
                        .then(|| owner_id.to_owned())
                });
            if let Some(knowledge_owner) = knowledge_owner
                && let Ok(records) = persistence.list_knowledge_bases(&knowledge_owner).await
            {
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
        additional_tools: Option<&BTreeSet<String>>,
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
                    additional_tools,
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
                additional_tools,
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
    pub async fn effective_config(&self, conversation_id: &str) -> anyhow::Result<EffectiveConfig> {
        let requested = if let Some(persistence) = &self.persistence {
            persistence
                .load_conversation_policy(crate::session::ANONYMOUS_SESSION_OWNER, conversation_id)
                .await?
        } else {
            None
        };
        let agent_id = requested
            .as_ref()
            .and_then(|record| record.policy.agent_id.clone())
            .unwrap_or_else(|| "default-agent".to_string());
        let agent = self.resolve_registered_agent(&agent_id).await?;
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
        Ok(EffectiveConfig {
            agent,
            requested_policy: requested,
            effective_policy: effective,
        })
    }

    /// Resolve an explicitly selected actor artifact without silently replacing
    /// an unknown ID or a failed storage read with the default agent.
    pub async fn resolve_registered_agent(
        &self,
        agent_id: &str,
    ) -> Result<AgentArtifact, crate::uar::domain::agent_store::AgentStoreError> {
        crate::uar::domain::agent_store::resolve_registered_agent(
            self.persistence.as_deref(),
            agent_id,
        )
        .await
    }

    /// Return the complete local runtime catalog without hiding store failures.
    pub async fn list_registered_agents(
        &self,
    ) -> Result<Vec<AgentArtifact>, crate::uar::domain::agent_store::AgentStoreError> {
        crate::uar::domain::agent_store::list_registered_agents(self.persistence.as_deref()).await
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
        additional_tools: Option<&BTreeSet<String>>,
        verified_owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
    ) -> EffectiveRunPolicy {
        let (universe, conversation) = self
            .build_universe_and_conversation(
                owner_id,
                conversation_id,
                thread_controls,
                mcp_catalog,
                additional_tools,
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
        let request = self
            .prepare_interaction_request(run_id, interaction, user, None)
            .await?;
        let marker = self
            .get_run_for_context(user, run_id)
            .await
            .and_then(|run| run.context.get("host_resources").cloned())
            .and_then(|value| {
                serde_json::from_value::<crate::uar::runtime::turn::host::HostResourcesMarker>(
                    value,
                )
                .ok()
            })
            .unwrap_or_default();
        if !marker.is_empty() || marker.artifact_inline {
            return Err("continuation must reattach the source run host resources".to_string());
        }
        Ok(self.execute_request(request).await)
    }

    /// Assemble a continuation request without retaining or resolving any host
    /// secret. The API adapter reattaches and compares request-scoped resources
    /// before it calls `execute_request`.
    pub async fn prepare_interaction_request(
        &self,
        run_id: &str,
        interaction: serde_json::Value,
        user: &crate::uar::security::claims::UserContext,
        inline_artifact: Option<AgentArtifact>,
    ) -> Result<crate::uar::runtime::turn::RunExecutionRequest, String> {
        let run = self
            .get_run_for_context(user, run_id)
            .await
            .ok_or_else(|| format!("run '{run_id}' not found"))?;
        let marker = run
            .context
            .get("host_resources")
            .cloned()
            .and_then(|value| {
                serde_json::from_value::<crate::uar::runtime::turn::host::HostResourcesMarker>(
                    value,
                )
                .ok()
            })
            .unwrap_or_default();
        let snapshot =
            crate::uar::domain::artifact::AgentArtifactSnapshot::from_run_context(&run.context)
                .map_err(str::to_string)?;
        if let Some(supplied) = inline_artifact
            && supplied.definition_revision() != snapshot.artifact.definition_revision()
        {
            return Err("continuation artifact does not match the source run snapshot".to_string());
        }
        let artifact = snapshot.artifact;
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
        request.host_resources_marker.artifact_inline = marker.artifact_inline;
        request.resolved_policy = effective_policy;
        if let Some(host_context) = run.context.get("host_context") {
            request.working_directory = host_context
                .get("working_directory")
                .and_then(serde_json::Value::as_str)
                .map(std::path::PathBuf::from);
            request.reasoning_effort = host_context
                .get("reasoning_effort")
                .cloned()
                .and_then(|value| serde_json::from_value(value).ok());
        }
        request.presentation_negotiation = match run.context.get("presentation_negotiation") {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|_| "Stored Presentation negotiation is invalid".to_string())?,
            None => Default::default(),
        };
        Ok(request)
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
        checkpoint_authorization_digest: String,
    ) -> String {
        self.execute_request(crate::uar::runtime::turn::RunExecutionRequest {
            artifact,
            input,
            session_id,
            user_id,
            memory_hits,
            verified_owner: None,
            mcp_resources: None,
            host_tool_admission: None,
            run_credentials: None,
            host_resources_marker: Default::default(),
            host_secret_scrubber: Default::default(),
            resolved_policy: None,
            target_selection: None,
            presentation_negotiation: Default::default(),
            host_policy_constraint: None,
            host_budget_constraint: None,
            host_usage_grant: None,
            a2a_delegation: None,
            host_sandbox_constraint: None,
            seed_history: Vec::new(),
            client_tools: None,
            output_token_cap: None,
            response_format: None,
            host_history: None,
            reasoning_effort: None,
            checkpoint_resume: Some(crate::uar::runtime::turn::CheckpointResume {
                state: restored_state,
                history: restored_history,
                authorization_digest: checkpoint_authorization_digest,
            }),
            inherited_history: None,
            collaboration_binding: None,
            instance_binding: None,
            service_binding: None,
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
        self.execute_request_with_run_id(request, run_id).await
    }

    /// Execute through the native run path using an identity reserved by an
    /// authenticated admission authority before kernel entry.
    pub(crate) async fn execute_request_with_run_id(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
        run_id: String,
    ) -> String {
        self.request_admission(request.clone(), run_id.clone())
            .await
    }

    // Construct the admission future in a separate frame that returns before
    // polling, keeping its inline state out of compatibility callers.
    #[inline(never)]
    fn request_admission(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
        run_id: String,
    ) -> futures::future::BoxFuture<'_, String> {
        Box::pin(async move {
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
            // Keep the large kernel future out of the HTTP admission future's
            // inline state, as the captured child-run entry point already does.
            Box::pin(self.execute_request_inner(request, run_id, None, None, None, None)).await
        })
    }

    /// Execute an ordinary agent only after its persisted private binding has
    /// been revalidated against the current catalog, policy, skills, and grants.
    pub async fn execute_bound_request(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
    ) -> Result<String, crate::uar::compiler::collaboration::CollaborationError> {
        let binding = request.collaboration_binding.as_ref().ok_or_else(|| {
            crate::uar::compiler::collaboration::CollaborationError::Invalid(
                "bound execution request has no effective binding receipt".to_owned(),
            )
        })?;
        binding
            .revalidate(&request.artifact)
            .await
            .map_err(crate::uar::compiler::collaboration::CollaborationError::from)?;
        Ok(self.execute_request(request).await)
    }

    /// Revalidate a bound request, then execute it with a pre-reserved run ID.
    pub(crate) async fn execute_bound_request_with_run_id(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
        run_id: String,
    ) -> Result<String, crate::uar::compiler::collaboration::CollaborationError> {
        let binding = request.collaboration_binding.as_ref().ok_or_else(|| {
            crate::uar::compiler::collaboration::CollaborationError::Invalid(
                "bound execution request has no effective binding receipt".to_owned(),
            )
        })?;
        binding
            .revalidate(&request.artifact)
            .await
            .map_err(crate::uar::compiler::collaboration::CollaborationError::from)?;
        Ok(self.execute_request_with_run_id(request, run_id).await)
    }

    pub(crate) async fn execute_checkpoint_approval(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
        envelope: &crate::uar::persistence::checkpoint_resume::CheckpointResumeEnvelope,
        bound: bool,
        approval: Option<crate::uar::persistence::approval_decisions::ApprovalRecord>,
    ) -> anyhow::Result<String> {
        self.execute_checkpoint_admitted(
            request,
            envelope,
            bound,
            approval,
            Uuid::new_v4().to_string(),
            false,
        )
        .await
    }

    /// A2A has durably correlated this host-reserved ID before entry. Retain the
    /// same checkpoint authority, but establish the actor-owned durable root.
    pub(crate) async fn execute_checkpoint_a2a_with_run_id(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
        envelope: &crate::uar::persistence::checkpoint_resume::CheckpointResumeEnvelope,
        bound: bool,
        approval: Option<crate::uar::persistence::approval_decisions::ApprovalRecord>,
        run_id: String,
    ) -> anyhow::Result<String> {
        self.execute_checkpoint_admitted(request, envelope, bound, approval, run_id, true)
            .await
    }

    async fn execute_checkpoint_admitted(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
        envelope: &crate::uar::persistence::checkpoint_resume::CheckpointResumeEnvelope,
        bound: bool,
        approval: Option<crate::uar::persistence::approval_decisions::ApprovalRecord>,
        continuation_run_id: String,
        actor_root: bool,
    ) -> anyhow::Result<String> {
        use crate::uar::persistence::checkpoint_resume::{
            CheckpointClaimOutcome, CheckpointContinuationClaim, CheckpointResumeError,
        };
        anyhow::ensure!(
            !self.root_cancellation.is_cancelled(),
            "checkpoint executor is shutting down"
        );
        envelope.validate()?;
        if actor_root && let Some(fresh) = &request.a2a_delegation {
            envelope.require_restorable_a2a(fresh)?;
        } else {
            envelope.require_restorable_execution()?;
        }
        if self.uses_agent_graph(&request.artifact) || self.primary_driver.is_some() {
            return Err(CheckpointResumeError::ExecutionUnavailable.into());
        }
        let principal = request
            .verified_owner
            .as_ref()
            .and_then(|owner| owner.credential_principal())
            .ok_or(CheckpointResumeError::Invalid)?;
        let checkpoint = envelope.checkpoint()?;
        let source = envelope.source_run()?;
        let source_artifact =
            crate::uar::domain::artifact::AgentArtifactSnapshot::from_run_context(&source.context)
                .map_err(|_| CheckpointResumeError::Invalid)?;
        let restoration = request
            .checkpoint_resume
            .as_ref()
            .ok_or(CheckpointResumeError::Invalid)?;
        if principal.storage_key() != envelope.owner_key
            || source.user_id.as_deref() != Some(principal.subject())
            || request.session_id.as_deref() != source.conversation_id.as_deref()
            || request.artifact.definition_revision()
                != source_artifact.artifact.definition_revision()
            || serde_json::to_value(&restoration.state)?
                != serde_json::to_value(checkpoint.try_restore_state()?)?
            || serde_json::to_value(&restoration.history)?
                != serde_json::to_value(crate::uar::runtime::checkpoint::history_from_checkpoint(
                    &checkpoint,
                )?)?
            || Some(restoration.authorization_digest.as_str())
                != checkpoint
                    .validate_integrity()?
                    .authorization_sha256
                    .as_deref()
        {
            return Err(CheckpointResumeError::Invalid.into());
        }
        let persistence = self
            .persistence
            .as_ref()
            .ok_or(CheckpointResumeError::Unsupported)?;
        let has_pending = restoration
            .state
            .data
            .contains_key(crate::uar::runtime::thread::approvals::recovery::PENDING_APPROVAL_KEY);
        if let Some(record) = &approval {
            crate::uar::runtime::thread::approvals::recovery::protected_call(record, envelope)?;
            if record.state != crate::uar::persistence::approval_decisions::ApprovalState::Pending
                || record.expires_at <= chrono::Utc::now()
            {
                return Err(CheckpointResumeError::Invalid.into());
            }
        } else if has_pending {
            return Err(CheckpointResumeError::Invalid.into());
        }
        let claim = CheckpointContinuationClaim {
            schema_version: 1,
            owner_key: envelope.owner_key.clone(),
            source_run_id: envelope.source_run_id.clone(),
            checkpoint_id: envelope.checkpoint_id.clone(),
            checkpoint_revision: envelope.checkpoint_revision.clone(),
            source_runtime_epoch: envelope.source_runtime_epoch.clone(),
            executor_runtime_epoch: self.tool_runtime_epoch.clone(),
            continuation_run_id,
        };
        let claim = match persistence
            .claim_checkpoint_resume(envelope, &claim)
            .await?
        {
            CheckpointClaimOutcome::Acquired(claim) => claim,
            CheckpointClaimOutcome::Existing(_) => {
                return Err(CheckpointResumeError::Conflict.into());
            }
        };
        claim.validate(envelope)?;
        anyhow::ensure!(
            !self.root_cancellation.is_cancelled(),
            "checkpoint executor is shutting down"
        );
        let continuation_run_id = claim.continuation_run_id.clone();
        let permit = CheckpointResumePermit {
            claim,
            envelope: envelope.clone(),
            owner: request
                .verified_owner
                .clone()
                .ok_or(CheckpointResumeError::Invalid)?,
            request_identity: checkpoint_request_identity(&request)?,
            approval,
        };
        self.checkpoint_resume_admitted
            .lock()
            .await
            .insert(continuation_run_id.clone(), permit);
        let run_id = if actor_root {
            let result = async {
                if bound {
                    let binding = request.collaboration_binding.as_ref().ok_or_else(|| {
                        anyhow::anyhow!("bound execution request has no effective binding receipt")
                    })?;
                    binding.revalidate(&request.artifact).await?;
                }
                let artifact_owner = request
                    .verified_owner
                    .clone()
                    .ok_or(CheckpointResumeError::Invalid)?;
                let artifacts = crate::uar::runtime::thread::artifacts::RunArtifactCollector::new(
                    artifact_owner,
                    continuation_run_id.clone(),
                );
                self.graph_roots
                    .start_with_artifacts(
                        self.clone(),
                        request.clone(),
                        continuation_run_id.clone(),
                        self.root_cancellation.child_token(),
                        Some(artifacts),
                    )
                    .await
            }
            .await;
            if let Err(error) = result {
                self.checkpoint_resume_admitted
                    .lock()
                    .await
                    .remove(&continuation_run_id);
                self.record_graph_root_failure(&request, &continuation_run_id)
                    .await;
                return Err(error);
            }
            continuation_run_id.clone()
        } else if bound {
            match self
                .execute_bound_request_with_run_id(request, continuation_run_id.clone())
                .await
            {
                Ok(run_id) => run_id,
                Err(error) => {
                    self.checkpoint_resume_admitted
                        .lock()
                        .await
                        .remove(&continuation_run_id);
                    return Err(error.into());
                }
            }
        } else {
            self.execute_request_with_run_id(request, continuation_run_id.clone())
                .await
        };
        self.checkpoint_resume_admitted
            .lock()
            .await
            .remove(&continuation_run_id);
        let current = self.get_run(&run_id).await;
        if current.as_ref().is_some_and(|run| {
            run.context
                .get("error_code")
                .and_then(serde_json::Value::as_str)
                == Some("approval_persistence_unavailable")
        }) {
            return Err(crate::uar::persistence::approval_decisions::ApprovalRecoveryError::PersistenceUnavailable.into());
        }
        if current.is_none_or(|run| matches!(run.status, RunStatus::Error | RunStatus::Cancelled)) {
            return Err(CheckpointResumeError::AdmissionDenied.into());
        }
        Ok(run_id)
    }

    /// Preserve an observable failure even when root persistence fails before
    /// kernel assembly creates its ordinary event stream. Never rerun the input.
    pub(crate) async fn record_graph_root_failure(
        &self,
        request: &crate::uar::runtime::turn::RunExecutionRequest,
        run_id: &str,
    ) {
        self.record_host_admission_failure(
            request,
            run_id,
            "graph_root_failed",
            "Graph root could not complete its owned execution and persistence",
        )
        .await;
    }

    async fn record_host_admission_failure(
        &self,
        request: &crate::uar::runtime::turn::RunExecutionRequest,
        run_id: &str,
        code: &str,
        message: &str,
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
                    pending_client_handoff: None,
                    client_handoff_failed: false,
                }));
                RunStreamState {
                    run: Run {
                        run_id: run_id.to_owned(),
                        agent_id: request.artifact.id.clone(),
                        conversation_id: request.session_id.clone(),
                        user_id: request.user_id.clone(),
                        status: RunStatus::Error,
                        context: serde_json::json!({ "agent_snapshot": null }),
                    },
                    verified_owner: request.verified_owner.clone(),
                    approval_root_run_id: None,
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
                    caller_handoff_confirmation: Arc::new(Mutex::new(())),
                    completion: None,
                    delegation: None,
                    tool_admission: None,
                    actor_spawn_descriptor: None,
                    actor_tool_approval: None,
                    started_at: Instant::now(),
                    terminal_at: Some(Instant::now()),
                    last_detached_at: None,
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
                runs: Arc::downgrade(&self.active_runs),
                secret_scrubber: request.host_secret_scrubber.clone(),
                admission_projection: Default::default(),
                run_history: self.run_history.clone(),
                retention_work: Arc::clone(&self.retention_work),
            }
        };
        emitter
            .emit(NormalizedEvent::Error {
                run_id: run_id.to_owned(),
                code: code.into(),
                message: message.into(),
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
        Box::pin(self.execute_request_inner(
            request,
            run_id,
            Some(capture),
            Some(cancellation),
            None,
            Some(root),
        ))
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
        let (tool_admission, descriptor, approval_gate) =
            {
                let runs = self.active_runs.read().await;
                let state = runs
                    .get(&root.record.thread.root_run_id)
                    .ok_or_else(|| anyhow::anyhow!("Actor root run is unavailable"))?;
                (
                    state
                        .tool_admission
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("Actor root admission is unavailable"))?,
                    state.actor_spawn_descriptor.clone().ok_or_else(|| {
                        anyhow::anyhow!("Actor root does not authorize spawn_agent")
                    })?,
                    state.actor_tool_approval.clone().ok_or_else(|| {
                        anyhow::anyhow!("Actor root approval gate is unavailable")
                    })?,
                )
            };
        let arguments = serde_json::to_value(&request)?;
        let argument_errors = descriptor
            .validator
            .iter_errors(&arguments)
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        anyhow::ensure!(
            argument_errors.is_empty(),
            "Invalid actor collaboration request: {}",
            argument_errors.join("; ")
        );
        let invocation = tool_admission.prepare(
            uuid::Uuid::new_v4().to_string(),
            descriptor.as_ref(),
            arguments,
            0,
        );
        let preparation = tool_admission.prepare_host(Arc::clone(&invocation)).await?;
        anyhow::ensure!(
            preparation.host_disposition
                != crate::uar::runtime::tool_admission::HostAdmissionDisposition::Deny,
            "Actor delegation is denied by the admission host"
        );
        let approval = crate::uar::runtime::tool_admission::ToolApprovalRequest {
            invocation: Arc::clone(&invocation),
            admission_id: preparation.admission_id.clone(),
            host_requires_approval: preparation.host_disposition
                == crate::uar::runtime::tool_admission::HostAdmissionDisposition::Ask,
            action_display: preparation.action_display.clone(),
            checkpoint_history: None,
        };
        let local_disposition = match (approval_gate.0)(approval).await {
            crate::llm::ToolApprovalResult::Approved => {
                crate::uar::runtime::tool_admission::LocalAdmissionDisposition::Approved
            }
            crate::llm::ToolApprovalResult::Allowed => {
                crate::uar::runtime::tool_admission::LocalAdmissionDisposition::Allowed
            }
            crate::llm::ToolApprovalResult::GovernanceBypassed => {
                crate::uar::runtime::tool_admission::LocalAdmissionDisposition::GovernanceBypassed
            }
            crate::llm::ToolApprovalResult::Rejected { reason } => {
                tool_admission
                    .resolve(
                        invocation,
                        preparation,
                        crate::uar::runtime::tool_admission::LocalAdmissionDisposition::Denied,
                        false,
                    )
                    .await?;
                anyhow::bail!("Actor delegation denied: {reason}")
            }
            crate::llm::ToolApprovalResult::Cancelled { reason } => {
                tool_admission
                    .cancel(
                        invocation.as_ref(),
                        &preparation.admission_id,
                        crate::uar::runtime::tool_admission::AdmissionCancellationReason::Cancelled,
                    )
                    .await?;
                anyhow::bail!("Actor delegation cancelled: {reason}")
            }
        };
        let admitted = tool_admission
            .resolve(invocation, preparation, local_disposition, true)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Actor delegation is denied by the admission host"))?;
        tool_admission.claim(&admitted).await?;
        let result = service.collaborate_from_user(owner, request).await;
        tool_admission.finish(&admitted, result.is_ok()).await?;
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
        let _assembly = tokio::select! {
            biased;
            () = self.root_cancellation.cancelled() => return run_id,
            guard = self.checkpoint_assemblies.read() => guard,
        };
        if self.root_cancellation.is_cancelled() {
            return run_id;
        }
        if let Some(format) = &request.response_format {
            if format.validate().is_err() {
                self.record_host_admission_failure(
                    &request,
                    &run_id,
                    "invalid_response_format",
                    "The requested response format is invalid or unsupported",
                )
                .await;
                return run_id;
            }
            if format.is_structured() && self.uses_agent_graph(&request.artifact) {
                self.record_host_admission_failure(&request, &run_id, "structured_graph_unsupported",
                    "Structured output requires a direct agent target; graph delegation does not inherit output contracts").await;
                return run_id;
            }
        }
        let recovered_approval = if request.checkpoint_resume.is_some() {
            let permit = self.checkpoint_resume_admitted.lock().await.remove(&run_id);
            if permit
                .as_ref()
                .is_none_or(|permit| !permit.accepts(&request, &run_id, &self.tool_runtime_epoch))
            {
                self.record_host_admission_failure(
                    &request,
                    &run_id,
                    "checkpoint_admission_required",
                    "Checkpoint execution requires its exact durable continuation claim",
                )
                .await;
                return run_id;
            }
            permit.and_then(|permit| permit.approval)
        } else {
            None
        };
        let execution_started_at = std::time::Instant::now();
        let checkpoint_a2a_renewable = request.a2a_delegation.as_ref().is_some_and(|contract| {
            request.host_budget_constraint.as_ref() == Some(&contract.budgets)
                && request.host_sandbox_constraint.as_ref() == Some(&contract.sandbox)
                && request.host_usage_grant.as_ref().is_some_and(|binding| {
                    binding.grant == contract.usage_grant
                        && contract
                            .digest()
                            .is_ok_and(|digest| binding.accounting_id == format!("a2a:{digest}"))
                })
        });
        let checkpoint_has_host_resources = request.host_tool_admission.is_some()
            || (!checkpoint_a2a_renewable
                && (request.host_usage_grant.is_some()
                    || request.host_budget_constraint.is_some()
                    || request.host_sandbox_constraint.is_some()))
            || !checkpoint_host_policy_renewable(&request)
            || request.instance_binding.is_some()
            || inherited.is_some()
            || request
                .collaboration_binding
                .as_ref()
                .is_some_and(|binding| {
                    binding.instance_authority.is_some()
                        || binding.team_attempt.is_some()
                        || binding.team_instructions.is_some()
                });
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
            mut memory_hits,
            resolved_policy,
            target_selection,
            presentation_negotiation,
            seed_history,
            client_tools,
            output_token_cap,
            response_format,
            mut host_history,
            reasoning_effort,
            checkpoint_resume,
            inherited_history,
            collaboration_binding,
            instance_binding,
            service_binding,
            skill_attachments: _,
            working_directory,
            verified_owner,
            mut mcp_resources,
            host_tool_admission,
            run_credentials,
            host_resources_marker,
            host_secret_scrubber,
            host_policy_constraint,
            host_budget_constraint,
            host_usage_grant,
            a2a_delegation,
            host_sandbox_constraint,
        } = request;
        let output_token_cap = match (
            output_token_cap,
            client_tools
                .as_ref()
                .and_then(|input| input.output_token_cap),
        ) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (left, right) => left.or(right),
        };
        if let Some(client) = &client_tools {
            if client.validate().is_err() {
                return run_id;
            }
            match serde_json::from_value::<Vec<Message>>(serde_json::Value::Array(
                client.history.clone(),
            )) {
                Ok(history) => host_history = Some(history),
                Err(_) => return run_id,
            }
        }
        let collaboration_binding = collaboration_binding.or_else(|| {
            inherited
                .as_ref()
                .and_then(|bindings| bindings.collaboration_binding.clone())
        });
        let instance_binding = instance_binding.or_else(|| {
            inherited
                .as_ref()
                .and_then(|bindings| bindings.instance_binding.clone())
        });
        let is_checkpoint_resume = checkpoint_resume.is_some();
        let (restored_state, mut checkpoint_history, checkpoint_authorization_digest) =
            match checkpoint_resume {
                Some(resume) => (
                    Some(resume.state),
                    Some(resume.history),
                    Some(resume.authorization_digest),
                ),
                None => (None, None, None),
            };
        // The current host assembles only a complete prefix. The protected
        // unresolved block enters execution after current admission is rebuilt.
        let pending_history =
            if let (Some(record), Some(history)) = (&recovered_approval, &mut checkpoint_history) {
                let index = history.iter().position(|message| {
                    message
                        .tool_calls
                        .iter()
                        .flatten()
                        .any(|call| call.id == record.tool_call_id)
                });
                match index {
                    Some(index) => history.split_off(index),
                    None => return run_id,
                }
            } else {
                Vec::new()
            };
        let client_history = client_tools.as_ref().and_then(|_| host_history.clone());
        let supplied_history = checkpoint_history
            .as_ref()
            .or(inherited_history.as_ref())
            .or(client_history.as_ref());
        let append_input = plan.append_input && client_tools.is_none();
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
            pending_client_handoff: None,
            client_handoff_failed: false,
        }));
        let mut emitter = RunEventEmitter {
            run_id: run_id.clone(),
            presentations: None,
            sender: tx.clone(),
            history: Arc::clone(&history),
            completion: completion.map(|capture| Arc::new(std::sync::Mutex::new(capture))),
            runs: Arc::downgrade(&self.active_runs),
            secret_scrubber: host_secret_scrubber,
            admission_projection: Default::default(),
            run_history: self.run_history.clone(),
            retention_work: Arc::clone(&self.retention_work),
        };
        let completion_guard = crate::uar::runtime::thread::execution::RunCompletionGuard::new(
            emitter.completion.clone(),
        );

        if client_tools
            .as_ref()
            .is_some_and(|input| !input.tools.is_empty() || input.budget_snapshot.is_some())
            && (verified_owner
                .as_ref()
                .and_then(|owner| owner.credential_principal())
                .is_none()
                || !self
                    .persistence
                    .as_ref()
                    .is_some_and(|store| store.supports_durable_client_tools()))
        {
            emitter
                .emit(NormalizedEvent::Error {
                    run_id: run_id.clone(),
                    code: "durable_client_tools_unavailable".into(),
                    message: "Caller tools require authenticated durable continuation storage"
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
            emitter.secret_scrubber.extend(resources.secret_scrubber());
            let invalid_capture = verified_owner.as_ref() != Some(resources.owner())
                || !resources.matches_agent(&artifact)
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
                && bindings.mcp.matches_agent(&artifact)
                && thread.artifact_id == bindings.policy.artifact().id
                && thread.artifact_id == artifact.id
                && user_id.as_deref() == Some(thread.owner_id.as_str())
                && session_id.as_deref() == Some(thread.thread_id.as_str())
                && inherited_history.is_some()
                && !is_checkpoint_resume
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
        // Capture committed settings once for this turn, including team members.
        let (turn_resilience_policy, _) =
            crate::uar::settings::persisted_resilience::resolve_persisted_resilience_policy(
                &self.resilience_policy,
                self.settings_manager.as_deref(),
                &artifact.id,
            )
            .await;
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
        self.checkpoint_evidence_owners
            .write()
            .await
            .insert(owner_id.clone());
        self.checkpoint_effect_roots
            .write()
            .await
            .insert((owner_id.clone(), run_id.clone()));
        if let Some(owner) = &verified_owner {
            self.checkpoint_evidence_owners
                .write()
                .await
                .insert(owner.presentation_owner_key());
        }
        let session_owner = match verified_owner.as_ref() {
            Some(owner) => owner.session_owner_key(),
            None if owner_id == crate::session::ANONYMOUS_SESSION_OWNER => Ok(owner_id.clone()),
            None => Err(anyhow::anyhow!("Verified conversation owner required")),
        };
        let session_owner = match session_owner {
            Ok(owner) => owner,
            Err(error) => {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "session_owner_invalid".into(),
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
        let retained_session = if let Some(id) = session_id {
            // A process restart clears the local cache, not the conversation.
            // Load through the same owner partition before staging this turn.
            let saved = if self.sessions.get_for_user(&id, &session_owner).is_none() {
                if let Some(db) = &self.persistence {
                    match db.load_session(&session_owner, &id).await {
                        Ok(saved) => saved,
                        Err(error) => {
                            tracing::warn!(%error, "Failed to restore conversation");
                            emitter
                                .emit(NormalizedEvent::Error {
                                    run_id: run_id.clone(),
                                    code: "session_restore_failed".into(),
                                    message: "Stored conversation could not be loaded".into(),
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
                } else {
                    None
                }
            } else {
                None
            };
            let retained = self.sessions.get_or_create_for_user(&id, &session_owner);
            if let Some(saved) = saved {
                // A concurrent request may already have warmed this session.
                retained.seed_if_empty(&saved.messages());
            }
            retained
        } else {
            self.sessions.create_for_user(&session_owner)
        };
        // Routing reads a private staged view. Admission has not yet captured
        // the selected provider corpus, so shared history must remain untouched.
        let session = crate::session::Session::from_state(retained_session.to_state());
        // SessionState excludes host-only world-state records. Restore those
        // records on the detached view without publishing shared history.
        if let Some(instructions) = retained_session.project_instructions() {
            session.record_world_state(
                &crate::uar::runtime::world_state::contributor::WorldStateUpdate {
                    fragments: Vec::new(),
                    messages: Vec::new(),
                    baseline: retained_session.world_state_baseline(),
                },
                instructions,
            );
        }
        let staged_replaces_history = supplied_history.is_some();
        let mut staged_seed: Option<Vec<Message>> = None;

        // 1b. Seed prior turns atomically. A checkpoint or inherited child
        // history is applied later after its authorization check and always
        // outranks request-supplied history.
        let mut history_status = crate::uar::runtime::turn::host::HistorySeedStatus::None;
        let mut seeded_messages = 0usize;
        if supplied_history.is_none() {
            let embedded_history = (!seed_history.is_empty()).then(|| {
                seed_history
                    .iter()
                    .filter_map(|message| {
                        let role = match message.role.as_str() {
                            "assistant" => MessageRole::Assistant,
                            "tool" => MessageRole::Tool,
                            "system" => return None,
                            _ => MessageRole::User,
                        };
                        Some(Message {
                            role,
                            content: crate::llm::MessageContent::text(&message.content),
                            tool_call_id: message.tool_call_id.clone(),
                            tool_calls: None,
                        })
                    })
                    .collect::<Vec<_>>()
            });
            let candidate = host_history.as_ref().or(embedded_history.as_ref());
            if let Some(candidate) = candidate {
                staged_seed = Some(candidate.clone());
                match session.seed_if_empty(candidate) {
                    crate::session::SeedOutcome::Seeded { messages } => {
                        history_status = crate::uar::runtime::turn::host::HistorySeedStatus::Seeded;
                        seeded_messages = messages;
                    }
                    crate::session::SeedOutcome::IgnoredWarmSession => {
                        history_status =
                            crate::uar::runtime::turn::host::HistorySeedStatus::IgnoredWarmSession;
                    }
                }
            }
        }

        // Attempt-bound peer handlers are created later, but their identities
        // must enter the universe before normal scopes can select or deny them.
        let mut attempt_native_tools = BTreeSet::new();
        if collaboration_binding
            .as_ref()
            .is_some_and(|binding| binding.team_attempt.is_some())
            && crate::uar::domain::execution_capabilities::team_execution_b_enabled()
        {
            attempt_native_tools.extend(
                crate::uar::runtime::native_skills::team_tools::TEAM_TOOL_NAMES
                    .into_iter()
                    .map(str::to_owned),
            );
        }
        let pre_resolved_policy = pre_resolved_policy_for_run(
            is_checkpoint_resume,
            inherited
                .as_ref()
                .map(|bindings| bindings.policy.effective().clone()),
            resolved_policy,
        );
        let policy_was_pre_resolved = pre_resolved_policy.is_some();
        let mut effective_policy = match pre_resolved_policy {
            Some(policy) => policy,
            None => {
                self.resolve_effective_policy_with_catalog(
                    &artifact,
                    &owner_id,
                    session.id(),
                    actor_root.is_some(),
                    host_policy_constraint.clone(),
                    mcp_resources
                        .as_ref()
                        .map(|resources| resources.catalog().as_ref()),
                    Some(&attempt_native_tools),
                    verified_owner.as_ref(),
                )
                .await
            }
        };
        if let Some(names) = mcp_resources
            .as_ref()
            .and_then(|resources| resources.run_scoped_names())
        {
            if effective_policy
                .mcp_servers
                .ids
                .iter()
                .any(|name| !names.contains(name))
            {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "mcp_server_not_run_scoped".into(),
                        message: "Run policy selects an MCP server outside this request".into(),
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

        // Remote descriptors do not exist until the authenticated run binding
        // performs tools/list. Resolve policy once more with those exact names
        // in the universe so every normal scope can allow, deny or narrow them.
        if !policy_was_pre_resolved
            && let Some(resources) = mcp_resources
                .as_ref()
                .filter(|resources| resources.run_scoped_names().is_some())
        {
            let mut discovered = match resources.discover_tool_ids(&effective_policy).await {
                Ok(discovered) => discovered,
                Err(error) => {
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
                    self.run_cancellations.write().await.remove(&run_id);
                    return run_id;
                }
            };
            discovered.append(&mut attempt_native_tools);
            effective_policy = self
                .resolve_effective_policy_with_catalog(
                    &artifact,
                    &owner_id,
                    session.id(),
                    actor_root.is_some(),
                    host_policy_constraint,
                    Some(resources.catalog().as_ref()),
                    Some(&discovered),
                    verified_owner.as_ref(),
                )
                .await;
            let names = resources
                .run_scoped_names()
                .expect("filtered run-scoped resources retain names");
            if effective_policy
                .mcp_servers
                .ids
                .iter()
                .any(|name| !names.contains(name))
            {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "mcp_server_not_run_scoped".into(),
                        message: "Run policy selects an MCP server outside this request".into(),
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

        // Only the host's verified owner can grant protected skills. This
        // narrowing is repeated on every authenticated continuation and also
        // constrains dynamically selected/activated skills and inherited policy.
        effective_policy
            .skills
            .ids
            .retain(|id| self.skill_role_allowed(id, verified_owner.as_ref()));

        let representation_context = match async {
            let Some(binding) = collaboration_binding
                .as_ref()
                .filter(|binding| binding.has_representation())
            else {
                return Ok::<_, anyhow::Error>(None);
            };
            anyhow::ensure!(
                self.governance_engine.is_some()
                    && self
                        .governance_gate
                        .as_ref()
                        .is_none_or(|gate| gate.effective_enabled()),
                "REPRESENTATION_CEDAR_REQUIRED"
            );
            anyhow::ensure!(
                session.message_count() == 0 && supplied_history.is_none() && !is_checkpoint_resume,
                "REPRESENTATION_HISTORY_SCOPE_UNSUPPORTED"
            );
            let context = binding.representation_context().await?;
            context.narrow(&mut effective_policy)?;
            memory_hits.clear();
            Ok(Some(context))
        }
        .await
        {
            Ok(context) => context,
            Err(error) => {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "representation_admission_denied".into(),
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

        let checkpoint_authorization_valid = match checkpoint_authorization_digest.as_deref() {
            Some(expected) => {
                crate::uar::runtime::checkpoint::authorization_digest(&effective_policy) == expected
            }
            None => !is_checkpoint_resume,
        };
        if !checkpoint_authorization_valid {
            let invalidation_unavailable = match &recovered_approval {
                Some(record) => self
                    .approvals
                    .finish_recovered(
                        record,
                        crate::uar::persistence::approval_decisions::ApprovalState::Invalidated,
                    )
                    .await
                    .is_err(),
                None => false,
            };
            let error_code = if invalidation_unavailable {
                "approval_persistence_unavailable"
            } else {
                "checkpoint_authorization_revoked"
            };
            let error_message = if invalidation_unavailable {
                "Recovered approval invalidation could not be persisted; continuation claim remains held"
            } else {
                "Checkpoint authorization is no longer valid"
            };
            let dialogue = redacted_checkpoint_dialogue(&session);
            let run = Run {
                run_id: run_id.clone(),
                agent_id: artifact.id.clone(),
                conversation_id: Some(session.id().to_string()),
                user_id: user_id.clone(),
                status: RunStatus::Error,
                context: serde_json::json!({
                    "error_code": error_code,
                    "agent_snapshot": null,
                }),
            };
            {
                let mut runs = self.active_runs.write().await;
                runs.insert(
                    run_id.clone(),
                    RunStreamState {
                        run: run.clone(),
                        approval_root_run_id: None,
                        verified_owner: verified_owner.clone(),
                        presentations: Some(Arc::clone(&presentation_snapshot)),
                        dialogue,
                        sender: tx.clone(),
                        history: Arc::clone(&history),
                        caller_handoff_confirmation: Arc::new(Mutex::new(())),
                        completion: emitter.completion.as_ref().map(Arc::downgrade),
                        delegation: None,
                        tool_admission: None,
                        actor_spawn_descriptor: None,
                        actor_tool_approval: None,
                        started_at: Instant::now(),
                        terminal_at: Some(Instant::now()),
                        last_detached_at: None,
                    },
                );
            }
            if let Some(writer) = &self.run_history {
                writer.write_record(&run, verified_owner.as_ref());
            }
            emitter
                .emit(NormalizedEvent::Error {
                    run_id: run_id.clone(),
                    code: error_code.into(),
                    message: error_message.into(),
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

        if let Some(history) = supplied_history {
            // Authorization has been re-established. Only now may the branch
            // replace a warm session with the checkpoint's protected history.
            session.clear();
            for message in history {
                if message.role != MessageRole::System {
                    session.add_message(message.clone());
                } else if client_tools.is_some() {
                    // Caller instructions remain ordered caller content; they cannot
                    // replace authoritative agent/system policy.
                    let mut caller = message.clone();
                    caller.role = MessageRole::User;
                    session.add_message(caller);
                }
            }
        }

        // 2. Add User Message
        if append_input {
            session.add_user_message(&input);
        }

        // Capture identity for credential resolution before `user_id` is moved
        // into the Run record and the resolved session id is the source of truth.
        let principal_for_creds = verified_owner
            .as_ref()
            .and_then(|owner| owner.credential_principal())
            .cloned();
        let session_id_for_creds = Some(session.id().to_string());

        let dialogue = redacted_checkpoint_dialogue(&session);
        let effective_service_binding = service_binding.clone();
        let effective_collaboration_binding = collaboration_binding.as_ref().map(|binding| {
            serde_json::json!({
                "ownerId": &binding.owner_id,
                "workspaceId": &binding.workspace_id,
                "bindingId": &binding.receipt.binding_ref.id,
                "bindingRevision": binding.receipt.binding_ref.revision,
            })
        });
        let agent_snapshot = artifact.snapshot(if host_resources_marker.artifact_inline {
            "inline"
        } else {
            "embedded"
        });
        let mut retained_policy = effective_policy.clone();
        retained_policy.warnings.clear();
        let run = Run {
            run_id: run_id.clone(),
            agent_id: artifact.id.clone(),
            conversation_id: Some(session.id().to_string()),
            user_id,
            status: RunStatus::Running,
            context: serde_json::json!({
                "input": null,
                "effective_run_policy": retained_policy,
                "effective_target": target_selection,
                "presentation_negotiation": presentation_snapshot.negotiation(),
                "presentation_selection": presentation_snapshot.selection(),
                "presentation_templates": presentation_snapshot.identities(),
                "agent_snapshot": null,
                "host_resources": host_resources_marker,
                "effective_service_binding": effective_service_binding.clone(),
                "effective_collaboration_binding": effective_collaboration_binding,
                "representation_disclosure": representation_context.as_ref().map(|context| context.disclosure()),
                "representation_grants": collaboration_binding.as_ref().map(|binding| binding.representation_grants()),
                "host_context": {
                    "working_directory": working_directory.as_ref().map(|path| path.display().to_string()),
                    "reasoning_effort": reasoning_effort.map(crate::config::ReasoningEffort::as_str),
                },
                "history": history_status,
                "seeded_messages": seeded_messages,
            }),
        };

        {
            let mut runs = self.active_runs.write().await;
            runs.insert(
                run_id.clone(),
                RunStreamState {
                    run: run.clone(),
                    approval_root_run_id: None,
                    verified_owner: verified_owner.clone(),
                    presentations: Some(Arc::clone(&presentation_snapshot)),
                    dialogue: dialogue.clone(),
                    sender: tx.clone(),
                    history: Arc::clone(&history),
                    caller_handoff_confirmation: Arc::new(Mutex::new(())),
                    completion: emitter.completion.as_ref().map(Arc::downgrade),
                    delegation: None,
                    tool_admission: None,
                    actor_spawn_descriptor: None,
                    actor_tool_approval: None,
                    started_at: Instant::now(),
                    terminal_at: None,
                    last_detached_at: None,
                },
            );
        }
        if let Some(writer) = &self.run_history {
            writer.write_record(&run, verified_owner.as_ref());
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
                crate::uar::persistence::tenant_storage_key(session.owner_id(), session.id()),
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
        self.run_admission_generation.send_modify(|generation| {
            *generation = generation.saturating_add(1);
        });
        if run_cancellation.is_cancelled() {
            if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                state.run.status = RunStatus::Cancelled;
            }
            emitter
                .emit(NormalizedEvent::Cancelled {
                    run_id: run_id.clone(),
                    usage: None,
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
            None => match self.approvals.register_scoped(
                run_id.clone(),
                owner_id.clone(),
                verified_owner
                    .as_ref()
                    .and_then(|owner| owner.credential_principal())
                    .map(|owner| owner.storage_key())
                    .unwrap_or_else(|| format!("v1:s:{}:{}", owner_id.len(), owner_id)),
                collaboration_binding
                    .as_ref()
                    .map(|binding| binding.workspace_id.clone()),
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

        if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
            state.approval_root_run_id = Some(approval_channel.root_run_id().to_owned());
        }

        let working_directory = inherited
            .as_ref()
            .map(|bindings| bindings.working_directory.clone())
            .or(working_directory)
            .or_else(|| {
                mcp_resources
                    .as_ref()
                    .map(|resources| resources.environment().directory().to_path_buf())
            });
        // A team attempt's context is its explicitly selected task/artifacts.
        // This applies to its delegated children as well as its fresh root.
        let project_instructions_config = if collaboration_binding
            .as_ref()
            .is_some_and(|binding| binding.team_attempt.is_some())
        {
            crate::uar::runtime::project_instructions::ProjectInstructionsConfig::default()
        } else {
            self.project_instructions_config.clone()
        };
        let mut world_state = match working_directory
            .map(Ok)
            .unwrap_or_else(std::env::current_dir)
            .and_then(|cwd| {
                crate::uar::runtime::world_state::runtime::WorldStateRuntime::new(
                    session.clone(),
                    cwd,
                    project_instructions_config,
                    self.world_state_config,
                    effective_policy.clone(),
                    Arc::clone(&self.world_state_clock),
                )
            }) {
            Ok(world_state) => world_state,
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
        let root_run_id = inherited.as_ref().map_or_else(
            || run_id.clone(),
            |bindings| bindings.thread.root_run_id.clone(),
        );
        let host_tool_admission =
            host_tool_admission.unwrap_or_else(|| Arc::clone(&self.host_tool_admission));
        let governance_policy_revision = match &self.governance_engine {
            Some(engine) => engine.policy_revision().await,
            None => "cedar:unavailable".to_string(),
        };
        let tool_admission = match (|| {
            // Team host effects use the admitted tenant/subject partition,
            // while ordinary session and thread ownership retain the subject.
            let admission_owner_id = match collaboration_binding.as_ref().and_then(|binding| {
                binding
                    .team_attempt
                    .as_ref()
                    .map(|attempt| (binding, attempt))
            }) {
                Some((binding, attempt)) => {
                    let owner = verified_owner.as_ref().ok_or_else(|| {
                        anyhow::anyhow!("Team tool admission requires a verified host owner")
                    })?;
                    let admitted_owner = owner.presentation_owner_key();
                    anyhow::ensure!(
                        admitted_owner == binding.owner_id && admitted_owner == attempt.owner_id,
                        "Team tool admission owner does not match its admitted binding and attempt"
                    );
                    admitted_owner
                }
                None => owner_id.clone(),
            };
            crate::uar::runtime::tool_admission::ToolAdmissionContext::new(
                root_run_id,
                run_id.clone(),
                admission_owner_id,
                artifact.id.clone(),
                world_state.directory().display().to_string(),
                self.tool_runtime_epoch.clone(),
                &artifact,
                &effective_policy,
                governance_policy_revision,
                host_tool_admission.binding(),
            )
        })()
        .and_then(|context| {
            crate::uar::runtime::tool_admission::ToolAdmissionRuntime::new_in_scope(
                context,
                Arc::clone(&host_tool_admission),
                self.persistence.clone(),
                run_cancellation.clone(),
                self.governance_engine.clone(),
                self.governance_gate.clone(),
                self.lifecycle_scope.clone(),
            )
        }) {
            Ok(admission) => {
                let admission = if let Some(binding) = &collaboration_binding {
                    admission.with_claim_revalidator(Arc::new(binding.clone()))
                } else {
                    admission
                };
                let admission = if let Some(binding) = &instance_binding {
                    admission.with_claim_revalidator(Arc::new(binding.clone()))
                } else {
                    admission
                };
                Arc::new(admission)
            }
            Err(error) => {
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "tool_admission_context_failed".into(),
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
        if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
            state.tool_admission = Some(Arc::clone(&tool_admission));
        }

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
        // Routing precedes generation-client resolution. Project with only the
        // auth/host/MCP corpus captured now; later keys cannot protect this call.
        let routing_input = emitter.secret_scrubber.scrub(&input);
        if !effective_policy.knowledge_bases.ids.is_empty()
            && let Some(db) = &self.persistence
        {
            let mut kb_ids = Vec::new();
            let mut kb_configs: Vec<crate::uar::domain::knowledge::KbConfig> = Vec::new();
            let rag_embedding = self.vector_matcher.pin_embedding();
            let active_space = rag_embedding.fingerprint();
            let mut space_mismatches: Vec<crate::uar::domain::knowledge::EmbeddingSpaceMismatch> =
                Vec::new();
            for id_or_name in &effective_policy.knowledge_bases.ids {
                let resolved = db
                    .get_knowledge_base(&session_owner, id_or_name)
                    .await
                    .ok()
                    .flatten();
                let resolved = match resolved {
                    Some(kb) => Some(kb),
                    None => db
                        .get_knowledge_base_by_name(&session_owner, id_or_name)
                        .await
                        .ok()
                        .flatten(),
                };
                if let Some(kb) = resolved
                    && !kb_ids.contains(&kb.id)
                {
                    if kb.regeneration_unavailable() {
                        emitter.emit(NormalizedEvent::RagDiagnostic {
                            run_id: run_id.clone(),
                            code: "knowledge_regeneration_incomplete".into(),
                            message: format!("Knowledge base '{}' is unavailable until regeneration completes", kb.id),
                        }).await;
                        continue;
                    }
                    match kb.ensure_embedding_space(&active_space) {
                        Ok(()) => {
                            kb_configs.push(kb.config.clone());
                            kb_ids.push(kb.id);
                        }
                        Err(mismatch) => {
                            if !space_mismatches.iter().any(|m| m.kb_id == mismatch.kb_id) {
                                space_mismatches.push(mismatch);
                            }
                        }
                    }
                }
            }
            // Searching a KB from another embedding space returns nothing;
            // skip it and say so instead of answering without its content.
            for mismatch in space_mismatches {
                tracing::warn!(
                    run_id = %run_id,
                    kb_id = %mismatch.kb_id,
                    indexed = %mismatch.indexed,
                    active = %mismatch.active,
                    "knowledge base skipped for RAG: embedding space mismatch"
                );
                emitter
                    .emit(NormalizedEvent::RagDiagnostic {
                        run_id: run_id.to_string(),
                        code: "embedding_space_mismatch".to_string(),
                        message: mismatch.to_string(),
                    })
                    .await;
            }

            // A configured selection that resolves to nothing is a safe empty
            // result, never an implicit search-all.
            let search_result = if kb_ids.is_empty() {
                Ok(Vec::new())
            } else {
                let backend = ChatRagSearchBackend {
                    persistence: db.as_ref(),
                    embedding: Arc::clone(&rag_embedding),
                    owner_id: &session_owner,
                    kb_ids: &kb_ids,
                };
                let kb_config_refs: Vec<&crate::uar::domain::knowledge::KbConfig> =
                    kb_configs.iter().collect();
                let (top_k, min_score) =
                    crate::uar::domain::knowledge::retrieval_params(&kb_config_refs);
                RagRetrievalPipeline::new()
                    .retrieve(
                        &backend,
                        &kb_ids.join(","),
                        &routing_input,
                        top_k,
                        min_score,
                    )
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
                                && let Ok(Some(doc)) = db.get_document(&session_owner, did).await
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
                Err(_) => tracing::error!("RAG retrieval pipeline failed during private admission"),
            }
        }

        use crate::uar::domain::skills::{SkillCandidate, SkillMatchResult};
        let skill_bindings = match &inherited {
            Some(bindings) => Arc::clone(&bindings.skills),
            None => match crate::uar::runtime::turn::bindings::RunSkillBindings::capture(
                &self.skills,
                self.skill_service.as_deref(),
            )
            .await
            {
                Ok(bindings) => Arc::new(bindings),
                Err(error) => {
                    tracing::warn!(
                        run_id = %run_id,
                        error = %error,
                        "skill selection snapshot is unavailable"
                    );
                    if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "skill_selection_unavailable".into(),
                            message: "Skill selection could not be reconciled; read selection state before retrying".into(),
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
        let is_team_attempt = collaboration_binding
            .as_ref()
            .is_some_and(|binding| binding.team_attempt.is_some());
        // Auxiliary ranking precedes skill model preferences. Its client has its
        // own credential capture, while every role shares this root accounting.
        let shared_model_budget_result = if let Some(bindings) = &inherited {
            Ok(bindings.models.budget().clone())
        } else {
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

            let model_budget = match model_budget {
                Ok(budget)
                    if self.llm_config.cost_tracking
                        || budget.has_configured_limits()
                        || (self
                            .persistence
                            .as_ref()
                            .is_some_and(|store| store.supports_durable_accounting())
                            && verified_owner
                                .as_ref()
                                .and_then(|owner| owner.credential_principal())
                                .is_some()) =>
                {
                    budget
                        .with_durable_accounting(self.persistence.clone(), verified_owner.as_ref())
                        .await
                }
                Ok(budget) => Ok(budget),
                Err(error) => Err(error),
            };

            let model_budget = match (
                model_budget,
                client_tools
                    .as_ref()
                    .and_then(|input| input.budget_snapshot.as_ref()),
            ) {
                (Ok(budget), Some(snapshot)) => match verified_owner.as_ref() {
                    Some(owner) => budget.restore_client_handoff(snapshot, owner).await,
                    None => Err(anyhow::anyhow!("client_tools_require_authenticated_owner")),
                },
                (budget, None) => budget,
                (Err(error), Some(_)) => Err(error),
            };

            model_budget
        };
        let shared_model_budget = match shared_model_budget_result {
            Ok(budget) => budget,
            Err(_) => {
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "budget_admission_failed".into(),
                        message: "Run accounting unavailable".into(),
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
        let matching_result = async {
            use crate::uar::runtime::skills::service::{SkillMatchingAlgorithm, SkillMatchingEngine};
            use crate::uar::runtime::matching::intent::ClassifierBackend;
            let engine = match &self.settings_manager {
                Some(settings) => settings.skill_matching_engine_for_admission().await?,
                None => skill_bindings.matching.as_ref().map_or(
                    SkillMatchingEngine::IntentClassifier, |matching| matching.config.engine),
            };
            let service_config = skill_bindings.matching.as_ref().map(|matching| &matching.config);
            let needs_llm = match engine {
                SkillMatchingEngine::Service => service_config
                    .ok_or_else(|| anyhow::anyhow!("skill_service_resource_required"))?.algorithm == SkillMatchingAlgorithm::Llm,
                SkillMatchingEngine::IntentClassifier => self.classifier_config.backend == ClassifierBackend::Llm,
            };
            let requested_model = if needs_llm {
                service_config.and_then(|config| config.model_name.clone())
                    .filter(|model| !model.trim().is_empty())
            } else { None };
            let auxiliary = if needs_llm {
                let bindings = if let Some(inherited) = &inherited {
                    inherited.models.for_policy(&inherited.policy)?
                } else {
                    let mut config = self.llm_config.clone();
                    if let Some(route) = &effective_policy.model {
                        config.model = format!("{}/{}", route.provider_id, route.model_id);
                        config.resolved_provider_id = Some(route.provider_id.clone());
                    } else if !artifact.policy.provider.default.model.trim().is_empty() {
                        config.model = format!("{}/{}", artifact.policy.provider.default.provider,
                            artifact.policy.provider.default.model);
                        config.resolved_provider_id = Some(artifact.policy.provider.default.provider.clone());
                    }
                    if let Some(model) = &requested_model {
                        let (provider, _) = crate::llm::registry::split_model_string_pub(&qualified_model_name(&config));
                        config.model = if model.contains('/') { model.clone() } else { format!("{provider}/{model}") };
                        let (selected_provider, _) = crate::llm::registry::split_model_string_pub(&config.model);
                        config.resolved_provider_id = Some(selected_provider);
                    }
                    if config.model.trim().is_empty()
                        && let Some(registry) = &self.provider_registry
                        && let Some((provider, model)) = registry.default_model().await {
                        config.model = format!("{provider}/{model}");
                    }
                    let (provider, model) = crate::llm::registry::split_model_string_pub(&qualified_model_name(&config));
                    if let Some(registry) = &self.provider_registry {
                        let routed = registry.resolve_to_llm_config(&provider, &model).await
                            .ok_or_else(|| anyhow::anyhow!("auxiliary_model_unavailable"))?;
                        config = apply_routed_connection(config, routed);
                    }
                    if let Some(credentials) = &run_credentials {
                        config = credentials.config_for(&provider, Some(&model), config)?;
                    }
                    config.resolved_provider_id = Some(provider.clone());
                    let mut profiles = BTreeMap::new();
                    let mut pricing_models = BTreeMap::new();
                    if is_team_attempt {
                        anyhow::ensure!(reasoning_effort.is_none(), "TEAM_REASONING_UNSUPPORTED");
                        let registry = self.provider_registry.as_deref()
                            .ok_or_else(|| anyhow::anyhow!("TEAM_PROFILE_UNSUPPORTED"))?;
                        let binding = collaboration_binding.as_ref()
                            .ok_or_else(|| anyhow::anyhow!("TEAM_PROFILE_UNSUPPORTED"))?;
                        for admitted in &binding.receipt.resolved_models {
                            let effective: crate::uar::domain::team_execution::EffectiveTeamModelReceipt =
                                serde_json::from_value(admitted["effectiveModel"].clone())
                                    .map_err(|_| anyhow::anyhow!("TEAM_PROFILE_UNSUPPORTED"))?;
                            let profile = registry.capture_team_endpoint_profile(&effective).await?;
                            if let Some(price) = effective.pricing_identity {
                                pricing_models.insert(profile.qualified_model.clone(),
                                    format!("{}/{}", price.provider_id, price.model_id));
                            }
                            profiles.insert(profile.qualified_model.clone(), profile);
                        }
                        let profile = profiles.get(&format!("{provider}/{model}"))
                            .ok_or_else(|| anyhow::anyhow!("TEAM_ROUTE_PROFILE_MISMATCH"))?;
                        config.model.clone_from(&profile.qualified_model);
                        config.catalog_pricing_model = pricing_models.get(&profile.qualified_model).cloned();
                    }
                    let (config, decision) = resolve_credential_binding(config,
                        self.credential_policy.as_ref(), self.provider_service.as_ref(),
                        principal_for_creds.as_ref(), session_id_for_creds.as_deref(),
                        Some(artifact.id.as_str()), run_credentials.is_some()).await?;
                    let mut failover = self.failover_config.clone();
                    failover.enabled = false;
                    failover.fallback_models.clear();
                    crate::uar::runtime::turn::bindings::RunModelBindings::capture(
                        config, None, decision, BTreeMap::new(), failover, None,
                        shared_model_budget.clone(), run_credentials.as_ref(),
                        profiles, pricing_models, &mut emitter.secret_scrubber).await?
                };
                let model = match requested_model.as_deref() {
                    Some(model) => bindings.auxiliary_model(model)?,
                    None => qualified_model_name(bindings.config()),
                };
                let driver = bindings.auxiliary(&model)?;
                let exact = self.destination_preparations.as_ref()
                    .and_then(|preparations| preparations.budget_contract(&model)).cloned();
                let (provider, model_id) = crate::llm::registry::split_model_string_pub(&model);
                let configured = if let Some(registry) = &self.provider_registry {
                    registry.models(&provider).await.and_then(|models| models.into_iter()
                        .find(|entry| entry.id == model_id).and_then(|entry| entry.context_window))
                        .map(|window| window as usize)
                } else { None };
                let capacity = run_credentials.as_ref().and_then(|credentials|
                    credentials.context_window_for(&provider, &model_id)).or(configured)
                    .or_else(|| crate::llm::catalog::ModelCatalog::global().model(&provider, &model_id)
                        .map(|entry| entry.limits.context_window as usize).filter(|window| *window > 0));
                let allowance = match exact {
                    Some(contract) => crate::uar::runtime::context::budget::AuxiliaryRequestAllowance::Exact(contract),
                    None => {
                        let capacity = capacity.ok_or_else(|| anyhow::anyhow!("auxiliary_capacity_unknown"))?;
                        anyhow::ensure!(capacity > 2304, "auxiliary_capacity_insufficient");
                        crate::uar::runtime::context::budget::AuxiliaryRequestAllowance::Estimated {
                            destination_model: model.clone(), input_allowance: capacity - 2048,
                            output_ceiling: 2048, reserve_tokens: 256,
                            counting_revision: "host-plaintext-estimate-v1".into(),
                        }
                    }
                };
                Some((driver, model, allowance))
            } else { None };
            let effective_model = auxiliary.as_ref().map(|(_, model, _)| model.clone());
            let (candidates, method, threshold, margin, top_k) = match engine {
                SkillMatchingEngine::Service => {
                    let matching = skill_bindings.matching.as_ref()
                        .ok_or_else(|| anyhow::anyhow!("skill_service_resource_required"))?;
                    let config = &matching.config;
                    let result = matching.match_skills_scoped_checked(&routing_input,
                        Some(&artifact.id), Some(session.id()), auxiliary.as_ref()
                            .map(|(driver, model, allowance)| (driver.as_ref(), model.as_str(), allowance))).await?;
                    (result.candidates, format!("skill_service.{}", match config.algorithm {
                        SkillMatchingAlgorithm::Keyword => "keyword", SkillMatchingAlgorithm::Embedding => "embedding",
                        SkillMatchingAlgorithm::LocalEmbedding => "local_embedding", SkillMatchingAlgorithm::Hybrid => "hybrid",
                        SkillMatchingAlgorithm::Llm => "llm",
                    }),
                        config.threshold, config.margin_threshold, config.top_k)
                }
                SkillMatchingEngine::IntentClassifier => {
                    let registry = skill_bindings.registry.read().await.clone();
                    let result = if let Some((driver, model, allowance)) = &auxiliary {
                        let skills = registry.list().into_iter().filter(|skill|
                            effective_policy.skills.ids.contains(&skill.skill_id)
                                && skill.enabled_for(Some(&artifact.id), Some(session.id()))).collect::<Vec<_>>();
                        crate::uar::runtime::matching::intent::classify_skills_with_driver(
                            &routing_input, &[], &skills, driver.as_ref(), model, allowance).await?
                    } else {
                        let pinned_vector = if self.classifier_config.backend == ClassifierBackend::LocalEmbedding {
                            let matcher = self.local_skill_matcher.as_ref()
                                .ok_or_else(|| anyhow::anyhow!("local_embedding_classifier_resource_required"))?;
                            Some(Arc::new(crate::uar::runtime::matching::vector::VectorMatcher::new(
                                Arc::new(crate::uar::rag::embeddings::swappable::SwappableEmbeddingBackend::new(
                                    matcher.pin_embedding())), self.classifier_config.accept_threshold)))
                        } else {
                            None
                        };
                        let classifier = crate::uar::runtime::matching::intent::create_classifier_with_resources(
                            &self.classifier_config, pinned_vector, None);
                        classifier.rebuild_index(&registry).await?;
                        classifier.classify(&routing_input, &[], &registry).await?
                    };
                    (result.scores.into_iter().filter_map(|score| score.skill.map(|skill|
                        SkillCandidate { skill, score: score.score })).collect(),
                        format!("legacy_classifier.{:?}", self.classifier_config.backend).to_lowercase(),
                        self.classifier_config.accept_threshold, self.classifier_config.margin_threshold,
                        self.classifier_config.topk)
                }
            };
            Ok::<_, anyhow::Error>((candidates, method, threshold, margin, top_k, requested_model, effective_model))
        }.await;
        let (
            candidates,
            skill_selection_method,
            threshold,
            margin,
            top_k,
            matching_requested_model,
            matching_effective_model,
        ) = match matching_result {
            Ok(result) => result,
            Err(_) => {
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "skill_matching_unavailable".into(),
                        message:
                            "Selected skill matching method or governed resource is unavailable"
                                .into(),
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
            context.insert("skill_matching".into(), serde_json::json!({
                "requested_method": skill_selection_method, "effective_method": skill_selection_method,
                "requested_model": matching_requested_model, "effective_model": matching_effective_model,
                "threshold": threshold, "margin": margin, "top_k": top_k,
                "candidates": skill_match_result.candidates.iter().map(|candidate| serde_json::json!({
                    "skill_id": candidate.skill.skill_id, "score": candidate.score,
                    "accepted": suggested_skill_ids.contains(&candidate.skill.skill_id),
                })).collect::<Vec<_>>()
            }));
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
        // Elicitation is a governed host interaction, never a tool approval.
        // Children inherit the root's frozen connection/channel projection.
        if !child_run
            && mcp_resources.is_some()
            && let Some(store) = self
                .persistence
                .as_ref()
                .filter(|store| store.supports_durable_mcp_elicitations())
            && let Some(owner) = verified_owner
                .as_ref()
                .filter(|owner| owner.credential_principal().is_some())
        {
            match crate::mcp::interaction::McpInteractionChannel::new(
                Arc::clone(store),
                owner,
                run_id.clone(),
                session.id().to_owned(),
                collaboration_binding
                    .as_ref()
                    .map(|binding| binding.workspace_id.clone()),
                run_cancellation.clone(),
                self.lifecycle_scope.clone(),
            ) {
                Ok(channel) => {
                    mcp_resources = mcp_resources
                        .take()
                        .map(|resources| resources.with_interactions(channel));
                }
                Err(error) => {
                    if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "mcp_interaction_admission_failed".into(),
                            message: error.to_string(),
                        })
                        .await;
                    self.run_cancellations.write().await.remove(&run_id);
                    return run_id;
                }
            }
        }
        let mut mcp_interaction_guard = if child_run {
            None
        } else {
            mcp_resources
                .as_ref()
                .and_then(McpRunResources::interaction_guard)
        };
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
                            usage: None,
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
        if let Some(binding) = &collaboration_binding {
            activation_context.lock().await.set_binding_configs(
                binding
                    .receipt
                    .resolved_skills
                    .iter()
                    .map(|resolved| (resolved.skill.id.clone(), resolved.skill.config.clone())),
            );
        }
        let register_turn_tools = async {
            if effective_policy.tools.mode != SelectionMode::None {
                if let Some(binding) = &collaboration_binding {
                    if let Some(attempt) = binding.team_attempt.as_ref().filter(|_| {
                        crate::uar::domain::execution_capabilities::team_execution_b_enabled()
                    }) {
                        crate::uar::runtime::native_skills::team_tools::register(
                            &native_skills,
                            crate::uar::runtime::native_skills::team_tools::TeamToolBinding {
                                catalog: Arc::clone(&binding.service),
                                attempt: attempt.clone(),
                                yielded: Arc::clone(&binding.team_yield),
                            },
                            &selected_tools,
                        )
                        .await?;
                    }
                }
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
            }
            if effective_policy.allows_model_control_tool(
                crate::uar::domain::policy::ACTIVATE_SKILL_TOOL,
                crate::uar::domain::policy::ACTIVATE_SKILL_TOOL,
            ) {
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
        // Built-in model-control tools were registered only when their
        // policy-specific conditions permit them. Filter every descriptor once
        // more before advertising it to the model.
        let native_skills = {
            let allowed = native_skills
                .descriptors()
                .await
                .into_iter()
                .filter(|descriptor| {
                    let model_control = descriptor.source
                        == crate::uar::tools::descriptor::ToolSource::BuiltIn
                        && descriptor.exposure
                            == crate::uar::tools::descriptor::Exposure::ModelOnly;
                    !model_control
                        || effective_policy
                            .allows_model_control_tool(&descriptor.provider_name, &descriptor.id)
                })
                .map(|descriptor| descriptor.provider_name.clone())
                .collect::<HashSet<_>>();
            Arc::new(native_skills.filtered(Some(&allowed)).await)
        };
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
            context.insert("activation_failures".to_string(), serde_json::Value::Null);
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
                Err(_) => {
                    tracing::warn!("Implicit skill activation refused during private admission")
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
        let policy_llm_config = if inherited.is_none()
            && let Some(ref registry) = self.provider_registry
        {
            let mut provider_policy = artifact.policy.provider.clone();
            if let Some(route) = &effective_policy.model {
                provider_policy.default.provider = route.provider_id.clone();
                provider_policy.default.model = route.model_id.clone();
            }
            let resolved = if is_team_attempt || target_selection.is_some() {
                registry
                    .resolve_to_llm_config(
                        &provider_policy.default.provider,
                        &provider_policy.default.model,
                    )
                    .await
            } else {
                registry
                    .resolve_llm_config_from_policy(&provider_policy)
                    .await
            };
            match resolved {
                Some(resolved) => {
                    tracing::info!(
                        provider = %provider_policy.default.provider,
                        model = %provider_policy.default.model,
                        "Using per-agent provider settings"
                    );
                    Some(resolved)
                }
                None if run_credentials.is_some()
                    && !is_team_attempt
                    && target_selection.is_none() =>
                {
                    None
                }
                None => {
                    tracing::warn!(
                        provider = %provider_policy.default.provider,
                        model = %provider_policy.default.model,
                        "Catalog provider/model assignment is unavailable"
                    );
                    if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: "provider_model_unavailable".into(),
                            message: format!(
                                "Catalog provider/model '{}/{}' is unavailable",
                                provider_policy.default.provider, provider_policy.default.model
                            )
                            .into(),
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
        // Resolve the model that will actually receive this run before applying
        // any model-keyed context budget. This includes provider-registry and
        // first-matched-skill overrides.
        let model_bindings_result = if let Some(bindings) = &inherited {
            bindings.models.for_policy(&bindings.policy)
        } else {
            let mut preferred_llm_config =
                policy_llm_config.unwrap_or_else(|| self.llm_config.clone());
            let skill_preferred_model = (collaboration_binding.is_none()
                && target_selection.is_none())
            .then(|| {
                matched_skills.iter().find_map(|skill| {
                    skill
                        .execution_config
                        .preferred_model
                        .as_ref()
                        .filter(|model| !model.trim().is_empty())
                        .map(|model| (skill.skill_id.as_str(), model.as_str()))
                })
            })
            .flatten();
            if preferred_llm_config.model.trim().is_empty()
                && skill_preferred_model.is_none()
                && let Some(registry) = &self.provider_registry
                && let Some((provider, model)) = registry.default_model().await
                && let Some(configured) = registry.resolve_to_llm_config(&provider, &model).await
            {
                preferred_llm_config = apply_routed_connection(preferred_llm_config, configured);
            }
            let run_llm_config = if is_team_attempt
                || target_selection.is_some()
                || (preferred_llm_config.model.trim().is_empty() && skill_preferred_model.is_none())
            {
                // Admission selected an exact route. Availability routing may not
                // replace it, or turn missing operator selection into a model choice.
                preferred_llm_config
            } else if let Some(ref registry) = self.provider_registry {
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
                    if config.model != model {
                        config.catalog_pricing_model = None;
                    }
                    config.model = model.to_string();
                }
                config
            };
            let mut run_llm_config = run_llm_config;
            if let Some(credentials) = &run_credentials {
                let policy_route = effective_policy.model.as_ref();
                let mut provider_id = policy_route
                    .map(|route| route.provider_id.as_str())
                    .unwrap_or(artifact.policy.provider.default.provider.as_str());
                let mut model_id = policy_route
                    .map(|route| route.model_id.as_str())
                    .unwrap_or(artifact.policy.provider.default.model.as_str());
                if let Some((_skill_id, preferred)) = skill_preferred_model {
                    if let Some((provider, model)) = preferred.split_once('/') {
                        provider_id = provider;
                        model_id = model;
                    } else {
                        model_id = preferred;
                    }
                }
                run_llm_config =
                    match credentials.config_for(provider_id, Some(model_id), run_llm_config) {
                        Ok(config) => config,
                        Err(error) => {
                            tracing::warn!(code = error.code, "Run provider binding refused");
                            if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                                state.run.status = RunStatus::Error;
                            }
                            emitter
                                .emit(NormalizedEvent::Error {
                                    run_id: run_id.clone(),
                                    code: error.code.into(),
                                    message: error.message.into(),
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
            }
            if let Some(effort) = reasoning_effort {
                run_llm_config.reasoning_effort = Some(effort);
                run_llm_config.thinking_budget = effort.thinking_budget();
            }
            let mut run_llm_config = run_llm_config;
            let mut run_failover_config = if target_selection.is_some() {
                // Explicit target selection never authorizes another model or billing route.
                let mut failover = self.failover_config.clone();
                failover.enabled = false;
                failover.fallback_models.clear();
                failover
            } else if run_credentials.is_some() || collaboration_binding.is_some() {
                let mut failover = self.failover_config.clone();
                failover.fallback_models = artifact
                    .policy
                    .provider
                    .fallbacks
                    .iter()
                    .map(|fallback| crate::config::FallbackModel {
                        model: format!("{}/{}", fallback.provider, fallback.model),
                        api_key: None,
                        base_url: None,
                    })
                    .collect();
                failover.enabled = !failover.fallback_models.is_empty();
                failover
            } else {
                self.failover_config.clone()
            };

            let team_profiles = async {
                let mut profiles = std::collections::BTreeMap::new();
                let mut pricing_models = std::collections::BTreeMap::new();
                if !is_team_attempt {
                    return Ok((profiles, pricing_models));
                }
                anyhow::ensure!(reasoning_effort.is_none(), "TEAM_REASONING_UNSUPPORTED");
                let registry = self
                    .provider_registry
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("TEAM_PROFILE_UNSUPPORTED"))?;
                let binding = collaboration_binding
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("TEAM_PROFILE_UNSUPPORTED"))?;
                for model in &binding.receipt.resolved_models {
                    let effective: crate::uar::domain::team_execution::EffectiveTeamModelReceipt =
                        serde_json::from_value(model["effectiveModel"].clone())
                            .map_err(|_| anyhow::anyhow!("TEAM_PROFILE_UNSUPPORTED"))?;
                    let profile = registry.capture_team_endpoint_profile(&effective).await?;
                    if let Some(price) = effective.pricing_identity {
                        pricing_models.insert(
                            profile.qualified_model.clone(),
                            format!("{}/{}", price.provider_id, price.model_id),
                        );
                    }
                    profiles.insert(profile.qualified_model.clone(), profile);
                }
                anyhow::ensure!(!profiles.is_empty(), "TEAM_PROFILE_UNSUPPORTED");
                let route = effective_policy.model.as_ref();
                let provider = route.map_or(
                    artifact.policy.provider.default.provider.as_str(),
                    |route| route.provider_id.as_str(),
                );
                let model = route
                    .map_or(artifact.policy.provider.default.model.as_str(), |route| {
                        route.model_id.as_str()
                    });
                let selected = profiles
                    .get(&format!("{provider}/{model}"))
                    .ok_or_else(|| anyhow::anyhow!("TEAM_ROUTE_PROFILE_MISMATCH"))?;
                anyhow::ensure!(
                    run_llm_config.resolved_provider_id.as_deref() == Some(provider),
                    "TEAM_ROUTE_PROFILE_MISMATCH"
                );
                // Request credentials supply connection authority only. Restore
                // the admitted complete alias and administrator-owned price.
                run_llm_config.model.clone_from(&selected.qualified_model);
                run_llm_config.catalog_pricing_model =
                    pricing_models.get(&selected.qualified_model).cloned();
                for fallback in &mut run_failover_config.fallback_models {
                    let profile = profiles
                        .get(&fallback.model)
                        .ok_or_else(|| anyhow::anyhow!("TEAM_PROFILE_UNSUPPORTED"))?;
                    let (_, model_id) =
                        crate::llm::registry::split_model_string_pub(&profile.qualified_model);
                    let config = registry
                        .resolve_to_llm_config(&profile.provider_id, &model_id)
                        .await
                        .ok_or_else(|| anyhow::anyhow!("TEAM_ROUTE_PROFILE_MISMATCH"))?;
                    let config = match &run_credentials {
                        Some(credentials) => {
                            credentials.config_for(&profile.provider_id, Some(&model_id), config)?
                        }
                        None => config,
                    };
                    fallback.api_key = config.api_key;
                    fallback.base_url = config.base_url;
                }
                Ok::<_, anyhow::Error>((profiles, pricing_models))
            }
            .await;

            // Team endpoint profiles can replace the selected model alias. Resolve
            // credentials only after that route is final so the policy decision,
            // bound client, and provider identity remain identical. A rejected team
            // profile remains its established public refusal; it does not trigger a
            // separate credential lookup.
            let primary_credential_binding = if team_profiles.is_ok() {
                Some(
                    resolve_credential_binding(
                        run_llm_config.clone(),
                        self.credential_policy.as_ref(),
                        self.provider_service.as_ref(),
                        principal_for_creds.as_ref(),
                        session_id_for_creds.as_deref(),
                        Some(artifact.id.as_str()),
                        run_credentials.is_some(),
                    )
                    .await,
                )
            } else {
                None
            };

            let model_budget = Ok::<_, anyhow::Error>(shared_model_budget.clone());

            let fallback_credential_decisions = async {
                let mut decisions = std::collections::BTreeMap::new();
                if !run_failover_config.enabled {
                    return Ok(decisions);
                }
                for fallback in &run_failover_config.fallback_models {
                    let fallback_config =
                        crate::llm::Orchestrator::fallback_llm_config(&run_llm_config, fallback);
                    let (fallback_provider, _) =
                        crate::llm::registry::split_model_string_pub(&fallback.model);
                    let fallback_config = match &run_credentials {
                        Some(credentials) => credentials
                            .config_for(&fallback_provider, Some(&fallback.model), fallback_config)
                            .map_err(anyhow::Error::from)?,
                        None => fallback_config,
                    };
                    let (_, decision) = resolve_credential_binding(
                        fallback_config,
                        self.credential_policy.as_ref(),
                        self.provider_service.as_ref(),
                        principal_for_creds.as_ref(),
                        session_id_for_creds.as_deref(),
                        Some(artifact.id.as_str()),
                        run_credentials.is_some(),
                    )
                    .await?;
                    decisions.insert(fallback.model.clone(), decision);
                }
                Ok::<_, anyhow::Error>(decisions)
            }
            .await;

            // Capture once before any summarization or model execution. Rebuilding
            // a client from the same config can resolve different environment or
            // provider credentials, and cannot serve as an inherited binding.
            match (
                model_budget,
                team_profiles,
                primary_credential_binding,
                fallback_credential_decisions,
            ) {
                (
                    Ok(model_budget),
                    Ok((profiles, pricing_models)),
                    Some(Ok((bound_config, primary_credential_decision))),
                    Ok(fallback_credential_decisions),
                ) => {
                    if self.primary_driver.is_some()
                        && self.credential_policy.is_shared()
                        && run_credentials.is_none()
                        && !is_team_attempt
                    {
                        Err(anyhow::anyhow!(
                            "shared credential policy cannot verify a supplied primary driver"
                        ))
                    } else {
                        crate::uar::runtime::turn::bindings::RunModelBindings::capture(
                            bound_config,
                            if run_credentials.is_some() || is_team_attempt {
                                None
                            } else {
                                self.primary_driver.clone()
                            },
                            primary_credential_decision,
                            fallback_credential_decisions,
                            run_failover_config,
                            if run_credentials.is_some() {
                                None
                            } else {
                                self.provider_registry
                                    .as_ref()
                                    .map(|registry| Arc::clone(registry.health()))
                            },
                            model_budget,
                            run_credentials.as_ref(),
                            profiles,
                            pricing_models,
                            &mut emitter.secret_scrubber,
                        )
                        .await
                    }
                }
                (Err(error), _, _, _) => Err(error),
                (_, Err(error), _, _) => Err(error),
                (_, _, Some(Err(error)), _) => Err(error),
                (_, _, _, Err(error)) => Err(error),
                (_, _, None, _) => Err(anyhow::anyhow!("credential binding unavailable")),
            }
        };
        let model_bindings = match model_bindings_result {
            Ok(bindings) => bindings,
            Err(error) => {
                // Constructor-owned/opaque credentials may not be captured.
                // Preserve only public contract codes at this early failure seam.
                // Model binding runs before normal admission publication. Its
                // error is reduced to a public stage/category below. Discard
                // private staged content and enable only this terminal failure;
                // otherwise the pre-admission guard replaces the diagnostic
                // reference with generic admission-failure text.
                emitter
                    .admission_projection
                    .lock()
                    .await
                    .begin_reduced_failure(emitter.secret_scrubber.clone());
                let raw_code = error.to_string();
                let team_error = match raw_code.as_str() {
                    "TEAM_PROFILE_UNSUPPORTED" => Some("TEAM_PROFILE_UNSUPPORTED"),
                    "TEAM_ROUTE_PROFILE_MISMATCH" => Some("TEAM_ROUTE_PROFILE_MISMATCH"),
                    "TEAM_REVISION_CONFLICT" => Some("TEAM_REVISION_CONFLICT"),
                    "TEAM_REASONING_UNSUPPORTED" => Some("TEAM_REASONING_UNSUPPORTED"),
                    _ => None,
                };
                let (stage, category) = run_binding_failure(&error);
                activation_context.lock().await.record_outcomes(false);
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                let code = if is_team_attempt {
                    team_error.unwrap_or("TEAM_PROVIDER_REQUEST_REJECTED")
                } else if stage == "budget_admission" {
                    category
                } else {
                    "orchestrator_start_failed"
                };
                let reference = uuid::Uuid::new_v4();
                tracing::error!(code, stage, category, diagnostic_reference = %reference,
                    "Run model capture refused");
                let message = if is_team_attempt {
                    format!("{code}; diagnostic reference {reference}")
                } else {
                    format!("Run binding refused during {stage}; diagnostic reference {reference}")
                };
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: code.into(),
                        message: message.into(),
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

        // The retained artifact is execution authority, not a display copy.
        // Refuse this unsupported case instead of changing its revision/content.
        let executable_snapshot = serde_json::to_value(&agent_snapshot);
        let admission_error = match &executable_snapshot {
            Ok(snapshot) if emitter.secret_scrubber.contains_captured(snapshot) => {
                Some("SECRET_IN_EXECUTABLE_ARTIFACT")
            }
            Err(_) => Some("EXECUTABLE_ARTIFACT_UNAVAILABLE"),
            _ => None,
        };
        let staged_events = if let Some(code) = admission_error {
            Err(code)
        } else {
            emitter
                .admission_projection
                .lock()
                .await
                .complete(emitter.secret_scrubber.clone())
        };
        let staged_events = match staged_events {
            Ok(events) => events,
            Err(code) => {
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: code.into(),
                        message: "Protected admission content cannot be published".into(),
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
        world_state
            .bind_retained_session(retained_session.clone(), emitter.secret_scrubber.clone());
        let world_state = Arc::new(world_state);
        let mut projected_history = session.messages();
        for message in &mut projected_history {
            emitter.secret_scrubber.project_message(message);
        }
        let projected_system = session
            .system_prompt()
            .map(|text| emitter.secret_scrubber.scrub(&text));
        session.clear();
        for message in &projected_history {
            session.add_message(message.clone());
            dialogue.0.add_message(message.clone());
        }
        if let Some(prompt) = projected_system {
            session.set_system_prompt(prompt.clone());
            dialogue.0.set_system_prompt(prompt);
        }
        // Preserve shared session identity and concurrent append authority. A
        // normal admission publishes only its seed-if-empty and new input;
        // authorized checkpoint history retains its existing replace semantics.
        if staged_replaces_history {
            retained_session.clear();
            for message in &projected_history {
                retained_session.add_message(message.clone());
            }
        } else {
            if let Some(mut seed) = staged_seed {
                for message in &mut seed {
                    emitter.secret_scrubber.project_message(message);
                }
                retained_session.seed_if_empty(&seed);
            }
            if append_input {
                retained_session.add_user_message(emitter.secret_scrubber.scrub(&input));
            }
        }
        emitter
            .secret_scrubber
            .project_fragments(&mut prompt_fragments);
        if let Some(state) = self.active_runs.write().await.get_mut(&run_id)
            && let Some(context) = state.run.context.as_object_mut()
        {
            context.insert("input".into(), emitter.secret_scrubber.scrub(&input).into());
            context.insert(
                "agent_snapshot".into(),
                executable_snapshot.expect("validated executable snapshot"),
            );
            if let Some(policy) = context
                .get_mut("effective_run_policy")
                .and_then(|policy| policy.as_object_mut())
            {
                policy.insert(
                    "warnings".into(),
                    emitter
                        .secret_scrubber
                        .project_value(serde_json::json!(&effective_policy.warnings)),
                );
            }
            context.insert(
                "activation_failures".into(),
                emitter
                    .secret_scrubber
                    .project_value(serde_json::json!(&attachment_failures)),
            );
        }
        for event in staged_events {
            emitter.emit(event).await;
        }

        // Capture before adding root-local handlers: retaining those handlers
        // here would form service -> kernel -> registry -> service ownership.
        let captured_native = Arc::new(native_skills.filtered(None).await);
        let delegation_lifetime = crate::uar::runtime::turn::bindings::RunDelegationLifetime(
            verified_owner.clone().filter(|_| !child_run).map(|owner| {
                Arc::new(crate::uar::runtime::turn::bindings::RunDelegationBindings {
                    secret_scrubber: emitter.secret_scrubber.clone(),
                    collaboration_binding: collaboration_binding.clone(),
                    instance_binding: instance_binding.clone(),
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
                    run_scoped_mcp: mcp_resources
                        .as_ref()
                        .filter(|resources| resources.run_scoped_names().is_some())
                        .map(|resources| resources.runtime().clone()),
                    run_mcp_grants: mcp_resources
                        .as_ref()
                        .and_then(|resources| resources.run_grants().cloned()),
                    service_binding: effective_service_binding.clone(),
                })
            }),
        );
        if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
            state.delegation = delegation_lifetime.0.as_ref().map(Arc::downgrade);
        }
        // The execution keeps the captured bindings alive, while the retained
        // producer owns the awaited release of its run-scoped MCP resources.
        let delegation_bindings = delegation_lifetime.0.clone();
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
                if effective_policy.tools.mode != SelectionMode::None {
                    let controls = crate::uar::runtime::native_skills::agents::registry_for_turn(
                        root_controls,
                    )
                    .await?;
                    for name in controls.names().await {
                        if let Some(handler) = controls.get(&name).await {
                            native_skills.register_arc(handler).await?;
                        }
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
        let qualified_model = if is_team_attempt {
            // Team profiles already capture the complete admitted provider/model.
            // Qualifying again would hide its administrator-configured capacity.
            run_llm_config.model.clone()
        } else {
            qualified_model_name(&run_llm_config)
        };
        let (catalog_provider, catalog_model_id) =
            crate::llm::registry::split_model_string_pub(&qualified_model);
        let configured_model = if let Some(registry) = &self.provider_registry {
            registry.models(&catalog_provider).await.and_then(|models| {
                models
                    .into_iter()
                    .find(|model| model.id == catalog_model_id)
            })
        } else {
            None
        };
        let configured_window = configured_model
            .as_ref()
            .and_then(|model| model.context_window)
            .map(|window| window as usize);
        let host_window = run_credentials.as_ref().and_then(|credentials| {
            credentials.context_window_for(&catalog_provider, &catalog_model_id)
        });
        let destination_budget_contract = self
            .destination_preparations
            .as_ref()
            .and_then(|preparations| preparations.budget_contract(&qualified_model))
            .cloned();
        let estimated_context_window = host_window
            .or(configured_window)
            .or_else(|| {
                crate::llm::catalog::ModelCatalog::global()
                    .model(&catalog_provider, &catalog_model_id)
                    .map(|model| model.limits.context_window as usize)
                    .filter(|window| *window > 0)
            })
            .or_else(|| {
                // Gateway aliases retain their admitted catalog identity separately
                // from the endpoint route. Explicit endpoint limits above still win;
                // catalog metadata does not certify the gateway's actual capacity.
                let identity = run_llm_config.catalog_pricing_model.as_deref()?;
                let (provider, model) = identity.split_once('/')?;
                crate::llm::catalog::ModelCatalog::global()
                    .model(provider, model)
                    .map(|model| model.limits.context_window as usize)
                    .filter(|window| *window > 0)
            });
        // Only a host-installed destination profile supplies an exact capacity.
        // The fallback remains a trimming estimate, never a fit guarantee.
        let model_context_window = destination_budget_contract
            .as_ref()
            .and_then(|contract| usize::try_from(contract.limits.context_tokens).ok())
            .or(estimated_context_window);
        let team_input_allowance = is_team_attempt
            .then(|| {
                host_usage_grant
                    .as_ref()
                    .filter(|grant| grant.accounting_id.starts_with("team-budget:"))
                    .and_then(|grant| grant.grant.max_total_tokens)
                    .and_then(|total| {
                        crate::uar::runtime::cost_budget::team_token_allocation(total).ok()
                    })
                    .and_then(|(input, _)| usize::try_from(input).ok())
            })
            .flatten();
        // Native/team callers can omit an explicit output cap. Bind the same
        // finite response reserve already withheld from their known context;
        // never infer an output allowance from the legacy unknown-window estimate.
        let bounded_output = (|| -> anyhow::Result<u32> {
            let capacity =
                model_context_window.ok_or_else(|| anyhow::anyhow!("output_capacity_unknown"))?;
            if is_team_attempt {
                anyhow::ensure!(
                    team_input_allowance.is_some(),
                    "team_input_allowance_unavailable"
                );
            }
            let mut cap = match output_token_cap {
                Some(cap) => cap,
                None => match &destination_budget_contract {
                    Some(contract) => u32::try_from(contract.limits.output_tokens)?,
                    None => crate::uar::runtime::context::reduce::RESPONSE_RESERVE_TOKENS as u32,
                },
            };
            if let Some(contract) = &destination_budget_contract {
                cap = cap.min(u32::try_from(contract.limits.output_tokens)?);
            }
            let maximum = configured_model
                .as_ref()
                .and_then(|model| model.max_output_tokens)
                .or_else(|| {
                    crate::llm::catalog::ModelCatalog::global()
                        .model(&catalog_provider, &catalog_model_id)
                        .and_then(|model| u32::try_from(model.limits.max_output).ok())
                        .filter(|limit| *limit > 0)
                });
            if let Some(maximum) = maximum {
                cap = cap.min(maximum);
            }
            if is_team_attempt {
                anyhow::ensure!(
                    u64::from(cap) <= crate::uar::runtime::cost_budget::TEAM_MAX_OUTPUT_TOKENS,
                    "team output token cap exceeds its host allocation"
                );
            }
            anyhow::ensure!(
                cap > 0 && capacity > cap as usize,
                "output_capacity_insufficient"
            );
            Ok(cap)
        })();
        let output_token_cap = match bounded_output {
            Ok(cap) => Some(cap),
            Err(_) => {
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                activation_context.lock().await.record_outcomes(false);
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: run_id.clone(),
                        code: "output_budget_unavailable".into(),
                        message:
                            "Selected model has no known capacity for a finite output reservation"
                                .into(),
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
            &qualified_model,
            model_context_window,
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
        if let Some(restored_history) = checkpoint_history.or(inherited_history) {
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
        for message in &mut messages {
            emitter.secret_scrubber.project_message(message);
        }
        emitter
            .secret_scrubber
            .project_fragments(&mut prompt_fragments);
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
        let (mut effective_strategy, context_model) = {
            let effective_context_tokens =
                model_context_window.map(|window| (window as f64 * 0.7) as u32);
            (
                crate::uar::context::resolve_effective_strategy(
                    &effective_policy.context_strategy,
                    effective_context_tokens,
                ),
                qualified_model.clone(),
            )
        };
        let summary_binding_result = async {
            use crate::uar::runtime::context::budget::AuxiliaryRequestAllowance;
            let requested = match &effective_strategy {
                crate::uar::context::ContextStrategy::Summarize { model, .. } => model.as_deref(),
                crate::uar::context::ContextStrategy::Hierarchical { .. } => None,
                _ => return Ok::<_, anyhow::Error>((None, None)),
            };
            let model = match requested {
                Some(requested) => model_bindings.auxiliary_model(requested)?,
                None => qualified_model.clone(),
            };
            let driver = model_bindings.auxiliary(&model)?;
            let exact = self
                .destination_preparations
                .as_ref()
                .and_then(|preparations| preparations.budget_contract(&model))
                .cloned();
            let (provider, model_id) = crate::llm::registry::split_model_string_pub(&model);
            let configured = if let Some(registry) = &self.provider_registry {
                registry
                    .models(&provider)
                    .await
                    .and_then(|models| {
                        models
                            .into_iter()
                            .find(|entry| entry.id == model_id)
                            .and_then(|entry| entry.context_window)
                    })
                    .map(|window| window as usize)
            } else {
                None
            };
            let capacity = run_credentials
                .as_ref()
                .and_then(|credentials| credentials.context_window_for(&provider, &model_id))
                .or(configured)
                .or_else(|| {
                    crate::llm::catalog::ModelCatalog::global()
                        .model(&provider, &model_id)
                        .map(|entry| entry.limits.context_window as usize)
                        .filter(|window| *window > 0)
                });
            let allowance = match exact {
                Some(contract) => AuxiliaryRequestAllowance::Exact(contract),
                None => {
                    let capacity =
                        capacity.ok_or_else(|| anyhow::anyhow!("auxiliary_capacity_unknown"))?;
                    anyhow::ensure!(capacity > 4352, "auxiliary_capacity_insufficient");
                    AuxiliaryRequestAllowance::Estimated {
                        destination_model: model.clone(),
                        input_allowance: capacity - 4096,
                        output_ceiling: 4096,
                        reserve_tokens: 256,
                        counting_revision: "host-plaintext-estimate-v1".into(),
                    }
                }
            };
            if let crate::uar::context::ContextStrategy::Summarize {
                model: selected, ..
            } = &mut effective_strategy
            {
                *selected = Some(model);
            }
            Ok((Some(driver), Some(allowance)))
        }
        .await;
        let (summarization_driver, summary_auxiliary_allowance) = match summary_binding_result {
            Ok(binding) => binding,
            Err(_) => {
                if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                    state.run.status = RunStatus::Error;
                }
                activation_context.lock().await.record_outcomes(false);
                emitter.emit(NormalizedEvent::Error { run_id: run_id.clone(), code: "summary_binding_unavailable".into(),
                    message: "Selected summary model has no permitted captured binding or known capacity".into() }).await;
                emitter
                    .emit(NormalizedEvent::RunDone {
                        run_id: run_id.clone(),
                    })
                    .await;
                self.run_cancellations.write().await.remove(&run_id);
                return run_id;
            }
        };
        // One reduction path: structural trimming, then the token budget, then
        // tool-call normalization, with the system message pinned throughout
        // (`uar::runtime::context::reduce`). The operator-declared strategy
        // drives both stages, so a run reduces once against one tokenizer.
        let context_limit = model_context_window.unwrap_or(8_192);
        let context_input_allowance = match &destination_budget_contract {
            Some(contract) => {
                crate::uar::runtime::context::budget::input_allowance_for_limits(&contract.limits)
                    .ok()
                    .and_then(|allowance| usize::try_from(allowance).ok())
                    .unwrap_or(0)
            }
            None => team_input_allowance
                .unwrap_or_else(|| {
                    context_limit.saturating_sub(output_token_cap.unwrap_or(0) as usize)
                })
                .min(context_limit.saturating_sub(output_token_cap.unwrap_or(0) as usize)),
        };
        let mut world_contributor = world_state.contributor(plan.restore_checkpoint).await;
        let world_reserved_tokens = match world_contributor
            .reserved_tokens(&messages, &context_model)
        {
            Ok(tokens) if tokens < context_input_allowance => tokens,
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
        // Tool definitions ride on every model call, so their serialized cost
        // is withheld from the history allowance like world state and the
        // response reserve. The tool set is resolved before any reduction.
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
        let tool_reserved_tokens = {
            let (mut definitions, has_deferred) = {
                let exposure =
                    crate::mcp::exposure::McpToolExposure::default().project(&authorized_tools);
                (
                    exposure
                        .visible()
                        .values()
                        .map(|tool| tool.openai_tool_json())
                        .collect::<Vec<_>>(),
                    exposure.has_deferred(),
                )
            };
            // The orchestrator adds the `search_tools` control whenever deferred
            // tools exist, so the first request carries it: count it here too.
            if has_deferred {
                match crate::uar::runtime::context::reduce::search_tools_definition().await {
                    Some(search) => definitions.push(search),
                    None => tracing::warn!(
                        "search_tools control could not be sized at admission; per-step counting applies"
                    ),
                }
            }
            crate::uar::runtime::context::reduce::tool_definition_tokens(
                &context_model,
                &definitions,
            )
        };
        let preparation_budget_receipt = serde_json::json!({
            "schema_version":"uar.request-budget.v1", "representation":"protected_preparation_candidate",
            "serialized_request":false, "count_quality":"approximate", "counting_revision":"model-message-and-tool-estimate-v1",
            "counted_input_tokens":crate::uar::runtime::context::token_service::TokenService::count_messages(&context_model, &messages)
                .saturating_add(tool_reserved_tokens).saturating_add(world_reserved_tokens),
            "input_allowance":context_input_allowance,
            "reserved_output_tokens":destination_budget_contract.as_ref().map(|contract| contract.limits.output_tokens)
                .unwrap_or(i64::from(output_token_cap.unwrap_or(0))),
            "tools_included":true, "instructions_included":true, "fit_guarantee":false,
        });
        let history_context_limit =
            match crate::uar::runtime::context::reduce::reserve_fixed_input_tokens(
                context_input_allowance,
                world_reserved_tokens,
                tool_reserved_tokens,
            ) {
                Ok(limit) => limit,
                Err(error) => {
                    emitter
                        .record_budget_refusal(error.code(), &preparation_budget_receipt)
                        .await;
                    if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    activation_context.lock().await.record_outcomes(false);
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: error.code().into(),
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
        // The window less the admission-time tool cost, for first-turn skill
        // reattachment and the graph request driver. The orchestrator's own
        // steps use the unreduced window and count their actual tools.
        // Per-step estimated reducers withhold the standard response reserve.
        // Withhold any larger explicit caller cap here as well; this does not
        // turn the model-window estimate into an exact final-wire guarantee.
        let response_aware_context_limit = if destination_budget_contract.is_none() {
            if let Some(input_allowance) = team_input_allowance {
                context_input_allowance.min(input_allowance)
            } else {
                context_limit.saturating_sub(
                    (output_token_cap.unwrap_or(0) as usize).saturating_sub(
                        crate::uar::runtime::context::reduce::RESPONSE_RESERVE_TOKENS,
                    ),
                )
            }
        } else {
            context_limit
        };
        let request_context_limit =
            response_aware_context_limit.saturating_sub(tool_reserved_tokens);
        let legacy_eligible_prose =
            crate::uar::runtime::context::reduce::canonical_prose_spans(&messages);
        let typed_eligible_prose =
            crate::uar::runtime::context::reduce::canonical_prose_spans(&unrendered_history);
        let legacy_reduction = if harness_config.mode != crate::config::HarnessMode::Typed {
            match crate::uar::runtime::context::reduce::reduce_history_with_input_allowance_and_auxiliary(
                messages,
                &effective_strategy,
                &context_model,
                history_context_limit,
                summarization_driver.as_deref(),
                summary_auxiliary_allowance.as_ref(),
                &legacy_eligible_prose,
            )
            .await
            {
                Ok(reduction) => Some(reduction),
                Err(error) => {
                    emitter.record_budget_refusal(error.code(), &preparation_budget_receipt).await;
                    if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    activation_context.lock().await.record_outcomes(false);
                    let message = crate::uar::runtime::context::reduce::liability_message(
                        &error,
                        tool_reserved_tokens.saturating_add(world_reserved_tokens),
                    );
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: error.code().into(),
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
            }
        } else {
            None
        };
        if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
            state.actor_spawn_descriptor = authorized_tools.get("spawn_agent").cloned();
        }
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
                history: unrendered_history.clone(),
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
            let mut registry = crate::uar::runtime::turn::builtin::registry_with_tool_tokens(
                crate::uar::runtime::turn::builtin::ContextStage {
                    model: context_model.clone(),
                    context_limit,
                    input_allowance: context_input_allowance,
                    strategy: effective_strategy.clone(),
                    reserved_tokens: world_reserved_tokens.saturating_add(tool_reserved_tokens),
                    options: render_options,
                    skill_budget: harness_config.skill_reattachment,
                    driver: summarization_driver.clone(),
                    summary_budget_contract: None,
                    summary_auxiliary_allowance: summary_auxiliary_allowance.clone(),
                    eligible_prose: typed_eligible_prose.clone(),
                },
                tool_reserved_tokens,
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
                    // A context-stage reduction failure keeps its stable code
                    // (context_overflow, ...) and liability text.
                    let (code, message) = match &error {
                        crate::uar::runtime::turn::contributors::AssemblyError::ContributorFailed {
                            name,
                            message,
                        } if name.as_str() == "context" => {
                            crate::uar::runtime::context::reduce::split_assembly_failure(message)
                                .map_or_else(
                                    || ("turn_assembly_rejected", error.to_string()),
                                    |(code, rest)| (code, rest.to_owned()),
                                )
                        }
                        _ => ("turn_assembly_rejected", error.to_string()),
                    };
                    emitter
                        .record_budget_refusal(code, &preparation_budget_receipt)
                        .await;
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: run_id.clone(),
                            code: code.into(),
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
            }
        } else {
            None
        };
        let (mut messages, reduce_report, world_update) =
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
        if let Some(state) = self.active_runs.write().await.get_mut(&run_id)
            && let Some(context) = state.run.context.as_object_mut()
        {
            context.insert(
                "activated_skills".into(),
                serde_json::json!(
                    active_for_manifest
                        .iter()
                        .map(|entry| entry.skill.skill_id.clone())
                        .collect::<Vec<_>>()
                ),
            );
            if let Some(receipt) = &reduce_report.structural_receipt {
                context.insert("context_reduction".into(), receipt.clone());
                if harness_config.mode == crate::config::HarnessMode::Typed
                    && let Some(attachment) = receipt.get("skill_reattachment")
                {
                    context.insert("skill_reattachment".into(), attachment.clone());
                }
            }
        }
        if let Some(update) = &world_update {
            world_state.commit(update).await;
        }
        for message in &mut messages {
            emitter.secret_scrubber.project_message(message);
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
            let (_, bounded_skill_fragments, receipt) =
                crate::uar::runtime::skills::retention::reattach_skills_with_receipt(
                    &messages,
                    &active_for_manifest,
                    &context_model,
                    request_context_limit,
                    harness_config.skill_reattachment,
                    RenderOptions {
                        prefers_xml_envelope: prompt_dialect.prefers_xml_envelope(),
                        markdown_averse: prompt_dialect.markdown_averse(),
                    },
                );
            prompt_fragments.extend(bounded_skill_fragments);
            if let Some(state) = self.active_runs.write().await.get_mut(&run_id)
                && let Some(context) = state.run.context.as_object_mut()
            {
                context.insert("skill_reattachment".into(), serde_json::json!(receipt));
            }
        }
        if let Some(binding) = collaboration_binding
            .as_ref()
            .filter(|b| b.team_attempt.is_some())
        {
            let mut member_guidance = Vec::new();
            for fragment in &mut prompt_fragments {
                if fragment.id == "agent.identity" {
                    let mut specialization = fragment.clone();
                    specialization.section = PromptSection::HostInstructions;
                    specialization.id = "10.member.specialization".into();
                    member_guidance.push(specialization);
                    *fragment = PromptFragment::new(
                        "agent.identity",
                        PromptSection::AgentIdentity,
                        format!("artifact:{}", artifact.id),
                        Authority::System,
                        PromptRole::System,
                        Retention::Session,
                        format!("Team member identity: {}", artifact.id),
                    );
                } else if fragment.id.starts_with("host.instruction.") {
                    fragment.section = PromptSection::HostInstructions;
                    fragment.id = format!("10.member.{}", fragment.id);
                }
            }
            prompt_fragments.extend(member_guidance);
            if let Some(guidance) = &binding.team_instructions {
                prompt_fragments.push(PromptFragment::new(
                    "00.team.instructions",
                    PromptSection::HostInstructions,
                    format!("team-guidance:{}:{}", guidance.revision, guidance.digest),
                    Authority::Host,
                    PromptRole::System,
                    Retention::Turn,
                    guidance.text.clone(),
                ));
            }
        }
        if let Some(context) = &representation_context {
            let disclosure = context.disclosure();
            prompt_fragments.push(PromptFragment::new(
                "00.representation.disclosure", PromptSection::HostInstructions,
                "representation-grants", Authority::Host, PromptRole::System,
                Retention::Turn, format!("{disclosure}\nInclude this exact disclosure in any team message. Office names grant no powers; no subdelegation is permitted."),
            ));
        }
        emitter
            .secret_scrubber
            .project_fragments(&mut prompt_fragments);
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
            effective_policy
                .warnings
                .iter()
                .map(|warning| emitter.secret_scrubber.scrub(warning)),
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
            .with_presentations(Arc::clone(&presentation_snapshot))
            .with_client_tools(client_tools.clone())
            .with_output_token_cap(output_token_cap)
            .with_response_format(response_format.clone()),
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
                        .with_presentations(Arc::clone(&presentation_snapshot))
                        .with_client_tools(client_tools.clone())
                        .with_output_token_cap(output_token_cap)
                        .with_response_format(response_format.clone()),
                    ),
                    assembled.history.clone(),
                )
            })
        } else {
            None
        };
        let primary_driver_for_graph: Arc<dyn LlmDriver> = Arc::new(DestinationBudgetDriver {
            inner: model_bindings.primary(),
            contract: destination_budget_contract.clone(),
            output_token_cap: output_token_cap.unwrap_or(0),
        });

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
                .with_resilience_policy(turn_resilience_policy)
                .with_tool_admission(Arc::clone(&tool_admission))
                .with_resolved_turn(Arc::clone(&resolved_turn))
                .with_client_tool_budget(Arc::new(model_bindings.budget().clone()))
                .with_canonical_receipt_store(self.persistence.clone())
                .with_secret_scrubber(emitter.secret_scrubber.clone())
                .with_team_model_handoff(
                    collaboration_binding
                        .clone()
                        .filter(|b| b.team_attempt.is_some()),
                )
                .with_world_state(Arc::clone(&world_state))
                .with_skill_activation(
                    Arc::clone(&activation_context),
                    effective_strategy.clone(),
                    context_model.clone(),
                    // No tool cost here: the orchestrator withholds each step's
                    // actual tool definitions itself.
                    response_aware_context_limit,
                    harness_config.skill_reattachment,
                )
                .with_cache_strategy(
                    effective_policy
                        .prompt_caching_enabled
                        .then(crate::llm::anthropic_cache::CacheStrategy::default),
                );
            let o = match &self.destination_preparations {
                Some(preparations) => o.with_destination_preparations(
                    Arc::clone(preparations),
                    unrendered_history.clone(),
                    prompt_fragments.clone(),
                ),
                None => o,
            };
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
            let approval_gate_channel = approval_channel.clone();
            let recovery_gate_record = recovered_approval.clone();
            let gate: crate::llm::ToolApprovalGate = Arc::new(move |invocation| {
                let run_id = approval_run_id.clone();
                let emitter = approval_emitter.clone();
                let channel = approval_gate_channel.clone();
                let cancellation = approval_cancellation.clone();
                let agent_id = approval_agent_id.clone();
                let governance = approval_governance.clone();
                let governance_gate = approval_governance_gate.clone();
                let recovered = recovery_gate_record.clone().filter(|record| {
                    record.tool_call_id == invocation.invocation.model_tool_call_id
                });
                Box::pin(async move {
                    let checkpoint_request = invocation.clone();
                    let host_requires_approval = invocation.host_requires_approval;
                    let invocation = invocation.invocation;
                    let admission_owner = invocation.admission_owner;
                    // Native effects retain their admission owner; a captured
                    // standalone binding resolves human decisions inside UAR.
                    let decision_owner = if admission_owner
                        == crate::uar::persistence::tool_admission::AdmissionOwner::UarRuntime
                        || invocation.host_epoch
                            == format!("standalone:{}", invocation.runtime_epoch)
                    {
                        crate::uar::persistence::tool_admission::AdmissionOwner::UarRuntime
                    } else {
                        crate::uar::persistence::tool_admission::AdmissionOwner::PairedHost
                    };
                    let tool_call_id = invocation.model_tool_call_id.clone();
                    let tool_name = invocation.provider_tool_name.clone();
                    let approval_class = invocation.approval_class;
                    let call_index = invocation.call_index;
                    if cancellation.is_cancelled() {
                        return crate::llm::ToolApprovalResult::Cancelled {
                            reason: "Tool call cancelled with its run".to_string(),
                        };
                    }
                    // A host may bypass automatic governance for a verified
                    // local root, but explicit Ask/Deny remains authoritative.
                    if !host_requires_approval
                        && recovered.is_none()
                        && !child_run
                        && approval_class != crate::uar::tools::descriptor::ApprovalClass::Required
                        && effective_tool_approval == ToolApprovalPolicy::Auto
                        && let Some(decision) = governance_bypass_decision(governance_gate.as_ref())
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
                        if let Some(record) = &recovered
                            && channel.invalidate_recovered(record).await.is_err()
                        {
                            emitter.emit(NormalizedEvent::Error { run_id: run_id.clone(), code: "approval_persistence_unavailable".into(),
                                message: "Recovered approval invalidation could not be persisted; continuation claim remains held".into() }).await;
                            return crate::llm::ToolApprovalResult::Cancelled {
                                reason: "Approval persistence unavailable".into(),
                            };
                        }
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
                    let approval_required = recovered.is_some()
                        || host_requires_approval
                        || effective_tool_approval == ToolApprovalPolicy::Ask
                        || approval_class == crate::uar::tools::descriptor::ApprovalClass::Required;
                    let decision = match &governance {
                        Some(engine) => {
                            engine
                                .tool_decision(&agent_id, &tool_name, approval_required)
                                .await
                        }
                        None if approval_required => {
                            crate::uar::governance::engine::ToolGovernanceDecision::RequireApproval
                        }
                        None => crate::uar::governance::engine::ToolGovernanceDecision::Allow,
                    };
                    match decision {
                        crate::uar::governance::engine::ToolGovernanceDecision::Allow
                            if !approval_required =>
                        {
                            return crate::llm::ToolApprovalResult::Allowed;
                        }
                        crate::uar::governance::engine::ToolGovernanceDecision::Deny => {
                            if let Some(record) = &recovered
                                && channel.invalidate_recovered(record).await.is_err()
                            {
                                emitter.emit(NormalizedEvent::Error { run_id: run_id.clone(), code: "approval_persistence_unavailable".into(),
                                    message: "Recovered approval invalidation could not be persisted; continuation claim remains held".into() }).await;
                                return crate::llm::ToolApprovalResult::Cancelled {
                                    reason: "Approval persistence unavailable".into(),
                                };
                            }
                            let reason =
                                format!("Tool '{tool_name}' is denied by governance policy");
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
                        crate::uar::governance::engine::ToolGovernanceDecision::Allow
                        | crate::uar::governance::engine::ToolGovernanceDecision::RequireApproval =>
                            {}
                    }
                    let risk_reason = if host_requires_approval {
                        format!("Tool '{tool_name}' requires approval under the paired host policy")
                    } else if effective_tool_approval == ToolApprovalPolicy::Ask {
                        format!(
                            "Tool '{tool_name}' requires approval under the effective run policy"
                        )
                    } else {
                        format!("Tool '{tool_name}' requires approval under its descriptor")
                    };
                    let outcome = if let Some(record) = &recovered {
                        match channel.approve_recovered(record, &invocation).await {
                            Ok(true) => ApprovalOutcome::Approved,
                            Ok(false) => ApprovalOutcome::Rejected,
                            Err(_) => ApprovalOutcome::ChannelClosed,
                        }
                    } else {
                        channel
                            .request_tool(
                                checkpoint_request,
                                decision_owner,
                                risk_reason,
                                &cancellation,
                            )
                            .await
                    };
                    match outcome {
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
                        ApprovalOutcome::Cancelled => crate::llm::ToolApprovalResult::Cancelled {
                            reason: "Approval cancelled with its run".to_string(),
                        },
                    }
                })
            });
            let tool_budget = model_bindings.budget().clone();
            let budget_emitter = emitter.clone();
            let budget_run_id = run_id.clone();
            let budget_gate: crate::llm::ToolApprovalGate = Arc::new(move |invocation| {
                let gate = Arc::clone(&gate);
                let budget = tool_budget.clone();
                let emitter = budget_emitter.clone();
                let run_id = budget_run_id.clone();
                Box::pin(async move {
                    let tool_call_id = invocation.invocation.model_tool_call_id.clone();
                    let tool_name = invocation.invocation.provider_tool_name.clone();
                    let call_index = invocation.invocation.call_index;
                    let decision = gate(invocation).await;
                    // Both Approved and GovernanceBypassed remain
                    // subject to the same host-owned root allowance.
                    if !matches!(
                        &decision,
                        crate::llm::ToolApprovalResult::Rejected { .. }
                            | crate::llm::ToolApprovalResult::Cancelled { .. }
                    ) && let Err(error) = budget.admit_tool_durable().await
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
            });
            if let Some(state) = self.active_runs.write().await.get_mut(&run_id) {
                state.actor_tool_approval = Some(ActorToolApprovalGate(Arc::clone(&budget_gate)));
            }
            let graph_thread_delegate = graph_controls
                .zip(authorized_tools.get("spawn_agent").cloned())
                .map(|(controls, descriptor)| {
                    let mut delegate =
                        crate::uar::runtime::graph::delegation::GraphThreadDelegate::new(
                            run_id.clone(),
                            controls,
                            Arc::clone(&budget_gate),
                            Arc::clone(&tool_admission),
                            descriptor,
                        )
                        .with_agent_routing(
                            self.clone(),
                            artifact.id.clone(),
                            Arc::new(emitter.clone()),
                        );
                    if let (Some(input), Some(store), Some(owner)) = (
                        client_tools
                            .as_ref()
                            .filter(|input| !input.tools.is_empty()),
                        self.persistence.as_ref(),
                        verified_owner.as_ref(),
                    ) {
                        let capture =
                            crate::uar::runtime::turn::client_tools::GraphCallerTurnCapture::new(
                                input.clone(),
                                artifact.id.clone(),
                                run_id.clone(),
                                session.id().to_owned(),
                                serde_json::Value::Null,
                            );
                        delegate = delegate.with_caller_tools(
                            capture,
                            self.clone(),
                            Arc::clone(store),
                            owner.clone(),
                            Arc::new(emitter.clone()),
                        );
                    }
                    Arc::new(delegate)
                });
            (
                Arc::new(o.with_tool_approval_gate(budget_gate)),
                graph_thread_delegate,
            )
        };

        let execute_run_id = run_id.clone();
        let execute_agent_id = artifact.id.clone();
        let emitter = emitter.clone();
        let execution_session = retained_session.clone();
        let skill_service_for_evolution = self.skill_service.clone();
        let skill_evolution_cfg = self.skill_evolution_config.clone();
        let lifecycle_for_evolution = self.lifecycle_scope.clone();
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
        let canonical_graph_history = self
            .destination_preparations
            .is_some()
            .then(|| unrendered_history.clone());

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
                    usage: None,
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
        let finalizer_checkpoint_cleanup = Arc::clone(&self.checkpoint_cleanup_failed);
        let checkpoint_actor_root = actor_root.clone();
        let checkpoint_source = {
            let mut runs = self.active_runs.write().await;
            runs.get_mut(&run_id).map(|state| {
                state.run.context["checkpoint_execution"] = serde_json::json!(
                    crate::uar::persistence::checkpoint_resume::CheckpointExecutionDefinition {
                        schema_version: 1,
                        kind: if graph_for_run.is_some() {
                            crate::uar::persistence::checkpoint_resume::CheckpointExecutionKind::HostGraph
                        } else if self.primary_driver.is_some() {
                            crate::uar::persistence::checkpoint_resume::CheckpointExecutionKind::HostDriver
                        } else if checkpoint_has_host_resources {
                            crate::uar::persistence::checkpoint_resume::CheckpointExecutionKind::HostResources
                        } else if checkpoint_a2a_renewable {
                            crate::uar::persistence::checkpoint_resume::CheckpointExecutionKind::A2aToolLoop
                        } else {
                            crate::uar::persistence::checkpoint_resume::CheckpointExecutionKind::ToolLoop
                        },
                    });
                if let Some(contract) = &a2a_delegation {
                    state.run.context["a2a_delegation"] = serde_json::json!(contract);
                }
                state.run.clone()
            })
        };
        let checkpoint_host = checkpoint_source.map(|source| {
            Arc::new(crate::uar::runtime::checkpoint_host::CheckpointHost::new(
                self.persistence.clone(),
                source,
                verified_owner.as_ref(),
                self.tool_runtime_epoch.clone(),
                crate::uar::runtime::checkpoint::authorization_digest(&effective_policy),
                run_cancellation.clone(),
                emitter.secret_scrubber.clone(),
                collaboration_binding.clone(),
            ))
        });
        if let Some(host) = &checkpoint_host {
            approval_channel.install_recovery_host(
                Arc::clone(host),
                self.root_cancellation.clone(),
                Arc::clone(&tool_admission),
            );
        }
        let actor_producer = actor_root.as_ref().map(|root| Arc::clone(&root.producer));
        if let Some(root) = &actor_root {
            root.ready.store(true, std::sync::atomic::Ordering::Release);
        }
        let representation_disclosure = representation_context
            .as_ref()
            .map(|context| context.disclosure());
        let representation_binding = collaboration_binding
            .clone()
            .filter(|binding| binding.has_representation());
        // The admitted producer captures the full tool loop, checkpoint host,
        // and finalizers. Keep both layers heap-indirected before handing them
        // to Tokio so a real native-tool turn does not build that state on the
        // caller's task stack.
        let execution = Box::pin(async move {
            let _delegation_bindings = delegation_bindings;
            let _sandbox_lease = sandbox_lease;
            let _terminal_lease = terminal_lease;
            if let Some(binding) = representation_binding {
                if binding.representation_context().await.is_err() {
                    run_cancellation.cancel();
                }
            }
            // 1. Run Start
            emitter
                .emit(NormalizedEvent::RunStart {
                    run_id: execute_run_id.clone(),
                    agent_id: execute_agent_id,
                })
                .await;

            if let Some(disclosure) = representation_disclosure {
                emitter
                    .emit(NormalizedEvent::ChatDelta {
                        run_id: execute_run_id.clone(),
                        text_delta: format!("{disclosure}\n\n"),
                    })
                    .await;
            }

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
                        context_limit: request_context_limit,
                        budget: skill_reattachment_budget,
                        cost_tracking: cost_tracking_enabled,
                    });
                let checkpoint_authorization_digest =
                    crate::uar::runtime::checkpoint::authorization_digest(&effective_policy);
                let graph_ctx = crate::uar::runtime::graph::GraphContext {
                    run_id: execute_run_id.clone(),
                    checkpoint_authorization_digest: checkpoint_authorization_digest.clone(),
                    session_id: Some(execution_session.id().to_string()),
                    llm_config: llm_config_for_graph,
                    driver: graph_driver,
                    cache_strategy: cache_strategy_for_graph,
                    thread_delegate: graph_thread_delegate,
                    tool_host: graph_tool_host.clone(),
                };

                let mut initial_state = client_tools
                    .as_ref()
                    .and_then(|input| input.graph_state.clone())
                    .and_then(|value| serde_json::from_value(value).ok())
                    .or(restored_state)
                    .unwrap_or_default();
                initial_state.set(
                    crate::uar::runtime::checkpoint::CHECKPOINT_AUTHORIZATION_DIGEST_KEY,
                    checkpoint_authorization_digest,
                );
                // Preserve the checkpoint's data bag and iteration while using
                // the fully assembled, normalized history for the resumed call.
                initial_state.messages = canonical_graph_history
                    .as_ref()
                    .unwrap_or(&messages)
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
                        if approval_channel.finish_cancelled_root(&execute_run_id).await.is_err() {
                            cleanup_failed = true;
                            emitter.emit(NormalizedEvent::Error {
                                run_id: execute_run_id.clone(), code: "approval_cleanup_unconfirmed".into(),
                                message: "Cancelled approval state could not be persisted".into(),
                            }).await;
                        }
                        if let Some(state) = runs_for_completion.write().await.get_mut(&execute_run_id) {
                            state.run.status = if cleanup_failed { RunStatus::Error } else { RunStatus::Cancelled };
                        }
                        if cleanup_failed {
                            emitter.emit(NormalizedEvent::RunDone { run_id: execute_run_id.clone() }).await;
                        } else {
                            emitter.emit(NormalizedEvent::Cancelled { run_id: execute_run_id.clone(), usage: None }).await;
                        }
                        cancellations_for_cleanup.write().await.remove(&cleanup_run_id);
                        return;
                    }
                    state = async {
                        match &checkpoint_host {
                            Some(host) => graph.execute_with_checkpoint_host(initial_state, &graph_ctx, &emitter, host).await,
                            None => {
                                let mut state = initial_state;
                                state.set("_error", "checkpoint_source_unavailable");
                                state
                            }
                        }
                    } => state,
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
                } else if final_state.get::<bool>("_client_tool_handoff") != Some(true)
                    && let Some(route) = final_state.get::<String>("_route")
                {
                    let output_key = if final_state
                        .get::<String>("_agent_roster_revision")
                        .is_some()
                    {
                        "_agent_output_selected-agent".to_string()
                    } else {
                        format!("_agent_output_{route}")
                    };
                    match final_state.get::<String>(&output_key) {
                        Some(output) => {
                            let attributed_output = emitter
                                .secret_scrubber
                                .scrub(&format!("[{route}]\n\n{output}"));
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
            let mut orchestrator_completed = false;
            let mut accumulated_tool_calls: Vec<crate::llm::ToolCall> = Vec::new();
            let mut tool_call_indices: HashMap<String, usize> = HashMap::new();
            let mut tool_call_names: HashMap<String, String> = HashMap::new();
            let mut partial_tool_calls: std::collections::BTreeMap<
                usize,
                (Option<String>, Option<String>, String),
            > = std::collections::BTreeMap::new();

            // Token usage tracking — accumulated across all LLM calls in this run.
            let mut total_input_tokens: u32 = 0;
            let mut total_output_tokens: u32 = 0;
            let mut total_cache_read_tokens: u32 = 0;

            // 2. Execute Orchestrator
            #[cfg(feature = "bauar-native-admission-gate")]
            let native_admission_gate = orchestrator.native_admission_gate();
            let orchestrator = if recovered_approval.is_some() {
                let completed = pending_history
                    .iter()
                    .filter_map(|message| message.tool_call_id.clone())
                    .collect::<BTreeSet<_>>();
                let pending_calls = pending_history
                    .iter()
                    .flat_map(|message| message.tool_calls.iter().flatten())
                    .cloned()
                    .enumerate()
                    .filter(|(_, call)| !completed.contains(&call.id))
                    .collect::<Vec<_>>();
                for message in &pending_history {
                    dialogue.record(&execution_session, |history| {
                        history.add_message(message.clone())
                    });
                }
                messages.extend(pending_history);
                for (index, call) in pending_calls {
                    emitter
                        .emit(NormalizedEvent::ToolStart {
                            run_id: execute_run_id.clone(),
                            call_index: index,
                            tool_call_id: call.id.clone(),
                            tool: call.function.name.clone(),
                            input: serde_json::from_str(&call.function.arguments)
                                .unwrap_or(serde_json::Value::Null),
                        })
                        .await;
                    match orchestrator
                        .execute_recovered_tool_call(&call, index, messages.clone())
                        .await
                    {
                        Ok((canonical, displayed, outcome)) => {
                            let message = Message {
                                role: MessageRole::Tool,
                                content: crate::llm::MessageContent::text(canonical.clone()),
                                tool_call_id: Some(call.id.clone()),
                                tool_calls: None,
                            };
                            dialogue.record(&execution_session, |history| {
                                history.add_message(message.clone())
                            });
                            messages.push(message);
                            tool_call_count += 1;
                            emitter
                                .emit(NormalizedEvent::ToolEnd {
                                    run_id: execute_run_id.clone(),
                                    call_index: index,
                                    tool_call_id: call.id,
                                    tool: call.function.name,
                                    output: serde_json::from_str(&displayed)
                                        .unwrap_or(serde_json::Value::String(displayed)),
                                    ok: outcome.succeeded(),
                                })
                                .await;
                            if outcome == crate::mcp::outcome::ToolOutcome::OutcomeUnknown {
                                run_failed = true;
                                emitter.emit(NormalizedEvent::Error { run_id: execute_run_id.clone(),
                                    code: "TOOL_OUTCOME_UNKNOWN".into(), message: "Recovered tool outcome is unknown; reconciliation required".into() }).await;
                                break;
                            }
                        }
                        Err(_) => {
                            run_failed = true;
                            let invalidation_unavailable = match &recovered_approval {
                                Some(record) => {
                                    approval_channel.invalidate_recovered(record).await.is_err()
                                }
                                None => false,
                            };
                            if invalidation_unavailable
                                && let Some(state) =
                                    runs_for_completion.write().await.get_mut(&execute_run_id)
                            {
                                state.run.context["error_code"] =
                                    "approval_persistence_unavailable".into();
                            }
                            emitter.emit(NormalizedEvent::Error { run_id: execute_run_id.clone(),
                                code: if invalidation_unavailable { "approval_persistence_unavailable" } else { "approval_recovery_admission_denied" }.into(),
                                message: if invalidation_unavailable { "Recovered approval invalidation could not be persisted; continuation claim remains held" }
                                    else { "Current admission did not authorize the recovered call" }.into() }).await;
                            break;
                        }
                    }
                }
                if !run_failed && let Some(host) = &checkpoint_host {
                    if host.has_resume_authority() {
                        let state = crate::uar::runtime::graph::GraphState {
                            data: HashMap::new(),
                            messages: execution_session
                                .messages()
                                .iter()
                                .map(|message| serde_json::json!(message))
                                .collect(),
                            iteration: tool_call_count as u32,
                        };
                        if host.persist("approval_recovered", &state).await.is_err() {
                            run_failed = true;
                            emitter
                                .emit(NormalizedEvent::Error {
                                    run_id: execute_run_id.clone(),
                                    code: "checkpoint_persistence_failed".into(),
                                    message:
                                        "Recovered tool result checkpoint could not be persisted"
                                            .into(),
                                })
                                .await;
                        }
                    } else {
                        tracing::debug!(run_id = %execute_run_id,
                            "Skipping approval-recovery checkpoint for anonymous run without resume authority");
                    }
                }
                Arc::new(
                    (*orchestrator)
                        .clone()
                        .with_recovered_history(messages.clone()),
                )
            } else {
                orchestrator
            };
            let recovered_stream = if run_failed {
                Err(anyhow::anyhow!(
                    "Approval recovery failed before model continuation"
                ))
            } else {
                orchestrator.chat_with_history(messages).await
            };
            match recovered_stream {
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
                        if let crate::normalized::NormalizedEvent::Custom {
                            source,
                            event_name,
                            payload,
                        } = &base_event
                            && source == "uar.request"
                            && event_name == "budget_receipt"
                        {
                            let mut receipt = payload.clone();
                            if payload
                                .get("destination_model")
                                .and_then(serde_json::Value::as_str)
                                == Some(run_model.as_str())
                            {
                                receipt["input_allowance"] =
                                    serde_json::json!(context_input_allowance);
                                if receipt
                                    .get("reserved_output_tokens")
                                    .is_none_or(serde_json::Value::is_null)
                                {
                                    receipt["reserved_output_tokens"] = preparation_budget_receipt
                                        ["reserved_output_tokens"]
                                        .clone();
                                }
                                let within = receipt
                                    .get("counted_input_tokens")
                                    .and_then(serde_json::Value::as_u64)
                                    .is_some_and(|tokens| tokens <= context_input_allowance as u64);
                                receipt["outcome"] = serde_json::json!(if within {
                                    "within_allowance"
                                } else {
                                    "exceeds_estimated_allowance"
                                });
                            }
                            if let Some(state) =
                                runs_for_completion.write().await.get_mut(&execute_run_id)
                                && let Some(context) = state.run.context.as_object_mut()
                            {
                                if let Some(receipts) = context
                                    .entry("request_budgets".to_owned())
                                    .or_insert_with(|| serde_json::json!([]))
                                    .as_array_mut()
                                {
                                    receipts.push(receipt.clone());
                                }
                                context.insert("request_budget".into(), receipt);
                            }
                        }
                        if let crate::normalized::NormalizedEvent::Custom {
                            source,
                            event_name,
                            payload,
                        } = &base_event
                            && source == "uar.request"
                            && event_name == "attempt_manifest"
                        {
                            if let Some(state) =
                                runs_for_completion.write().await.get_mut(&execute_run_id)
                                && let Some(context) = state.run.context.as_object_mut()
                            {
                                let manifests = context
                                    .entry("attempt_manifests".to_string())
                                    .or_insert_with(|| serde_json::json!([]));
                                if let Some(manifests) = manifests.as_array_mut() {
                                    manifests.push(payload.clone());
                                }
                            }
                        }
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
                            } if source == "uar.request" && event_name == "attempt_manifest" => {
                                Some(NormalizedEvent::Artifact {
                                    run_id: execute_run_id.clone(),
                                    artifact: ArtifactPayload {
                                        artifact_id: format!(
                                            "attempt-manifest-{}-{}",
                                            execute_run_id,
                                            payload
                                                .get("destination_model")
                                                .and_then(serde_json::Value::as_str)
                                                .unwrap_or("unknown")
                                        ),
                                        artifact_type: "attempt_manifest".to_string(),
                                        title: "Provider attempt manifest".to_string(),
                                        content: payload.to_string(),
                                        language: Some("json".to_string()),
                                        metadata: serde_json::json!({
                                            "source": source,
                                            "budgeting_label": payload
                                                .pointer("/budgeting/label")
                                                .and_then(serde_json::Value::as_str),
                                        }),
                                    },
                                })
                            }
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
                                let partial = partial_tool_calls.entry(call_index).or_default();
                                if let Some(tid) = &id {
                                    partial.0 = Some(tid.clone());
                                    tool_call_indices.insert(tid.clone(), call_index);
                                }
                                if let Some(tool_name) = &name {
                                    partial.1 = Some(tool_name.clone());
                                    if let Some(tid) = &partial.0 {
                                        tool_call_names.insert(tid.clone(), tool_name.clone());
                                    }
                                }
                                if let Some(delta) = &arguments_delta {
                                    partial.2.push_str(delta);
                                }
                                if let (Some(tid), Some(delta)) = (id, arguments_delta) {
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
                                partial_tool_calls.remove(&call_index);
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
                            crate::normalized::NormalizedEvent::ClientToolsRequested {
                                batch_id,
                                calls,
                            } => {
                                if let Some(state) =
                                    runs_for_completion.write().await.get_mut(&execute_run_id)
                                {
                                    state.run.context["pending_client_tool_batch_id"] =
                                        batch_id.clone().into();
                                    state.run.context["pending_client_tool_call_ids"] = serde_json::json!(
                                        calls.iter().map(|call| &call.id).collect::<Vec<_>>()
                                    );
                                }
                                Some(NormalizedEvent::ClientToolsRequested {
                                    run_id: execute_run_id.clone(),
                                    batch_id,
                                    calls,
                                })
                            }
                            crate::normalized::NormalizedEvent::Done => {
                                orchestrator_completed = true;
                                None
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

                                // The host awaits the atomic checkpoint/envelope write.
                                // Anonymous runs have no verified checkpoint
                                // principal: no envelope they produce could ever
                                // be resumed, so the write is skipped rather than
                                // treated as a durability failure.
                                let checkpoint_writable = persistence_for_run.is_some()
                                    && match &checkpoint_host {
                                        Some(host) => host.has_resume_authority(),
                                        None => true, // preserve the existing source-unavailable failure
                                    };
                                if checkpoint_writable {
                                    let checkpoint_state = crate::uar::runtime::graph::GraphState {
                                        data: std::collections::HashMap::new(),
                                        messages: execution_session
                                            .messages()
                                            .iter()
                                            .map(|message| {
                                                serde_json::to_value(message)
                                                    .expect("Message serialization is infallible")
                                            })
                                            .collect(),
                                        iteration: tool_call_count as u32,
                                    };
                                    let saved = match &checkpoint_host {
                                        Some(host) => {
                                            host.persist(
                                                &format!("tool_loop_{tool_call_count}"),
                                                &checkpoint_state,
                                            )
                                            .await
                                        }
                                        None => {
                                            Err(anyhow::anyhow!("checkpoint source unavailable"))
                                        }
                                    };
                                    if saved.is_err() {
                                        run_failed = true;
                                        emitter
                                            .emit(NormalizedEvent::Error {
                                                run_id: execute_run_id.clone(),
                                                code: "checkpoint_persistence_failed".into(),
                                                message: "Atomic checkpoint persistence failed"
                                                    .into(),
                                            })
                                            .await;
                                    }
                                } else {
                                    tracing::debug!(run_id = %execute_run_id,
                                        "Skipping tool-loop checkpoint for anonymous run without resume authority");
                                }
                            }
                            // Derive a memory mutation event before consuming the ToolEnd.
                            let mutation_evt = memory_mutation_from_tool_end(&evt, &execute_run_id);
                            emitter.emit(evt).await;
                            if let Some(m) = mutation_evt {
                                emitter.emit(m).await;
                            }
                            if run_failed {
                                break;
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

            if !run_cancelled && !run_failed && !orchestrator_completed {
                run_failed = true;
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: execute_run_id.clone(),
                        code: "orchestrator_completion_missing".into(),
                        message: "Orchestrator stream ended before completion".into(),
                    })
                    .await;
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

            if approval_channel
                .finish_cancelled_root(&execute_run_id)
                .await
                .is_err()
            {
                run_cancelled = false;
                run_failed = true;
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: execute_run_id.clone(),
                        code: "approval_cleanup_unconfirmed".into(),
                        message: "Cancelled approval state could not be persisted".into(),
                    })
                    .await;
            }

            let mut interrupted_fragment = None;
            for (_call_index, (id, name, arguments)) in partial_tool_calls {
                let (Some(id), Some(name)) = (id, name) else {
                    continue;
                };
                if accumulated_tool_calls.iter().any(|call| call.id == id) {
                    continue;
                }
                accumulated_tool_calls.push(crate::llm::ToolCall {
                    id,
                    call_type: "function".to_string(),
                    function: crate::llm::ToolCallFunction { name, arguments },
                });
            }
            let pending_tool_call_ids = accumulated_tool_calls
                .iter()
                .map(|call| call.id.clone())
                .collect::<Vec<_>>();
            if !accumulated_tool_calls.is_empty() {
                if !run_cancelled && !run_failed {
                    run_failed = true;
                    emitter
                        .emit(NormalizedEvent::Error {
                            run_id: execute_run_id.clone(),
                            code: "pending_tool_calls_unsettled".into(),
                            message: "Provider stream ended before pending tool calls reached a durable terminal result".into(),
                        })
                        .await;
                }
                if run_cancelled || run_failed {
                    interrupted_fragment = Some(
                        TurnInterrupted {
                            run_id: execute_run_id.clone(),
                            reason: if run_cancelled {
                                TurnInterruptionReason::Cancelled
                            } else {
                                TurnInterruptionReason::ProviderError
                            },
                        }
                        .into_fragment(),
                    );
                }
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
                if let Some(db) = &persistence_for_run
                    && let Err(error) = db.save_session(&execution_session).await
                {
                    tracing::warn!(
                        run_id = %execute_run_id,
                        error = %error,
                        "Failed to persist pending assistant tool turn"
                    );
                }
            } else if !accumulated_content.is_empty() {
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

            // A successful text-only turn must be durable before its terminal
            // event. Otherwise a restart silently loses both user and answer.
            if !run_cancelled
                && !run_failed
                && let Some(db) = &persistence_for_run
                && let Err(error) = db.save_session(&execution_session).await
            {
                run_failed = true;
                tracing::warn!(%error, run_id = %execute_run_id, "Failed to persist completed conversation");
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: execute_run_id.clone(),
                        code: "session_persist_failed".into(),
                        message: "Completed conversation could not be saved".into(),
                    })
                    .await;
            }

            let completed_active_skills = activation_context
                .lock()
                .await
                .active()
                .into_iter()
                .map(|entry| entry.skill.skill_id)
                .collect::<Vec<_>>();
            if let Some(state) = runs_for_completion.write().await.get_mut(&execute_run_id) {
                if let Some(context) = state.run.context.as_object_mut() {
                    context.insert(
                        "activated_skills".into(),
                        serde_json::json!(completed_active_skills),
                    );
                }
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
                if !pending_tool_call_ids.is_empty()
                    && let Some(context) = state.run.context.as_object_mut()
                {
                    context.insert(
                        "pending_tool_calls".to_string(),
                        serde_json::json!(pending_tool_call_ids),
                    );
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

            // The dispatch future has been dropped and ordinary cleanup has completed.
            #[cfg(feature = "bauar-native-admission-gate")]
            if native_admission_gate
                .finalize(run_cancelled, run_failed)
                .await
                .is_err()
            {
                emitter
                    .emit(NormalizedEvent::Error {
                        run_id: execute_run_id.clone(),
                        code: "NATIVE_ADMISSION_GATE_FINALIZATION_FAILED".into(),
                        message: "Native acceptance observer could not finalize".into(),
                    })
                    .await;
            }
            // Settle cost for any run that reported usage, cancelled or not, so a
            // cancelled run's spend reaches the durable ledger and the event stream.
            let cost_usd_estimate = crate::uar::runtime::run_cost::settle_run_cost(
                crate::uar::runtime::run_cost::RunCostInputs {
                    run_id: &execute_run_id,
                    session_id: execution_session.id(),
                    agent_id: &cost_scope_agent_id,
                    model: &run_model,
                    input_tokens: total_input_tokens,
                    output_tokens: total_output_tokens,
                    cache_read_tokens: total_cache_read_tokens,
                    cost_tracking_enabled,
                },
                &cost_budget_for_run,
                persistence_for_run.as_ref(),
                &emitter,
            )
            .await;
            if run_cancelled {
                tracing::info!(run_id = %execute_run_id, "Run cancelled; emitting terminal Cancelled event");
                let usage = has_usage.then(|| crate::uar::domain::events::RunUsage {
                    input_tokens: Some(total_input_tokens),
                    output_tokens: Some(total_output_tokens),
                    total_tokens: Some(total_tokens),
                    cost_usd_estimate,
                    model: Some(run_model),
                });
                emitter
                    .emit(NormalizedEvent::Cancelled {
                        run_id: execute_run_id,
                        usage,
                    })
                    .await;
            } else if has_usage {
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
                    let evolution = async move {
                        tracing::info!(
                            run_id = %run_id_ev,
                            tool_calls = tool_call_count,
                            "Triggering skill evolution reflection"
                        );
                        svc.evolve_from_run(&run_id_ev, tool_call_count, &cfg_ev)
                            .await
                            .map_err(anyhow::Error::from)
                    };
                    if let Err(error) = lifecycle_for_evolution.spawn("skill-evolution", evolution)
                    {
                        tracing::warn!(%error, "Skill evolution was rejected during lifecycle shutdown");
                    }
                } else {
                    tracing::debug!(
                        run_id = %evolution_run_id,
                        "Skill evolution enabled but no SkillService configured — skipping"
                    );
                }
            }
        });
        // Acquire the actor's empty producer slot before launch. Publishing the
        // join handle then has no await/cancellation gap after tokio::spawn.
        let mut actor_producer_slot = match &actor_producer {
            Some(producer) => Some(producer.lock().await),
            None => None,
        };
        let mut checkpoint_producers = self.checkpoint_producers.lock().await;
        let producer_scope = self.lifecycle_scope.clone();
        let producer = tokio::spawn(async move {
            producer_scope.run_in_scope(Box::pin(async move {
                use futures::FutureExt;
                // Keep completion outside the fallible execution body so a
                // caught unwind cannot release its actor before sandbox drain.
                let completion_guard = completion_guard;
                let result = std::panic::AssertUnwindSafe(execution).catch_unwind().await;
                let delegation_cleanup = delegation_lifetime.release().await;
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
                let mcp_cleanup = match mcp_interaction_guard.as_mut() {
                    Some(guard) => guard.finish().await,
                    None => Ok(()),
                };
                let cleanup = finalizer_sandboxes.finish_run(&finalizer_scope).await;
                if result.is_err() || delegation_cleanup.is_err() || graph_cleanup.is_err()
                    || terminal_cleanup.is_err() || thread_cleanup.is_err() || mcp_cleanup.is_err()
                    || cleanup.is_err() {
                    finalizer_checkpoint_cleanup.store(true, std::sync::atomic::Ordering::Release);
                }
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
                if let Err(error) = delegation_cleanup {
                    completion_guard.fail("run_scoped_mcp_cleanup_unconfirmed", "Run-scoped MCP cleanup remains unconfirmed");
                    if let Some(state) = finalizer_runs.write().await.get_mut(&finalizer_run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    tracing::error!(run_id = %finalizer_run_id, %error, "Run retains unconfirmed run-scoped MCP resources");
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
                if let Err(error) = mcp_cleanup {
                    completion_guard.fail("mcp_interaction_cleanup_unconfirmed", "MCP interaction cleanup remains unconfirmed");
                    if let Some(state) = finalizer_runs.write().await.get_mut(&finalizer_run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    tracing::error!(run_id = %finalizer_run_id, %error, "Run retains unconfirmed MCP interactions");
                }
                if let Err(error) = terminal_cleanup {
                    completion_guard.fail("terminal_cleanup_unconfirmed", "Terminal cleanup remains unconfirmed");
                    if let Some(state) = finalizer_runs.write().await.get_mut(&finalizer_run_id) {
                        state.run.status = RunStatus::Error;
                    }
                    tracing::error!(run_id = %finalizer_run_id, %error, "Run retains unconfirmed terminal processes");
                }
            }.instrument(run_span))).await
        });
        if let Some(slot) = actor_producer_slot.as_mut() {
            slot.handle = Some(producer);
            if let Some(root) = checkpoint_actor_root {
                checkpoint_producers.insert(
                    run_id.clone(),
                    Arc::new(Mutex::new(CheckpointProducer::Actor(root))),
                );
            }
        } else {
            checkpoint_producers.insert(
                run_id.clone(),
                Arc::new(Mutex::new(CheckpointProducer::Ordinary(Some(producer)))),
            );
        }

        run_id
    }

    /// Capture startup replay and the live receiver under the publication lock.
    /// A truncated startup is an explicit failed observation, never a success.
    pub async fn execute_request_observed(
        &self,
        request: crate::uar::runtime::turn::RunExecutionRequest,
    ) -> anyhow::Result<(String, Vec<StreamEvent>, broadcast::Receiver<StreamEvent>)> {
        let run_id = self.execute_request(request).await;
        let snapshot = {
            let runs = self.active_runs.read().await;
            match runs.get(&run_id) {
                Some(state) => {
                    let history = state.history.lock().await;
                    if history.buffer.front().is_some_and(|event| event.id != 1) {
                        None
                    } else {
                        let receiver = state.sender.subscribe();
                        Some((history.buffer.iter().cloned().collect(), receiver))
                    }
                }
                None => None,
            }
        };
        match snapshot {
            Some((events, receiver)) => Ok((run_id, events, receiver)),
            None => {
                self.cancel_run(&run_id).await;
                anyhow::bail!("run_startup_observation_lost")
            }
        }
    }

    pub async fn subscribe(&self, run_id: &str) -> Option<broadcast::Receiver<StreamEvent>> {
        let runs = self.active_runs.read().await;
        let receiver = runs.get(run_id).map(|state| state.sender.subscribe());
        drop(runs);
        if receiver.is_some() {
            self.notify_retention_work();
        }
        receiver
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
            let terminal = matches!(
                &event,
                NormalizedEvent::RunDone { .. }
                    | NormalizedEvent::RunDoneWithUsage { .. }
                    | NormalizedEvent::Cancelled { .. }
                    | NormalizedEvent::Error { .. }
            );
            let terminal_record =
                terminal.then(|| (state.run.clone(), state.verified_owner.clone()));
            let event =
                super::a2ui_output::enforce_output_ceiling(event, state.presentations.as_deref());
            let published = {
                let mut history = state.history.lock().await;
                history.record(
                    run_id,
                    event,
                    state.presentations.as_deref(),
                    true,
                    &state.sender,
                    None,
                )
            };
            if let Some(writer) = &self.run_history {
                if let Some((run, owner)) = terminal_record {
                    writer.write_record(&run, owner.as_ref());
                }
                writer.write_events(run_id, published);
            }
        }
    }

    pub async fn history_since(
        &self,
        run_id: &str,
        last_event_id: Option<u64>,
    ) -> Option<Vec<StreamEvent>> {
        let runs = self.active_runs.read().await;
        if let Some(state) = runs.get(run_id) {
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
            return Some(events);
        }
        drop(runs);
        // Restart-durable fallback: replay the published events exactly as they
        // were acknowledged; durable rows already include presentation
        // projections, so no ring compensation applies here.
        let persistence = self
            .persistence
            .as_ref()
            .filter(|layer| layer.supports_durable_run_history())?;
        let rows = persistence.load_run_history_events(run_id).await.ok()?;
        let mut events: Vec<StreamEvent> = rows
            .into_iter()
            .filter_map(|row| {
                serde_json::from_value::<NormalizedEvent>(row.event)
                    .ok()
                    .map(|event| StreamEvent {
                        id: row.event_id,
                        event,
                    })
            })
            .collect();
        events.sort_by_key(|event| event.id);
        events.retain(|event| last_event_id.is_none_or(|id| event.id > id));
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

    /// Whether this process retains a run belonging to the authenticated owner.
    /// Durable history is inspectable, but does not recreate a local approval lane.
    pub(crate) async fn has_resident_run_for_context(
        &self,
        user: &crate::uar::security::claims::UserContext,
        run_id: &str,
    ) -> bool {
        let owner = if user.user_id == crate::session::ANONYMOUS_SESSION_OWNER {
            if user.claims.sub != user.user_id || user.tenant_id.is_some() {
                return false;
            }
            None
        } else {
            let Ok(owner) =
                crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)
            else {
                return false;
            };
            Some(owner)
        };
        let runs = self.active_runs.read().await;
        runs.get(run_id).is_some_and(|state| {
            state.verified_owner == owner
                && state
                    .run
                    .user_id
                    .as_deref()
                    .unwrap_or(crate::session::ANONYMOUS_SESSION_OWNER)
                    == user.user_id
        })
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
        if let Some(state) = runs.get(run_id) {
            return (state.verified_owner == owner
                && state
                    .run
                    .user_id
                    .as_deref()
                    .unwrap_or(crate::session::ANONYMOUS_SESSION_OWNER)
                    == user.user_id)
                .then(|| state.run.clone());
        }
        drop(runs);
        // Restart-durable fallback: the record persists only the ownership
        // namespace key (`presentation_owner_key`), never a deserialized
        // principal; recompute the key from the live middleware-verified
        // context and apply the same ownership predicate as the active path.
        let persistence = self
            .persistence
            .as_ref()
            .filter(|layer| layer.supports_durable_run_history())?;
        let record = persistence.load_run_history_record(run_id).await.ok()??;
        if record.run_id != run_id {
            return None;
        }
        let namespace_matches = match (&owner, &record.owner_key) {
            (None, None) => true,
            (Some(live), Some(stored)) => live.presentation_owner_key() == *stored,
            _ => false,
        };
        if !namespace_matches {
            return None;
        }
        let run: Run = serde_json::from_value(record.run).ok()?;
        (run.run_id == run_id
            && run
                .user_id
                .as_deref()
                .unwrap_or(crate::session::ANONYMOUS_SESSION_OWNER)
                == user.user_id)
            .then_some(run)
    }

    /// List runs visible to the exact middleware-verified subject and tenant.
    ///
    /// This is intentionally the same ownership predicate as
    /// [`Self::get_run_for_context`]. Administration clients can enumerate
    /// their runs without receiving another owner's identifiers or context.
    pub(crate) async fn list_runs_for_context(
        &self,
        user: &crate::uar::security::claims::UserContext,
    ) -> Vec<Run> {
        let owner = if user.user_id == crate::session::ANONYMOUS_SESSION_OWNER {
            if user.claims.sub != user.user_id || user.tenant_id.is_some() {
                return Vec::new();
            }
            None
        } else {
            match crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user) {
                Ok(owner) => Some(owner),
                Err(_) => return Vec::new(),
            }
        };
        let runs = self.active_runs.read().await;
        let mut visible = runs
            .values()
            .filter(|state| {
                state.verified_owner == owner
                    && state
                        .run
                        .user_id
                        .as_deref()
                        .unwrap_or(crate::session::ANONYMOUS_SESSION_OWNER)
                        == user.user_id
            })
            .map(|state| state.run.clone())
            .collect::<Vec<_>>();
        visible.sort_by(|left, right| left.run_id.cmp(&right.run_id));
        visible
    }

    /// Return a run only when it belongs to the authenticated subject.
    pub async fn get_run_for_user(&self, owner_id: &str, run_id: &str) -> Option<Run> {
        self.get_run(run_id).await.filter(|run| match &run.user_id {
            Some(run_owner) => run_owner == owner_id,
            None => owner_id == crate::session::ANONYMOUS_SESSION_OWNER,
        })
    }

    /// Return a run only for the exact verified subject and tenant identity.
    /// Wire projection observes final status only after all owned producer
    /// cleanup, including interaction connections and graph persistence.
    pub(crate) async fn settled_run_status_for_context(
        &self,
        user: &crate::uar::security::claims::UserContext,
        run_id: &str,
    ) -> Option<RunStatus> {
        self.get_run_for_context(user, run_id).await?;
        self.settled_run_status(run_id).await
    }

    /// Host-only status query; ingress must establish ownership separately.
    pub(crate) async fn settled_run_status(&self, run_id: &str) -> Option<RunStatus> {
        let run = self.get_run(run_id).await?;
        if !matches!(
            run.status,
            RunStatus::Done | RunStatus::Error | RunStatus::Cancelled
        ) || !self.graph_roots.is_settled(run_id)
        {
            return None;
        }
        let producer = self.checkpoint_producers.lock().await.get(run_id).cloned();
        if let Some(producer) = producer {
            let producer = producer.try_lock().ok()?;
            let finished = match &*producer {
                CheckpointProducer::Ordinary(handle) => handle
                    .as_ref()
                    .is_none_or(tokio::task::JoinHandle::is_finished),
                CheckpointProducer::Actor(root) => root.producer.try_lock().is_ok_and(|producer| {
                    producer
                        .handle
                        .as_ref()
                        .is_none_or(tokio::task::JoinHandle::is_finished)
                }),
            };
            if !finished {
                return None;
            }
        }
        Some(run.status)
    }

    /// Serialize local handoff acknowledgments for one retained source run.
    pub(crate) async fn caller_handoff_confirmation_lock(
        &self,
        run_id: &str,
    ) -> Option<Arc<Mutex<()>>> {
        self.active_runs
            .read()
            .await
            .get(run_id)
            .map(|state| Arc::clone(&state.caller_handoff_confirmation))
    }

    /// Join the exact admitted producer and graph root, including outer cleanup.
    /// Called only after an adapter has authenticated the run or durable row.
    /// Handles stay in their slots across await so cancellation cannot detach work.
    pub(crate) async fn await_settled_run_status(&self, run_id: &str) -> anyhow::Result<RunStatus> {
        anyhow::ensure!(self.get_run(run_id).await.is_some(), "source_run_not_found");
        let producer = self.checkpoint_producers.lock().await.get(run_id).cloned();
        let joined = async {
            if let Some(producer) = producer {
                let mut producer = producer.lock().await;
                match &mut *producer {
                    CheckpointProducer::Ordinary(handle) => {
                        if let Some(join) = handle.as_mut() {
                            let outcome = join.await;
                            *handle = None;
                            if outcome.is_err() {
                                // Publish failure while retaining the producer-slot
                                // lock, before another joiner can observe it empty.
                                if let Some(state) = self.active_runs.write().await.get_mut(run_id)
                                {
                                    state.run.status = RunStatus::Error;
                                }
                                anyhow::bail!("run_producer_join_failed");
                            }
                        }
                    }
                    CheckpointProducer::Actor(root) => root.finish().await?,
                }
            }
            self.graph_roots.join_run(run_id).await?;
            Ok::<(), anyhow::Error>(())
        }
        .await;
        if let Err(error) = joined {
            if let Some(state) = self.active_runs.write().await.get_mut(run_id) {
                state.run.status = RunStatus::Error;
            }
            return Err(error);
        }
        // Joining establishes settlement for this waiter. A concurrent waiter
        // may now hold the same producer slot for an idempotent join; probing
        // that mutex with try_lock would incorrectly revoke this proof.
        let status = self
            .get_run(run_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("run_settlement_unconfirmed"))?
            .status;
        anyhow::ensure!(
            matches!(
                status,
                RunStatus::Done | RunStatus::Error | RunStatus::Cancelled
            ),
            "run_settlement_unconfirmed"
        );
        Ok(status)
    }

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

    /// Resolve a session's live run using the complete verified identity.
    pub async fn get_run_by_session_id_for_context(
        &self,
        user: &crate::uar::security::claims::UserContext,
        session_id: &str,
    ) -> Option<Run> {
        let owner = user.session_owner_key().ok()?;
        let key = crate::uar::persistence::tenant_storage_key(&owner, session_id);
        let run_id = self.session_current_run.read().await.get(&key).cloned()?;
        self.get_run_for_context(user, &run_id).await
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
mod native_identity_tests {
    use super::*;
    use crate::llm::mock_driver::MockLlmDriver;
    use crate::normalized::NormalizedEvent as ProviderEvent;
    use crate::uar::persistence::{PersistenceLayer, providers::memory::InMemoryProvider};
    use crate::uar::runtime::native_skill::NativeSkill;
    use crate::uar::tools::descriptor::ToolEffect;
    use futures::FutureExt;
    use serde_json::{Value, json};
    use std::panic::AssertUnwindSafe;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const WAIT: Duration = Duration::from_secs(3);

    struct IdentitySkill(Arc<AtomicUsize>);

    #[async_trait::async_trait]
    impl NativeSkill for IdentitySkill {
        fn name(&self) -> &str {
            "identity_probe"
        }
        fn description(&self) -> &str {
            "Returns the local identity fixture input"
        }
        fn parameters_schema(&self) -> Value {
            json!({"type":"object","properties":{"value":{"type":"integer"}},"required":["value"]})
        }
        fn effect(&self) -> ToolEffect {
            ToolEffect::ReadOnly
        }
        async fn execute(&self, arguments: Value) -> anyhow::Result<Value> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(arguments)
        }
    }

    fn delta(index: usize, id: Option<&str>, arguments: &str) -> ProviderEvent {
        ProviderEvent::ToolCallDelta {
            call_index: index,
            id: id.map(str::to_owned),
            name: Some("identity_probe".into()),
            arguments_delta: Some(arguments.into()),
        }
    }

    fn complete(index: usize, id: &str, value: u32) -> ProviderEvent {
        ProviderEvent::ToolCallComplete {
            call_index: index,
            id: id.into(),
            name: "identity_probe".into(),
            arguments_json: json!({"value":value}).to_string(),
        }
    }

    async fn cleanup_step(
        name: &str,
        future: impl std::future::Future<Output = anyhow::Result<()>>,
    ) -> Option<String> {
        match AssertUnwindSafe(tokio::time::timeout(WAIT, future))
            .catch_unwind()
            .await
        {
            Ok(Ok(Ok(()))) => None,
            Ok(Ok(Err(error))) => Some(format!("{name}: {error}")),
            Ok(Err(_)) => Some(format!("{name}: cleanup deadline exceeded")),
            Err(_) => Some(format!("{name}: cleanup panicked")),
        }
    }

    fn assert_dialogue_pairs(messages: &[Message], expected: &[(&str, u32)]) {
        let calls = messages
            .iter()
            .flat_map(|message| message.tool_calls.iter().flatten())
            .map(|call| {
                (
                    call.id.as_str(),
                    call.function.name.as_str(),
                    serde_json::from_str::<Value>(&call.function.arguments).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            calls,
            expected
                .iter()
                .map(|(id, value)| (*id, "identity_probe", json!({"value":value})))
                .collect::<Vec<_>>()
        );
        let results = messages
            .iter()
            .filter(|message| message.role == MessageRole::Tool)
            .collect::<Vec<_>>();
        assert_eq!(results.len(), expected.len());
        for (result, (id, value)) in results.iter().zip(expected) {
            assert_eq!(result.tool_call_id.as_deref(), Some(*id));
            assert_eq!(
                serde_json::from_str::<Value>(result.content.as_text().unwrap()).unwrap(),
                json!({"value":value})
            );
            let call_position = messages
                .iter()
                .position(|message| {
                    message
                        .tool_calls
                        .iter()
                        .flatten()
                        .any(|call| call.id == *id)
                })
                .unwrap();
            let result_position = messages
                .iter()
                .position(|message| {
                    message.role == MessageRole::Tool
                        && message.tool_call_id.as_deref() == Some(*id)
                })
                .unwrap();
            assert!(call_position < result_position);
        }
    }

    async fn exercise_manager(batch: Vec<ProviderEvent>, parallel: bool, succeeds: bool) {
        let store: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
        let sessions = SessionStore::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let skills = Arc::new(NativeSkillRegistry::new());
        skills
            .register(IdentitySkill(Arc::clone(&calls)))
            .await
            .unwrap();
        let driver = Arc::new(MockLlmDriver::new(vec![
            batch,
            vec![
                ProviderEvent::MessageDelta {
                    text: "finished".into(),
                },
                ProviderEvent::Done,
            ],
        ]));
        let manager = RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".into(),
                api_key: Some("identity-fixture-key".into()),
                cost_tracking: false,
                parallel_tool_calls: Some(parallel),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            sessions.clone(),
            Arc::new(RwLock::new(SkillRegistry::new(None, None))),
            Arc::new(crate::uar::runtime::matching::VectorMatcher::new(
                Arc::new(
                    crate::uar::rag::embeddings::swappable::SwappableEmbeddingBackend::new(
                        Arc::new(
                            crate::uar::rag::embeddings::UnavailableEmbeddingBackend::new(
                                384,
                                "identity fixture performs no embedding inference",
                            ),
                        ),
                    ),
                ),
                0.75,
            )),
            Some(Arc::clone(&store)),
        )
        .await
        .with_native_skills(skills)
        .with_llm_driver(driver.clone());

        // Retain the manager outside the fallible scenario so every owned family
        // is closed even when an assertion or bounded wait fails.
        let outcome = AssertUnwindSafe(async {
            tokio::time::timeout(WAIT, async {
                let mut artifact = crate::uar::defaults::default_agent();
                artifact.extensions.insert("uar.run_policy".into(), json!({"version":1,
                    "tool_approval": if succeeds { ToolApprovalPolicy::Auto } else { ToolApprovalPolicy::Ask }}));
                let mut request = crate::uar::runtime::turn::RunExecutionRequest::new(artifact, "identity fixture".into());
                request.session_id = Some("native-identity-conversation".into());
                let run_id = manager.execute_request(request).await;
                let events = loop {
                    let events = manager.history_since(&run_id, None).await.expect("real run history");
                    if events.iter().any(|event| matches!(event.event, NormalizedEvent::RunDone { .. })) {
                        break events;
                    }
                    tokio::task::yield_now().await;
                };
                let (dialogue, conversation_id, owner) = {
                    let runs = manager.active_runs.read().await;
                    let state = runs.get(&run_id).expect("executed run remains retained");
                    (state.dialogue.0.messages(), state.run.conversation_id.clone().unwrap(), state.dialogue.0.owner_id().to_owned())
                };
                let conversation = sessions.get_for_user(&conversation_id, &owner).expect("real retained conversation").messages();
                let receipts = store.list_canonical_tool_receipts(&owner, &run_id).await.unwrap();
                let requests = driver.requests();
                assert!(events.iter().all(|event| !matches!(event.event,
                    NormalizedEvent::ToolCallApprovalRequired { .. } | NormalizedEvent::ToolCallDenied { .. })));
                if succeeds {
                    assert!(events.iter().all(|event| !matches!(event.event, NormalizedEvent::Error { .. })));
                    let starts = events.iter().enumerate().filter_map(|(position, event)| match &event.event {
                        NormalizedEvent::ToolStart { call_index, tool_call_id, tool, input, .. } =>
                            Some((position, *call_index, tool_call_id.as_str(), tool.as_str(), input.clone())),
                        _ => None,
                    }).collect::<Vec<_>>();
                    assert_eq!(starts.iter().map(|(_, index, id, tool, input)| (*index, *id, *tool, input.clone())).collect::<Vec<_>>(),
                        vec![(2, "native-two", "identity_probe", json!({"value":2})),
                            (9, " native-nine ", "identity_probe", json!({"value":9}))]);
                    let ends = events.iter().enumerate().filter_map(|(position, event)| match &event.event {
                        NormalizedEvent::ToolEnd { call_index, tool_call_id, tool, output, ok, .. } =>
                            Some((position, *call_index, tool_call_id.as_str(), tool.as_str(), output.clone(), *ok)),
                        _ => None,
                    }).collect::<Vec<_>>();
                    assert_eq!(ends.iter().map(|(_, index, id, tool, output, ok)| (*index, *id, *tool, output.clone(), *ok)).collect::<Vec<_>>(),
                        vec![(2, "native-two", "identity_probe", json!({"value":2}), true),
                            (9, " native-nine ", "identity_probe", json!({"value":9}), true)]);
                    assert!(starts.last().unwrap().0 < ends.first().unwrap().0);
                    let deltas = events.iter().filter_map(|event| match &event.event {
                        NormalizedEvent::ToolDelta { call_index, tool_call_id, delta, .. } =>
                            Some((*call_index, tool_call_id.as_str(), delta.clone())),
                        _ => None,
                    }).collect::<Vec<_>>();
                    assert_eq!(deltas, vec![
                        (9, " native-nine ", json!(r#"{"value":"#)),
                        (2, "native-two", json!(r#"{"value":"#)),
                        (9, " native-nine ", json!("9")), (2, "native-two", json!("2")),
                        (9, " native-nine ", json!("}")), (2, "native-two", json!("}")),
                    ]);
                    let expected = [("native-two", 2), (" native-nine ", 9)];
                    assert_dialogue_pairs(&dialogue, &expected);
                    assert_dialogue_pairs(&conversation, &expected);
                    assert_eq!(requests.len(), 2);
                    let provider_history = requests[1].messages.iter().cloned().map(serde_json::from_value::<Message>)
                        .collect::<Result<Vec<_>, _>>().unwrap();
                    assert_dialogue_pairs(&provider_history, &expected);
                    assert_eq!(calls.load(Ordering::SeqCst), 2);
                    assert_eq!(receipts.len(), 2);
                    for (receipt, (id, value)) in receipts.iter().zip(expected) {
                        receipt.validate().unwrap();
                        assert_eq!(receipt.call_id, id);
                        assert_eq!(receipt.tool, "identity_probe");
                        assert_eq!(receipt.typed_value, Some(json!({"value":value})));
                    }
                } else {
                    assert!(events.iter().any(|event| matches!(&event.event,
                        NormalizedEvent::Error { code, .. } if code == "provider_stream_failed")));
                    assert!(events.iter().all(|event| !matches!(event.event,
                        NormalizedEvent::ToolStart { .. } | NormalizedEvent::ToolEnd { .. })));
                    assert_dialogue_pairs(&dialogue, &[]);
                    assert_dialogue_pairs(&conversation, &[]);
                    assert_eq!(requests.len(), 1);
                    assert_eq!(calls.load(Ordering::SeqCst), 0);
                    assert!(receipts.is_empty());
                }
            }).await.expect("real manager identity scenario completes");
        }).catch_unwind().await;

        let mut cleanup_errors = Vec::new();
        if let Some(error) = cleanup_step("manager lifecycle", async {
            let report = manager.shutdown_lifecycle().await;
            anyhow::ensure!(
                report.released
                    && report.failures.is_empty()
                    && report.unresolved_effects.is_empty()
                    && report.dependency_failures.is_empty(),
                "manager close did not acknowledge release: {report:?}"
            );
            Ok(())
        })
        .await
        {
            cleanup_errors.push(error);
        }
        if let Some(error) = cleanup_step("graph roots", manager.shutdown_graph_roots()).await {
            cleanup_errors.push(error);
        }
        if let Some(error) = cleanup_step("sandboxes", manager.shutdown_sandboxes()).await {
            cleanup_errors.push(error);
        }
        if let Some(error) = cleanup_step("terminals", manager.shutdown_terminals()).await {
            cleanup_errors.push(error);
        }
        if let Some(error) = cleanup_step("run history", manager.shutdown_run_history()).await {
            cleanup_errors.push(error);
        }
        if let Err(panic) = outcome {
            if !cleanup_errors.is_empty() {
                eprintln!("identity fixture cleanup: {cleanup_errors:?}");
            }
            std::panic::resume_unwind(panic);
        }
        assert!(
            cleanup_errors.is_empty(),
            "identity fixture cleanup: {cleanup_errors:?}"
        );
    }

    #[tokio::test]
    async fn native_identity_manager_correlates_live_events_dialogues_and_receipts() {
        for parallel in [false, true] {
            exercise_manager(
                vec![
                    delta(9, Some(" \t"), ""),
                    delta(9, Some(" native-nine "), r#"{"value":"#),
                    delta(2, Some("native-two"), r#"{"value":"#),
                    delta(9, None, "9"),
                    delta(2, Some(""), "2"),
                    delta(9, Some(" \t"), "}"),
                    delta(2, Some("native-two"), "}"),
                    complete(9, "", 9),
                    complete(2, " \t", 2),
                    complete(9, " native-nine ", 9),
                    ProviderEvent::Done,
                ],
                parallel,
                true,
            )
            .await;
        }
    }

    #[tokio::test]
    async fn native_identity_manager_rejects_late_conflict_and_duplicates_without_dialogue_calls() {
        for parallel in [false, true] {
            for batch in [
                vec![
                    complete(9, "private-first", 9),
                    delta(9, Some("private-second"), ""),
                    ProviderEvent::Done,
                ],
                vec![
                    complete(9, "private-first", 9),
                    ProviderEvent::Done,
                    complete(9, "private-second", 9),
                ],
                vec![
                    complete(2, "duplicate", 2),
                    complete(9, "duplicate", 9),
                    ProviderEvent::Done,
                ],
            ] {
                exercise_manager(batch, parallel, false).await;
            }
        }
    }
}

#[cfg(test)]
mod approval_gate_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::{ApprovalWaitOutcome, await_approval, governance_bypass_decision};
    use crate::llm::ToolApprovalResult;
    use crate::uar::governance::runtime_control::governance_runtime_handles;
    use std::{
        sync::{Arc, Barrier},
        time::Duration,
    };
    use tokio::sync::oneshot;

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

    use super::resolve_credential_binding;
    use crate::config::LlmConfig;
    use crate::uar::security::credentials::{
        CredentialEncryption, CredentialRecord, CredentialScope, CredentialTenancyPolicy,
        InMemoryCredentialStore, PrincipalCredentialStore, ProviderCredentialPolicyConfig,
        ProviderService,
    };
    use std::sync::Arc;

    const KEY: &[u8; 32] = b"0123456789abcdef0123456789abcdef";

    fn config_with_key() -> LlmConfig {
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            api_key: Some("configured-key".to_string()),
            ..Default::default()
        }
    }

    fn policy() -> CredentialTenancyPolicy {
        CredentialTenancyPolicy::from_config(ProviderCredentialPolicyConfig {
            installation_id: Some("6dcf767e-0888-44cb-b52a-9806fa13c1f6".to_string()),
            ..Default::default()
        })
        .expect("valid fixture credential policy")
    }

    async fn service_with_user_key(
        policy: &CredentialTenancyPolicy,
        user: &str,
        provider: &str,
        key: &str,
    ) -> Arc<ProviderService> {
        let store = Arc::new(InMemoryCredentialStore::new());
        let encryption = CredentialEncryption::from_key(KEY);
        let now = chrono::Utc::now();
        let principal = crate::uar::security::claims::CredentialPrincipal::for_test(user);
        let owner = policy
            .bind_principal(&principal)
            .expect("bound fixture owner");
        store
            .put_principal(
                &owner,
                CredentialRecord {
                    scope: CredentialScope::User,
                    scope_id: user.to_string(),
                    provider_id: provider.to_string(),
                    api_key_encrypted: encryption.encrypt(key).expect("encrypt"),
                    api_key_hint: CredentialEncryption::key_hint(key, 4),
                    created_at: now,
                    updated_at: now,
                    row_revision: None,
                },
            )
            .await
            .expect("put");
        Arc::new(ProviderService::new(store, Arc::new(encryption)))
    }

    #[tokio::test]
    async fn configured_key_becomes_host_supplied_without_environment_lookup() {
        let policy = policy();
        let principal = crate::uar::security::claims::CredentialPrincipal::for_test("alice");
        let (config, _) = resolve_credential_binding(
            config_with_key(),
            &policy,
            None,
            Some(&principal),
            None,
            Some("agent-1"),
            false,
        )
        .await
        .expect("configured credential decision");
        assert_eq!(config.api_key.as_deref(), Some("configured-key"));
        assert!(config.host_supplied_connection);
        assert!(config.api_key_env.is_none());
        assert!(config.provider_keys.is_empty());
    }

    #[tokio::test]
    async fn stored_user_key_replaces_the_configured_key_before_capture() {
        let policy = policy();
        let service = service_with_user_key(&policy, "alice", "openai", "sk-alice-user").await;
        let principal = crate::uar::security::claims::CredentialPrincipal::for_test("alice");
        let (config, _) = resolve_credential_binding(
            config_with_key(),
            &policy,
            Some(&service),
            Some(&principal),
            None,
            Some("agent-1"),
            false,
        )
        .await
        .expect("stored credential decision");
        assert_eq!(config.api_key.as_deref(), Some("sk-alice-user"));
        assert!(config.host_supplied_connection);
    }

    #[tokio::test]
    async fn provider_miss_retains_only_the_policy_approved_config_source() {
        let policy = policy();
        let service = service_with_user_key(&policy, "alice", "anthropic", "sk-anthropic").await;
        let principal = crate::uar::security::claims::CredentialPrincipal::for_test("alice");
        let (config, _) = resolve_credential_binding(
            config_with_key(),
            &policy,
            Some(&service),
            Some(&principal),
            None,
            Some("agent-1"),
            false,
        )
        .await
        .expect("config source decision");
        assert_eq!(config.api_key.as_deref(), Some("configured-key"));
        assert!(config.host_supplied_connection);
    }
}
