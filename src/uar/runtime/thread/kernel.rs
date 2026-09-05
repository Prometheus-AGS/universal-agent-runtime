//! Captured entry into the shared run kernel for the trusted thread scheduler.
//! Implements the trusted execution host over those exact resources. The thread
//! service owns policy intersection, admission reservations and durable writes.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

use crate::llm::{Message, MessageContent, MessageRole};
use crate::mcp::registry::McpRegistry;
use crate::uar::domain::runs::RunStatus;
use crate::uar::persistence::{PersistenceLayer, agent_threads::PersistedAgentThread};
use crate::uar::runtime::actor::messages::ActorOwner;
use crate::uar::runtime::manager::RunManager;
use crate::uar::runtime::native_skill::NativeSkillRegistry;
use crate::uar::runtime::turn::{
    RunExecutionRequest,
    bindings::{InheritedRunBindings, RunDelegationBindings},
};

use super::policy_intersection::{
    CredentialGrant, CredentialTarget, McpToolBinding, SandboxPermissions, ThreadBudgets,
    ThreadPermissions, ThreadPolicy, ThreadToolBinding,
};
use super::{AgentThread, AgentThreadResult, AgentThreadStatus, service::HostedThreadTurn};

/// An owner-qualified root capture. There is no public constructor accepting
/// clients, credentials or a replacement root; use RunManager's live-run lookup.
pub struct CapturedThreadKernel {
    manager: RunManager,
    root: PersistedAgentThread,
    persistence: Arc<dyn PersistenceLayer>,
    resources: Arc<RunDelegationBindings>,
    mcp: Arc<McpRegistry>,
    native: Arc<NativeSkillRegistry>,
    mcp_grants: BTreeSet<CredentialGrant>,
}

struct ChildExecutionLifetime(tokio_util::sync::CancellationToken);

impl Drop for ChildExecutionLifetime {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

impl std::fmt::Debug for CapturedThreadKernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapturedThreadKernel")
            .field("root_thread_id", &self.root.thread.thread_id)
            .finish_non_exhaustive()
    }
}

impl CapturedThreadKernel {
    pub(crate) fn trusted_a2a_peer(
        &self,
        endpoint: &str,
        agent_id: &str,
    ) -> anyhow::Result<crate::uar::api::a2a::peer::TrustedA2APeer> {
        self.manager.trusted_a2a_peer(endpoint, agent_id)
    }

    pub(crate) fn a2a_instance_id(&self) -> &str {
        self.manager.a2a_instance_id()
    }

    pub(crate) fn remote_presentation_negotiation(
        &self,
        policy: &super::policy_intersection::ThreadPolicy,
    ) -> Option<crate::uar::a2ui::presentation_selection::PresentationNegotiation> {
        self.resources
            .presentations
            .narrow(policy.effective())
            .delegation_negotiation()
    }

    pub(crate) async fn capture(
        manager: RunManager,
        owner: &ActorOwner,
        root: &PersistedAgentThread,
        persistence: Arc<dyn PersistenceLayer>,
        resources: Arc<RunDelegationBindings>,
    ) -> anyhow::Result<Self> {
        root.validate(owner.user_id())?;
        anyhow::ensure!(
            owner == &resources.owner
                && root.thread.parent_thread_id.is_none()
                && root.thread.run_id.as_ref() == Some(&resources.run_id)
                && root.thread.root_run_id == resources.run_id
                && resources.policy.agent_id.as_ref() == Some(&root.thread.artifact_id),
            "Root thread does not match the captured host identity"
        );
        Self::require_root(&manager, root, persistence.as_ref(), &resources).await?;
        let cancellation = resources.cancellation.clone();
        let mcp = tokio::select! {
            biased;
            () = cancellation.cancelled() => anyhow::bail!("Root delegation has closed"),
            result = async {
                resources.activation.lock().await.freeze_mcp_bindings().await
            } => result?,
        };
        let native = Arc::new(resources.native.filtered(None).await);
        // Opaque identities refer to this exact frozen connection capture, not
        // credentials, server configuration, or a recipe for reconnecting.
        let mcp_grants = mcp
            .server_names()
            .into_iter()
            .map(|server| CredentialGrant {
                target: CredentialTarget::McpServer(server),
                binding_id: uuid::Uuid::new_v4().to_string(),
            })
            .collect();
        let captured = Self {
            manager,
            root: root.clone(),
            persistence,
            resources,
            mcp: Arc::new(mcp),
            native,
            mcp_grants,
        };
        captured.require_live_root().await?;
        Ok(captured)
    }

