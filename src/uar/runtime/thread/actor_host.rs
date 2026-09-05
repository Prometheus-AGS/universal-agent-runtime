//! Trusted-host adapter for a user-addressed actor session. Root records are
//! committed before kernel entry; terminal records before mailbox replies.

use std::sync::Arc;

use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use crate::uar::domain::artifact::AgentArtifact;
use crate::uar::persistence::{
    PersistenceLayer,
    agent_threads::{self, PersistedAgentThread},
};
use crate::uar::runtime::{manager::RunManager, turn::RunExecutionRequest};

use super::{AgentThread, AgentThreadResult};

/// Host-owned root handoff. The actor retains this through producer completion
/// so even an assembly failure leaves an attached tree available for cleanup.
#[derive(Clone)]
pub(crate) struct ActorRootBinding {
    pub(crate) record: PersistedAgentThread,
    pub(crate) artifacts: Option<super::artifacts::RunArtifactCollector>,
    pub(crate) graph_tools:
        Arc<std::sync::OnceLock<Arc<crate::uar::runtime::graph::tools::GraphToolHost>>>,
    pub(crate) sandbox: Arc<
        std::sync::OnceLock<(
            Arc<crate::sandbox::execution::SandboxSupervisor>,
            crate::sandbox::execution::SandboxRun,
        )>,
    >,
    pub(crate) terminal: Arc<
        std::sync::OnceLock<(
            Arc<crate::uar::tools::terminal_process::TerminalSupervisor>,
            crate::uar::tools::terminal_process::TerminalRun,
        )>,
    >,
    pub(crate) persistence: Arc<dyn PersistenceLayer>,
    pub(crate) service: Arc<std::sync::OnceLock<Arc<super::service::ThreadService>>>,
    pub(crate) ready: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) producer: Arc<tokio::sync::Mutex<ActorProducer>>,
}

#[derive(Default)]
pub(crate) struct ActorProducer {
    pub(crate) handle: Option<tokio::task::JoinHandle<()>>,
    failure: Option<String>,
}

impl ActorRootBinding {
    pub(crate) async fn shutdown(&self) -> anyhow::Result<()> {
        self.ready
            .store(false, std::sync::atomic::Ordering::Release);
        // Retain every exact scope after a failed cleanup. A terminal producer
        // receipt alone cannot prove its sandbox/process resources were reaped.
        // Attempt every family even if an earlier one remains unresolved.
        let mut failure = None;
        if let Some(host) = self.graph_tools.get() {
            if let Err(error) = host.shutdown().await {
                failure.get_or_insert(error);
            }
        }
        if let Some((host, scope)) = self.terminal.get() {
            if let Err(error) = host.finish_run(scope).await {
                failure.get_or_insert(anyhow::Error::from(error));
            }
        }
        if let Some(service) = self.service.get() {
            if let Err(error) = service.shutdown().await {
                failure.get_or_insert(error);
            }
        }
        if let Some((host, scope)) = self.sandbox.get() {
            if let Err(error) = host.finish_run(scope).await {
                failure.get_or_insert(anyhow::Error::from(error));
            }
        }
        failure.map_or(Ok(()), Err)
    }

    /// Called by the actor/registry, never by its own executing producer. Keep
    /// the handle in its slot across await so an abandoned stop is resumable.
    pub(crate) async fn finish(&self) -> anyhow::Result<()> {
        let joined = {
            let mut producer = self.producer.lock().await;
            if let Some(handle) = producer.handle.as_mut() {
                let outcome = handle.await;
                producer.handle = None;
                producer.failure = outcome.err().map(|error| error.to_string());
            }
            match &producer.failure {
                Some(error) => Err(anyhow::anyhow!("Actor root producer failed: {error}")),
                None => Ok(()),
            }
        };
        let cleaned = self.shutdown().await;
        joined.and(cleaned)
    }
}

