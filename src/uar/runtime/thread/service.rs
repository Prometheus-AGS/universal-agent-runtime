//! Trusted, root-scoped thread host. The host owns scheduling and persistence;
//! model execution is supplied by the shared run kernel, never another LLM loop.

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex as JobMutex};

use tokio::sync::{Mutex, oneshot, watch};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::llm::{Message, MessageContent, MessageRole};
use crate::uar::domain::{artifact::AgentArtifact, events::RuntimeEventSink};
use crate::uar::persistence::{
    PersistenceLayer,
    agent_threads::{self, PersistedAgentThread},
};

use super::control::{
    AgentControlScope, AgentInterruptReceipt, AgentThreadHost, AgentToolContext,
    RootDelegationGrant, SendAgentMessageRequest,
};
use super::limits::{ActiveChildPermit, AgentTreeAdmission, AgentTreeLimits, ChildReservation};
use super::messages::InterAgentMessage;
use super::policy_intersection::ThreadPolicy;
use super::spawn::{AgentSpawnRequest, HistoryForkMode, RemoteAgentSpawnRequest};
use super::{
    AgentEdge, AgentHandle, AgentThread, AgentThreadResult, AgentThreadStatus, RemoteThreadBinding,
};

/// One committed child turn. Neither this input nor its authority is decoded
/// from model arguments. The executor must preserve the exact policy bindings.
pub struct HostedThreadTurn {
    pub record: PersistedAgentThread,
    pub policy: Arc<ThreadPolicy>,
    pub original_artifact: AgentArtifact,
    pub messages: Vec<Message>,
    pub controls: Arc<AgentToolContext>,
    pub cancellation: CancellationToken,
}

impl std::fmt::Debug for HostedThreadTurn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostedThreadTurn")
            .field("thread_id", &self.record.thread.thread_id)
            .field("message_count", &self.messages.len())
            .finish_non_exhaustive()
    }
}

/// Required bridge to the trusted shared-kernel host. There are no default
/// methods: an adapter cannot silently skip binding, budget, or execution work.
#[async_trait::async_trait]
pub trait ThreadExecutionHost: Send + Sync {
    /// Resolve the exact registered artifact or fail; never substitute a default.
    async fn artifact(&self, owner_id: &str, artifact_id: &str) -> anyhow::Result<AgentArtifact>;
    /// Read canonical history for this host-resolved thread, not a client ID.
    async fn history(&self, thread: &AgentThread) -> anyhow::Result<Vec<Message>>;
    /// Check frozen executable bindings and sandbox enforcement. The service
    /// independently checks the captured root budget before admission and turns.
    async fn check_admission(&self, policy: &ThreadPolicy) -> anyhow::Result<()>;
    /// Run the same turn kernel, including root approvals and per-call budgets.
    /// Cancellation must unwind local/remote work before this method returns.
    async fn execute(&self, turn: HostedThreadTurn) -> anyhow::Result<AgentThreadResult>;
}

struct Entry {
    record: PersistedAgentThread,
    confirmed: bool,
    pending: Option<PersistedAgentThread>,
    reservation: Option<ChildReservation>,
    parent: Option<PersistedAgentThread>,
    policy: Arc<ThreadPolicy>,
    original_artifact: AgentArtifact,
    cancellation: CancellationToken,
    handle: AgentHandle,
    publisher: watch::Sender<AgentThread>,
    first_turn_handle: AgentHandle,
    first_turn_publisher: watch::Sender<AgentThread>,
    mailbox: VecDeque<InterAgentMessage>,
    sequence: u64,
    worker_running: bool,
    target: ChildTarget,
}

enum ChildTarget {
    Local,
    Remote(RemoteChild),
}

struct RemoteChild {
    peer: crate::uar::api::a2a::peer::TrustedA2APeer,
    contract: crate::uar::api::a2a::contract::UarDelegationContract,
    reservation: crate::uar::runtime::cost_budget::RemoteBudgetReservation,
    execution: Option<Arc<Mutex<crate::uar::api::a2a::task_execution::A2ATaskExecution>>>,
    // Monotonic host fact: absence of a current execution alone does not prove
    // the peer was never contacted (completed turns clear that handle).
    execution_admitted: bool,
}

impl RemoteChild {
    fn release_if_never_dispatched(&self) -> anyhow::Result<()> {
        if !self.execution_admitted {
            self.reservation.release_confirmed()?;
        }
        Ok(())
    }
}

enum TurnWork {
    Local(HostedThreadTurn),
    Remote {
        execution: Arc<Mutex<crate::uar::api::a2a::task_execution::A2ATaskExecution>>,
        reservation: crate::uar::runtime::cost_budget::RemoteBudgetReservation,
    },
}

struct ThreadJob {
    state: Mutex<ThreadJobState>,
}

struct ThreadJobState {
    handle: Option<JoinHandle<()>>,
    failure: Option<String>,
}

impl ThreadJob {
    fn is_finished(&self) -> bool {
        self.state
            .try_lock()
            .is_ok_and(|state| state.handle.as_ref().is_none_or(JoinHandle::is_finished))
    }

    fn is_joined(&self) -> bool {
        self.state
            .try_lock()
            .is_ok_and(|state| state.handle.is_none() && state.failure.is_none())
    }

    async fn join(&self) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        if let Some(handle) = state.handle.as_mut() {
            let result = handle.await;
            state.failure = result.err().map(|error| error.to_string());
            state.handle = None;
        }
        // Retain a failed receipt for concurrent/subsequent shutdown callers;
        // joining an already-consumed handle must not turn failure into success.
        match &state.failure {
            Some(error) => Err(anyhow::anyhow!("Thread host job failed: {error}")),
            None => Ok(()),
        }
    }
}

#[derive(Default)]
struct ThreadJobs {
    closed: bool,
    entries: Vec<Arc<ThreadJob>>,
}

struct RootHost {
    root: AgentThread,
    persistence: Arc<dyn PersistenceLayer>,
    executor: Arc<dyn ThreadExecutionHost>,
    kernel: Arc<super::kernel::CapturedThreadKernel>,
    events: Arc<dyn RuntimeEventSink>,
    grant: std::sync::OnceLock<RootDelegationGrant>,
    cancellation: CancellationToken,
    admission: AgentTreeAdmission,
    // Serialize root-wide admission and storage transitions, never model work.
    entries: Mutex<BTreeMap<String, Entry>>,
    jobs: JobMutex<ThreadJobs>,
}

/// Cloneable host for exactly one root run. All adapters in that run must share
/// this instance, so independent callers cannot reset the admission counters.
#[derive(Clone)]
pub struct ThreadService {
    inner: Arc<RootHost>,
}

impl std::fmt::Debug for ThreadService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadService")
            .field("root_thread_id", &self.inner.root.thread_id)
            .finish_non_exhaustive()
    }
}

