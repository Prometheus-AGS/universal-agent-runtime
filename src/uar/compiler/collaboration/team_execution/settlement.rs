use super::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, attempt, fence, key, team,
    team_key,
};
use crate::uar::domain::team_execution::{
    TeamControlRequest, TeamExecutionAttempt, TeamExecutionCommandReceipt, TeamReservation,
};
use chrono::Utc;
use serde_json::{Value, json};

impl CollaborationCatalogService {
    pub async fn claim_team_dispatch(
        &self,
        expected: &TeamExecutionAttempt,
    ) -> Result<TeamExecutionAttempt, CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            fence(&current, expected)?;
            let mut item = attempt(&current, expected)?.clone();
            if item.status != "queued" {
                return Err(CollaborationError::Conflict(
                    "team dispatch was already claimed".to_owned(),
                ));
            }
            item.status = "running".to_owned();
            item.updated_at = Utc::now();
            let mut next = current.clone();
            next.generation += 1;
            next.team_execution_attempts.insert(
                key(&item.owner_id, &item.workspace_id, &item.team_id, &item.id),
                item.clone(),
            );
            if self.cas(current.generation, &next).await? {
                return Ok(item);
            }
        }
        Err(CollaborationError::Conflict(
            "team dispatch changed concurrently".to_owned(),
        ))
    }

    pub async fn settle_team_attempt(
        &self,
        expected: &TeamExecutionAttempt,
        status: &str,
        usage: Option<TeamReservation>,
        output: Option<Value>,
        reason: Option<String>,
    ) -> Result<TeamExecutionAttempt, CollaborationError> {
        if !matches!(status, "succeeded" | "failed" | "cancelled" | "uncertain") {
            return Err(CollaborationError::Invalid(
                "settlement requires a terminal or uncertain outcome".to_owned(),
            ));
        }
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let mut item = attempt(&current, expected)?.clone();
            if item.run_id != expected.run_id || item.execution_epoch != expected.execution_epoch {
                return Err(CollaborationError::Conflict(
                    "settlement belongs to a stale execution epoch".to_owned(),
                ));
            }
            if let Some(prior) = &item.usage {
                if usage.as_ref().is_some_and(|u| {
                    u.tokens != prior.tokens
                        || u.cost_microunits != prior.cost_microunits
                        || u.elapsed_seconds != prior.elapsed_seconds
                }) || item.execution_outcome.as_deref() != Some(status)
                    || item.output != output
                {
                    return Err(CollaborationError::Conflict(
                        "settled attempt outcome is immutable".to_owned(),
                    ));
                }
                return Ok(item);
            }
            let now = Utc::now();
            item.status = if usage.is_none() {
                "uncertain".to_owned()
            } else {
                status.to_owned()
            };
            item.execution_outcome = Some(status.to_owned());
            item.usage = usage.clone();
            item.usage_revision += 1;
            item.output = output.clone();
            item.state_reason = reason.clone().or_else(|| {
                usage.is_none().then(|| {
                    "Provider usage or effects are uncertain; reservation retained".to_owned()
                })
            });
            item.updated_at = now;
            let mut next = current.clone();
            next.generation += 1;
            let selected = next
                .team_instances
                .get_mut(&team_key(&item.owner_id, &item.workspace_id, &item.team_id))
                .ok_or_else(|| CollaborationError::NotFound(item.team_id.clone()))?;
            if let Some(task) = selected.tasks.iter_mut().find(|t| t.id == item.task_id)
                && task.ownership_epoch == item.ownership_epoch
                && task.assignee_member_id.as_deref() == Some(item.member_id.as_str())
            {
                task.status = if status == "uncertain" {
                    "blocked".to_owned()
                } else {
                    status.to_owned()
                };
                task.output = output.clone();
                task.state_reason = item.state_reason.clone();
                task.revision += 1;
                task.updated_at = now;
            }
            selected.revision += 1;
            selected.updated_at = now;
            if status == "succeeded" {
                let succeeded: std::collections::BTreeSet<_> = selected
                    .tasks
                    .iter()
                    .filter(|t| t.status == "succeeded")
                    .map(|t| t.id.clone())
                    .collect();
                for dependent in &mut selected.tasks {
                    if dependent.status == "queued"
                        && dependent.depends_on.iter().all(|d| succeeded.contains(d))
                    {
                        dependent.status = "ready".to_owned();
                        dependent.revision += 1;
                        dependent.updated_at = now;
                    }
                }
            }
            next.team_execution_attempts.insert(
                key(&item.owner_id, &item.workspace_id, &item.team_id, &item.id),
                item.clone(),
            );
            if self.cas(current.generation, &next).await? {
                return Ok(item);
            }
        }
        Err(CollaborationError::Conflict(
            "team usage settlement changed concurrently".to_owned(),
        ))
    }

    pub async fn cancel_team_attempt(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
        attempt_id: &str,
        request: TeamControlRequest,
    ) -> Result<TeamExecutionAttempt, CollaborationError> {
        super::super::validation::validate_id(&request.command_id)?;
        if request.reason.trim().is_empty() {
            return Err(CollaborationError::Invalid(
                "cancellation requires a reason".to_owned(),
            ));
        }
        let digest = super::super::validation::canonical_digest(
            &json!({"teamId":team_id,"attemptId":attempt_id,"request":request}),
        )?;
        let receipt_key = format!("{owner}\u{1f}{workspace}\u{1f}{}", request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let selected = team(&current, owner, workspace, team_id)?;
            let mut item = current
                .team_execution_attempts
                .get(&key(owner, workspace, team_id, attempt_id))
                .cloned()
                .ok_or_else(|| CollaborationError::NotFound(attempt_id.to_owned()))?;
            if let Some(receipt) = current.team_execution_command_receipts.get(&receipt_key) {
                if receipt.operation != "cancel-attempt" || receipt.request_digest != digest {
                    return Err(CollaborationError::Conflict(
                        "commandId belongs to another operation".to_owned(),
                    ));
                }
                return Ok(item);
            }
            if selected.revision != request.expected_team_revision {
                return Err(CollaborationError::Conflict(
                    "team revision changed before cancellation".to_owned(),
                ));
            }
            let now = Utc::now();
            match item.status.as_str() {
                "queued" => { item.status = "cancelled".to_owned(); item.execution_outcome = Some("cancelled".to_owned()); item.usage = Some(TeamReservation::default()); item.usage_revision += 1; }
                "running" | "cancellation_requested" => item.status = "cancellation_requested".to_owned(),
                "uncertain" => return Err(CollaborationError::Conflict("uncertain execution cannot be released by cancellation; reconcile provider/effect receipts".to_owned())),
                _ => return Err(CollaborationError::Conflict("attempt is already terminal".to_owned())),
            }
            item.state_reason = Some(request.reason.clone());
            item.updated_at = now;
            let mut next = current.clone();
            next.generation += 1;
            let updated = next
                .team_instances
                .get_mut(&team_key(owner, workspace, team_id))
                .ok_or_else(|| CollaborationError::NotFound(team_id.to_owned()))?;
            updated.revision += 1;
            updated.updated_at = now;
            if item.status == "cancelled"
                && let Some(task) = updated.tasks.iter_mut().find(|t| t.id == item.task_id)
            {
                task.status = "cancelled".to_owned();
                task.state_reason = item.state_reason.clone();
                task.revision += 1;
                task.updated_at = now;
            }
            next.team_execution_attempts
                .insert(key(owner, workspace, team_id, attempt_id), item.clone());
            next.team_execution_command_receipts.insert(
                receipt_key.clone(),
                TeamExecutionCommandReceipt {
                    owner_id: owner.to_owned(),
                    workspace_id: workspace.to_owned(),
                    team_id: team_id.to_owned(),
                    command_id: request.command_id.clone(),
                    operation: "cancel-attempt".to_owned(),
                    request_digest: digest.clone(),
                    attempt_id: Some(attempt_id.to_owned()),
                    recovery_attempt_ids: Vec::new(),
                    committed_at: now,
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(item);
            }
        }
        Err(CollaborationError::Conflict(
            "team cancellation changed concurrently".to_owned(),
        ))
    }

    pub async fn recover_team_attempts(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
    ) -> Result<Vec<TeamExecutionAttempt>, CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            team(&current, owner, workspace, team_id)?;
            let mut next = current.clone();
            let mut queued = Vec::new();
            let mut changed = false;
            for item in next.team_execution_attempts.values_mut().filter(|a| {
                a.owner_id == owner && a.workspace_id == workspace && a.team_id == team_id
            }) {
                match item.status.as_str() {
                    "queued" => queued.push(item.clone()),
                    "running" | "cancellation_requested" => {
                        item.status = "uncertain".to_owned();
                        item.updated_at = Utc::now();
                        item.state_reason = Some("Runtime interrupted after dispatch; inspect original run and provider/effect receipts before retrying".to_owned());
                        changed = true;
                    }
                    _ => {}
                }
            }
            if !changed {
                return Ok(queued);
            }
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                return Ok(queued);
            }
        }
        Err(CollaborationError::Conflict(
            "team recovery changed concurrently".to_owned(),
        ))
    }
}
