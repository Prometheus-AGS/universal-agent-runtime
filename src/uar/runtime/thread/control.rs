//! Governed model-tool boundary for the trusted thread host. This module owns
//! argument/identity checks and read-only waits, not model execution. A host
//! implementation is mandatory; there is no fake or no-op execution fallback.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use futures::{StreamExt, stream::FuturesUnordered};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::uar::domain::artifact::AgentArtifact;
use crate::uar::domain::policy::{
    RUN_POLICY_VERSION, RunPolicy, SelectionMode, ToolApprovalPolicy,
};
use crate::uar::persistence::agent_threads::PersistedAgentThread;

use super::messages::InterAgentMessage;
use super::policy_intersection::{ThreadPolicy, ThreadToolBinding};
use super::spawn::{AgentSpawnRequest, RemoteAgentSpawnRequest};
use super::{AgentEdge, AgentHandle, AgentThread, AgentThreadResult, AgentThreadStatus};

/// Names reserved for parent-turn-bound agent controls.
pub const AGENT_TOOL_NAMES: [&str; 5] = [
    "spawn_agent",
    "send_agent_message",
    "wait_agents",
    "list_agents",
    "interrupt_agent",
];

/// A user grant minted by the verified root host, never decoded from tool args
/// or inferred from a child's text. Revocation denies subsequent spawn calls.
#[derive(Clone)]
pub struct RootDelegationGrant {
    owner_id: String,
    root_run_id: String,
    revoked: CancellationToken,
}

impl std::fmt::Debug for RootDelegationGrant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RootDelegationGrant")
            .field("root_run_id", &self.root_run_id)
            .field("revoked", &self.revoked.is_cancelled())
            .finish_non_exhaustive()
    }
}

impl RootDelegationGrant {
    /// Record an explicit, already verified root-user delegation decision.
    /// This is a host API, not a parser for user/model message content.
    ///
    /// # Errors
    /// Rejects a descendant or a record outside its original live root run.
    pub fn from_verified_user(root: &AgentThread) -> Result<Self, AgentControlError> {
        root.validate()
            .map_err(|_| AgentControlError::InvalidContext)?;
        if root.parent_thread_id.is_some()
            || root.run_id.as_ref() != Some(&root.root_run_id)
            || !live(root.status)
        {
            return Err(AgentControlError::InvalidContext);
        }
        Ok(Self {
            owner_id: root.owner_id.clone(),
            root_run_id: root.root_run_id.clone(),
            revoked: CancellationToken::new(),
        })
    }

    /// Revoke the grant without converting child text into a user decision.
    pub fn revoke(&self) {
        self.revoked.cancel();
    }

    fn permits(&self, caller: &AgentThread) -> bool {
        self.owner_id == caller.owner_id
            && self.root_run_id == caller.root_run_id
            && !self.revoked.is_cancelled()
    }
}

/// Immutable identity/policy supplied to the host by the governed tool boundary.
/// No deserializer or mutation methods are exposed.
#[derive(Debug, Clone)]
pub struct AgentControlScope {
    caller: AgentThread,
    policy: Arc<ThreadPolicy>,
}

impl AgentControlScope {
    /// Host-resolved caller, including root identity and current run ID.
    pub fn caller(&self) -> &AgentThread {
        &self.caller
    }

    /// Frozen effective authority; child policy must intersect this snapshot.
    pub fn policy(&self) -> &ThreadPolicy {
        &self.policy
    }
}

/// Body-only model intent. Sender identity and sequence are assigned by the host.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendAgentMessageRequest {
    pub recipient_thread_id: String,
    pub content: String,
    #[serde(default)]
    pub trigger_turn: bool,
}

impl std::fmt::Debug for SendAgentMessageRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendAgentMessageRequest")
            .field("recipient_thread_id", &self.recipient_thread_id)
            .field("trigger_turn", &self.trigger_turn)
            .finish_non_exhaustive()
    }
}

/// Wait for any observed terminal turn. Zero performs a non-blocking snapshot.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaitAgentsRequest {
    pub thread_ids: Vec<String>,
    #[serde(default = "default_wait_timeout")]
    pub timeout_ms: u64,
}

fn default_wait_timeout() -> u64 {
    30_000
}

