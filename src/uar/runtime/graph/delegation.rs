//! Host-bound graph delegation through the same controls as native agent tools.
//! Graph data supplies intent, never the caller, root, approval or credentials.

use std::fmt::Write as _;
use std::sync::Arc;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::llm::{ToolApprovalGate, ToolApprovalResult};
use crate::uar::runtime::thread::control::{AgentToolContext, AgentTurnOutcome};
use crate::uar::runtime::thread::spawn::AgentSpawnRequest;
use crate::uar::runtime::thread::spawn::RemoteAgentSpawnRequest;
use crate::uar::runtime::tool_admission::{
    HostAdmissionDisposition, LocalAdmissionDisposition, ToolAdmissionRuntime,
    ToolApprovalRequest,
};
use crate::uar::tools::descriptor::ToolDescriptor;
use crate::uar::runtime::turn::client_tools::GraphCallerTurnCapture;
use crate::uar::domain::events::{NormalizedEvent, RuntimeEventSink};

pub(crate) struct GraphChildOutcome {
    pub(crate) turn: AgentTurnOutcome,
    pub(crate) handoff: Option<GraphCallerHandoff>,
}

pub(crate) struct GraphCallerHandoff {
    batch_id: String,
    calls: Vec<crate::llm::ToolCall>,
    text: String,
}

struct GraphCallerHost {
    capture: GraphCallerTurnCapture,
    manager: crate::uar::runtime::manager::RunManager,
    persistence: Arc<dyn crate::uar::persistence::PersistenceLayer>,
    owner: crate::uar::runtime::actor::messages::ActorOwner,
    events: Arc<dyn RuntimeEventSink>,
}

/// Public catalog metadata available to the intent classifier. Agent prompts
/// and credentials never become classification metadata.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AgentRoutingCandidate {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) tags: Vec<String>,
    pub(crate) revision: String,
}

/// Content-addressed snapshot of this parent's eligible registered agents.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AgentRoutingRoster {
    pub(crate) revision: String,
    pub(crate) candidates: Vec<AgentRoutingCandidate>,
}

struct GraphRoutingHost {
    manager: crate::uar::runtime::manager::RunManager,
    source_agent_id: String,
    events: Arc<dyn RuntimeEventSink>,
}

/// Opaque delegation capability installed by the run host, not deserialized
/// from graph state. Its controls retain the root-owned thread service.
pub struct GraphThreadDelegate {
    run_id: String,
    controls: Arc<AgentToolContext>,
    gate: ToolApprovalGate,
    admission: Arc<ToolAdmissionRuntime>,
    descriptor: Arc<ToolDescriptor>,
    caller: Option<GraphCallerHost>,
    routing: Option<GraphRoutingHost>,
}

impl std::fmt::Debug for GraphThreadDelegate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphThreadDelegate")
            .field("run_id", &self.run_id)
            .finish_non_exhaustive()
    }
}

impl GraphThreadDelegate {
    pub(crate) fn new(
        run_id: String,
        controls: Arc<AgentToolContext>,
        gate: ToolApprovalGate,
        admission: Arc<ToolAdmissionRuntime>,
        descriptor: Arc<ToolDescriptor>,
    ) -> Self {
        Self {
            run_id,
            controls,
            gate,
            admission,
            descriptor,
            caller: None,
            routing: None,
        }
    }

    /// Install host-owned catalog access independently of optional caller tools.
    pub(crate) fn with_agent_routing(
        mut self,
        manager: crate::uar::runtime::manager::RunManager,
        source_agent_id: String,
        events: Arc<dyn RuntimeEventSink>,
    ) -> Self {
        self.routing = Some(GraphRoutingHost { manager, source_agent_id, events });
        self
    }

