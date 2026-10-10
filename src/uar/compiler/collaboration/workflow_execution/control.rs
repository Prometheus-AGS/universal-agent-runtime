use super::*;
use crate::uar::domain::team_execution::TeamControlRequest;
use chrono::Utc;
use serde_json::json;

impl CollaborationCatalogService {
    pub async fn decide_workflow(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: WorkflowDecisionRequest,
    ) -> Result<WorkflowRun, CollaborationError> {
        super::super::validation::validate_id(&request.command_id)?;
        let digest =
            super::super::validation::canonical_digest(&json!({"runId":id,"request":request}))?;
        let command = key(owner, workspace, &format!("{id}:{}", request.command_id));
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            let original = run(&current, owner, workspace, id)?;
            authority(&current, original)?;
            if let Some(receipt) = current.workflow_commands.get(&command) {
                if receipt.request_digest != digest || receipt.operation != "decide" {
                    return Err(conflict("WORKFLOW_COMMAND_CONFLICT"));
                }
                return Ok(receipt.result.clone());
            }
            if original.revision != request.expected_run_revision
                || original.status != "awaiting_decision"
            {
                return Err(conflict("WORKFLOW_STALE_DECISION"));
            }
            let wait = original
                .wait
                .as_ref()
                .ok_or_else(|| conflict("WORKFLOW_WAIT_UNAVAILABLE"))?;
            if wait.id != request.wait_id
                || wait.artifact_id != request.artifact_id
                || wait.artifact_digest != request.artifact_digest
                || !wait.decisions.contains(&request.decision)
            {
                return Err(conflict("WORKFLOW_STALE_DECISION"));
            }
            let final_output = json!({"decision":request.decision,"artifactId":request.artifact_id,"artifactDigest":request.artifact_digest});
            if !jsonschema::validator_for(&original.plan.output)
                .map_err(|_| conflict("WORKFLOW_OUTPUT_SCHEMA_INVALID"))?
                .is_valid(&final_output)
            {
                return Err(conflict("WORKFLOW_FINAL_OUTPUT_INVALID"));
            }
            let mut updated = view(&current, original)?;
            updated.status = match request.decision.as_str() {
                "accept" => "accepted",
                "reject" => "rejected",
                _ => "cancelled",
            }
            .into();
            updated.revision += 1;
            updated.updated_at = Utc::now();
            updated.state_reason = None;
            updated.decision = Some(WorkflowDecisionReceipt {
                id: uuid::Uuid::new_v4().to_string(),
                command_id: request.command_id.clone(),
                wait_id: request.wait_id.clone(),
                decision: request.decision.clone(),
                artifact_id: request.artifact_id.clone(),
                artifact_digest: request.artifact_digest.clone(),
                run_revision: updated.revision,
                committed_at: updated.updated_at,
            });
            let mut next = current.clone();
            next.generation += 1;
            next.workflow_runs
                .insert(key(owner, workspace, id), updated.clone());
            next.workflow_commands.insert(
                command.clone(),
                WorkflowCommandReceipt {
                    request_digest: digest.clone(),
                    operation: "decide".into(),
                    run_id: id.into(),
                    result: updated.clone(),
                    completed: true,
                    team_control: None,
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(updated);
            }
        }
        Err(conflict("WORKFLOW_CONCURRENT_CHANGE"))
    }

    pub async fn prepare_workflow_control(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        operation: &str,
        request: &WorkflowControlRequest,
    ) -> Result<WorkflowCommandReceipt, CollaborationError> {
        super::super::validation::validate_id(&request.command_id)?;
        if !matches!(operation, "cancel" | "recover") || request.reason.trim().is_empty() {
            return Err(conflict("WORKFLOW_CONTROL_INVALID"));
        }
        let digest = super::super::validation::canonical_digest(
            &json!({"runId":id,"operation":operation,"request":request}),
        )?;
        let command = key(owner, workspace, &format!("{id}:{}", request.command_id));
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            let original = run(&current, owner, workspace, id)?;
            if let Some(receipt) = current.workflow_commands.get(&command) {
                if receipt.request_digest != digest || receipt.operation != operation {
                    return Err(conflict("WORKFLOW_COMMAND_CONFLICT"));
                }
                return Ok(receipt.clone());
            }
            if original.revision != request.expected_run_revision {
                return Err(conflict("WORKFLOW_REVISION_CHANGED"));
            }
            if matches!(
                original.status.as_str(),
                "accepted" | "rejected" | "cancelled" | "failed"
            ) {
                return Err(conflict("WORKFLOW_TERMINAL"));
            }
            let team = current
                .team_instances
                .get(&key(owner, workspace, &original.team_id))
                .ok_or_else(|| conflict("WORKFLOW_TEAM_UNAVAILABLE"))?;
            let mut updated = view(&current, original)?;
            if operation == "cancel" {
                updated.status = "cancellation_requested".into();
                updated.state_reason = Some(request.reason.clone());
            }
            updated.revision += 1;
            updated.updated_at = Utc::now();
            let receipt = WorkflowCommandReceipt {
                request_digest: digest.clone(),
                operation: operation.into(),
                run_id: id.into(),
                result: updated.clone(),
                completed: false,
                team_control: Some(TeamControlRequest {
                    command_id: format!("workflow-control-{}", uuid::Uuid::new_v4()),
                    expected_team_revision: team.revision,
                    reason: request.reason.clone(),
                }),
            };
            let mut next = current.clone();
            next.generation += 1;
            next.workflow_runs
                .insert(key(owner, workspace, id), updated);
            next.workflow_commands
                .insert(command.clone(), receipt.clone());
            if self.cas(current.generation, &next).await? {
                return Ok(receipt);
            }
        }
        Err(conflict("WORKFLOW_CONCURRENT_CHANGE"))
    }
    pub async fn finish_workflow_control(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        command_id: &str,
    ) -> Result<WorkflowRun, CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            let result = view(&current, run(&current, owner, workspace, id)?)?;
            let mut next = current.clone();
            let receipt = next
                .workflow_commands
                .get_mut(&key(owner, workspace, &format!("{id}:{command_id}")))
                .ok_or_else(|| conflict("WORKFLOW_COMMAND_UNAVAILABLE"))?;
            if receipt.completed {
                return Ok(receipt.result.clone());
            }
            receipt.completed = true;
            receipt.result = result.clone();
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                return Ok(result);
            }
        }
        Err(conflict("WORKFLOW_CONCURRENT_CHANGE"))
    }
}
