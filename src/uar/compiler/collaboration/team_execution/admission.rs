use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use super::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, add, key, team, team_key,
};
use crate::uar::domain::team_execution::{
    AdmitTeamTaskRequest, TeamExecutionAttempt, TeamExecutionCommandReceipt, TeamReservation,
};

impl CollaborationCatalogService {
    pub async fn admit_team_task(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
        task_id: &str,
        request: AdmitTeamTaskRequest,
    ) -> Result<TeamExecutionAttempt, CollaborationError> {
        super::super::validation::validate_id(&request.command_id)?;
        super::super::validation::validate_id(task_id)?;
        if request.reservation.tokens == 0 || request.reservation.elapsed_seconds == 0 {
            return Err(CollaborationError::Invalid(
                "reserve positive tokens and elapsed seconds before execution".to_owned(),
            ));
        }
        let digest = super::super::validation::canonical_digest(
            &json!({"teamId":team_id,"taskId":task_id,"request":request}),
        )?;
        let command_key = format!("{owner}\u{1f}{workspace}\u{1f}{}", request.command_id);
        let id = Uuid::new_v4().to_string();
        let run_id = Uuid::new_v4().to_string();
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let execution_fence = self.require_execution_owner(&current, false)?;
            let selected = team(&current, owner, workspace, team_id)?;
            if let Some(receipt) = current.team_execution_command_receipts.get(&command_key) {
                if receipt.request_digest != digest
                    || receipt.operation != "admit-task"
                    || receipt.team_id != team_id
                {
                    return Err(CollaborationError::Conflict(
                        "commandId was used for another execution request".to_owned(),
                    ));
                }
                return current
                    .team_execution_attempts
                    .get(&key(
                        owner,
                        workspace,
                        team_id,
                        receipt.attempt_id.as_deref().ok_or_else(|| {
                            CollaborationError::Storage(
                                "admission receipt has no attempt".to_owned(),
                            )
                        })?,
                    ))
                    .cloned()
                    .ok_or_else(|| {
                        CollaborationError::Storage("admission receipt lost its attempt".to_owned())
                    });
            }
            if current.team_command_receipts.contains_key(&command_key)
                || current
                    .team_mailbox_command_receipts
                    .contains_key(&command_key)
            {
                return Err(CollaborationError::Conflict(
                    "commandId already belongs to a planning or mailbox command".to_owned(),
                ));
            }
            let task = selected
                .tasks
                .iter()
                .find(|task| task.id == task_id)
                .ok_or_else(|| CollaborationError::NotFound(task_id.to_owned()))?;
            let member = selected
                .members
                .iter()
                .find(|member| member.id == request.member_id)
                .ok_or_else(|| CollaborationError::NotFound(request.member_id.clone()))?;
            if selected.revision != request.expected_team_revision
                || task.revision != request.expected_task_revision
            {
                return Err(CollaborationError::Conflict(
                    "team or task revision changed before admission".to_owned(),
                ));
            }
            if selected.status == "cancelled"
                || matches!(member.status.as_str(), "revoked" | "stopped")
                || member.role != task.role
                || task.status != "ready"
                || task.assignee_member_id.as_deref() != Some(member.id.as_str())
                || !task.depends_on.iter().all(|dep| {
                    selected
                        .tasks
                        .iter()
                        .any(|t| &t.id == dep && t.status == "succeeded")
                })
            {
                return Err(CollaborationError::Conflict(
                    "task is not ready for its current assigned member".to_owned(),
                ));
            }
            let binding = current
                .bindings
                .get(&super::super::bindings::binding_key(
                    owner,
                    workspace,
                    &selected.binding.id,
                ))
                .ok_or_else(|| CollaborationError::NotFound(selected.binding.id.clone()))?;
            if binding.revision != selected.binding.revision {
                return Err(CollaborationError::Conflict(
                    "team deployment binding changed; reinstall the team binding".to_owned(),
                ));
            }
            let definition = current
                .definitions
                .get(&selected.definition.storage_key())
                .ok_or_else(|| CollaborationError::NotFound(selected.definition.id.clone()))?;
            if definition.document["budget"]["currency"].as_str() != Some("USD")
                || binding.document["effectiveBudget"]["currency"].as_str() != Some("USD")
            {
                return Err(CollaborationError::Invalid("Runtime pricing is in USD; team and effective binding budget currency must be USD".to_owned()));
            }
            let mut consumed = TeamReservation::default();
            let mut active = 0_u64;
            for prior in current.team_execution_attempts.values().filter(|a| {
                a.owner_id == owner && a.workspace_id == workspace && a.team_id == team_id
            }) {
                add(
                    &mut consumed,
                    prior.usage.as_ref().unwrap_or(&prior.reservation),
                )?;
                if matches!(
                    prior.status.as_str(),
                    "queued" | "running" | "cancellation_requested"
                ) || (prior.status == "uncertain"
                    && !matches!(
                        prior.execution_outcome.as_deref(),
                        Some("succeeded" | "failed" | "cancelled")
                    ))
                {
                    active += 1;
                    if prior.member_id == member.id {
                        return Err(CollaborationError::Conflict("member has an active or uncertain attempt; resolve it before another turn".to_owned()));
                    }
                }
            }
            let concurrency = definition.document["limits"]["concurrentTurns"]
                .as_u64()
                .ok_or_else(|| {
                    CollaborationError::Storage("team concurrentTurns is missing".to_owned())
                })?;
            let narrowed = binding.document["effectiveLimits"]["concurrentTurns"]
                .as_u64()
                .unwrap_or(concurrency)
                .min(concurrency);
            if active >= narrowed {
                return Err(CollaborationError::Conflict(
                    "team concurrent turn limit reached".to_owned(),
                ));
            }
            add(&mut consumed, &request.reservation)?;
            for (field, amount) in [
                ("maxTokens", consumed.tokens),
                ("maxCostMicrounits", consumed.cost_microunits),
                ("maxElapsedSeconds", consumed.elapsed_seconds),
            ] {
                let declared = definition.document["budget"][field]
                    .as_u64()
                    .ok_or_else(|| {
                        CollaborationError::Storage(format!("team budget {field} is missing"))
                    })?;
                let cap = binding.document["effectiveBudget"][field]
                    .as_u64()
                    .unwrap_or(declared)
                    .min(declared);
                if amount > cap {
                    return Err(CollaborationError::Conflict(format!(
                        "aggregate team {field} reservation exhausted ({amount}/{cap})"
                    )));
                }
            }
            let mut artifact_ids = std::collections::BTreeSet::new();
            for artifact_id in &request.context_artifact_ids {
                if !artifact_ids.insert(artifact_id) {
                    return Err(CollaborationError::Invalid(
                        "context contains duplicate artifact IDs".to_owned(),
                    ));
                }
                let artifact = current
                    .team_artifacts
                    .get(&key(owner, workspace, team_id, artifact_id))
                    .ok_or_else(|| CollaborationError::NotFound(artifact_id.clone()))?;
                if artifact.owner_id != owner
                    || artifact.workspace_id != workspace
                    || artifact.team_id != team_id
                {
                    return Err(CollaborationError::Conflict(
                        "artifact is outside this team workspace".to_owned(),
                    ));
                }
            }
            let now = Utc::now();
            let attempt = TeamExecutionAttempt {
                id: id.clone(),
                run_id: run_id.clone(),
                owner_id: owner.to_owned(),
                workspace_id: workspace.to_owned(),
                team_id: team_id.to_owned(),
                task_id: task_id.to_owned(),
                member_id: member.id.clone(),
                member_revision: member.revision,
                ownership_epoch: task.ownership_epoch,
                binding_revision: binding.revision,
                execution_epoch: 1,
                execution_fence: Some(execution_fence),
                effective_models: self.capture_team_models(&current, selected, member).await?,
                effect_disposition: "confirmed".into(),
                accounting_state: "reserved-unknown".into(),
                status: "queued".to_owned(),
                execution_outcome: None,
                reservation: request.reservation.clone(),
                context_artifact_ids: request.context_artifact_ids.clone(),
                usage: None,
                usage_revision: 0,
                output: None,
                state_reason: None,
                diagnostic: None,
                created_at: now,
                updated_at: now,
            };
            let mut next = current.clone();
            let updated = next
                .team_instances
                .get_mut(&team_key(owner, workspace, team_id))
                .ok_or_else(|| CollaborationError::NotFound(team_id.to_owned()))?;
            let task = updated
                .tasks
                .iter_mut()
                .find(|t| t.id == task_id)
                .ok_or_else(|| CollaborationError::NotFound(task_id.to_owned()))?;
            task.status = "running".to_owned();
            task.revision += 1;
            task.updated_at = now;
            updated.revision += 1;
            updated.updated_at = now;
            next.generation += 1;
            next.team_execution_attempts
                .insert(key(owner, workspace, team_id, &id), attempt.clone());
            next.team_execution_command_receipts.insert(
                command_key.clone(),
                TeamExecutionCommandReceipt {
                    owner_id: owner.to_owned(),
                    workspace_id: workspace.to_owned(),
                    team_id: team_id.to_owned(),
                    command_id: request.command_id.clone(),
                    operation: "admit-task".to_owned(),
                    request_digest: digest.clone(),
                    attempt_id: Some(id.clone()),
                    recovery_attempt_ids: Vec::new(),
                    committed_at: now,
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(attempt);
            }
        }
        Err(CollaborationError::Conflict(
            "team admission changed concurrently".to_owned(),
        ))
    }
}