/// Serialized by the actor mailbox. Conversation history is owned by the run
/// kernel's session, not a second vector in the actor.
pub(crate) struct ActorThreadSession {
    owner: crate::uar::runtime::actor::messages::ActorOwner,
    artifact: AgentArtifact,
    session_id: String,
    manager: Arc<RunManager>,
    persistence: Arc<dyn PersistenceLayer>,
    cancellation: CancellationToken,
    current: Option<PersistedAgentThread>,
    /// An uncertain write cannot authorize another turn or a blind retry.
    uncertain: Option<PersistedAgentThread>,
    state: watch::Sender<Option<PersistedAgentThread>>,
    owned_root: Arc<tokio::sync::Mutex<Option<ActorRootBinding>>>,
    remote_constraints: Option<RemoteRootConstraints>,
}

#[derive(Clone)]
pub(crate) struct RemoteRootConstraints {
    pub(crate) presentation_negotiation:
        Option<crate::uar::a2ui::presentation_selection::PresentationNegotiation>,
    pub(crate) policy: crate::uar::domain::policy::EffectiveRunPolicy,
    pub(crate) budgets: super::policy_intersection::ThreadBudgets,
    pub(crate) usage_grant: crate::uar::runtime::cost_budget::RemoteUsageGrantBinding,
    pub(crate) sandbox: super::policy_intersection::SandboxPermissions,
}

impl ActorThreadSession {
    pub(crate) fn new(
        owner: crate::uar::runtime::actor::messages::ActorOwner,
        artifact: AgentArtifact,
        session_id: String,
        manager: Arc<RunManager>,
        persistence: Arc<dyn PersistenceLayer>,
        cancellation: CancellationToken,
        state: watch::Sender<Option<PersistedAgentThread>>,
        owned_root: Arc<tokio::sync::Mutex<Option<ActorRootBinding>>>,
    ) -> Self {
        Self::new_with_constraints(
            owner,
            artifact,
            session_id,
            manager,
            persistence,
            cancellation,
            state,
            owned_root,
            None,
        )
    }

    pub(crate) fn new_with_constraints(
        owner: crate::uar::runtime::actor::messages::ActorOwner,
        artifact: AgentArtifact,
        session_id: String,
        manager: Arc<RunManager>,
        persistence: Arc<dyn PersistenceLayer>,
        cancellation: CancellationToken,
        state: watch::Sender<Option<PersistedAgentThread>>,
        owned_root: Arc<tokio::sync::Mutex<Option<ActorRootBinding>>>,
        remote_constraints: Option<RemoteRootConstraints>,
    ) -> Self {
        Self {
            owner,
            artifact,
            session_id,
            manager,
            persistence,
            cancellation,
            current: None,
            uncertain: None,
            state,
            owned_root,
            remote_constraints,
        }
    }

    /// Execute one direct user turn through RunManager with an exact artifact.
    /// This entry is not a model-controlled child spawn or authorization grant.
    pub(crate) async fn execute(
        &mut self,
        content: String,
    ) -> anyhow::Result<PersistedAgentThread> {
        self.execute_named(content, uuid::Uuid::new_v4().to_string(), None)
            .await
    }

    pub(crate) async fn execute_named(
        &mut self,
        content: String,
        run_id: String,
        artifacts: Option<super::artifacts::RunArtifactCollector>,
    ) -> anyhow::Result<PersistedAgentThread> {
        let mut request = RunExecutionRequest::new(self.artifact.clone(), content)
            .with_verified_owner(self.owner.clone());
        if let Some(constraints) = &self.remote_constraints {
            request.presentation_negotiation = constraints
                .presentation_negotiation
                .clone()
                .unwrap_or_default();
            request.resolved_policy = Some(constraints.policy.clone());
            request.host_budget_constraint = Some(constraints.budgets.clone());
            request.host_usage_grant = Some(constraints.usage_grant.clone());
            request.host_sandbox_constraint = Some(constraints.sandbox.clone());
        }
        request.session_id = Some(self.session_id.clone());
        self.execute_request(request, run_id, None, artifacts).await
    }