/// Cancellation acknowledgment is distinct from a terminal thread result.
#[derive(Debug)]
pub struct AgentInterruptReceipt {
    pub thread: AgentThread,
    pub cancellation_requested: bool,
}

/// Mandatory trusted-host operations shared by the native tools and adapters.
/// Implementations must recheck live authority at the mutation boundary, persist
/// before publishing, intersect child policy, enforce tree/root budgets, retain
/// exact host bindings, and route all approvals to the root. Mutation lifetimes
/// must not become untracked merely because a tool caller drops its future.
///
/// This contract deliberately supplies no implementation that fabricates child
/// execution, approval, delivery, completion, or cancellation.
#[async_trait::async_trait]
pub trait AgentThreadHost: Send + Sync {
    async fn spawn(
        &self,
        scope: &AgentControlScope,
        request: AgentSpawnRequest,
    ) -> anyhow::Result<AgentThread>;
    async fn spawn_remote(
        &self,
        scope: &AgentControlScope,
        request: RemoteAgentSpawnRequest,
    ) -> anyhow::Result<AgentThread>;
    async fn send_message(
        &self,
        scope: &AgentControlScope,
        request: SendAgentMessageRequest,
    ) -> anyhow::Result<InterAgentMessage>;
    async fn load_thread(
        &self,
        scope: &AgentControlScope,
        thread_id: &str,
    ) -> anyhow::Result<Option<AgentThread>>;
    async fn list_threads(&self, scope: &AgentControlScope) -> anyhow::Result<Vec<AgentThread>>;
    async fn subscribe_thread(
        &self,
        scope: &AgentControlScope,
        thread_id: &str,
    ) -> anyhow::Result<AgentHandle>;
    /// Subscribe to the first invocation's retained result, not latest state.
    /// Hosts without a receipt cannot substitute a later resumed turn.
    async fn subscribe_first_turn(
        &self,
        _scope: &AgentControlScope,
        _thread_id: &str,
    ) -> anyhow::Result<AgentHandle> {
        anyhow::bail!("Thread host does not retain first-turn results")
    }
    async fn interrupt(
        &self,
        scope: &AgentControlScope,
        thread_id: &str,
    ) -> anyhow::Result<AgentInterruptReceipt>;
}

/// Safe model-facing listing. Prompts, history, credentials, and result bodies
/// are omitted; wait results carry only the separately requested terminal result.
#[derive(Debug, Clone, Serialize)]
pub struct AgentSummary {
    pub thread_id: String,
    pub root_thread_id: String,
    pub parent_thread_id: Option<String>,
    pub canonical_path: String,
    pub artifact_id: String,
    pub run_id: Option<String>,
    pub status: AgentThreadStatus,
}

impl From<&AgentThread> for AgentSummary {
    fn from(thread: &AgentThread) -> Self {
        Self {
            thread_id: thread.thread_id.clone(),
            root_thread_id: thread.root_thread_id.clone(),
            parent_thread_id: thread.parent_thread_id.clone(),
            canonical_path: thread.canonical_path.clone(),
            artifact_id: thread.artifact_id.clone(),
            run_id: thread.run_id.clone(),
            status: thread.status,
        }
    }
}

/// Typed delivery acknowledgment without echoing message content into history.
#[derive(Debug, Serialize)]
pub struct AgentMessageReceipt {
    pub message_id: String,
    pub recipient_thread_id: String,
    pub sequence: u64,
    pub trigger_turn: bool,
}

/// The run ID identifies which observed turn produced this result. Watches are
/// latest-state subscriptions, not queries for earlier resumed turns.
#[derive(Debug, Serialize)]
pub struct AgentTurnOutcome {
    pub agent: AgentSummary,
    pub result: Option<AgentThreadResult>,
}

/// Timeout leaves unfinished children running. It never invents completion.
#[derive(Debug, Serialize)]
pub struct WaitAgentsResult {
    pub threads: Vec<AgentTurnOutcome>,
    pub timed_out: bool,
}