    async fn require_live_root(&self) -> anyhow::Result<()> {
        Self::require_root(
            &self.manager,
            &self.root,
            self.persistence.as_ref(),
            &self.resources,
        )
        .await
    }

    pub(crate) fn root_record(&self) -> &PersistedAgentThread {
        &self.root
    }

    pub(crate) async fn check_actor_owner(&self, owner: &ActorOwner) -> anyhow::Result<()> {
        anyhow::ensure!(
            owner == &self.resources.owner,
            "Foreign actor delegation owner"
        );
        self.require_live_root().await
    }

    pub(crate) fn admit_host_tool(&self) -> anyhow::Result<()> {
        self.resources.models.budget().admit_tool()
    }

    pub(crate) fn reserve_remote_budget(
        &self,
        requested: &ThreadBudgets,
    ) -> anyhow::Result<crate::uar::runtime::cost_budget::RemoteBudgetReservation> {
        self.resources.models.budget().reserve_remote(requested)
    }

    pub(crate) fn original_artifact(&self) -> &crate::uar::domain::artifact::AgentArtifact {
        &self.resources.artifact
    }

    pub(crate) fn persistence(&self) -> Arc<dyn PersistenceLayer> {
        Arc::clone(&self.persistence)
    }

    pub(crate) fn cancellation(&self) -> tokio_util::sync::CancellationToken {
        self.resources.cancellation.child_token()
    }

    /// Trusted-host single-attachment claim, shared even across repeated kernel
    /// captures. A failed attachment closes this root's admission attempt; it
    /// does not authorize retrying with fresh zeroed tree counters.
    pub(crate) fn claim_attachment(&self) -> anyhow::Result<()> {
        use std::sync::atomic::Ordering;
        anyhow::ensure!(
            self.resources
                .thread_attachment_claimed
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok(),
            "Root thread service attachment has already been attempted"
        );
        Ok(())
    }