    /// Reuse the owned root lifecycle for a graph request without discarding
    /// checkpoint history, resolved policy, attachments or captured resources.
    /// A preparation observer does not keep an otherwise abandoned graph alive.
    pub(crate) async fn execute_request(
        &mut self,
        request: RunExecutionRequest,
        run_id: String,
        prepared: Option<tokio::sync::oneshot::Sender<()>>,
        artifacts: Option<super::artifacts::RunArtifactCollector>,
    ) -> anyhow::Result<PersistedAgentThread> {
        anyhow::ensure!(
            request.verified_owner.as_ref() == Some(&self.owner)
                && request.user_id.as_deref() == Some(self.owner.user_id())
                && request.artifact.id == self.artifact.id
                && request.session_id.as_deref() == Some(self.session_id.as_str()),
            "Root request does not match its host session"
        );
        if let Some(artifacts) = &artifacts {
            artifacts.check_binding(&self.owner, &run_id)?;
        }
        self.settle_uncertain().await?;
        if self.cancellation.is_cancelled() {
            anyhow::bail!("Actor has been stopped");
        }
        if self
            .current
            .as_ref()
            .is_some_and(|current| !current.thread.status.is_terminal())
        {
            anyhow::bail!("The preceding actor root has not reached a terminal state");
        }
        // A conversation may span many root runs. Reusing a root thread while
        // changing only run_id would retain the previous run's tree identity,
        // approval root and admission counters. History remains in session_id;
        // the durable roots remain separate and are never overwritten.
        let next = AgentThread::root(
            self.owner.user_id().to_owned(),
            self.artifact.id.clone(),
            run_id.clone(),
        )?;
        let record = self.persist(next).await?;
        let root = ActorRootBinding {
            record,
            artifacts,
            persistence: Arc::clone(&self.persistence),
            graph_tools: Arc::new(std::sync::OnceLock::new()),
            sandbox: Arc::new(std::sync::OnceLock::new()),
            terminal: Arc::new(std::sync::OnceLock::new()),
            service: Arc::new(std::sync::OnceLock::new()),
            ready: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            producer: Arc::new(tokio::sync::Mutex::new(ActorProducer::default())),
        };
        *self.owned_root.lock().await = Some(root.clone());

        let completion = self
            .manager
            .start_hosted_root_turn(
                request,
                root.clone(),
                self.cancellation.child_token(),
                prepared.is_none(),
            )
            .await;
        if let Some(prepared) = prepared {
            let _ = prepared.send(());
        }
        let mut result = completion
            .await
            .unwrap_or_else(|_| AgentThreadResult::Failed {
                code: "kernel_completion_closed".into(),
                message: "Run ended without a terminal completion record".into(),
            });
        if let Err(error) = root.finish().await {
            tracing::error!(%error, "Actor root retains unresolved child cleanup");
            result = AgentThreadResult::Failed {
                code: "thread_cleanup_unconfirmed".into(),
                message: "Child thread cleanup remains unconfirmed".into(),
            };
        } else {
            self.owned_root.lock().await.take();
        }
        let mut next = self
            .current
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Actor thread registration is missing"))?
            .thread
            .clone();
        next.finish_turn(result)?;
        self.persist(next).await
    }

    /// After the owning worker has ended, settle its exact receipts and close
    /// any root whose execution was interrupted before a terminal write.
    pub(crate) async fn finish_abandoned(&mut self) -> anyhow::Result<()> {
        self.settle_uncertain().await?;
        if let Some(current) = &self.current
            && !current.thread.status.is_terminal()
        {
            let mut next = current.thread.clone();
            next.finish_turn(if self.cancellation.is_cancelled() {
                AgentThreadResult::Cancelled
            } else {
                AgentThreadResult::Failed {
                    code: "root_worker_interrupted".into(),
                    message: "Root worker ended before its terminal record".into(),
                }
            })?;
            self.persist(next).await?;
        }
        Ok(())
    }