/// Model-facing failures omit raw arguments, child prompts, and backend details.
#[derive(Debug, thiserror::Error)]
pub enum AgentControlError {
    #[error("invalid host agent-tool context")]
    InvalidContext,
    #[error("agent tool is not authorized in this turn")]
    NotAuthorized,
    #[error("invalid agent-tool arguments")]
    InvalidArguments,
    #[error("agent-tool caller is no longer the active turn")]
    StaleTurn,
    #[error("agent-tool caller was cancelled")]
    Cancelled,
    #[error("agent thread is unavailable in this root tree")]
    ThreadUnavailable,
    #[error("agent host returned an invalid or foreign record")]
    InvalidHostResult,
    #[error("agent thread observation ended before a terminal result")]
    ObservationClosed,
    #[error("agent host operation failed")]
    Host(#[source] anyhow::Error),
}

/// One live parent turn's tool authority. Construct a fresh registry for each
/// context; never reuse equivalent descriptors with another parent's handlers.
pub struct AgentToolContext {
    scope: AgentControlScope,
    host: Arc<dyn AgentThreadHost>,
    cancellation: CancellationToken,
    user_grant: Option<RootDelegationGrant>,
    artifact_authorized: bool,
}

impl std::fmt::Debug for AgentToolContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentToolContext")
            .field("scope", &self.scope)
            .field("artifact_authorized", &self.artifact_authorized)
            .finish_non_exhaustive()
    }
}

impl AgentToolContext {
    /// The trusted kernel checks this identity before installing turn handlers.
    pub(crate) fn scope(&self) -> &AgentControlScope {
        &self.scope
    }

    /// Bind controls to a persisted live caller and its original registered
    /// artifact. Do not pass the materialized narrowed artifact: an expanded
    /// wildcard allow-list is eligibility, not explicit delegation authorization.
    ///
    /// # Errors
    /// Rejects identity/policy mismatches and malformed artifact authorization.
    pub fn for_turn(
        caller: &PersistedAgentThread,
        policy: Arc<ThreadPolicy>,
        original_artifact: &AgentArtifact,
        host: Arc<dyn AgentThreadHost>,
        cancellation: CancellationToken,
        user_grant: Option<RootDelegationGrant>,
    ) -> Result<Self, AgentControlError> {
        caller
            .validate(policy.owner_id())
            .map_err(|_| AgentControlError::InvalidContext)?;
        let thread = &caller.thread;
        if !live(thread.status)
            || thread.run_id.is_none()
            || thread.artifact_id != original_artifact.id
            || policy.artifact().id != thread.artifact_id
            || policy.approval_root_run_id() != thread.root_run_id
            || user_grant.as_ref().is_some_and(|grant| {
                grant.owner_id != thread.owner_id || grant.root_run_id != thread.root_run_id
            })
        {
            return Err(AgentControlError::InvalidContext);
        }
        let artifact_authorized = explicitly_authorizes_spawn(original_artifact)?;
        Ok(Self {
            scope: AgentControlScope {
                caller: thread.clone(),
                policy,
            },
            host,
            cancellation,
            user_grant,
            artifact_authorized,
        })
    }

    /// Registration and execution both consult this predicate. Authorization
    /// does not bypass schema validation, Cedar policy, or root approval.
    pub fn permits(&self, tool: &str) -> bool {
        let policy = self.scope.policy.effective();
        !self.cancellation.is_cancelled()
            && AGENT_TOOL_NAMES.contains(&tool)
            && policy.tool_approval != ToolApprovalPolicy::Deny
            && policy.tools.mode == SelectionMode::Selected
            && policy.tools.ids.iter().any(|id| id == tool)
            && matches!(
                self.scope.policy.permissions().tool_bindings.get(tool),
                Some(ThreadToolBinding::Native)
            )
            && (tool != "spawn_agent"
                || self.artifact_authorized
                || self
                    .user_grant
                    .as_ref()
                    .is_some_and(|grant| grant.permits(&self.scope.caller)))
    }

    /// Dispatch a validated spawn intent to the trusted host.
    pub async fn spawn(
        &self,
        request: AgentSpawnRequest,
    ) -> Result<AgentSummary, AgentControlError> {
        request
            .validate()
            .map_err(|_| AgentControlError::InvalidArguments)?;
        self.require_current("spawn_agent").await?;
        let artifact_id = request.artifact_id.clone();
        let task_name = request.task_name.clone();
        let thread = self
            .host
            .spawn(&self.scope, request)
            .await
            .map_err(AgentControlError::Host)?;
        self.validate_thread(&thread)?;
        AgentEdge::between(&self.scope.caller, &thread)
            .map_err(|_| AgentControlError::InvalidHostResult)?;
        if thread.artifact_id != artifact_id
            || task_name.is_some_and(|name| {
                thread.canonical_path != format!("{}/{name}", self.scope.caller.canonical_path)
            })
        {
            return Err(AgentControlError::InvalidHostResult);
        }
        Ok(AgentSummary::from(&thread))
    }

