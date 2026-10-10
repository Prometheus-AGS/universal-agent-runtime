use super::*;
use crate::uar::domain::team_execution::AdmitTeamTaskRequest;
use chrono::Utc;
use serde_json::json;

pub(crate) fn admission_guard(
    state: &CollaborationCatalogState,
    owner: &str,
    workspace: &str,
    team: &str,
    task: &str,
    workflow: Option<&str>,
) -> Result<(), CollaborationError> {
    let selected = state.workflow_runs.values().find(|r| {
        r.owner_id == owner
            && r.workspace_id == workspace
            && r.team_id == team
            && r.steps.iter().any(|s| s.task_id == task)
    });
    match (selected, workflow) {
        (None, None) => Ok(()),
        (Some(run), Some(id)) if run.id == id => {
            authority(state, run)?;
            if !matches!(run.status.as_str(), "ready" | "running") {
                return Err(conflict("WORKFLOW_NOT_EXECUTABLE"));
            }
            let index = run
                .steps
                .iter()
                .position(|s| s.task_id == task)
                .ok_or_else(|| conflict("WORKFLOW_TASK_UNAVAILABLE"))?;
            if run.steps[index].attempt_id.is_some()
                || (index == 1 && run.steps[0].artifact.is_none())
            {
                return Err(conflict("WORKFLOW_STEP_NOT_READY"));
            }
            Ok(())
        }
        _ => Err(conflict("WORKFLOW_OWNED_TASK")),
    }
}

pub(crate) fn link_attempt(
    state: &mut CollaborationCatalogState,
    workflow: Option<&str>,
    attempt: &TeamExecutionAttempt,
) -> Result<(), CollaborationError> {
    let Some(id) = workflow else { return Ok(()) };
    let run = state
        .workflow_runs
        .get_mut(&key(&attempt.owner_id, &attempt.workspace_id, id))
        .ok_or_else(|| conflict("WORKFLOW_RUN_UNAVAILABLE"))?;
    let step = run
        .steps
        .iter_mut()
        .find(|s| s.task_id == attempt.task_id)
        .ok_or_else(|| conflict("WORKFLOW_TASK_UNAVAILABLE"))?;
    step.attempt_id = Some(attempt.id.clone());
    step.status = "running".into();
    run.status = "running".into();
    run.state_reason = None;
    run.revision += 1;
    run.updated_at = Utc::now();
    Ok(())
}