    /// Reconcile a previous mailbox operation before accepting another one.
    /// A recovered start belongs to the failed request: never substitute the
    /// new prompt or replay a model call whose caller already received failure.
    pub(crate) async fn settle_uncertain(&mut self) -> anyhow::Result<()> {
        {
            let mut owned_root = self.owned_root.lock().await;
            if let Some(root) = owned_root.as_ref() {
                root.finish().await?;
            }
            owned_root.take();
        }
        if self.uncertain.is_none() {
            return Ok(());
        }
        let recovered = self.reconcile_exact().await?;
        if !recovered.thread.status.is_terminal() {
            // A start write is awaited before kernel entry. If it remained
            // uncertain when execute returned, no model/tool work was started.
            let mut next = recovered.thread;
            next.finish_turn(if self.cancellation.is_cancelled() {
                AgentThreadResult::Cancelled
            } else {
                AgentThreadResult::Failed {
                    code: "kernel_not_started".into(),
                    message: "Turn could not start while its registration was unconfirmed".into(),
                }
            })?;
            self.persist(next).await?;
        }
        Ok(())
    }

    /// Only an exact owner-qualified snapshot proves this write committed.
    /// Absence, an older revision, or a failed read does not prove rollback and
    /// must not authorize a replacement ID, a repeated write, or another turn.
    async fn reconcile_exact(&mut self) -> anyhow::Result<PersistedAgentThread> {
        let expected = self
            .uncertain
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Actor has no uncertain transition"))?
            .clone();
        let stored = self
            .persistence
            .load_agent_thread(self.owner.user_id(), &expected.thread.thread_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Actor transition is not yet confirmed in storage"))?;
        stored.validate(self.owner.user_id())?;
        if stored != expected {
            anyhow::bail!("Stored actor transition does not match the pending write");
        }
        self.publish_committed(stored.clone());
        Ok(stored)
    }

    async fn persist(&mut self, next: AgentThread) -> anyhow::Result<PersistedAgentThread> {
        if self.uncertain.is_some() {
            anyhow::bail!("Actor has an unresolved persistence write; refusing another write");
        }
        let previous = self
            .current
            .as_ref()
            .filter(|current| current.thread.thread_id == next.thread_id);
        if previous.is_none()
            && self
                .current
                .as_ref()
                .is_some_and(|current| !current.thread.status.is_terminal())
        {
            anyhow::bail!("Cannot replace a live actor root");
        }
        let expected = match previous {
            Some(current) => {
                agent_threads::next_record(self.owner.user_id(), current, current.revision, &next)?
            }
            None => agent_threads::new_root(self.owner.user_id(), &next)?,
        };
        self.uncertain = Some(expected.clone());
        let write = match previous {
            Some(current) => {
                self.persistence
                    .update_agent_thread(self.owner.user_id(), current.revision, &next)
                    .await
            }
            None => {
                self.persistence
                    .create_agent_root(self.owner.user_id(), &next)
                    .await
            }
        };
        let stored = match write {
            Ok(stored) => stored,
            Err(write_error) => {
                // An acknowledged read may recover a lost commit response in
                // this same request. This never retries the mutation itself.
                return self.reconcile_exact().await.map_err(|read_error| {
                    write_error.context(format!("Actor write remains unresolved: {read_error}"))
                });
            }
        };
        stored.validate(self.owner.user_id())?;
        if stored != expected {
            anyhow::bail!("Thread store returned a different actor transition");
        }
        self.publish_committed(stored.clone());
        Ok(stored)
    }

    fn publish_committed(&mut self, stored: PersistedAgentThread) {
        self.current = Some(stored.clone());
        self.uncertain = None;
        // The actor registry retains the receiver; an absent observer does not
        // undo the confirmed database write or turn it into a failed operation.
        let _ = self.state.send(Some(stored));
    }
}