    /// Host-only graph adapter. Endpoint and credentials are resolved by the
    /// retained thread host; this operation is not exposed as model arguments.
    pub(crate) async fn spawn_remote(
        &self,
        request: RemoteAgentSpawnRequest,
    ) -> Result<AgentSummary, AgentControlError> {
        request
            .validate()
            .map_err(|_| AgentControlError::InvalidArguments)?;
        let expected_agent_id = self
            .scope
            .policy
            .remote_agent_for_endpoint(&request.endpoint)
            .map_err(|_| AgentControlError::InvalidArguments)?;
        self.require_current("spawn_agent").await?;
        let task_name = request.task_name.clone();
        let thread = self
            .host
            .spawn_remote(&self.scope, request)
            .await
            .map_err(AgentControlError::Host)?;
        self.validate_thread(&thread)?;
        AgentEdge::between(&self.scope.caller, &thread)
            .map_err(|_| AgentControlError::InvalidHostResult)?;
        if thread.artifact_id != expected_agent_id
            || task_name.is_some_and(|name| {
                thread.canonical_path != format!("{}/{name}", self.scope.caller.canonical_path)
            })
        {
            return Err(AgentControlError::InvalidHostResult);
        }
        Ok(AgentSummary::from(&thread))
    }

    /// Send a body-only intent; the host owns sender identity and mailbox order.
    pub async fn send_message(
        &self,
        request: SendAgentMessageRequest,
    ) -> Result<AgentMessageReceipt, AgentControlError> {
        nonempty(&request.recipient_thread_id)?;
        nonempty(&request.content)?;
        self.require_current("send_agent_message").await?;
        self.target(&request.recipient_thread_id).await?;
        let message = self
            .host
            .send_message(&self.scope, request.clone())
            .await
            .map_err(AgentControlError::Host)?;
        let caller = &self.scope.caller;
        if message.message_id.trim().is_empty()
            || message.owner_id != caller.owner_id
            || message.root_thread_id != caller.root_thread_id
            || message.root_run_id != caller.root_run_id
            || message.sender_thread_id != caller.thread_id
            || message.sender_artifact_id != caller.artifact_id
            || message.recipient_thread_id != request.recipient_thread_id
            || message.content != request.content
            || message.trigger_turn != request.trigger_turn
        {
            return Err(AgentControlError::InvalidHostResult);
        }
        Ok(AgentMessageReceipt {
            message_id: message.message_id,
            recipient_thread_id: message.recipient_thread_id,
            sequence: message.sequence,
            trigger_turn: message.trigger_turn,
        })
    }

    /// List only children in the caller's root tree, without prompt/result bodies.
    pub async fn list_agents(&self) -> Result<Vec<AgentSummary>, AgentControlError> {
        self.require_current("list_agents").await?;
        let mut threads = self
            .host
            .list_threads(&self.scope)
            .await
            .map_err(AgentControlError::Host)?;
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for thread in &threads {
            self.validate_thread(thread)?;
            if !ids.insert(thread.thread_id.clone()) || !paths.insert(thread.canonical_path.clone())
            {
                return Err(AgentControlError::InvalidHostResult);
            }
        }
        threads.retain(|thread| thread.parent_thread_id.is_some());
        if threads.len() > 16 {
            return Err(AgentControlError::InvalidHostResult);
        }
        threads.sort_by(|left, right| left.order_key().cmp(&right.order_key()));
        Ok(threads.iter().map(AgentSummary::from).collect())
    }

    /// Request descendant cancellation. The acknowledgment does not claim that
    /// local or remote execution has already stopped; wait observes that state.
    pub async fn interrupt(
        &self,
        thread_id: &str,
    ) -> Result<(AgentSummary, bool), AgentControlError> {
        nonempty(thread_id)?;
        self.require_current("interrupt_agent").await?;
        let target = self.target(thread_id).await?;
        if !target
            .canonical_path
            .starts_with(&format!("{}/", self.scope.caller.canonical_path))
        {
            return Err(AgentControlError::NotAuthorized);
        }
        let receipt = self
            .host
            .interrupt(&self.scope, thread_id)
            .await
            .map_err(AgentControlError::Host)?;
        self.validate_target(&receipt.thread, thread_id)?;
        Ok((
            AgentSummary::from(&receipt.thread),
            receipt.cancellation_requested,
        ))
    }

