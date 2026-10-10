//! Workflow commands reuse this runtime's durable task queue and live cancellation owner.
use super::TeamExecutionRuntime;
use crate::uar::{
    compiler::collaboration::CollaborationError,
    domain::{
        team_execution::TeamControlRequest,
        workflow_execution::{WorkflowControlRequest, WorkflowRun},
    },
    runtime::actor::messages::ActorOwner,
};
use std::sync::Arc;

impl TeamExecutionRuntime {
    pub(super) async fn drive_workflows(
        self: &Arc<Self>,
        owner: &ActorOwner,
    ) -> Result<(), CollaborationError> {
        let owner_key = owner.presentation_owner_key();
        for run in self.catalog.list_owner_workflow_runs(&owner_key).await? {
            if run.status != "cancellation_requested" {
                continue;
            }
            self.cancel_workflow_attempts(owner, &run).await?;
        }
        self.catalog.advance_workflows(&owner_key).await?;
        Ok(())
    }
    async fn cancel_workflow_attempts(
        &self,
        owner: &ActorOwner,
        run: &WorkflowRun,
    ) -> Result<(), CollaborationError> {
        for step in &run.steps {
            let Some(id) = &step.attempt_id else { continue };
            let attempt = self
                .catalog
                .get_team_attempt(&run.owner_id, &run.workspace_id, &run.team_id, id)
                .await?;
            if !matches!(
                attempt.status.as_str(),
                "queued" | "running" | "cancellation_requested"
            ) {
                continue;
            }
            let team = self
                .catalog
                .get_team_instance(&run.owner_id, &run.workspace_id, &run.team_id)
                .await?;
            self.cancel(
                owner,
                &run.workspace_id,
                &run.team_id,
                id,
                TeamControlRequest {
                    command_id: format!("workflow-cancel-{}-{id}-{}", run.id, team.revision),
                    expected_team_revision: team.revision,
                    reason: run
                        .state_reason
                        .clone()
                        .unwrap_or_else(|| "workflow cancellation".into()),
                },
            )
            .await?;
        }
        Ok(())
    }
    pub async fn control_workflow(
        self: &Arc<Self>,
        owner: ActorOwner,
        workspace: &str,
        id: &str,
        operation: &str,
        request: WorkflowControlRequest,
    ) -> Result<WorkflowRun, CollaborationError> {
        let owner_key = owner.presentation_owner_key();
        let receipt = self
            .catalog
            .prepare_workflow_control(&owner_key, workspace, id, operation, &request)
            .await?;
        if receipt.completed {
            return Ok(receipt.result);
        }
        if operation == "recover" {
            self.recover(
                owner.clone(),
                workspace,
                &receipt.result.team_id,
                receipt.team_control.ok_or_else(|| {
                    CollaborationError::Conflict("WORKFLOW_CONTROL_UNAVAILABLE".into())
                })?,
            )
            .await?;
        } else {
            self.cancel_workflow_attempts(&owner, &receipt.result)
                .await?;
        }
        self.activate_workflows(owner).await;
        self.catalog
            .finish_workflow_control(&owner_key, workspace, id, &request.command_id)
            .await
    }
}