impl ThreadService {
    /// Attach to a newly committed root. Existing child trees require recovery,
    /// not fresh zeroed counters. Call once, before exposing root agent tools.
    ///
    /// # Errors
    /// Rejects a second attachment, stale roots, existing children, mismatched
    /// authority or an unavailable/exceeded captured root budget. Root policy,
    /// original artifact, persistence and cancellation come only from the
    /// manager's capture, never from adapter-supplied replacements.
    pub async fn attach(
        kernel: Arc<super::kernel::CapturedThreadKernel>,
        events: Arc<dyn RuntimeEventSink>,
        grant: Option<RootDelegationGrant>,
    ) -> anyhow::Result<Self> {
        kernel.claim_attachment()?;
        let root = kernel.root_record().clone();
        let original_artifact = kernel.original_artifact().clone();
        let persistence = kernel.persistence();
        let cancellation = kernel.cancellation();
        let policy = Arc::new(kernel.root_policy().await?);
        root.validate(policy.owner_id())?;
        if root.thread.parent_thread_id.is_some()
            || root.thread.run_id.as_ref() != Some(&root.thread.root_run_id)
            || !live(root.thread.status)
            || policy.artifact().id != original_artifact.id
            || root.thread.artifact_id != original_artifact.id
            || root.thread.root_run_id != policy.approval_root_run_id()
            || cancellation.is_cancelled()
        {
            anyhow::bail!("Invalid root thread host authority");
        }
        let records = persistence
            .list_agent_threads(policy.owner_id(), &root.thread.root_run_id)
            .await?;
        if records != vec![root.clone()]
            || !persistence
                .list_agent_edges(policy.owner_id(), &root.thread.root_run_id)
                .await?
                .is_empty()
        {
            anyhow::bail!("Root host attachment requires a fresh committed tree");
        }
        kernel.check_attachment(&root, &policy).await?;
        // Admission and execution must use the same captured resources. An
        // adapter-supplied callback cannot replace the real host checks.
        let executor: Arc<dyn ThreadExecutionHost> = kernel.clone();
        let admission = AgentTreeAdmission::new(root.thread.clone(), AgentTreeLimits::default())?;
        let (handle, publisher) = AgentHandle::channel(root.thread.clone())?;
        let (first_turn_handle, first_turn_publisher) = AgentHandle::channel(root.thread.clone())?;
        let root_thread = root.thread.clone();
        let entry = Entry {
            record: root,
            confirmed: true,
            pending: None,
            reservation: None,
            parent: None,
            policy,
            original_artifact,
            cancellation: cancellation.clone(),
            handle,
            publisher,
            first_turn_handle,
            first_turn_publisher,
            mailbox: VecDeque::new(),
            sequence: 0,
            worker_running: true,
            target: ChildTarget::Local,
        };
        Ok(Self {
            inner: Arc::new(RootHost {
                entries: Mutex::new(BTreeMap::from([(root_thread.thread_id.clone(), entry)])),
                root: root_thread,
                persistence,
                executor,
                kernel,
                events,
                grant: grant.map(std::sync::OnceLock::from).unwrap_or_default(),
                cancellation,
                admission,
                jobs: JobMutex::new(ThreadJobs::default()),
            }),
        })
    }

    fn context(&self, entry: &Entry) -> anyhow::Result<Arc<AgentToolContext>> {
        if !entry.confirmed || entry.pending.is_some() {
            anyhow::bail!("Thread write is unresolved");
        }
        Ok(Arc::new(AgentToolContext::for_turn(
            &entry.record,
            Arc::clone(&entry.policy),
            &entry.original_artifact,
            Arc::new(self.clone()),
            entry.cancellation.clone(),
            self.inner.grant.get().cloned(),
        )?))
    }

    /// Obtain root-bound controls after attachment, for the root kernel only.
    pub async fn root_controls(&self) -> anyhow::Result<Arc<AgentToolContext>> {
        let entries = self.inner.entries.lock().await;
        self.live_root().await?;
        self.context(
            entries
                .get(&self.inner.root.thread_id)
                .ok_or_else(|| anyhow::anyhow!("Root host record missing"))?,
        )
    }

    /// The authenticated actor endpoint is the explicit root-user delegation
    /// decision. Its host checks Cedar before entering here; child tools still
    /// use the root approval channel. Never expose this as a model tool.
    pub(crate) async fn collaborate_from_user(
        &self,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
        request: AgentSpawnRequest,
    ) -> anyhow::Result<AgentThread> {
        request.validate()?;
        self.inner.kernel.check_actor_owner(owner).await?;
        let root = self.live_root().await?;
        let grant = RootDelegationGrant::from_verified_user(&root.thread)?;
        self.inner.grant.get_or_init(|| grant);
        let controls = self.root_controls().await?;
        anyhow::ensure!(
            controls.permits("spawn_agent"),
            "Actor root policy denies delegation"
        );
        self.inner.kernel.admit_host_tool()?;
        let child = controls.spawn(request).await?;
        let handle = {
            let entries = self.inner.entries.lock().await;
            let entry = entries
                .get(&child.thread_id)
                .ok_or_else(|| anyhow::anyhow!("Child disappeared"))?;
            anyhow::ensure!(
                entry.confirmed && entry.pending.is_none(),
                "Child registration is unresolved"
            );
            entry.handle.clone()
        };
        // Waiting for this accepted operation's result is not a model-selected
        // wait_agents call. Dropping the HTTP waiter leaves the child root-owned.
        Ok(handle.wait_until_terminal().await?)
    }

