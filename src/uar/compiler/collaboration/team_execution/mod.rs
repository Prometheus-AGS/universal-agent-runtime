//! Catalog-owned team admission. Runtime workers consume durable dispatch intents.
mod admission;
mod context;
mod context_evidence;
mod continuations;
pub(crate) mod peer;
mod peer_commands;
mod waits;
pub use context::SelectedTeamContext;
mod dispatch;
mod model_capture;
mod ownership;
mod recovery;
mod resolution;
mod scope;
mod settlement;

use super::service::{CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS};
use crate::uar::domain::collaboration::CollaborationCatalogState;
use crate::uar::domain::team_execution::{TeamExecutionAttempt, TeamReservation};
use crate::uar::domain::team_planning::TeamInstance;

fn key(owner: &str, workspace: &str, team: &str, id: &str) -> String {
    format!("{owner}\u{1f}{workspace}\u{1f}{team}\u{1f}{id}")
}

fn team_key(owner: &str, workspace: &str, team: &str) -> String {
    format!("{owner}\u{1f}{workspace}\u{1f}{team}")
}

fn team<'a>(
    state: &'a CollaborationCatalogState,
    owner: &str,
    workspace: &str,
    id: &str,
) -> Result<&'a TeamInstance, CollaborationError> {
    super::service::validate_owner(owner)?;
    super::validation::validate_id(workspace)?;
    super::validation::validate_id(id)?;
    state
        .team_instances
        .get(&team_key(owner, workspace, id))
        .ok_or_else(|| CollaborationError::NotFound(id.to_owned()))
}

fn attempt<'a>(
    state: &'a CollaborationCatalogState,
    expected: &TeamExecutionAttempt,
) -> Result<&'a TeamExecutionAttempt, CollaborationError> {
    state
        .team_execution_attempts
        .get(&key(
            &expected.owner_id,
            &expected.workspace_id,
            &expected.team_id,
            &expected.id,
        ))
        .ok_or_else(|| CollaborationError::NotFound(expected.id.clone()))
}

fn fence(
    state: &CollaborationCatalogState,
    expected: &TeamExecutionAttempt,
) -> Result<(), CollaborationError> {
    self::ownership::require_attempt_fence(state, expected, false)?;
    let actual = attempt(state, expected)?;
    let team = team(
        state,
        &expected.owner_id,
        &expected.workspace_id,
        &expected.team_id,
    )?;
    let member = team
        .members
        .iter()
        .find(|m| m.id == expected.member_id)
        .ok_or_else(|| CollaborationError::Conflict("team member was removed".to_owned()))?;
    let task = team
        .tasks
        .iter()
        .find(|t| t.id == expected.task_id)
        .ok_or_else(|| CollaborationError::NotFound(expected.task_id.clone()))?;
    let binding = state
        .bindings
        .get(&super::bindings::binding_key(
            &expected.owner_id,
            &expected.workspace_id,
            &team.binding.id,
        ))
        .ok_or_else(|| CollaborationError::NotFound(team.binding.id.clone()))?;
    if matches!(member.status.as_str(), "revoked" | "stopped")
        || member.revision != expected.member_revision
        || task.assignee_member_id.as_deref() != Some(expected.member_id.as_str())
        || task.ownership_epoch != expected.ownership_epoch
        || !matches!(task.status.as_str(), "running" | "ready" | "queued")
        || binding.revision != expected.binding_revision
        || actual.execution_epoch != expected.execution_epoch
        || actual.run_id != expected.run_id
        || actual.member_id != expected.member_id
        || actual.task_id != expected.task_id
        || actual.member_revision != expected.member_revision
        || actual.ownership_epoch != expected.ownership_epoch
        || actual.binding_revision != expected.binding_revision
        || !matches!(actual.status.as_str(), "queued" | "running")
    {
        return Err(CollaborationError::Conflict(
            "team attempt authority is stale, cancelled, revoked or stopped".to_owned(),
        ));
    }
    Ok(())
}

fn add(left: &mut TeamReservation, right: &TeamReservation) -> Result<(), CollaborationError> {
    left.tokens = left
        .tokens
        .checked_add(right.tokens)
        .ok_or_else(|| CollaborationError::Conflict("token budget overflow".to_owned()))?;
    left.cost_microunits = left
        .cost_microunits
        .checked_add(right.cost_microunits)
        .ok_or_else(|| CollaborationError::Conflict("cost budget overflow".to_owned()))?;
    left.elapsed_seconds = left
        .elapsed_seconds
        .checked_add(right.elapsed_seconds)
        .ok_or_else(|| CollaborationError::Conflict("elapsed budget overflow".to_owned()))?;
    Ok(())
}

impl CollaborationCatalogService {
    pub async fn queued_team_attempts(
        &self,
    ) -> Result<Vec<TeamExecutionAttempt>, CollaborationError> {
        let state = self.load_state().await?;
        self.require_execution_owner(&state, false)?;
        Ok(state
            .team_execution_attempts
            .values()
            .filter(|a| a.status == "queued")
            .cloned()
            .collect())
    }

    pub async fn get_team_attempt(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
        id: &str,
    ) -> Result<TeamExecutionAttempt, CollaborationError> {
        let state = self.load_state().await?;
        team(&state, owner, workspace, team_id)?;
        state
            .team_execution_attempts
            .get(&key(owner, workspace, team_id, id))
            .cloned()
            .ok_or_else(|| CollaborationError::NotFound(id.to_owned()))
    }

    pub async fn revalidate_team_attempt(
        &self,
        expected: &TeamExecutionAttempt,
    ) -> Result<(), CollaborationError> {
        let state = self.load_state().await?;
        self.require_execution_owner(&state, false)?;
        fence(&state, expected)
    }

