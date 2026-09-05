//! Host-bound graph delegation through the same controls as native agent tools.
//! Graph data supplies intent, never the caller, root, approval or credentials.

use std::sync::Arc;

use crate::llm::{ToolApprovalGate, ToolApprovalResult};
use crate::uar::runtime::thread::control::{AgentToolContext, AgentTurnOutcome};
use crate::uar::runtime::thread::spawn::AgentSpawnRequest;
use crate::uar::runtime::thread::spawn::RemoteAgentSpawnRequest;
use crate::uar::tools::descriptor::ApprovalClass;

/// Opaque delegation capability installed by the run host, not deserialized
/// from graph state. Its controls retain the root-owned thread service.
pub struct GraphThreadDelegate {
    run_id: String,
    controls: Arc<AgentToolContext>,
    gate: ToolApprovalGate,
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
    ) -> Self {
        Self {
            run_id,
            controls,
            gate,
        }
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
        let decision = (self.gate)(
            uuid::Uuid::new_v4().to_string(),
            "spawn_agent".to_string(),
            ApprovalClass::Required,
            serde_json::to_string(&request)?,
            step as usize,
        )
        .await;
        if let ToolApprovalResult::Rejected { reason } = decision {
            anyhow::bail!("Graph delegation denied: {reason}");
        }
        // The same host gate charges the root tool budget. The service then
        // intersects policies and enforces the shared tree/model budget.
        let child = self.controls.spawn(request).await.map_err(|error| {
            tracing::error!(
                run_id,
                step,
                error = ?error,
                "Graph child spawn failed"
            );
            error
        })?;
        Ok(self.controls.wait_first_turn(&child.thread_id).await?)
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
        let decision = (self.gate)(
            uuid::Uuid::new_v4().to_string(),
            "spawn_agent".to_string(),
            ApprovalClass::Required,
            serde_json::to_string(&serde_json::json!({ "endpoint": request.endpoint }))?,
            step as usize,
        )
        .await;
        if let ToolApprovalResult::Rejected { reason } = decision {
            anyhow::bail!("Graph delegation denied: {reason}");
        }
        let child = self.controls.spawn_remote(request).await?;
        Ok(self.controls.wait_first_turn(&child.thread_id).await?)
    }
}