impl CollaborationCatalogService {
    /// Called by the existing runtime drain and authenticated start/recovery, never by reads.
    pub async fn advance_workflows(
        &self,
        owner: &str,
    ) -> Result<Vec<TeamExecutionAttempt>, CollaborationError> {
        if !crate::uar::api::capabilities::workflow_execution_enabled() {
            return Ok(Vec::new());
        }
        let ids = self
            .load_state()
            .await?
            .workflow_runs
            .values()
            .filter(|r| {
                r.owner_id == owner
                    && matches!(
                        r.status.as_str(),
                        "ready" | "running" | "cancellation_requested" | "reconciling"
                    )
            })
            .map(|r| (r.workspace_id.clone(), r.id.clone()))
            .collect::<Vec<_>>();
        let mut queued = Vec::new();
        for (workspace, id) in ids {
            match self.advance_workflow(owner, &workspace, &id).await {
                Ok(Some(attempt)) => queued.push(attempt),
                Ok(None) => {}
                Err(error) => {
                    self.workflow_reason(owner, &workspace, &id, error.to_string())
                        .await?;
                }
            }
        }
        Ok(queued)
    }
    async fn workflow_reason(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        reason: String,
    ) -> Result<(), CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let selected = run(&current, owner, workspace, id)?;
            if selected.state_reason.as_deref() == Some(reason.as_str()) {
                return Ok(());
            }
            let mut next = current.clone();
            let selected = next
                .workflow_runs
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("WORKFLOW_RUN_UNAVAILABLE"))?;
            selected.state_reason = Some(reason.clone());
            selected.revision += 1;
            selected.updated_at = Utc::now();
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                return Ok(());
            }
        }
        Err(conflict("WORKFLOW_CONCURRENT_CHANGE"))
    }
    pub async fn advance_workflow(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
    ) -> Result<Option<TeamExecutionAttempt>, CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            let original = run(&current, owner, workspace, id)?;
            if matches!(
                original.status.as_str(),
                "awaiting_decision" | "accepted" | "rejected" | "cancelled" | "failed"
            ) {
                return Ok(None);
            }
            let mut updated = original.clone();
            let cancelling = updated.status == "cancellation_requested";
            let mut active = false;
            for step in &mut updated.steps {
                let Some(attempt_id) = &step.attempt_id else {
                    continue;
                };
                let attempt = current
                    .team_execution_attempts
                    .values()
                    .find(|a| {
                        a.id == *attempt_id
                            && a.owner_id == owner
                            && a.workspace_id == workspace
                            && a.team_id == updated.team_id
                    })
                    .ok_or_else(|| conflict("WORKFLOW_ATTEMPT_UNAVAILABLE"))?;
                if matches!(
                    attempt.status.as_str(),
                    "queued" | "running" | "cancellation_requested"
                ) {
                    active = true;
                    continue;
                }
                match attempt.execution_outcome.as_deref() {
                    Some("succeeded") if attempt.effect_disposition == "confirmed" => {
                        let artifact = current
                            .team_artifacts
                            .values()
                            .find(|a| {
                                a.attempt_id == attempt.id
                                    && a.owner_id == owner
                                    && a.workspace_id == workspace
                                    && a.team_id == updated.team_id
                                    && a.task_id == step.task_id
                            })
                            .ok_or_else(|| conflict("WORKFLOW_ARTIFACT_UNAVAILABLE"))?;
                        if attempt.output.as_ref() != Some(&artifact.content) {
                            return Err(conflict("WORKFLOW_ARTIFACT_OUTCOME_MISMATCH"));
                        }
                        step.artifact = Some(WorkflowArtifact {
                            id: artifact.id.clone(),
                            digest: super::super::validation::canonical_digest(&artifact.content)?,
                            content: artifact.content.clone(),
                            attempt_id: attempt.id.clone(),
                        });
                        step.status = "succeeded".into();
                    }
                    Some("failed" | "cancelled") if attempt.effect_disposition == "confirmed" => {
                        step.status = attempt.execution_outcome.clone().unwrap_or_default();
                        updated.status = if cancelling { "cancelled" } else { "failed" }.into();
                        updated.state_reason = attempt.state_reason.clone();
                    }
                    _ => {
                        updated.status = if cancelling {
                            "cancellation_requested"
                        } else {
                            "reconciling"
                        }
                        .into();
                        updated.state_reason = Some("WORKFLOW_EXECUTION_UNCERTAIN".into());
                        active = true;
                    }
                }
            }
            if cancelling && !active {
                updated.status = "cancelled".into();
                for step in updated
                    .steps
                    .iter_mut()
                    .filter(|step| step.attempt_id.is_none())
                {
                    step.status = "cancelled".into();
                }
            }
            if !cancelling && !active && updated.steps.iter().all(|s| s.status == "succeeded") {
                authority(&current, &updated)?;
                let draft = updated.steps[1]
                    .artifact
                    .as_ref()
                    .ok_or_else(|| conflict("WORKFLOW_DRAFT_UNAVAILABLE"))?;
                updated.wait = Some(WorkflowWait {
                    id: format!("workflow-wait-{}", updated.id),
                    artifact_id: draft.id.clone(),
                    artifact_digest: draft.digest.clone(),
                    decisions: vec!["accept".into(), "reject".into(), "cancel".into()],
                    created_at: Utc::now(),
                });
                updated.status = "awaiting_decision".into();
                updated.state_reason = None;
            }
            let changed = serde_json::to_value(&updated)? != serde_json::to_value(original)?;
            if changed {
                updated.revision += 1;
                updated.updated_at = Utc::now();
                let mut next = current.clone();
                next.generation += 1;
                if updated.status == "cancelled" {
                    let team = next
                        .team_instances
                        .get_mut(&key(owner, workspace, &updated.team_id))
                        .ok_or_else(|| conflict("WORKFLOW_TEAM_UNAVAILABLE"))?;
                    for step in updated
                        .steps
                        .iter()
                        .filter(|step| step.attempt_id.is_none())
                    {
                        let task = team
                            .tasks
                            .iter_mut()
                            .find(|task| task.id == step.task_id)
                            .ok_or_else(|| conflict("WORKFLOW_TASK_UNAVAILABLE"))?;
                        task.status = "cancelled".into();
                        task.revision += 1;
                        task.updated_at = updated.updated_at;
                    }
                    team.revision += 1;
                }
                if updated.steps[0].artifact.is_some()
                    && updated.steps[1].attempt_id.is_none()
                    && !cancelling
                    && updated.status != "failed"
                {
                    let team = next
                        .team_instances
                        .get_mut(&key(owner, workspace, &updated.team_id))
                        .ok_or_else(|| conflict("WORKFLOW_TEAM_UNAVAILABLE"))?;
                    let task = team
                        .tasks
                        .iter_mut()
                        .find(|t| t.id == updated.steps[1].task_id)
                        .ok_or_else(|| conflict("WORKFLOW_TASK_UNAVAILABLE"))?;
                    task.input = json!({"feedback":updated.input["feedback"],"classification":updated.steps[0].artifact.as_ref().map(|a|a.content.clone())});
                    task.revision += 1;
                    team.revision += 1;
                    updated.steps[1].status = "ready".into();
                }
                next.workflow_runs
                    .insert(key(owner, workspace, id), updated);
                if !self.cas(current.generation, &next).await? {
                    continue;
                }
                continue;
            }
            if active
                || cancelling
                || matches!(
                    updated.status.as_str(),
                    "failed" | "cancelled" | "awaiting_decision" | "reconciling"
                )
            {
                return Ok(None);
            }
            let team = authority(&current, &updated)?;
            let Some((index, step)) = updated
                .steps
                .iter()
                .enumerate()
                .find(|(_, s)| s.attempt_id.is_none())
            else {
                return Ok(None);
            };
            let task = team
                .tasks
                .iter()
                .find(|t| t.id == step.task_id)
                .ok_or_else(|| conflict("WORKFLOW_TASK_UNAVAILABLE"))?;
            let request = AdmitTeamTaskRequest {
                command_id: format!("workflow-{}-{}", id, step.step_id),
                expected_team_revision: team.revision,
                expected_task_revision: task.revision,
                member_id: step.member_id.clone(),
                reservation: step.reservation.clone(),
                context_artifact_ids: if index == 0 {
                    vec![]
                } else {
                    vec![
                        updated.steps[0]
                            .artifact
                            .as_ref()
                            .ok_or_else(|| conflict("WORKFLOW_PREDECESSOR_UNAVAILABLE"))?
                            .id
                            .clone(),
                    ]
                },
            };
            return self
                .admit_workflow_task(
                    owner,
                    workspace,
                    &updated.team_id,
                    &step.task_id,
                    request,
                    id,
                )
                .await
                .map(Some);
        }
        Err(conflict("WORKFLOW_CONCURRENT_CHANGE"))
    }
}
