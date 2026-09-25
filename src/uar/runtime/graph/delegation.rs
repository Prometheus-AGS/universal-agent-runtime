//! Host-bound graph delegation through the same controls as native agent tools.
//! Graph data supplies intent, never the caller, root, approval or credentials.

use std::sync::Arc;

use anyhow::Context;
use crate::llm::{ToolApprovalGate, ToolApprovalResult};
use crate::uar::runtime::thread::control::{AgentToolContext, AgentTurnOutcome};
use crate::uar::runtime::thread::spawn::AgentSpawnRequest;
use crate::uar::runtime::thread::spawn::RemoteAgentSpawnRequest;
use crate::uar::runtime::tool_admission::{
    HostAdmissionDisposition, LocalAdmissionDisposition, ToolAdmissionRuntime,
    ToolApprovalRequest,
};
use crate::uar::tools::descriptor::ToolDescriptor;

/// Opaque delegation capability installed by the run host, not deserialized
/// from graph state. Its controls retain the root-owned thread service.
pub struct GraphThreadDelegate {
    run_id: String,
    controls: Arc<AgentToolContext>,
    gate: ToolApprovalGate,
    admission: Arc<ToolAdmissionRuntime>,
    descriptor: Arc<ToolDescriptor>,
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
        }
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
    ) -> anyhow::Result<AgentTurnOutcome> {
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
        // The same host gate charges the root tool budget. The service then
        // intersects policies and enforces the shared tree/model budget.
        let result = self.controls.spawn(request).await.map_err(|error| {
            tracing::error!(
                run_id,
                step,
                error = ?error,
                "Graph child spawn failed"
            );
            error
        });
        let outcome: anyhow::Result<AgentTurnOutcome> = match result {
            Ok(child) => self
                .controls
                .wait_first_turn(&child.thread_id)
                .await
                .map_err(anyhow::Error::from),
            Err(error) => Err(error),
        };
        self.admission.finish(&admitted, outcome.is_ok()).await?;
        outcome
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