    async fn live_root(&self) -> anyhow::Result<PersistedAgentThread> {
        if self.inner.cancellation.is_cancelled() {
            anyhow::bail!("Root run cancelled");
        }
        let root = self
            .inner
            .persistence
            .load_agent_thread(&self.inner.root.owner_id, &self.inner.root.thread_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Root thread is unavailable"))?;
        root.validate(&self.inner.root.owner_id)?;
        if root.thread.root_run_id != self.inner.root.root_run_id
            || root.thread.run_id != self.inner.root.run_id
            || !live(root.thread.status)
        {
            anyhow::bail!("Root run is no longer active");
        }
        Ok(root)
    }

    async fn authorize(
        &self,
        entries: &BTreeMap<String, Entry>,
        scope: &AgentControlScope,
        operation: Option<&str>,
    ) -> anyhow::Result<PersistedAgentThread> {
        self.live_root().await?;
        let caller = scope.caller();
        if caller.owner_id != self.inner.root.owner_id
            || caller.root_run_id != self.inner.root.root_run_id
            || caller.root_thread_id != self.inner.root.thread_id
        {
            anyhow::bail!("Foreign thread host scope");
        }
        let entry = entries
            .get(&caller.thread_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown caller thread"))?;
        if !std::ptr::eq(scope.policy(), entry.policy.as_ref())
            || entry.record.thread.run_id != caller.run_id
            || entry.cancellation.is_cancelled()
        {
            anyhow::bail!("Stale thread host scope");
        }
        let context = self.context(entry)?;
        if operation.is_some_and(|name| !context.permits(name)) {
            anyhow::bail!("Agent operation not authorized");
        }
        let stored = self
            .inner
            .persistence
            .load_agent_thread(&caller.owner_id, &caller.thread_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Caller thread is unavailable"))?;
        stored.validate(&caller.owner_id)?;
        if stored != entry.record || !live(stored.thread.status) {
            anyhow::bail!("Caller turn changed");
        }
        Ok(stored)
    }

    // A dropped API/tool future abandons its reply, not a half-finished mutation.
    // Jobs remain owned by the root host until they complete.
    fn track(&self, job: impl Future<Output = ()> + Send + 'static) -> anyhow::Result<()> {
        let mut jobs = self
            .inner
            .jobs
            .lock()
            .map_err(|_| anyhow::anyhow!("Thread job registry unavailable"))?;
        anyhow::ensure!(
            !jobs.closed && !self.inner.cancellation.is_cancelled(),
            "Thread host is shutting down"
        );
        jobs.entries.retain(|job| !job.is_joined());
        jobs.entries.push(Arc::new(ThreadJob {
            state: Mutex::new(ThreadJobState {
                handle: Some(tokio::spawn(job)),
                failure: None,
            }),
        }));
        Ok(())
    }

    async fn reap_finished(&self) -> anyhow::Result<()> {
        let finished = {
            let jobs = self
                .inner
                .jobs
                .lock()
                .map_err(|_| anyhow::anyhow!("Thread job registry unavailable"))?;
            jobs.entries
                .iter()
                .filter(|job| job.is_finished())
                .cloned()
                .collect::<Vec<_>>()
        };
        for job in finished {
            if let Err(error) = job.join().await {
                // A panicked host mutation may have stopped between a storage
                // write and its receipt. Do not admit another operation blindly.
                self.inner.cancellation.cancel();
                return Err(error);
            }
        }
        Ok(())
    }

    /// Close admission, cancel the tree, and join every accepted host job.
    /// The root adapter still owns its root record's terminal transition.
    ///
    /// # Errors
    /// Reports a panicked job or unresolved child write after attempting all
    /// cleanup. No uncertain mutation is retried. A cancelled shutdown waiter
    /// leaves join handles in this service for a later caller to finish joining.
    pub async fn shutdown(&self) -> anyhow::Result<()> {
        self.inner.cancellation.cancel();
        let jobs = {
            let mut jobs = self
                .inner
                .jobs
                .lock()
                .map_err(|_| anyhow::anyhow!("Thread job registry unavailable"))?;
            jobs.closed = true;
            jobs.entries.clone()
        };
        let mut failure = None;
        for job in jobs {
            if let Err(error) = job.join().await {
                tracing::error!(%error, "Thread host job failed during shutdown");
                if failure.is_none() {
                    failure = Some(error);
                }
            }
        }
        let remote_children = {
            let entries = self.inner.entries.lock().await;
            entries
                .values()
                .filter_map(|entry| match &entry.target {
                    ChildTarget::Remote(remote) if !remote.reservation.is_released() => {
                        Some(entry.record.thread.thread_id.clone())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        for thread_id in remote_children {
            if let Err(error) = self.close_idle_remote(&thread_id).await {
                tracing::error!(%thread_id, %error, "Remote child cleanup remains unresolved");
                if failure.is_none() {
                    failure = Some(error);
                }
            }
        }
        // All producers have unwound. Finish a persisted pending child whose
        // launch raced closure, or a live record left by a panicked producer.
        let mut entries = self.inner.entries.lock().await;
        for entry in entries.values_mut().filter(|entry| entry.parent.is_some()) {
            entry.cancellation.cancel();
            entry.mailbox.clear();
            if let ChildTarget::Remote(remote) = &entry.target
                && !remote.reservation.is_released()
            {
                let error = anyhow::anyhow!("Remote child cleanup remains unresolved");
                if failure.is_none() {
                    failure = Some(error);
                }
            }
            let settled = async {
                if entry.pending.is_some() {
                    self.confirm(entry).await?;
                }
                if !entry.confirmed {
                    anyhow::bail!("Child registration remains unconfirmed");
                }
                if !entry.record.thread.status.is_terminal() {
                    let mut next = entry.record.thread.clone();
                    next.finish_turn(AgentThreadResult::Cancelled)?;
                    self.persist(entry, next).await?;
                }
                entry.worker_running = false;
                Ok::<(), anyhow::Error>(())
            }
            .await;
            if let Err(error) = settled {
                tracing::error!(thread_id = %entry.record.thread.thread_id, %error,
                    "Child shutdown transition remains unresolved");
                if failure.is_none() {
                    failure = Some(error);
                }
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    async fn tracked<T: Send + 'static>(
        &self,
        operation: impl Future<Output = anyhow::Result<T>> + Send + 'static,
    ) -> anyhow::Result<T> {
        self.reap_finished().await?;
        let (reply, receiver) = oneshot::channel();
        self.track(async move {
            let _ = reply.send(operation.await);
        })?;
        receiver
            .await
            .map_err(|_| anyhow::anyhow!("Thread host operation ended without a receipt"))?
    }

    async fn confirm(&self, entry: &mut Entry) -> anyhow::Result<()> {
        let expected = entry
            .pending
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No pending thread write"))?
            .clone();
        let stored = self
            .inner
            .persistence
            .load_agent_thread(&self.inner.root.owner_id, &expected.thread.thread_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Pending thread write is not confirmed"))?;
        stored.validate(&self.inner.root.owner_id)?;
        if stored != expected {
            anyhow::bail!("Stored thread differs from pending transition");
        }
        if !entry.confirmed {
            let parent = entry
                .parent
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing child parent"))?;
            let edge = AgentEdge::between(&parent.thread, &stored.thread)?;
            if !self
                .inner
                .persistence
                .list_agent_edges(&self.inner.root.owner_id, &self.inner.root.root_run_id)
                .await?
                .contains(&edge)
            {
                anyhow::bail!("Child edge write is not confirmed");
            }
        }
        self.publish(entry, stored).await
    }

    async fn publish(&self, entry: &mut Entry, stored: PersistedAgentThread) -> anyhow::Result<()> {
        let event = match &entry.parent {
            Some(parent) => stored.lifecycle_event(
                &self.inner.root.owner_id,
                parent,
                entry.confirmed.then_some(&entry.record),
            )?,
            None => None,
        };
        entry.record = stored;
        entry.confirmed = true;
        entry.pending = None;
        let _ = entry.publisher.send(entry.record.thread.clone());
        // Graph delegation consumes the first invocation, not whichever
        // follow-up happens to be latest when its waiter next gets scheduled.
        // Keep this one receipt after later turns replace the ordinary watch.
        if !entry.first_turn_publisher.borrow().status.is_terminal() {
            let _ = entry.first_turn_publisher.send(entry.record.thread.clone());
        }
        if let Some(event) = event {
            self.inner.events.emit(event).await;
        }
        Ok(())
    }

    async fn persist(&self, entry: &mut Entry, next: AgentThread) -> anyhow::Result<()> {
        if entry.pending.is_some() {
            anyhow::bail!("Previous thread write is unresolved");
        }
        let parent = entry
            .parent
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Root writes belong to the root adapter"))?;
        let edge = AgentEdge::between(&parent.thread, &next)?;
        let expected = if entry.confirmed {
            agent_threads::next_record(
                &self.inner.root.owner_id,
                &entry.record,
                entry.record.revision,
                &next,
            )?
        } else {
            let root = self.live_root().await?;
            agent_threads::new_child(&self.inner.root.owner_id, &next, &edge, parent, &root)?
        };
        entry.pending = Some(expected.clone());
        let result = if entry.confirmed {
            self.inner
                .persistence
                .update_agent_thread(&self.inner.root.owner_id, entry.record.revision, &next)
                .await
        } else {
            let reservation = entry
                .reservation
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("Child reservation missing"))?;
            reservation.begin_persistence();
            self.inner
                .persistence
                .create_agent_child(&self.inner.root.owner_id, &next, &edge)
                .await
        };
        match result {
            Ok(stored) if stored == expected => self.publish(entry, stored).await,
            Ok(_) => anyhow::bail!("Thread store returned a different transition"),
            Err(error) => self.confirm(entry).await.map_err(|read_error| {
                error.context(format!("Thread write remains unresolved: {read_error}"))
            }),
        }
    }

    async fn spawn_inner(
        &self,
        scope: AgentControlScope,
        request: AgentSpawnRequest,
    ) -> anyhow::Result<AgentThread> {
        request.validate()?;
        let mut entries = self.inner.entries.lock().await;
        let parent = self
            .authorize(&entries, &scope, Some("spawn_agent"))
            .await?;
        let artifact = self
            .inner
            .executor
            .artifact(&parent.thread.owner_id, &request.artifact_id)
            .await?;
        if artifact.id != request.artifact_id {
            anyhow::bail!("Executor returned another artifact");
        }
        let policy = Arc::new(scope.policy().intersect(&artifact)?);
        self.inner.kernel.check_budget(&policy).await?;
        self.inner.executor.check_admission(&policy).await?;
        let history = if matches!(
            request.history_fork,
            HistoryForkMode::None | HistoryForkMode::LastTurns(0)
        ) {
            Vec::new()
        } else {
            self.inner.executor.history(&parent.thread).await?
        };
        let messages = request.initial_messages(&history)?;
        // Resolution/history I/O cannot preserve a revoked grant or stale turn.
        self.authorize(&entries, &scope, Some("spawn_agent"))
            .await?;
        // History and adapter preflight await other work. Recheck shared usage
        // immediately before reserving a child, not only before that I/O.
        self.inner.kernel.check_budget(&policy).await?;
        let child = AgentThread::child(
            &parent.thread,
            artifact.id.clone(),
            request.task_name.as_deref(),
        )?;
        let reservation = self.inner.admission.reserve_child(&child)?;
        let (handle, publisher) = AgentHandle::channel(child.clone())?;
        let (first_turn_handle, first_turn_publisher) = AgentHandle::channel(child.clone())?;
        let parent_cancel = entries
            .get(&parent.thread.thread_id)
            .ok_or_else(|| anyhow::anyhow!("Parent disappeared"))?
            .cancellation
            .clone();
        let entry = Entry {
            record: PersistedAgentThread {
                thread: child.clone(),
                revision: 0,
            },
            confirmed: false,
            pending: None,
            reservation: Some(reservation),
            parent: Some(parent),
            policy,
            original_artifact: artifact,
            cancellation: parent_cancel.child_token(),
            handle,
            publisher,
            first_turn_handle,
            first_turn_publisher,
            mailbox: VecDeque::new(),
            sequence: 0,
            worker_running: false,
            target: ChildTarget::Local,
        };
        entries.insert(child.thread_id.clone(), entry);
        let entry = entries
            .get_mut(&child.thread_id)
            .ok_or_else(|| anyhow::anyhow!("Child registration missing"))?;
        if let Err(error) = self.persist(entry, child.clone()).await {
            // Local validation can fail before the persistence boundary. Only
            // that case proves no write and permits dropping the reservation.
            if !entry.confirmed && entry.pending.is_none() {
                entries.remove(&child.thread_id);
            }
            return Err(error);
        }
        let permit = entry
            .reservation
            .take()
            .ok_or_else(|| anyhow::anyhow!("Child reservation missing"))?
            .commit()?;
        self.launch(entry, messages, permit)?;
        Ok(child)
    }

    async fn spawn_remote_inner(
        &self,
        scope: AgentControlScope,
        request: RemoteAgentSpawnRequest,
    ) -> anyhow::Result<AgentThread> {
        request.validate()?;
        let mut entries = self.inner.entries.lock().await;
        let parent = self
            .authorize(&entries, &scope, Some("spawn_agent"))
            .await?;
        let target_agent_id = scope
            .policy()
            .remote_agent_for_endpoint(&request.endpoint)?;
        let peer = self
            .inner
            .kernel
            .trusted_a2a_peer(&request.endpoint, &target_agent_id)?;
        let (policy, contract_policy, requested_budgets, sandbox) =
            scope.policy().for_remote_child(&target_agent_id)?;
        let policy = Arc::new(policy);
        self.inner.kernel.check_budget(&policy).await?;
        self.authorize(&entries, &scope, Some("spawn_agent"))
            .await?;
        self.inner.kernel.check_budget(&policy).await?;
        let child = AgentThread::child(
            &parent.thread,
            target_agent_id.clone(),
            request.task_name.as_deref(),
        )?;
        let reservation = self.inner.admission.reserve_child(&child)?;
        let (handle, publisher) = AgentHandle::channel(child.clone())?;
        let (first_turn_handle, first_turn_publisher) = AgentHandle::channel(child.clone())?;
        let parent_cancel = entries
            .get(&parent.thread.thread_id)
            .ok_or_else(|| anyhow::anyhow!("Parent disappeared"))?
            .cancellation
            .clone();
        let messages = vec![Message {
            role: MessageRole::User,
            content: MessageContent::text(request.delegated_prompt),
            tool_call_id: None,
            tool_calls: None,
        }];
        // Complete fallible local admission before leasing peer capacity. No
        // remote operation exists until run_turn publishes its owned execution.
        let budget_reservation = self
            .inner
            .kernel
            .reserve_remote_budget(&requested_budgets)?;
        let presentation_negotiation = self.inner.kernel.remote_presentation_negotiation(&policy);
        let contract = crate::uar::api::a2a::contract::UarDelegationContract {
            version: crate::uar::api::a2a::contract::UAR_DELEGATION_CONTRACT_VERSION,
            source_instance_id: self.inner.kernel.a2a_instance_id().to_owned(),
            target_instance_id: peer.instance_id.clone(),
            owner_id: parent.thread.owner_id.clone(),
            root_run_id: parent.thread.root_run_id.clone(),
            parent_thread_id: parent.thread.thread_id.clone(),
            child_thread_id: child.thread_id.clone(),
            target_agent_id,
            policy: crate::uar::api::a2a::contract::UarDelegationPolicy::for_peer(
                contract_policy,
                &presentation_negotiation,
            ),
            budgets: requested_budgets,
            usage_grant: budget_reservation.grant().clone(),
            sandbox,
            presentation_negotiation,
        };
        if let Err(error) = contract.validate() {
            budget_reservation.release_confirmed()?;
            return Err(error);
        }
        let entry = Entry {
            record: PersistedAgentThread {
                thread: child.clone(),
                revision: 0,
            },
            confirmed: false,
            pending: None,
            reservation: Some(reservation),
            parent: Some(parent),
            policy: Arc::clone(&policy),
            original_artifact: policy.artifact().clone(),
            cancellation: parent_cancel.child_token(),
            handle,
            publisher,
            first_turn_handle,
            first_turn_publisher,
            mailbox: VecDeque::new(),
            sequence: 0,
            worker_running: false,
            target: ChildTarget::Remote(RemoteChild {
                peer,
                contract,
                reservation: budget_reservation,
                execution: None,
                execution_admitted: false,
            }),
        };
        entries.insert(child.thread_id.clone(), entry);
        let entry = entries
            .get_mut(&child.thread_id)
            .ok_or_else(|| anyhow::anyhow!("Remote child registration missing"))?;
        if let Err(error) = self.persist(entry, child.clone()).await {
            if let ChildTarget::Remote(remote) = &entry.target {
                remote.release_if_never_dispatched()?;
            }
            if !entry.confirmed && entry.pending.is_none() {
                entries.remove(&child.thread_id);
            }
            return Err(error);
        }
        let permit = entry
            .reservation
            .take()
            .ok_or_else(|| anyhow::anyhow!("Remote child reservation missing"))?
            .commit()?;
        if let Err(error) = self.launch(entry, messages, permit) {
            if let ChildTarget::Remote(remote) = &entry.target {
                remote.release_if_never_dispatched()?;
            }
            return Err(error);
        }
        Ok(child)
    }

    fn launch(
        &self,
        entry: &mut Entry,
        messages: Vec<Message>,
        permit: ActiveChildPermit,
    ) -> anyhow::Result<()> {
        let service = self.clone();
        let thread_id = entry.record.thread.thread_id.clone();
        self.track(async move {
            service.run_child(thread_id, messages, permit).await;
        })?;
        entry.worker_running = true;
        Ok(())
    }

    async fn run_child(
        &self,
        thread_id: String,
        mut messages: Vec<Message>,
        _permit: ActiveChildPermit,
    ) {
        loop {
            let outcome = self.run_turn(&thread_id, messages).await;
            let mut entries = self.inner.entries.lock().await;
            let Some(entry) = entries.get_mut(&thread_id) else {
                return;
            };
            entry.worker_running = false;
            if let Err(error) = outcome {
                tracing::error!(%thread_id, %error, "Child thread host operation failed");
                if let ChildTarget::Remote(remote) = &entry.target
                    && let Err(error) = remote.release_if_never_dispatched()
                {
                    tracing::error!(%thread_id, %error, "Undispatched remote budget release failed");
                }
                if entry.pending.is_none() && !entry.record.thread.status.is_terminal() {
                    let mut next = entry.record.thread.clone();
                    let result = if entry.cancellation.is_cancelled() {
                        AgentThreadResult::Cancelled
                    } else {
                        AgentThreadResult::Failed {
                            code: "child_turn_rejected".into(),
                            message: "Child turn could not execute".into(),
                        }
                    };
                    if let Err(error) = async {
                        next.finish_turn(result)?;
                        self.persist(entry, next).await
                    }
                    .await
                    {
                        tracing::error!(%thread_id, %error, "Child failure transition remains unresolved");
                    }
                }
                return;
            }
            if entry.cancellation.is_cancelled()
                || !entry.mailbox.iter().any(|message| message.trigger_turn)
            {
                return;
            }
            // Retain the same child slot while draining accepted triggers.
            // Releasing and reacquiring here could strand an acknowledged
            // message if another child took the newly freed slot.
            messages = take_triggered_messages(&mut entry.mailbox);
            entry.worker_running = true;
        }
    }

    async fn run_turn(&self, thread_id: &str, mut messages: Vec<Message>) -> anyhow::Result<()> {
        let work = {
            let mut entries = self.inner.entries.lock().await;
            let entry = entries
                .get_mut(thread_id)
                .ok_or_else(|| anyhow::anyhow!("Child registration missing"))?;
            if entry.cancellation.is_cancelled()
                && entry.record.thread.status == AgentThreadStatus::Pending
            {
                if let ChildTarget::Remote(remote) = &entry.target {
                    remote.release_if_never_dispatched()?;
                }
                let mut next = entry.record.thread.clone();
                next.finish_turn(AgentThreadResult::Cancelled)?;
                self.persist(entry, next).await?;
                return Ok(());
            }
            let previous = entry.record.thread.clone();
            // An accepted trigger owns a new turn even when admission/history
            // fails. Publish that turn before preflight so its failure cannot
            // be mistaken for the preceding turn's successful result.
            let mut next = previous.clone();
            next.begin_turn(uuid::Uuid::new_v4().to_string())?;
            self.persist(entry, next).await?;
            self.live_root().await?;
            if entry.cancellation.is_cancelled() {
                anyhow::bail!("Child cancelled before kernel entry");
            }
            match &mut entry.target {
                ChildTarget::Local => {
                    self.inner.kernel.check_budget(&entry.policy).await?;
                    self.inner.executor.check_admission(&entry.policy).await?;
                    if previous.status.is_terminal() {
                        // A resumed child keeps its own canonical tool-paired history.
                        // Parent fork filtering applies only to its initial delegation.
                        let mut history = self.inner.executor.history(&previous).await?;
                        history.extend(messages);
                        messages = history;
                    }
                    self.live_root().await?;
                    if entry.cancellation.is_cancelled() {
                        anyhow::bail!("Child cancelled before kernel entry");
                    }
                    self.inner.kernel.check_budget(&entry.policy).await?;
                    TurnWork::Local(HostedThreadTurn {
                        record: entry.record.clone(),
                        policy: Arc::clone(&entry.policy),
                        original_artifact: entry.original_artifact.clone(),
                        messages,
                        controls: self.context(entry)?,
                        cancellation: entry.cancellation.clone(),
                    })
                }
                ChildTarget::Remote(remote) => {
                    remote.reservation.check_active()?;
                    let content = remote_message_text(&messages)?;
                    self.live_root().await?;
                    if entry.cancellation.is_cancelled() {
                        anyhow::bail!("Remote child cancelled before A2A entry");
                    }
                    anyhow::ensure!(
                        remote.execution.is_none(),
                        "Remote child has an unresolved prior operation"
                    );
                    let message = crate::uar::api::a2a::types::Message::user_text(content);
                    let execution = match &entry.record.thread.remote {
                        Some(binding) => {
                            anyhow::ensure!(
                                binding.target_instance_id == remote.peer.instance_id
                                    && binding.endpoint == remote.peer.endpoint
                                    && binding.contract_digest == remote.contract.digest()?,
                                "Persisted remote task binding does not match its trusted peer contract"
                            );
                            remote.peer.client.governed_task_execution_for_task(
                                remote.peer.endpoint.clone(),
                                message,
                                remote.contract.clone(),
                                binding.task_id.clone(),
                                binding.context_id.clone(),
                                &entry.cancellation,
                            )?
                        }
                        None => remote.peer.client.governed_task_execution(
                            remote.peer.endpoint.clone(),
                            message,
                            remote.contract.clone(),
                            &entry.cancellation,
                        )?,
                    };
                    let execution = Arc::new(Mutex::new(execution));
                    remote.execution_admitted = true;
                    remote.execution = Some(Arc::clone(&execution));
                    TurnWork::Remote {
                        execution,
                        reservation: remote.reservation.clone(),
                    }
                }
            }
        };
        let (result, remote_settled, remote_cleanup) = match work {
            TurnWork::Local(turn) => {
                let executor = Arc::clone(&self.inner.executor);
                // Supervise the executor: a panicking/dropped producer cannot be
                // mistaken for empty success or leave a permanently running child.
                let result = match tokio::spawn(async move { executor.execute(turn).await }).await {
                    Ok(Ok(result)) => result,
                    Ok(Err(error)) => {
                        tracing::error!(%thread_id, %error, "Child kernel failed");
                        AgentThreadResult::Failed {
                            code: "child_kernel_failed".into(),
                            message: "Child kernel execution failed".into(),
                        }
                    }
                    Err(error) => {
                        tracing::error!(%thread_id, %error, "Child kernel producer disappeared");
                        AgentThreadResult::Failed {
                            code: "child_kernel_closed".into(),
                            message: "Child kernel ended without completion".into(),
                        }
                    }
                };
                (result, true, false)
            }
            TurnWork::Remote {
                execution,
                reservation,
            } => {
                let started = execution.lock().await.start().await;
                if let Ok(task) = &started {
                    let context_id = task.context_id.clone().ok_or_else(|| {
                        anyhow::anyhow!("Remote UAR task receipt omitted its context identity")
                    })?;
                    let (target_instance_id, endpoint, contract_digest) = {
                        let entries = self.inner.entries.lock().await;
                        let entry = entries
                            .get(thread_id)
                            .ok_or_else(|| anyhow::anyhow!("Child registration missing"))?;
                        let ChildTarget::Remote(remote) = &entry.target else {
                            anyhow::bail!("Remote task was rebound to a local child");
                        };
                        (
                            remote.peer.instance_id.clone(),
                            remote.peer.endpoint.clone(),
                            remote.contract.digest()?,
                        )
                    };
                    let binding = RemoteThreadBinding {
                        target_instance_id,
                        endpoint,
                        task_id: task.id.clone(),
                        context_id,
                        contract_digest,
                    };
                    let mut entries = self.inner.entries.lock().await;
                    let entry = entries
                        .get_mut(thread_id)
                        .ok_or_else(|| anyhow::anyhow!("Child registration missing"))?;
                    if entry.record.thread.remote.as_ref() != Some(&binding) {
                        let mut next = entry.record.thread.clone();
                        next.bind_remote(binding)?;
                        self.persist(entry, next).await?;
                    }
                }
                let outcome = match started {
                    Ok(_) => execution.lock().await.execute().await,
                    Err(error) => Err(error),
                };
                match outcome {
                    Ok(task) => {
                        let result = self.remote_task_result(&task, &reservation);
                        match result {
                            Ok(result) => {
                                let execution = execution.lock().await;
                                (
                                    result,
                                    execution.terminal_confirmed(),
                                    execution.cleanup_confirmed(),
                                )
                            }
                            Err(error) => {
                                tracing::error!(%thread_id, %error, "Remote child receipt was rejected");
                                (
                                    AgentThreadResult::Failed {
                                        code: "remote_receipt_invalid".into(),
                                        message:
                                            "Remote child returned an invalid terminal receipt"
                                                .into(),
                                    },
                                    false,
                                    false,
                                )
                            }
                        }
                    }
                    Err(crate::uar::api::a2a::task_execution::A2AExecutionError::NotStarted) => {
                        (AgentThreadResult::Cancelled, true, true)
                    }
                    Err(error) => {
                        tracing::error!(%thread_id, %error, "Remote child cleanup is unconfirmed");
                        (
                            AgentThreadResult::Failed {
                                code: "remote_cleanup_unconfirmed".into(),
                                message: "Remote child cleanup remains unconfirmed".into(),
                            },
                            false,
                            false,
                        )
                    }
                }
            }
        };
        let mut entries = self.inner.entries.lock().await;
        let entry = entries
            .get_mut(thread_id)
            .ok_or_else(|| anyhow::anyhow!("Child registration missing"))?;
        if remote_settled && let ChildTarget::Remote(remote) = &mut entry.target {
            let cancelled = entry.cancellation.is_cancelled();
            if !cancelled || remote_cleanup {
                remote.execution = None;
            }
            if cancelled && remote_cleanup {
                remote.reservation.release_confirmed()?;
            }
        }
        let mut next = entry.record.thread.clone();
        next.finish_turn(result)?;
        self.persist(entry, next).await
    }

    fn remote_task_result(
        &self,
        task: &crate::uar::api::a2a::types::Task,
        reservation: &crate::uar::runtime::cost_budget::RemoteBudgetReservation,
    ) -> anyhow::Result<AgentThreadResult> {
        let usage = remote_usage(task)?;
        reservation.record_cumulative(usage)?;
        Ok(match task.status.state {
            crate::uar::api::a2a::types::TaskState::Completed => {
                let output = remote_task_text(task)
                    .ok_or_else(|| anyhow::anyhow!("Remote UAR task completed without output"))?;
                AgentThreadResult::Completed { output }
            }
            crate::uar::api::a2a::types::TaskState::Canceled => AgentThreadResult::Cancelled,
            crate::uar::api::a2a::types::TaskState::Failed => AgentThreadResult::Failed {
                code: "remote_child_failed".into(),
                message: "Remote child execution failed".into(),
            },
            _ => anyhow::bail!("Remote UAR task receipt is not terminal"),
        })
    }

    /// Recover only an exact committed write. A start recovered after its
    /// failed request is closed as failed/cancelled, never replayed. Missing or
    /// different reads preserve uncertainty and do not release lifetime quota.
    pub async fn reconcile(&self, thread_id: &str) -> anyhow::Result<AgentThread> {
        let service = self.clone();
        let thread_id = thread_id.to_string();
        self.tracked(async move {
            let mut entries = service.inner.entries.lock().await;
            let entry = entries
                .get_mut(&thread_id)
                .ok_or_else(|| anyhow::anyhow!("Thread unavailable"))?;
            if entry.worker_running {
                anyhow::bail!("Child worker still owns this transition");
            }
            if entry.pending.is_some() {
                service.confirm(entry).await?;
            }
            if !entry.confirmed {
                anyhow::bail!("Child creation remains unconfirmed");
            }
            if let Some(reservation) = entry.reservation.take() {
                drop(reservation.commit()?);
            }
            if !entry.record.thread.status.is_terminal() {
                let mut next = entry.record.thread.clone();
                next.finish_turn(if entry.cancellation.is_cancelled() {
                    AgentThreadResult::Cancelled
                } else {
                    AgentThreadResult::Failed {
                        code: "child_not_started".into(),
                        message: "Child could not start while its registration was unconfirmed"
                            .into(),
                    }
                })?;
                service.persist(entry, next).await?;
            }
            Ok(entry.record.thread.clone())
        })
        .await
    }

    /// Drain root messages at a root-kernel boundary. Identity stays in the
    /// envelope; the kernel appends bodies without identity prompt prefixes.
    pub async fn take_root_messages(&self) -> anyhow::Result<Vec<InterAgentMessage>> {
        self.live_root().await?;
        let mut entries = self.inner.entries.lock().await;
        let entry = entries
            .get_mut(&self.inner.root.thread_id)
            .ok_or_else(|| anyhow::anyhow!("Root record missing"))?;
        Ok(entry.mailbox.drain(..).collect())
    }

    /// Publish a root transition only after the root adapter has committed it.
    /// A completed root closes descendant work; its old authority cannot be
    /// carried into another root turn through an already-captured tool context.
    pub async fn refresh_root(&self) -> anyhow::Result<AgentThread> {
        let mut entries = self.inner.entries.lock().await;
        let stored = self
            .inner
            .persistence
            .load_agent_thread(&self.inner.root.owner_id, &self.inner.root.thread_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Root record unavailable"))?;
        stored.validate(&self.inner.root.owner_id)?;
        if stored.thread.run_id != self.inner.root.run_id {
            anyhow::bail!("Root turn was replaced");
        }
        let entry = entries
            .get_mut(&self.inner.root.thread_id)
            .ok_or_else(|| anyhow::anyhow!("Root record missing"))?;
        if stored != entry.record {
            let expected = agent_threads::next_record(
                &self.inner.root.owner_id,
                &entry.record,
                entry.record.revision,
                &stored.thread,
            )?;
            if expected != stored {
                anyhow::bail!("Root transition revision was skipped");
            }
            entry.record = stored.clone();
            let _ = entry.publisher.send(stored.thread.clone());
        }
        if stored.thread.status.is_terminal() {
            for child in entries.values_mut().filter(|entry| entry.parent.is_some()) {
                child.cancellation.cancel();
                child.mailbox.clear();
            }
        }
        Ok(stored.thread)
    }

    async fn send_inner(
        &self,
        scope: AgentControlScope,
        request: SendAgentMessageRequest,
    ) -> anyhow::Result<InterAgentMessage> {
        if request.content.trim().is_empty() {
            anyhow::bail!("Empty agent message");
        }
        let mut entries = self.inner.entries.lock().await;
        let sender = self
            .authorize(&entries, &scope, Some("send_agent_message"))
            .await?;
        let entry = entries
            .get(&request.recipient_thread_id)
            .ok_or_else(|| anyhow::anyhow!("Recipient unavailable"))?;
        if !entry.confirmed || entry.pending.is_some() || entry.cancellation.is_cancelled() {
            anyhow::bail!("Recipient cannot accept messages");
        }
        if matches!(&entry.target, ChildTarget::Remote(remote) if remote.execution.is_some()) {
            anyhow::bail!("Remote child has an unresolved prior operation");
        }
        let permit = if request.trigger_turn && !entry.worker_running && entry.parent.is_some() {
            match &entry.target {
                ChildTarget::Local => {
                    self.inner.executor.check_admission(&entry.policy).await?;
                    self.inner.kernel.check_budget(&entry.policy).await?;
                }
                ChildTarget::Remote(remote) => remote.reservation.check_active()?,
            }
            Some(
                self.inner
                    .admission
                    .reserve_turn(&entry.record.thread.thread_id)?,
            )
        } else {
            None
        };
        self.authorize(&entries, &scope, Some("send_agent_message"))
            .await?;
        let entry = entries
            .get_mut(&request.recipient_thread_id)
            .ok_or_else(|| anyhow::anyhow!("Recipient unavailable"))?;
        if entry.cancellation.is_cancelled() {
            anyhow::bail!("Recipient cancelled before delivery");
        }
        if request.trigger_turn && entry.parent.is_some() {
            match &entry.target {
                ChildTarget::Local => self.inner.kernel.check_budget(&entry.policy).await?,
                ChildTarget::Remote(remote) => remote.reservation.check_active()?,
            }
        }
        let sequence = entry
            .sequence
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("Mailbox sequence exhausted"))?;
        let message = InterAgentMessage::between(
            &sender.thread,
            &entry.record.thread,
            sequence,
            request.content,
            request.trigger_turn,
        )?;
        let previous_mailbox = entry.mailbox.clone();
        let previous_sequence = entry.sequence;
        entry.sequence = sequence;
        entry.mailbox.push_back(message.clone());
        if let Some(permit) = permit {
            let messages = take_triggered_messages(&mut entry.mailbox);
            if let Err(error) = self.launch(entry, messages, permit) {
                entry.mailbox = previous_mailbox;
                entry.sequence = previous_sequence;
                return Err(error);
            }
        }
        Ok(message)
    }

    async fn close_idle_remote(&self, thread_id: &str) -> anyhow::Result<()> {
        let (execution, reservation) = {
            let mut entries = self.inner.entries.lock().await;
            let entry = entries
                .get_mut(thread_id)
                .ok_or_else(|| anyhow::anyhow!("Remote child disappeared during cancellation"))?;
            let ChildTarget::Remote(remote) = &mut entry.target else {
                return Ok(());
            };
            if remote.reservation.is_released() {
                return Ok(());
            }
            if !remote.execution_admitted {
                return remote.release_if_never_dispatched();
            }
            let execution = match &remote.execution {
                Some(execution) => Arc::clone(execution),
                None => {
                    let binding = entry.record.thread.remote.clone().ok_or_else(|| {
                        anyhow::anyhow!("Remote child has no confirmed task identity")
                    })?;
                    let execution =
                        Arc::new(Mutex::new(remote.peer.client.governed_task_cleanup(
                            remote.peer.endpoint.clone(),
                            remote.contract.clone(),
                            binding.task_id,
                            &entry.cancellation,
                        )?));
                    remote.execution = Some(Arc::clone(&execution));
                    execution
                }
            };
            (execution, remote.reservation.clone())
        };
        let usage_status = {
            let mut execution = execution.lock().await;
            match execution.cancel_and_wait().await {
                Ok(task) if execution.cleanup_confirmed() => {
                    if reservation.is_released() {
                        Ok(())
                    } else {
                        let status = match remote_usage(&task) {
                            Ok(usage) => reservation.record_cumulative(usage),
                            Err(error) => match reservation.charge_reserved_capacity() {
                                Ok(()) => Err(anyhow::anyhow!(
                                    "Remote usage receipt was unavailable; reserved capacity was charged: {error}"
                                )),
                                Err(charge) => Err(anyhow::anyhow!(
                                    "Remote usage receipt was unavailable and reserved capacity could not be charged: {error}; {charge}"
                                )),
                            },
                        };
                        reservation.release_confirmed()?;
                        status
                    }
                }
                Err(crate::uar::api::a2a::task_execution::A2AExecutionError::NotStarted)
                    if execution.cleanup_confirmed() =>
                {
                    if !reservation.is_released() {
                        reservation.release_confirmed()?;
                    }
                    Ok(())
                }
                Ok(_) => anyhow::bail!("Remote child cleanup receipt is not terminal"),
                Err(error) => return Err(error.into()),
            }
        };
        let mut entries = self.inner.entries.lock().await;
        if let Some(entry) = entries.get_mut(thread_id)
            && let ChildTarget::Remote(remote) = &mut entry.target
            && remote
                .execution
                .as_ref()
                .is_some_and(|stored| Arc::ptr_eq(stored, &execution))
        {
            remote.execution = None;
        }
        usage_status
    }
}

fn remote_usage(
    task: &crate::uar::api::a2a::types::Task,
) -> anyhow::Result<crate::uar::api::a2a::contract::UarUsageReceipt> {
    let usage = task
        .metadata
        .get(crate::uar::api::a2a::contract::UAR_USAGE_METADATA)
        .ok_or_else(|| anyhow::anyhow!("Remote UAR task omitted its usage receipt"))?;
    Ok(serde_json::from_value(usage.clone())?)
}

#[async_trait::async_trait]
impl AgentThreadHost for ThreadService {
    async fn spawn(
        &self,
        scope: &AgentControlScope,
        request: AgentSpawnRequest,
    ) -> anyhow::Result<AgentThread> {
        let service = self.clone();
        let scope = scope.clone();
        self.tracked(async move { service.spawn_inner(scope, request).await })
            .await
    }

    async fn spawn_remote(
        &self,
        scope: &AgentControlScope,
        request: RemoteAgentSpawnRequest,
    ) -> anyhow::Result<AgentThread> {
        let service = self.clone();
        let scope = scope.clone();
        self.tracked(async move { service.spawn_remote_inner(scope, request).await })
            .await
    }

    async fn send_message(
        &self,
        scope: &AgentControlScope,
        request: SendAgentMessageRequest,
    ) -> anyhow::Result<InterAgentMessage> {
        let service = self.clone();
        let scope = scope.clone();
        self.tracked(async move { service.send_inner(scope, request).await })
            .await
    }

    async fn load_thread(
        &self,
        scope: &AgentControlScope,
        thread_id: &str,
    ) -> anyhow::Result<Option<AgentThread>> {
        let entries = self.inner.entries.lock().await;
        self.authorize(&entries, scope, None).await?;
        let Some(entry) = entries.get(thread_id) else {
            return Ok(None);
        };
        if !entry.confirmed || entry.pending.is_some() {
            anyhow::bail!("Thread write unresolved");
        }
        Ok(Some(entry.record.thread.clone()))
    }

    async fn list_threads(&self, scope: &AgentControlScope) -> anyhow::Result<Vec<AgentThread>> {
        let entries = self.inner.entries.lock().await;
        self.authorize(&entries, scope, Some("list_agents")).await?;
        let mut threads = Vec::new();
        for entry in entries.values() {
            if !entry.confirmed || entry.pending.is_some() {
                anyhow::bail!("Tree contains an unresolved write");
            }
            threads.push(entry.record.thread.clone());
        }
        threads.sort_by(|left, right| left.order_key().cmp(&right.order_key()));
        Ok(threads)
    }

    async fn subscribe_thread(
        &self,
        scope: &AgentControlScope,
        thread_id: &str,
    ) -> anyhow::Result<AgentHandle> {
        let entries = self.inner.entries.lock().await;
        self.authorize(&entries, scope, Some("wait_agents")).await?;
        let entry = entries
            .get(thread_id)
            .ok_or_else(|| anyhow::anyhow!("Thread unavailable"))?;
        if !entry.confirmed || entry.pending.is_some() {
            anyhow::bail!("Thread write unresolved");
        }
        Ok(entry.handle.clone())
    }

    async fn subscribe_first_turn(
        &self,
        scope: &AgentControlScope,
        thread_id: &str,
    ) -> anyhow::Result<AgentHandle> {
        let entries = self.inner.entries.lock().await;
        self.authorize(&entries, scope, Some("wait_agents")).await?;
        let entry = entries
            .get(thread_id)
            .ok_or_else(|| anyhow::anyhow!("Thread unavailable"))?;
        let first = entry.first_turn_handle.snapshot()?;
        // A later unconfirmed transition cannot invalidate this already
        // committed receipt. Before the first result, uncertainty still fails
        // closed instead of publishing a completion that storage did not prove.
        anyhow::ensure!(
            entry.confirmed && (first.status.is_terminal() || entry.pending.is_none()),
            "First-turn write unresolved"
        );
        anyhow::ensure!(
            first.parent_thread_id.as_deref() == Some(&scope.caller().thread_id),
            "First-turn receipt belongs to another parent"
        );
        Ok(entry.first_turn_handle.clone())
    }

    async fn interrupt(
        &self,
        scope: &AgentControlScope,
        thread_id: &str,
    ) -> anyhow::Result<AgentInterruptReceipt> {
        let mut entries = self.inner.entries.lock().await;
        self.authorize(&entries, scope, Some("interrupt_agent"))
            .await?;
        let entry = entries
            .get(thread_id)
            .ok_or_else(|| anyhow::anyhow!("Thread unavailable"))?;
        if !entry
            .record
            .thread
            .canonical_path
            .starts_with(&format!("{}/", scope.caller().canonical_path))
        {
            anyhow::bail!("Only descendants can be interrupted");
        }
        if !entry.confirmed || entry.pending.is_some() {
            anyhow::bail!("Thread write unresolved");
        }
        let thread = entry.record.thread.clone();
        let descendant_prefix = format!("{}/", thread.canonical_path);
        let mut requested = false;
        let mut idle_remote = Vec::new();
        for target in entries.values_mut().filter(|entry| {
            entry.record.thread.thread_id == thread_id
                || entry
                    .record
                    .thread
                    .canonical_path
                    .starts_with(&descendant_prefix)
        }) {
            let remote_needs_close = matches!(&target.target, ChildTarget::Remote(remote)
                if remote.execution.is_none()
                    && remote.reservation.check_active().is_ok());
            requested |= !target.record.thread.status.is_terminal()
                || !target.mailbox.is_empty()
                || remote_needs_close;
            target.cancellation.cancel();
            target.mailbox.clear();
            if remote_needs_close {
                idle_remote.push(target.record.thread.thread_id.clone());
            }
        }
        drop(entries);
        for remote_thread_id in idle_remote {
            self.close_idle_remote(&remote_thread_id).await?;
        }
        Ok(AgentInterruptReceipt {
            thread,
            cancellation_requested: requested,
        })
    }
}

fn take_triggered_messages(mailbox: &mut VecDeque<InterAgentMessage>) -> Vec<Message> {
    let mut messages = Vec::new();
    while let Some(message) = mailbox.pop_front() {
        messages.push(message.user_message());
        if message.trigger_turn {
            break;
        }
    }
    messages
}

fn remote_message_text(messages: &[Message]) -> anyhow::Result<String> {
    let mut parts = Vec::with_capacity(messages.len());
    for message in messages {
        anyhow::ensure!(
            message.role == MessageRole::User
                && message.tool_call_id.is_none()
                && message.tool_calls.as_ref().is_none_or(Vec::is_empty),
            "Remote A2A delegation accepts body-only user messages"
        );
        let text = match &message.content {
            MessageContent::Text { content } => content,
            MessageContent::Parts { .. } => {
                anyhow::bail!("Remote A2A delegation does not accept multimodal history")
            }
        };
        anyhow::ensure!(
            !text.trim().is_empty(),
            "Remote A2A delegation message is empty"
        );
        parts.push(text.as_str());
    }
    anyhow::ensure!(!parts.is_empty(), "Remote A2A delegation has no message");
    Ok(parts.join("\n"))
}

fn remote_task_text(task: &crate::uar::api::a2a::types::Task) -> Option<String> {
    use crate::uar::api::a2a::types::{Part, Role};

    let text_from_parts = |parts: &[Part]| {
        let text = parts
            .iter()
            .filter_map(|part| match part {
                Part::Text { text } => Some(text.as_str()),
                Part::File { .. } | Part::Data { .. } => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        (!text.trim().is_empty()).then_some(text)
    };
    task.status
        .message
        .as_ref()
        .and_then(|message| text_from_parts(&message.parts))
        .or_else(|| {
            task.history
                .iter()
                .rev()
                .find(|message| message.role == Role::Agent)
                .and_then(|message| text_from_parts(&message.parts))
        })
        .or_else(|| {
            task.artifacts
                .iter()
                .rev()
                .find_map(|artifact| text_from_parts(&artifact.parts))
        })
}

fn live(status: AgentThreadStatus) -> bool {
    matches!(
        status,
        AgentThreadStatus::Running | AgentThreadStatus::Waiting
    )
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
