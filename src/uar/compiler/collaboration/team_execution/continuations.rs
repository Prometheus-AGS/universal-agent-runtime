//! Existing controller continuation readiness and aggregate CAS admission.
use super::{CollaborationCatalogService, CollaborationError, key, peer::*, team, team_key};
use crate::uar::domain::{
    collaboration::CollaborationCatalogState, team_execution::*, team_wait::*,
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

impl CollaborationCatalogService {
    pub(crate) async fn evaluate_team_waits(
        &self,
        state: &mut CollaborationCatalogState,
    ) -> Result<(), CollaborationError> {
        let ids: Vec<_> = state
            .team_waits
            .values()
            .filter(|w| matches!(w.state.as_str(), "waiting" | "blocked"))
            .map(|w| w.wait_id.clone())
            .collect();
        for id in ids {
            let mut wait = state.team_waits[&id].clone();
            let prior = state
                .team_execution_attempts
                .values()
                .find(|a| a.id == wait.authority.attempt_id)
                .cloned()
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
            let selected =
                team(state, &prior.owner_id, &prior.workspace_id, &prior.team_id)?.clone();
            let member = selected
                .members
                .iter()
                .find(|m| m.id == prior.member_id)
                .cloned();
            let task = selected.tasks.iter().find(|t| t.id == prior.task_id);
            if member.as_ref().is_none_or(|m| {
                m.revision != prior.member_revision
                    || matches!(m.status.as_str(), "revoked" | "stopped")
            }) || task.is_none_or(|t| {
                t.ownership_epoch != prior.ownership_epoch
                    || t.assignee_member_id.as_deref() != Some(prior.member_id.as_str())
                    || t.status != "waiting"
            }) || selected.binding.revision != prior.binding_revision
                || state
                    .bindings
                    .get(&super::super::bindings::binding_key(
                        &prior.owner_id,
                        &prior.workspace_id,
                        &selected.binding.id,
                    ))
                    .is_none_or(|b| b.revision != prior.binding_revision)
                || selected.status == "cancelled"
            {
                wait.state = "invalidated".into();
                wait.reason_code = Some("TEAM_WAIT_INVALIDATED".into());
                state.team_waits.insert(id, wait);
                continue;
            }
            let mut outcomes = Vec::new();
            let mut reason = None;
            for target in &wait.target_task_ids {
                let attempt = state
                    .team_execution_attempts
                    .values()
                    .filter(|a| {
                        a.owner_id == prior.owner_id
                            && a.workspace_id == prior.workspace_id
                            && a.team_id == prior.team_id
                            && a.task_id == *target
                    })
                    .max_by_key(|a| a.created_at);
                let Some(attempt) = attempt else {
                    outcomes.clear();
                    break;
                };
                if require_result_edge(state, &prior, &attempt.member_id).is_err() {
                    reason = Some("TEAM_EDGE_DENIED");
                    outcomes.clear();
                    break;
                }
                if attempt.effect_disposition != "confirmed"
                    || !matches!(
                        attempt.execution_outcome.as_deref(),
                        Some("succeeded" | "failed" | "cancelled")
                    )
                {
                    outcomes.clear();
                    break;
                }
                let target_task = selected
                    .tasks
                    .iter()
                    .find(|t| t.id == *target)
                    .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
                if target_task.ownership_epoch != attempt.ownership_epoch
                    || target_task.assignee_member_id.as_deref() != Some(attempt.member_id.as_str())
                {
                    reason = Some("TEAM_WAIT_INVALIDATED");
                    outcomes.clear();
                    break;
                }
                outcomes.push(TargetOutcome {
                    task_id: target.clone(),
                    member_id: attempt.member_id.clone(),
                    attempt_id: attempt.id.clone(),
                    execution_outcome: attempt.execution_outcome.clone().unwrap_or_default(),
                    effect_disposition: "confirmed".into(),
                    artifact_ids: state
                        .team_artifacts
                        .values()
                        .filter(|a| {
                            a.attempt_id == attempt.id
                                && a.owner_id == prior.owner_id
                                && a.workspace_id == prior.workspace_id
                                && a.team_id == prior.team_id
                        })
                        .map(|a| a.id.clone())
                        .collect(),
                });
            }
            if let Some(code) = reason {
                wait.state = "invalidated".into();
                wait.reason_code = Some(code.into());
                state.team_waits.insert(id, wait);
                continue;
            }
            if outcomes.len() != wait.target_task_ids.len() {
                continue;
            }
            if state
                .execution_claim
                .as_ref()
                .is_none_or(|o| o.state != "held")
            {
                continue;
            }
            wait.wake_outcomes = outcomes.clone();
            if reserve(state, &prior, &wait.continuation_reservation).is_err() {
                wait.state = "blocked".into();
                wait.reason_code = Some("TEAM_BUDGET_EXHAUSTED".into());
                state.team_waits.insert(id, wait);
                continue;
            }
            let Some(member) = member else {
                continue;
            };
            let models = match self.capture_team_models(state, &selected, &member).await {
                Ok(models) => models,
                Err(_) => {
                    wait.state = "blocked".into();
                    wait.reason_code = Some("TEAM_ROUTE_PROFILE_MISMATCH".into());
                    state.team_waits.insert(id, wait);
                    continue;
                }
            };
            let fence = self.require_execution_owner(state, false)?;
            let now = Utc::now();
            let root_id = Uuid::new_v4().to_string();
            let artifact_ids: Vec<_> = outcomes
                .iter()
                .flat_map(|o| o.artifact_ids.clone())
                .chain(wait.continuation_input.artifact_ids.clone())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            let mut next = prior.clone();
            next.id = Uuid::new_v4().to_string();
            next.run_id = Uuid::new_v4().to_string();
            next.root_id = root_id.clone();
            next.approval_scope_id = root_id.clone();
            next.queue_sequence = state.generation;
            next.continuation_of_wait_id = Some(id.clone());
            next.execution_fence = Some(fence.clone());
            next.effective_models = models;
            next.status = "queued".into();
            next.execution_epoch = prior
                .execution_epoch
                .checked_add(1)
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
            next.execution_outcome = None;
            next.effect_disposition = "confirmed".into();
            next.accounting_state = "reserved-unknown".into();
            next.reservation = wait.continuation_reservation.clone();
            next.context_artifact_ids = artifact_ids.clone();
            next.usage = None;
            next.usage_revision = 0;
            next.output = None;
            next.state_reason = None;
            next.diagnostic = None;
            next.created_at = now;
            next.updated_at = now;
            let receipt = ContinuationReceipt {
                wait_id: id.clone(),
                uniqueness_key: format!("team-wait:{id}:continuation:1"),
                previous_attempt_id: prior.id.clone(),
                authority: authority(state, &next)?,
                root_id: root_id.clone(),
                approval_scope_id: root_id,
                authorization_revision: state.generation,
                task_id: next.task_id.clone(),
                continuation_attempt_id: next.id.clone(),
                run_id: next.run_id.clone(),
                execution_fence: fence,
                reservation: next.reservation.clone(),
                selected_artifact_ids: artifact_ids,
                target_outcomes: outcomes.clone(),
                committed_at: now,
            };
            wait.state = "resumed".into();
            wait.reason_code = None;
            wait.continuation_attempt_id = Some(next.id.clone());
            state.team_continuations.insert(id.clone(), receipt);
            let internal_command = format!("team-wait:{id}:continuation:1");
            let internal_key = format!(
                "{}\u{1f}{}\u{1f}{}",
                next.owner_id, next.workspace_id, internal_command
            );
            state.team_execution_command_receipts.insert(
                internal_key,
                TeamExecutionCommandReceipt {
                    owner_id: next.owner_id.clone(),
                    workspace_id: next.workspace_id.clone(),
                    team_id: next.team_id.clone(),
                    command_id: internal_command,
                    operation: "continue-team-wait".into(),
                    request_digest: super::super::validation::canonical_digest(
                        &json!({"waitId":id,"targetOutcomes":outcomes}),
                    )?,
                    attempt_id: Some(next.id.clone()),
                    recovery_attempt_ids: vec![],
                    committed_at: now,
                },
            );
            state.team_waits.insert(id, wait);
            state.team_execution_attempts.insert(
                key(&next.owner_id, &next.workspace_id, &next.team_id, &next.id),
                next.clone(),
            );
            let selected = state
                .team_instances
                .get_mut(&team_key(&next.owner_id, &next.workspace_id, &next.team_id))
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
            let task = selected
                .tasks
                .iter_mut()
                .find(|t| t.id == next.task_id)
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
            task.status = "ready".into();
            task.revision += 1;
            task.updated_at = now;
            selected.revision += 1;
            selected.updated_at = now;
        }
        Ok(())
    }
}
