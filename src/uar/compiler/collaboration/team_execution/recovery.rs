use super::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, key, team, team_key,
};
use crate::uar::domain::team_execution::{
    TeamControlRequest, TeamExecutionAttempt, TeamExecutionCommandReceipt,
};
use chrono::Utc;
use serde_json::json;

impl CollaborationCatalogService {
    pub async fn pending_team_yield_recoveries(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
    ) -> Result<
        Vec<(
            TeamExecutionAttempt,
            crate::uar::domain::team_wait::KernelTeamYield,
        )>,
        CollaborationError,
    > {
        let state = self.load_state().await?;
        self.require_execution_owner(&state, false)?;
        team(&state, owner, workspace, team_id)?;
        Ok(state
            .team_execution_attempts
            .values()
            .filter(|a| {
                a.owner_id == owner
                    && a.workspace_id == workspace
                    && a.team_id == team_id
                    && matches!(a.status.as_str(), "running" | "uncertain")
            })
            .filter_map(|a| {
                state
                    .team_yields
                    .get(&a.id)
                    .filter(|y| {
                        state
                            .team_waits
                            .get(&y.wait_id)
                            .is_some_and(|w| w.state == "yield_requested")
                    })
                    .map(|y| (a.clone(), y.clone()))
            })
            .collect())
    }

    pub async fn team_recovery_receipt(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
        request: &TeamControlRequest,
    ) -> Result<Option<Vec<TeamExecutionAttempt>>, CollaborationError> {
        let state = self.load_state().await?;
        self.require_execution_owner(&state, false)?;
        team(&state, owner, workspace, team_id)?;
        let receipt_key = format!("{owner}\u{1f}{workspace}\u{1f}{}", request.command_id);
        let Some(receipt) = state.team_execution_command_receipts.get(&receipt_key) else {
            return Ok(None);
        };
        let digest = super::super::validation::canonical_digest(
            &json!({"teamId":team_id,"request":request}),
        )?;
        if receipt.operation != "recover-team"
            || receipt.team_id != team_id
            || receipt.request_digest != digest
        {
            return Err(CollaborationError::Conflict(
                "commandId already belongs to another recovery or operation".to_owned(),
            ));
        }
        receipt
            .recovery_attempt_ids
            .iter()
            .map(|id| {
                state
                    .team_execution_attempts
                    .get(&key(owner, workspace, team_id, id))
                    .cloned()
                    .ok_or_else(|| {
                        CollaborationError::Storage("recovery receipt lost its attempt".to_owned())
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }

    pub async fn recover_team_attempts_control(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
        request: TeamControlRequest,
        joined_yields: Vec<(
            TeamExecutionAttempt,
            crate::uar::domain::team_wait::KernelTeamYield,
        )>,
    ) -> Result<Vec<TeamExecutionAttempt>, CollaborationError> {
        super::super::validation::validate_id(&request.command_id)?;
        if request.reason.trim().is_empty() {
            return Err(CollaborationError::Invalid(
                "recovery requires a reason".to_owned(),
            ));
        }
        let digest = super::super::validation::canonical_digest(
            &json!({"teamId":team_id,"request":request}),
        )?;
        let receipt_key = format!("{owner}\u{1f}{workspace}\u{1f}{}", request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            if let Some(result) = self
                .team_recovery_receipt(owner, workspace, team_id, &request)
                .await?
            {
                return Ok(result);
            }
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            let selected = team(&current, owner, workspace, team_id)?;
            // Retry when another writer committed between the two reads.
            if current
                .team_execution_command_receipts
                .contains_key(&receipt_key)
            {
                continue;
            }
            if current.team_command_receipts.contains_key(&receipt_key)
                || current
                    .team_mailbox_command_receipts
                    .contains_key(&receipt_key)
            {
                return Err(CollaborationError::Conflict(
                    "commandId belongs to another team command".to_owned(),
                ));
            }
            if selected.revision != request.expected_team_revision {
                return Err(CollaborationError::Conflict(
                    "team revision changed before recovery".to_owned(),
                ));
            }
            let now = Utc::now();
            let mut next = current.clone();
            for (attempt, control) in &joined_yields {
                if attempt.owner_id != owner
                    || attempt.workspace_id != workspace
                    || attempt.team_id != team_id
                {
                    return Err(CollaborationError::Conflict("TEAM_SCOPE_DENIED".into()));
                }
                super::waits::apply_confirmed_team_yield(&mut next, attempt, control, None)?;
            }
            let mut queued = Vec::new();
            for item in next.team_execution_attempts.values_mut().filter(|a| {
                a.owner_id == owner && a.workspace_id == workspace && a.team_id == team_id
            }) {
                match item.status.as_str() {
                    "queued" => queued.push(item.clone()),
                    "running" | "cancellation_requested" => {
                        item.status = "uncertain".to_owned();
                        item.effect_disposition = "uncertain".into();
                        item.accounting_state = "reserved-unknown".into();
                        item.updated_at = now;
                        item.state_reason=Some("Runtime interrupted after dispatch; reservation retained until original provider/effect receipts are reconciled".to_owned());
                    }
                    _ => {}
                }
            }
            let selected = next
                .team_instances
                .get_mut(&team_key(owner, workspace, team_id))
                .ok_or_else(|| CollaborationError::NotFound(team_id.to_owned()))?;
            for task in &mut selected.tasks {
                if next.team_execution_attempts.values().any(|a| {
                    a.owner_id == owner
                        && a.workspace_id == workspace
                        && a.team_id == team_id
                        && a.task_id == task.id
                        && a.status == "uncertain"
                        && a.ownership_epoch == task.ownership_epoch
                }) {
                    task.status = "blocked".to_owned();
                    task.state_reason =
                        Some("Original execution or usage remains uncertain".to_owned());
                    task.revision += 1;
                    task.updated_at = now;
                }
            }
            selected.revision += 1;
            selected.updated_at = now;
            self.evaluate_team_waits(&mut next).await?;
            queued = next
                .team_execution_attempts
                .values()
                .filter(|a| {
                    a.owner_id == owner
                        && a.workspace_id == workspace
                        && a.team_id == team_id
                        && a.status == "queued"
                })
                .cloned()
                .collect();
            next.generation += 1;
            next.team_execution_command_receipts.insert(
                receipt_key.clone(),
                TeamExecutionCommandReceipt {
                    owner_id: owner.to_owned(),
                    workspace_id: workspace.to_owned(),
                    team_id: team_id.to_owned(),
                    command_id: request.command_id.clone(),
                    operation: "recover-team".to_owned(),
                    request_digest: digest.clone(),
                    attempt_id: None,
                    recovery_attempt_ids: queued.iter().map(|a| a.id.clone()).collect(),
                    committed_at: now,
                },
            );
            if self.cas(current.generation, &next).await? {
                self.team_execution_notify.notify_one();
                return Ok(queued);
            }
        }
        Err(CollaborationError::Conflict(
            "team recovery changed concurrently".to_owned(),
        ))
    }
}