    /// Observe child completion without spawning polling jobs or cancelling any
    /// child on timeout. Waits stop when the caller's cancellation token fires.
    pub async fn wait_agents(
        &self,
        request: WaitAgentsRequest,
    ) -> Result<WaitAgentsResult, AgentControlError> {
        if request.thread_ids.is_empty()
            || request.thread_ids.len() > 16
            || request.timeout_ms > 60_000
        {
            return Err(AgentControlError::InvalidArguments);
        }
        let mut ids = BTreeSet::new();
        for id in &request.thread_ids {
            nonempty(id)?;
            if !ids.insert(id) {
                return Err(AgentControlError::InvalidArguments);
            }
        }
        self.require_current("wait_agents").await?;
        let mut handles = Vec::with_capacity(request.thread_ids.len());
        for id in &request.thread_ids {
            let handle = self
                .host
                .subscribe_thread(&self.scope, id)
                .await
                .map_err(AgentControlError::Host)?;
            let snapshot = handle
                .snapshot()
                .map_err(|_| AgentControlError::InvalidHostResult)?;
            self.validate_target(&snapshot, id)?;
            handles.push(handle);
        }
        // Subscription setup can await I/O while the caller is cancelled or
        // replaced. Snapshot-only waits must not bypass the live-turn check.
        self.require_current("wait_agents").await?;
        let initial = self.snapshots(&handles)?;
        if initial.iter().any(|thread| thread.status.is_terminal()) {
            return Ok(wait_result(initial, false));
        }
        if request.timeout_ms == 0 {
            return Ok(wait_result(initial, true));
        }
        let mut pending = FuturesUnordered::new();
        for handle in &handles {
            let handle = handle.clone();
            pending.push(async move { handle.wait_until_terminal().await });
        }
        let completed = tokio::select! {
            biased;
            _ = self.cancellation.cancelled() => return Err(AgentControlError::Cancelled),
            result = pending.next() => Some(result.ok_or(AgentControlError::ObservationClosed)?
                .map_err(|_| AgentControlError::ObservationClosed)?),
            _ = tokio::time::sleep(Duration::from_millis(request.timeout_ms)) => None,
        };
        self.require_current("wait_agents").await?;
        let mut snapshots = self.snapshots(&handles)?;
        let timed_out =
            completed.is_none() && !snapshots.iter().any(|thread| thread.status.is_terminal());
        if let Some(completed) = completed {
            self.validate_thread(&completed)?;
            // Preserve the actual terminal observation even if a host has already
            // started another turn before the remaining snapshots are collected.
            let target = snapshots
                .iter_mut()
                .find(|thread| thread.thread_id == completed.thread_id)
                .ok_or(AgentControlError::InvalidHostResult)?;
            *target = completed;
        }
        Ok(wait_result(snapshots, timed_out))
    }

    /// Host-adapter wait for the invocation just spawned by this parent.
    /// This is not a model tool and does not reinterpret a newer turn's result.
    pub(crate) async fn wait_first_turn(
        &self,
        thread_id: &str,
    ) -> Result<AgentTurnOutcome, AgentControlError> {
        nonempty(thread_id)?;
        self.require_current("wait_agents").await?;
        let handle = self
            .host
            .subscribe_first_turn(&self.scope, thread_id)
            .await
            .map_err(AgentControlError::Host)?;
        let initial = handle
            .snapshot()
            .map_err(|_| AgentControlError::InvalidHostResult)?;
        self.validate_target(&initial, thread_id)?;
        let completed = tokio::select! {
            biased;
            _ = self.cancellation.cancelled() => return Err(AgentControlError::Cancelled),
            result = handle.wait_until_terminal() => result.map_err(|_| AgentControlError::ObservationClosed)?,
        };
        self.require_current("wait_agents").await?;
        self.validate_target(&completed, thread_id)?;
        if completed.parent_thread_id.as_deref() != Some(&self.scope.caller.thread_id)
            || completed.history_revision > 1
        {
            return Err(AgentControlError::InvalidHostResult);
        }
        Ok(AgentTurnOutcome {
            agent: AgentSummary::from(&completed),
            result: completed.result,
        })
    }