    pub async fn team_execution_summary(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
    ) -> Result<crate::uar::domain::team_execution::TeamExecutionSummary, CollaborationError> {
        let state = self.load_state().await?;
        let selected = team(&state, owner, workspace, team_id)?;
        let definition = state
            .definitions
            .get(&selected.definition.storage_key())
            .ok_or_else(|| CollaborationError::NotFound(selected.definition.id.clone()))?;
        let binding = state
            .bindings
            .get(&super::bindings::binding_key(
                owner,
                workspace,
                &selected.binding.id,
            ))
            .ok_or_else(|| CollaborationError::NotFound(selected.binding.id.clone()))?;
        let mut budget = definition.document["budget"].clone();
        for field in ["maxTokens", "maxCostMicrounits", "maxElapsedSeconds"] {
            if let (Some(cap), Some(declared)) = (
                binding.document["effectiveBudget"][field].as_u64(),
                budget[field].as_u64(),
            ) {
                budget[field] = serde_json::json!(cap.min(declared));
            }
        }
        let mut limits = definition.document["limits"].clone();
        for field in [
            "concurrentTurns",
            "maxMembers",
            "maxDepth",
            "maxPendingTasks",
        ] {
            if let (Some(cap), Some(declared)) = (
                binding.document["effectiveLimits"][field].as_u64(),
                limits[field].as_u64(),
            ) {
                limits[field] = serde_json::json!(cap.min(declared));
            }
        }
        let mut result = crate::uar::domain::team_execution::TeamExecutionSummary {
            attempts: Vec::new(),
            command_receipts: state
                .team_peer_commands
                .values()
                .filter(|r| {
                    r.scope.owner_id == owner
                        && r.scope.workspace_id == workspace
                        && r.scope.team_id == team_id
                })
                .cloned()
                .collect(),
            waits: state
                .team_waits
                .values()
                .filter(|w| {
                    w.authority.owner_id == owner
                        && w.authority.workspace_id == workspace
                        && w.authority.team_id == team_id
                })
                .cloned()
                .collect(),
            continuations: state
                .team_continuations
                .values()
                .filter(|w| {
                    w.authority.owner_id == owner
                        && w.authority.workspace_id == workspace
                        && w.authority.team_id == team_id
                })
                .cloned()
                .collect(),
            context_receipts: state
                .team_context_receipts
                .values()
                .filter(|w| {
                    w.authority.owner_id == owner
                        && w.authority.workspace_id == workspace
                        && w.authority.team_id == team_id
                })
                .cloned()
                .collect(),
            committed: TeamReservation::default(),
            reserved: TeamReservation::default(),
            uncertain_attempts: Vec::new(),
            budget,
            limits,
        };
        for item in state
            .team_execution_attempts
            .values()
            .filter(|a| a.owner_id == owner && a.workspace_id == workspace && a.team_id == team_id)
        {
            if let Some(usage) = &item.usage {
                add(&mut result.committed, usage)?;
            } else {
                add(&mut result.reserved, &item.reservation)?;
            }
            if item.status == "uncertain" {
                result.uncertain_attempts.push(item.id.clone());
            }
            result.attempts.push(item.clone());
        }
        // Historical context retains its provenance privately; disclosure is current-authority only.
        result.context_receipts.retain(|receipt| {
            let Some(attempt) = state.team_execution_attempts.get(&key(
                owner,
                workspace,
                team_id,
                &receipt.authority.attempt_id,
            )) else {
                return false;
            };
            peer::authorized_roster(&state, attempt).is_ok_and(|(visible, _, _)| {
                receipt
                    .roster
                    .iter()
                    .all(|m| visible.iter().any(|v| v.member_id == m.member_id))
            }) && receipt
                .target_outcomes
                .iter()
                .all(|o| peer::require_result_edge(&state, attempt, &o.member_id).is_ok())
                && receipt
                    .selections
                    .iter()
                    .filter(|s| {
                        s.source_kind
                            == crate::uar::domain::team_context::ContextSourceKind::Artifact
                            && s.disposition
                                == crate::uar::domain::team_context::ContextDisposition::Selected
                    })
                    .all(|selection| {
                        state
                            .team_artifacts
                            .values()
                            .find(|a| {
                                a.id == selection.source_id
                                    && a.owner_id == owner
                                    && a.workspace_id == workspace
                                    && a.team_id == team_id
                            })
                            .is_some_and(|a| {
                                peer::require_artifact_edge(&state, attempt, &a.member_id).is_ok()
                            })
                    })
        });
        for wait in &mut result.waits {
            let Some(attempt) = state.team_execution_attempts.get(&key(
                owner,
                workspace,
                team_id,
                &wait.authority.attempt_id,
            )) else {
                continue;
            };
            if wait
                .wake_outcomes
                .iter()
                .any(|o| peer::require_result_edge(&state, attempt, &o.member_id).is_err())
            {
                // Never return a partial outcome list as a successful projection.
                return Err(peer::denied("TEAM_EDGE_DENIED"));
            }
        }
        result.continuations.retain(|receipt| {
            state
                .team_execution_attempts
                .get(&key(
                    owner,
                    workspace,
                    team_id,
                    &receipt.authority.attempt_id,
                ))
                .is_some_and(|attempt| {
                    receipt
                        .target_outcomes
                        .iter()
                        .all(|o| peer::require_result_edge(&state, attempt, &o.member_id).is_ok())
                })
        });
        result.attempts.sort_by_key(|item| item.created_at);
        Ok(result)
    }
}