    /// Read the current catalog and narrow candidates against captured parent
    /// authority. Final execution admission remains with the thread service.
    pub(crate) async fn eligible_local_agents(&self) -> anyhow::Result<AgentRoutingRoster> {
        let host = self.routing.as_ref()
            .ok_or_else(|| anyhow::anyhow!("graph_routing_host_missing"))?;
        anyhow::ensure!(
            host.source_agent_id == self.controls.scope().caller().artifact_id
                && self.controls.scope().caller().run_id.as_deref() == Some(self.run_id.as_str()),
            "graph_routing_host_scope_mismatch"
        );
        anyhow::ensure!(
            self.controls.permits("spawn_agent") && self.controls.permits("wait_agents"),
            "Graph policy does not authorize child spawning and waiting"
        );
        let agents = host.manager.list_registered_agents().await?;
        let mut candidates = Vec::new();
        for agent in agents {
            if agent.id == host.source_agent_id || agent.runtime.entry == "orchestrator"
                || self.controls.scope().policy().intersect(&agent).is_err()
            {
                continue;
            }
            candidates.push(AgentRoutingCandidate {
                revision: agent.content_revision(),
                id: agent.id,
                title: agent.metadata.title,
                description: agent.metadata.description,
                tags: agent.metadata.tags,
            });
        }
        candidates.sort_by(|left, right| left.id.cmp(&right.id));
        anyhow::ensure!(!candidates.is_empty(), "graph_routing_roster_empty");
        let digest = Sha256::digest(serde_json::to_vec(&candidates)?);
        let mut revision = String::with_capacity(digest.len() * 2);
        for byte in digest {
            write!(&mut revision, "{byte:02x}")?;
        }
        Ok(AgentRoutingRoster { revision, candidates })
    }

    pub(crate) fn with_caller_tools(
        mut self,
        capture: GraphCallerTurnCapture,
        manager: crate::uar::runtime::manager::RunManager,
        persistence: Arc<dyn crate::uar::persistence::PersistenceLayer>,
        owner: crate::uar::runtime::actor::messages::ActorOwner,
        events: Arc<dyn RuntimeEventSink>,
    ) -> Self {
        self.caller = Some(GraphCallerHost { capture, manager, persistence, owner, events });
        self
    }

    pub(crate) async fn publish_handoff(&self, handoff: GraphCallerHandoff) -> anyhow::Result<()> {
        let host = self.caller.as_ref().ok_or_else(|| anyhow::anyhow!("graph_caller_host_missing"))?;
        if !handoff.text.is_empty() {
            host.events.emit(NormalizedEvent::ChatDelta { run_id: self.run_id.clone(), text_delta: handoff.text }).await;
        }
        host.events.emit(NormalizedEvent::ClientToolsRequested {
            run_id: self.run_id.clone(), batch_id: handoff.batch_id, calls: handoff.calls,
        }).await;
        Ok(())
    }