    pub(crate) async fn check_attachment(
        &self,
        root: &PersistedAgentThread,
        policy: &super::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            root == &self.root,
            "Thread service must attach to its captured budget root"
        );
        self.check_bindings(policy, false).await
    }

    /// Build root authority from the actual run capture, never an adapter's
    /// replacement artifact, credential list or permissive resource defaults.
    ///
    /// # Errors
    /// Rejects closed roots, unbound tools and unsupported policy restrictions.
    pub async fn root_policy(&self) -> anyhow::Result<ThreadPolicy> {
        self.require_live_root().await?;
        let artifact = &self.resources.artifact;
        let mut credentials = self.resources.models.credential_grants();
        credentials.extend(self.mcp_grants.iter().cloned());
        let mut tool_bindings = BTreeMap::new();
        for name in &self.resources.policy.tools.ids {
            let binding = if self.has_control_factory(name)
                || self.native.contains(name).await
                || self.mcp.is_native_tool(name)
            {
                ThreadToolBinding::Native
            } else {
                let (server_id, tool_name) = self.mcp.resolve_mcp_tool(name).ok_or_else(|| {
                    anyhow::anyhow!("Root tool '{name}' has no captured executable binding")
                })?;
                ThreadToolBinding::Mcp(McpToolBinding {
                    server_id,
                    tool_name,
                })
            };
            tool_bindings.insert(name.clone(), binding);
        }
        let mode = artifact.policy.tools.execution_mode.clone();
        let sandbox = self.resources.sandbox.as_ref().map_or_else(
            || SandboxPermissions {
                execution_mode: mode.clone(),
                network_enabled: false,
                filesystem: BTreeMap::new(),
                environment: BTreeSet::new(),
            },
            |binding| binding.permissions(mode.clone()),
        );
        let policy = ThreadPolicy::for_root(
            &self.root.thread,
            &self.resources.policy,
            artifact,
            ThreadPermissions {
                credentials,
                tool_bindings,
                sandbox,
                budgets: ThreadBudgets::from_artifact(artifact)?,
                max_active_skills: artifact.policy.skills.max_active,
                max_concurrent_tools: artifact.policy.tools.max_concurrent,
            },
        )?;
        self.check_bindings(&policy, false).await?;
        Ok(policy)
    }

    fn has_control_factory(&self, name: &str) -> bool {
        self.resources.thread_controls && super::control::AGENT_TOOL_NAMES.contains(&name)
    }

    async fn check_execution_bindings(&self, policy: &ThreadPolicy) -> anyhow::Result<()> {
        self.check_bindings(policy, true).await
    }

    // Capturing root authority does not execute a delegated tool. Preserve
    // existing root tools, but require child execution adapters at admission.
    async fn check_bindings(
        &self,
        policy: &ThreadPolicy,
        child_execution: bool,
    ) -> anyhow::Result<()> {
        self.check_budget(policy).await?;
        let parent = &self.resources.policy;
        let child = policy.effective();
        for (selected, inherited) in [
            (&child.skills.ids, &parent.skills.ids),
            (&child.tools.ids, &parent.tools.ids),
            (&child.mcp_servers.ids, &parent.mcp_servers.ids),
            (&child.knowledge_bases.ids, &parent.knowledge_bases.ids),
            (&child.presentations.ids, &parent.presentations.ids),
        ] {
            anyhow::ensure!(
                selected.iter().all(|id| inherited.contains(id)),
                "Child selection exceeds the captured root resources"
            );
        }
        anyhow::ensure!(
            policy.permissions().max_active_skills
                <= self.resources.artifact.policy.skills.max_active
                && policy.permissions().max_concurrent_tools
                    <= self.resources.artifact.policy.tools.max_concurrent,
            "Child limits exceed the captured root artifact"
        );
        self.resources.models.for_policy(policy)?.budget().admit()?;
        for grant in &policy.permissions().credentials {
            if matches!(grant.target, CredentialTarget::McpServer(_)) {
                anyhow::ensure!(
                    self.mcp_grants.contains(grant),
                    "Child MCP credential binding is not inherited"
                );
            }
        }
        self.mcp
            .require_bound_servers(child.mcp_servers.ids.iter().map(String::as_str))?;
        match &self.resources.sandbox {
            Some(binding) => {
                binding.for_permissions(&policy.permissions().sandbox)?;
            }
            None => anyhow::ensure!(
                !policy.permissions().sandbox.network_enabled
                    && policy.permissions().sandbox.filesystem.is_empty()
                    && policy.permissions().sandbox.environment.is_empty(),
                "Child sandbox grants have no captured enforcement binding"
            ),
        }
        for name in &child.tools.ids {
            let sandbox_required = |required, effect| {
                use crate::uar::domain::artifact::ToolExecutionMode;
                required
                    || match policy.permissions().sandbox.execution_mode {
                        ToolExecutionMode::Direct => false,
                        ToolExecutionMode::Sandboxed => true,
                        ToolExecutionMode::Auto => {
                            effect == crate::uar::tools::descriptor::ToolEffect::CodeExecution
                        }
                    }
            };
            match policy.permissions().tool_bindings.get(name) {
                Some(ThreadToolBinding::Native) => {
                    if self.has_control_factory(name) {
                        anyhow::ensure!(
                            !self.native.contains(name).await
                                && self.mcp.descriptor(name).is_none(),
                            "Agent control name collides with an ambient tool"
                        );
                        anyhow::ensure!(
                            !child_execution
                                || policy.permissions().sandbox.execution_mode
                                    != crate::uar::domain::artifact::ToolExecutionMode::Sandboxed,
                            "Agent controls have no sandbox execution adapter"
                        );
                        continue;
                    }
                    if self.mcp.is_native_tool(name) {
                        if child_execution {
                            self.mcp.check_native_thread_policy(name, policy)?;
                            let descriptor = self.mcp.descriptor(name).ok_or_else(|| {
                                anyhow::anyhow!("In-process descriptor is unavailable")
                            })?;
                            anyhow::ensure!(
                                !sandbox_required(descriptor.sandbox_required, descriptor.effect),
                                "In-process tool has no sandbox execution adapter"
                            );
                        }
                        continue;
                    }
                    let tool = self.native.get(name).await.ok_or_else(|| {
                        anyhow::anyhow!("Native tool has no context-aware captured binding")
                    })?;
                    if child_execution && sandbox_required(tool.sandbox_required(), tool.effect()) {
                        anyhow::ensure!(
                            self.resources.sandbox.is_some() && tool.supports_sandbox_execution(),
                            "Required child sandbox or tool adapter is unavailable"
                        );
                    }
                    // Direct calls go through execute_native, which checks the
                    // child owner and implementation policy before every effect.
                    // Turn-local controls are rebuilt, not reused from this copy.
                }
                Some(ThreadToolBinding::Mcp(binding)) => {
                    anyhow::ensure!(
                        self.mcp.resolve_mcp_tool(name)
                            == Some((binding.server_id.clone(), binding.tool_name.clone()))
                            && child.mcp_servers.ids.contains(&binding.server_id)
                            && policy.permissions().credentials.iter().any(|grant| {
                                grant.target
                                    == CredentialTarget::McpServer(binding.server_id.clone())
                                    && self.mcp_grants.contains(grant)
                            }),
                        "Child MCP tool does not match its frozen server binding"
                    );
                    let descriptor = self
                        .mcp
                        .descriptor(name)
                        .ok_or_else(|| anyhow::anyhow!("Child MCP descriptor is unavailable"))?;
                    anyhow::ensure!(
                        !child_execution
                            || !sandbox_required(descriptor.sandbox_required, descriptor.effect),
                        "MCP tool has no sandbox execution adapter"
                    );
                }
                None => anyhow::bail!("Child tool has no captured identity"),
            }
        }
        self.check_budget(policy).await
    }

    pub(crate) async fn check_budget(
        &self,
        policy: &super::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        self.require_live_root().await?;
        anyhow::ensure!(
            policy.owner_id() == self.resources.owner.user_id()
                && policy.approval_root_run_id() == self.resources.run_id,
            "Thread budget belongs to another root"
        );
        self.resources
            .models
            .budget()
            .narrowed(&policy.permissions().budgets)?
            .admit()
    }

    async fn require_root(
        manager: &RunManager,
        root: &PersistedAgentThread,
        persistence: &dyn PersistenceLayer,
        resources: &RunDelegationBindings,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            !resources.cancellation.is_cancelled(),
            "Root delegation has closed"
        );
        let owner = resources.owner.user_id();
        let stored = persistence
            .load_agent_thread(owner, &root.thread.thread_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Delegation root is unavailable"))?;
        anyhow::ensure!(
            &stored == root && !stored.thread.status.is_terminal(),
            "Delegation root changed or completed"
        );
        let run = manager
            .get_run_for_user(owner, &resources.run_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Root kernel is unavailable"))?;
        anyhow::ensure!(
            run.status == RunStatus::Running && !resources.cancellation.is_cancelled(),
            "Root kernel is no longer running"
        );
        Ok(())
    }

    /// Resolve a named artifact for this owner through the manager's existing
    /// registered-artifact path, without falling back to a default artifact.
    ///
    /// # Errors
    /// Rejects a different owner, a closed root, missing artifacts or store errors.
    pub async fn artifact(
        &self,
        owner_id: &str,
        artifact_id: &str,
    ) -> anyhow::Result<crate::uar::domain::artifact::AgentArtifact> {
        anyhow::ensure!(
            owner_id == self.resources.owner.user_id(),
            "Foreign artifact lookup owner"
        );
        self.require_live_root().await?;
        let artifact = self.manager.resolve_registered_agent(artifact_id).await?;
        self.require_live_root().await?;
        Ok(artifact)
    }

    /// Read this tree's canonical kernel history. Fork filtering belongs to
    /// ThreadService; resumed children retain their own complete tool pairs.
    ///
    /// # Errors
    /// Rejects foreign lineage, stale turns and unavailable/superseded sessions.
    pub async fn history(&self, thread: &AgentThread) -> anyhow::Result<Vec<Message>> {
        self.require_live_root().await?;
        thread.validate()?;
        anyhow::ensure!(
            thread.owner_id == self.resources.owner.user_id()
                && thread.root_thread_id == self.root.thread.thread_id
                && thread.root_run_id == self.root.thread.root_run_id,
            "Foreign thread history request"
        );
        let stored = self
            .persistence
            .load_agent_thread(&thread.owner_id, &thread.thread_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Thread history record is unavailable"))?;
        stored.validate(&thread.owner_id)?;
        let current = &stored.thread;
        // A resumed child has committed its new turn before asking for the
        // preceding turn's history. Permit that one transition, not an arbitrary
        // older record whose private session now belongs to later execution.
        let resuming = thread.parent_thread_id.is_some()
            && thread.status.is_terminal()
            && current.status == AgentThreadStatus::Running
            && thread.history_revision.checked_add(1) == Some(current.history_revision)
            && current.run_id != thread.run_id
            && current.owner_id == thread.owner_id
            && current.root_thread_id == thread.root_thread_id
            && current.root_run_id == thread.root_run_id
            && current.parent_thread_id == thread.parent_thread_id
            && current.canonical_path == thread.canonical_path
            && current.artifact_id == thread.artifact_id;
        anyhow::ensure!(
            current == thread || resuming,
            "Thread history revision changed"
        );
        let run_id = thread
            .run_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Thread has no kernel history"))?;
        let history = self
            .manager
            .canonical_thread_history(
                &thread.owner_id,
                run_id,
                thread
                    .parent_thread_id
                    .as_ref()
                    .map(|_| thread.thread_id.as_str()),
            )
            .await?;
        self.require_live_root().await?;
        Ok(history)
    }

    /// Execute a committed, admitted child turn, retaining the parent's actual
    /// clients, frozen MCP connections and request-only approval channel.
    ///
    /// ThreadService checks admission before reserving work. This method also
    /// rechecks the exact executable bindings before kernel entry; it does not
    /// reserve spawns, mint controls or write thread transitions.
    ///
    /// # Errors
    /// Rejects stale/foreign records, widened resource selections, absent
    /// inherited bindings, or a missing final delegated user message.
    pub async fn execute(&self, mut turn: HostedThreadTurn) -> anyhow::Result<AgentThreadResult> {
        if turn.cancellation.is_cancelled() || self.resources.cancellation.is_cancelled() {
            return Ok(AgentThreadResult::Cancelled);
        }
        self.check_execution_bindings(&turn.policy).await?;
        turn.record.validate(self.resources.owner.user_id())?;
        let thread = &turn.record.thread;
        anyhow::ensure!(
            thread.parent_thread_id.is_some()
                && thread.root_thread_id == self.root.thread.thread_id
                && thread.root_run_id == self.root.thread.root_run_id
                && turn.policy.owner_id() == self.resources.owner.user_id()
                && turn.policy.approval_root_run_id() == self.resources.run_id,
            "Child turn belongs to another root"
        );
        let stored = self
            .persistence
            .load_agent_thread(self.resources.owner.user_id(), &thread.thread_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Child turn is not committed"))?;
        anyhow::ensure!(
            stored == turn.record,
            "Child turn changed before kernel entry"
        );
        let child = turn.policy.effective();
        let servers = child
            .mcp_servers
            .ids
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let tools = child.tools.ids.iter().cloned().collect::<HashSet<_>>();
        let mcp = self.mcp.filtered(Some(&servers), Some(&tools));
        mcp.require_bound_servers(servers.iter().map(String::as_str))?;
        let models = self.resources.models.for_policy(&turn.policy)?;
        let sandbox = self
            .resources
            .sandbox
            .as_ref()
            .map(|binding| {
                binding
                    .for_permissions(&turn.policy.permissions().sandbox)
                    .map(Arc::new)
            })
            .transpose()?;

        // ThreadService supplies canonical history, including the new user
        // message. Split only that final message so matching sees its real text
        // and the kernel appends it once. Do not re-fork a resumed child's tools.
        let input = turn
            .messages
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Child turn has no delegated input"))?;
        anyhow::ensure!(
            input.role == MessageRole::User
                && input.tool_call_id.is_none()
                && input.tool_calls.as_ref().is_none_or(Vec::is_empty),
            "Child turn must end in an ordinary user message"
        );
        let MessageContent::Text { content: input } = input.content else {
            // as_text() returns only the first text part of multimodal input.
            // Do not silently discard other parts at this text-only boundary.
            anyhow::bail!("Delegated input must be a text message");
        };
        let mut request = RunExecutionRequest::new(turn.original_artifact, input)
            .with_verified_owner(self.resources.owner.clone());
        request.session_id = Some(thread.thread_id.clone());
        request.checkpoint_history = Some(turn.messages);
        let run_id = thread
            .run_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Child run ID is missing"))?;
        let bindings = InheritedRunBindings {
            policy: turn.policy,
            presentations: Arc::clone(&self.resources.presentations),
            thread: thread.clone(),
            controls: turn.controls,
            models,
            skills: Arc::clone(&self.resources.skills),
            mcp: Arc::new(mcp),
            native: Arc::clone(&self.native),
            harness: self.resources.harness.clone(),
            sandbox,
            working_directory: self.resources.working_directory.clone(),
            approvals: self.resources.approvals.for_child(),
        };
        self.require_live_root().await?;
        let cancellation = self.resources.cancellation.child_token();
        let _lifetime = ChildExecutionLifetime(cancellation.clone());
        let execution =
            self.manager
                .execute_captured_thread(request, run_id, cancellation.clone(), bindings);
        tokio::pin!(execution);
        tokio::select! {
            biased;
            () = turn.cancellation.cancelled() => {
                cancellation.cancel();
                // Cancellation requests unwind; they do not detach the kernel
                // or publish a terminal thread before its producer has stopped.
                execution.await
            }
            result = &mut execution => result,
        }
    }
}

#[async_trait::async_trait]
impl super::service::ThreadExecutionHost for CapturedThreadKernel {
    async fn artifact(
        &self,
        owner_id: &str,
        artifact_id: &str,
    ) -> anyhow::Result<crate::uar::domain::artifact::AgentArtifact> {
        CapturedThreadKernel::artifact(self, owner_id, artifact_id).await
    }

    async fn history(&self, thread: &AgentThread) -> anyhow::Result<Vec<Message>> {
        CapturedThreadKernel::history(self, thread).await
    }

    async fn check_admission(&self, policy: &ThreadPolicy) -> anyhow::Result<()> {
        self.check_execution_bindings(policy).await
    }

    async fn execute(&self, turn: HostedThreadTurn) -> anyhow::Result<AgentThreadResult> {
        CapturedThreadKernel::execute(self, turn).await
    }
}