    async fn require_current(&self, tool: &str) -> Result<(), AgentControlError> {
        if self.cancellation.is_cancelled() {
            return Err(AgentControlError::Cancelled);
        }
        if !self.permits(tool) {
            return Err(AgentControlError::NotAuthorized);
        }
        let current = self.target(&self.scope.caller.thread_id).await?;
        if current.run_id != self.scope.caller.run_id
            || !live(current.status)
            || current.artifact_id != self.scope.caller.artifact_id
            || current.history_revision < self.scope.caller.history_revision
        {
            return Err(AgentControlError::StaleTurn);
        }
        // A host lookup can await I/O; recheck revocation/cancellation afterward.
        if self.cancellation.is_cancelled() {
            return Err(AgentControlError::Cancelled);
        }
        if !self.permits(tool) {
            return Err(AgentControlError::NotAuthorized);
        }
        Ok(())
    }

    async fn target(&self, thread_id: &str) -> Result<AgentThread, AgentControlError> {
        let thread = self
            .host
            .load_thread(&self.scope, thread_id)
            .await
            .map_err(AgentControlError::Host)?
            .ok_or(AgentControlError::ThreadUnavailable)?;
        self.validate_target(&thread, thread_id)?;
        Ok(thread)
    }

    fn validate_target(
        &self,
        thread: &AgentThread,
        thread_id: &str,
    ) -> Result<(), AgentControlError> {
        self.validate_thread(thread)?;
        if thread.thread_id != thread_id {
            return Err(AgentControlError::InvalidHostResult);
        }
        Ok(())
    }

    fn validate_thread(&self, thread: &AgentThread) -> Result<(), AgentControlError> {
        thread
            .validate()
            .map_err(|_| AgentControlError::InvalidHostResult)?;
        let caller = &self.scope.caller;
        if thread.owner_id != caller.owner_id
            || thread.root_thread_id != caller.root_thread_id
            || thread.root_run_id != caller.root_run_id
        {
            return Err(AgentControlError::InvalidHostResult);
        }
        Ok(())
    }

    fn snapshots(&self, handles: &[AgentHandle]) -> Result<Vec<AgentThread>, AgentControlError> {
        handles
            .iter()
            .map(|handle| {
                let snapshot = handle
                    .snapshot()
                    .map_err(|_| AgentControlError::InvalidHostResult)?;
                self.validate_thread(&snapshot)?;
                Ok(snapshot)
            })
            .collect()
    }
}

fn explicitly_authorizes_spawn(artifact: &AgentArtifact) -> Result<bool, AgentControlError> {
    let extended = artifact
        .extensions
        .get("uar.run_policy")
        .map(|value| {
            serde_json::from_value::<RunPolicy>(value.clone())
                .map_err(|_| AgentControlError::InvalidContext)
        })
        .transpose()?;
    if extended
        .as_ref()
        .is_some_and(|policy| policy.version != RUN_POLICY_VERSION)
    {
        return Err(AgentControlError::InvalidContext);
    }
    Ok(artifact
        .policy
        .tools
        .allow
        .iter()
        .any(|tool| tool == "spawn_agent")
        || extended.is_some_and(|policy| {
            policy.tools.mode == SelectionMode::Selected
                && policy.tools.ids.iter().any(|tool| tool == "spawn_agent")
        }))
}

fn nonempty(value: &str) -> Result<(), AgentControlError> {
    if value.trim().is_empty() {
        return Err(AgentControlError::InvalidArguments);
    }
    Ok(())
}

fn live(status: AgentThreadStatus) -> bool {
    matches!(
        status,
        AgentThreadStatus::Running | AgentThreadStatus::Waiting
    )
}

fn wait_result(threads: Vec<AgentThread>, timed_out: bool) -> WaitAgentsResult {
    WaitAgentsResult {
        threads: threads
            .into_iter()
            .map(|thread| AgentTurnOutcome {
                agent: AgentSummary::from(&thread),
                result: thread.result,
            })
            .collect(),
        timed_out,
    }
}