    async fn caller_handoff(&self, turn: &AgentTurnOutcome) -> anyhow::Result<Option<GraphCallerHandoff>> {
        let Some(host) = &self.caller else { return Ok(None); };
        // A child can fail admission before a manager run is registered. Keep
        // its exact terminal failure instead of replacing it with lookup failure.
        if !matches!(&turn.result, Some(crate::uar::runtime::thread::AgentThreadResult::Completed { .. })) {
            return Ok(None);
        }
        let child_run = turn.agent.run_id.as_deref().ok_or_else(|| anyhow::anyhow!("graph_child_run_missing"))?;
        let status = host.manager.await_settled_run_status(child_run).await?;
        if status != crate::uar::domain::runs::RunStatus::Done { return Ok(None); }
        let owner = host.owner.session_owner_key()?;
        let Some(batch) = host.persistence.load_client_tool_continuation_for_run(
            &owner, host.owner.tenant_id(), host.capture.root_agent_id(), host.capture.root_run_id(),
        ).await? else { return Ok(None); };
        batch.validate()?;
        anyhow::ensure!(batch.owner_key == owner && batch.tenant_id.as_deref() == host.owner.tenant_id()
            && batch.agent_id == host.capture.root_agent_id() && batch.source_run_id == self.run_id
            && batch.session_id.as_deref() == Some(host.capture.root_session_id())
            && batch.state == crate::uar::persistence::client_tools::ClientToolContinuationState::Pending,
            "graph_caller_batch_scope_conflict");
        let source = batch.execution_source.as_ref().ok_or_else(|| anyhow::anyhow!("graph_caller_source_missing"))?;
        anyhow::ensure!(source.run_id == child_run && source.thread_id == turn.agent.thread_id
            && source.agent_id == turn.agent.artifact_id,
            "graph_caller_child_scope_conflict");
        let assistant = batch.visible_transcript.as_array().and_then(|history| history.last())
            .ok_or_else(|| anyhow::anyhow!("graph_caller_visible_history_missing"))?;
        let calls: Vec<crate::llm::ToolCall> = serde_json::from_value(assistant.get("tool_calls").cloned()
            .ok_or_else(|| anyhow::anyhow!("graph_caller_calls_missing"))?)?;
        anyhow::ensure!(calls.iter().map(|call| &call.id).eq(batch.pending_call_ids.iter()),
            "graph_caller_call_ids_conflict");
        anyhow::ensure!(assistant.get("role").and_then(serde_json::Value::as_str) == Some("assistant"),
            "graph_caller_assistant_missing");
        let text = assistant.get("content").and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("graph_caller_text_invalid"))?.to_owned();
        Ok(Some(GraphCallerHandoff { batch_id: batch.batch_id, calls, text }))
    }

    async fn admit(
        &self,
        step: u32,
        arguments: serde_json::Value,
    ) -> anyhow::Result<crate::uar::runtime::tool_admission::AdmittedToolInvocation> {
        let prepared = self.admission.prepare(
            uuid::Uuid::new_v4().to_string(),
            &self.descriptor,
            arguments,
            step as usize,
        );
        let host = self.admission.prepare_host(Arc::clone(&prepared)).await?;
        anyhow::ensure!(
            host.host_disposition != HostAdmissionDisposition::Deny,
            "Graph delegation is denied by the host policy"
        );
        let request = ToolApprovalRequest {
            invocation: Arc::clone(&prepared),
            admission_id: host.admission_id.clone(),
            host_requires_approval: host.host_disposition == HostAdmissionDisposition::Ask,
            action_display: host.action_display.clone(),
            checkpoint_history: None,
        };
        let local = match (self.gate)(request).await {
            ToolApprovalResult::Approved => LocalAdmissionDisposition::Approved,
            ToolApprovalResult::Allowed => LocalAdmissionDisposition::Allowed,
            ToolApprovalResult::GovernanceBypassed => {
                LocalAdmissionDisposition::GovernanceBypassed
            }
            ToolApprovalResult::Rejected { reason } => {
                self.admission
                    .resolve(
                        prepared,
                        host,
                        LocalAdmissionDisposition::Denied,
                        false,
                    )
                    .await?;
                anyhow::bail!("Graph delegation denied: {reason}")
            }
            ToolApprovalResult::Cancelled { reason } => {
                self.admission
                    .cancel(
                        prepared.as_ref(),
                        &host.admission_id,
                        crate::uar::runtime::tool_admission::AdmissionCancellationReason::Cancelled,
                    )
                    .await?;
                anyhow::bail!("Graph delegation cancelled: {reason}")
            }
        };
        let admitted = self
            .admission
            .resolve(prepared, host, local, true)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph delegation was not authorized by the host"))?;
        if let Err(error) = self.admission.claim(&admitted).await {
            let _ = self
                .admission
                .cancel(
                    admitted.prepared.as_ref(),
                    &admitted.host_receipt.admission_id,
                    crate::uar::runtime::tool_admission::AdmissionCancellationReason::Invalidated,
                )
                .await;
            return Err(error.context("Graph delegation claim intent was not persisted"));
        }
        Ok(admitted)
    }

    pub(crate) async fn execute(
        &self,
        run_id: &str,
        step: u32,
        request: AgentSpawnRequest,
        graph_state: serde_json::Value,
    ) -> anyhow::Result<GraphChildOutcome> {
        self.execute_local_inner(run_id, step, request, graph_state, None).await
    }

    /// Revalidate the classified roster after approval and before dispatch.
    /// A catalog mutation rejects this selection rather than choosing a fallback.
    pub(crate) async fn execute_selected(
        &self,
        run_id: &str,
        step: u32,
        request: AgentSpawnRequest,
        graph_state: serde_json::Value,
        roster_revision: &str,
    ) -> anyhow::Result<GraphChildOutcome> {
        self.execute_local_inner(run_id, step, request, graph_state, Some(roster_revision)).await
    }

    async fn execute_local_inner(
        &self,
        run_id: &str,
        step: u32,
        request: AgentSpawnRequest,
        graph_state: serde_json::Value,
        roster_revision: Option<&str>,
    ) -> anyhow::Result<GraphChildOutcome> {
        request.validate()?;
        anyhow::ensure!(
            run_id == self.run_id
                && self.controls.scope().caller().run_id.as_deref() == Some(run_id),
            "Graph delegation belongs to another run"
        );
        // Check both operations before spawning: a graph cannot leave an
        // accepted child running merely because its wait was never authorized.
        anyhow::ensure!(
            self.controls.permits("spawn_agent") && self.controls.permits("wait_agents"),
            "Graph policy does not authorize child spawning and waiting"
        );
        anyhow::ensure!(
            self.controls
                .scope()
                .policy()
                .permissions()
                .sandbox
                .execution_mode
                != crate::uar::domain::artifact::ToolExecutionMode::Sandboxed,
            "Graph agent controls have no sandbox execution adapter"
        );
        let admitted = self.admit(step, serde_json::to_value(&request)?).await?;
        let selected = if let Some(expected_revision) = roster_revision {
            let selection = async {
                let roster = self.eligible_local_agents().await?;
                anyhow::ensure!(roster.revision == expected_revision, "graph_routing_roster_changed");
                let candidate = roster.candidates.into_iter()
                    .find(|candidate| candidate.id == request.artifact_id)
                    .ok_or_else(|| anyhow::anyhow!("graph_routing_selection_unavailable"))?;
                Ok::<_, anyhow::Error>((candidate, roster.revision))
            }.await;
            match selection {
                Ok(selected) => Some(selected),
                Err(error) => {
                    self.admission.finish(&admitted, false).await?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        // The same host gate charges the root tool budget. The service then
        // intersects policies and enforces the shared tree/model budget.
        let result = match &self.caller {
            Some(host) => {
                let capture = GraphCallerTurnCapture::new(host.capture.input().clone(),
                    host.capture.root_agent_id().to_owned(), host.capture.root_run_id().to_owned(),
                    host.capture.root_session_id().to_owned(), graph_state);
                self.controls.spawn_graph(request, capture).await
            }
            None => self.controls.spawn(request).await,
        }.map_err(|error| {
            tracing::error!(
                run_id,
                step,
                error = ?error,
                "Graph child spawn failed"
            );
            error
        });
        let outcome: anyhow::Result<AgentTurnOutcome> = match result {
            Ok(child) => {
                if let Some((candidate, revision)) = selected
                    && let Some(host) = &self.routing
                {
                    // The child is persisted. Publish before waiting or forwarding
                    // its response; reconnect replays this root event, not spawn.
                    host.events.emit(NormalizedEvent::AgentSelected {
                        run_id: self.run_id.clone(),
                        source_agent_id: host.source_agent_id.clone(),
                        selected_agent_id: candidate.id,
                        selected_agent_name: candidate.title,
                        selected_thread_id: child.thread_id.clone(),
                        selection_method: "intent".to_owned(),
                        roster_revision: revision,
                    }).await;
                }
                self.controls.wait_first_turn(&child.thread_id).await.map_err(anyhow::Error::from)
            }
            Err(error) => Err(error.into()),
        };
        self.admission.finish(&admitted, outcome.is_ok()).await?;
        let turn = outcome?;
        let handoff = self.caller_handoff(&turn).await?;
        Ok(GraphChildOutcome { turn, handoff })
    }

    pub(crate) async fn execute_remote(
        &self,
        run_id: &str,
        step: u32,
        endpoint: String,
        delegated_prompt: String,
    ) -> anyhow::Result<AgentTurnOutcome> {
        let request = RemoteAgentSpawnRequest {
            endpoint,
            delegated_prompt,
            task_name: None,
        };
        request.validate()?;
        anyhow::ensure!(
            run_id == self.run_id
                && self.controls.scope().caller().run_id.as_deref() == Some(run_id),
            "Graph delegation belongs to another run"
        );
        anyhow::ensure!(
            self.controls.permits("spawn_agent") && self.controls.permits("wait_agents"),
            "Graph policy does not authorize child spawning and waiting"
        );
        let admitted = self.admit(
            step,
            serde_json::json!({
                "endpoint": request.endpoint,
                "delegatedPrompt": request.delegated_prompt,
                "taskName": request.task_name,
            }),
        )
        .await?;
        let outcome: anyhow::Result<AgentTurnOutcome> = match self.controls.spawn_remote(request).await {
            Ok(child) => self
                .controls
                .wait_first_turn(&child.thread_id)
                .await
                .map_err(anyhow::Error::from),
            Err(error) => Err(error.into()),
        };
        self.admission.finish(&admitted, outcome.is_ok()).await?;
        outcome
    }
}
