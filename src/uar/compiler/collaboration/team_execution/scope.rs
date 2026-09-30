//! Explicit artifact disclosure and revisioned membership revocation.

use chrono::Utc;
use serde_json::{Value, json};

use crate::uar::domain::team_execution::{
    TeamArtifact, TeamControlRequest, TeamExecutionAttempt, TeamExecutionCommandReceipt,
    TeamReservation,
};
use crate::uar::domain::team_planning::TeamInstance;

use super::super::validation::{request_digest, validate_id};
use super::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, attempt, fence, key, team,
    team_key,
};

impl CollaborationCatalogService {
    /// Lists artifacts only within the authenticated owner, workspace and team.
    pub async fn list_team_artifacts(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
    ) -> Result<Vec<TeamArtifact>, CollaborationError> {
        let state = self.load_state().await?;
        team(&state, owner, workspace, team_id)?;
        let mut artifacts: Vec<_> = state
            .team_artifacts
            .into_values()
            .filter(|artifact| {
                artifact.owner_id == owner
                    && artifact.workspace_id == workspace
                    && artifact.team_id == team_id
            })
            .collect();
        artifacts.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(artifacts)
    }

    /// Projects declared task input and only explicitly selected durable artifacts.
    pub async fn selected_team_context(
        &self,
        expected: &TeamExecutionAttempt,
    ) -> Result<Value, CollaborationError> {
        let state = self.load_state().await?;
        fence(&state, expected)?;
        let actual = attempt(&state, expected)?;
        let current_team = team(
            &state,
            &actual.owner_id,
            &actual.workspace_id,
            &actual.team_id,
        )?;
        let task = current_team
            .tasks
            .iter()
            .find(|task| task.id == actual.task_id)
            .ok_or_else(|| CollaborationError::NotFound(actual.task_id.clone()))?;
        let artifacts = actual
            .context_artifact_ids
            .iter()
            .map(|id| {
                validate_id(id)?;
                state
                    .team_artifacts
                    .get(&key(
                        &actual.owner_id,
                        &actual.workspace_id,
                        &actual.team_id,
                        id,
                    ))
                    .filter(|artifact| {
                        artifact.owner_id == actual.owner_id
                            && artifact.workspace_id == actual.workspace_id
                            && artifact.team_id == actual.team_id
                            && artifact.id == *id
                    })
                    .cloned()
                    .ok_or_else(|| CollaborationError::NotFound(id.clone()))
            })
            .collect::<Result<Vec<_>, CollaborationError>>()?;
        Ok(json!({
            "teamInput": current_team.input,
            "taskInput": task.input,
            "taskOutputContract": task.output_contract,
            "artifacts": artifacts,
        }))
    }

    /// Persists one immutable output artifact for the currently authorized attempt.
    pub async fn add_team_artifact(
        &self,
        expected: &TeamExecutionAttempt,
        content: Value,
    ) -> Result<TeamArtifact, CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            fence(&current, expected)?;
            let actual = attempt(&current, expected)?;
            let id = format!("artifact-{}", actual.id);
            let artifact_key = key(&actual.owner_id, &actual.workspace_id, &actual.team_id, &id);
            if let Some(existing) = current.team_artifacts.get(&artifact_key) {
                if existing.owner_id != actual.owner_id
                    || existing.workspace_id != actual.workspace_id
                    || existing.team_id != actual.team_id
                    || existing.task_id != actual.task_id
                    || existing.member_id != actual.member_id
                    || existing.attempt_id != actual.id
                    || existing.content != content
                {
                    return Err(CollaborationError::Conflict(
                        "attempt output artifact is immutable".to_owned(),
                    ));
                }
                return Ok(existing.clone());
            }
            let artifact = TeamArtifact {
                id,
                owner_id: actual.owner_id.clone(),
                workspace_id: actual.workspace_id.clone(),
                team_id: actual.team_id.clone(),
                task_id: actual.task_id.clone(),
                member_id: actual.member_id.clone(),
                attempt_id: actual.id.clone(),
                content: content.clone(),
                created_at: Utc::now(),
            };
            let mut next = current.clone();
            next.generation = increment(current.generation, "catalog generation")?;
            next.team_artifacts.insert(artifact_key, artifact.clone());
            if self.cas(current.generation, &next).await? {
                return Ok(artifact);
            }
        }
        Err(CollaborationError::Conflict(
            "team artifacts changed during output persistence".to_owned(),
        ))
    }

    /// Fences the member and all current task claims without releasing unknown usage.
    pub async fn revoke_team_member(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
        member_id: &str,
        request: TeamControlRequest,
    ) -> Result<TeamInstance, CollaborationError> {
        validate_id(member_id)?;
        validate_id(&request.command_id)?;
        if request.reason.trim().is_empty() {
            return Err(CollaborationError::Invalid(
                "membership revocation requires a reason".to_owned(),
            ));
        }
        let digest = request_digest(&json!({"memberId": member_id, "request": request}))?;
        let operation = format!("revoke-member:{member_id}");
        let receipt_key = format!("{owner}\u{1f}{workspace}\u{1f}{}", request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            let mut current_team = team(&current, owner, workspace, team_id)?.clone();
            if let Some(receipt) = current.team_execution_command_receipts.get(&receipt_key) {
                if receipt.owner_id != owner
                    || receipt.workspace_id != workspace
                    || receipt.team_id != team_id
                    || receipt.operation != operation
                    || receipt.request_digest != digest
                {
                    return Err(CollaborationError::Conflict(
                        "commandId already belongs to another team operation or payload".to_owned(),
                    ));
                }
                return Ok(current_team);
            }
            if current.team_command_receipts.contains_key(&receipt_key)
                || current
                    .team_mailbox_command_receipts
                    .contains_key(&receipt_key)
            {
                return Err(CollaborationError::Conflict(
                    "commandId already belongs to a planning or mailbox operation".to_owned(),
                ));
            }
            if current_team.revision != request.expected_team_revision {
                return Err(CollaborationError::Conflict(format!(
                    "expectedTeamRevision {} does not match {}",
                    request.expected_team_revision, current_team.revision,
                )));
            }
            let member = current_team
                .members
                .iter_mut()
                .find(|member| member.id == member_id)
                .ok_or_else(|| CollaborationError::NotFound(member_id.to_owned()))?;
            if member.status == "revoked" {
                return Err(CollaborationError::Conflict(
                    "membership is already revoked".to_owned(),
                ));
            }
            member.status = "revoked".to_owned();
            member.revision = increment(member.revision, "member revision")?;
            let now = Utc::now();
            for task in &mut current_team.tasks {
                let owns = task.assignee_member_id.as_deref() == Some(member_id);
                let reviews = task.reviewer_member_id.as_deref() == Some(member_id);
                if owns || reviews {
                    if owns {
                        task.ownership_epoch = increment(task.ownership_epoch, "ownership epoch")?;
                        task.assignment_authority = None;
                    }
                    if reviews {
                        task.reviewer_member_id = None;
                        task.reviewer_epoch = increment(task.reviewer_epoch, "reviewer epoch")?;
                    }
                    task.revision = increment(task.revision, "task revision")?;
                    task.state_reason = Some(request.reason.trim().to_owned());
                    task.updated_at = now;
                }
            }
            current_team.revision = increment(current_team.revision, "team revision")?;
            current_team.updated_at = now;
            let mut next = current.clone();
            next.generation = increment(current.generation, "catalog generation")?;
            for active in next.team_execution_attempts.values_mut().filter(|active| {
                active.owner_id == owner
                    && active.workspace_id == workspace
                    && active.team_id == team_id
                    && active.member_id == member_id
                    && matches!(active.status.as_str(), "queued" | "running")
            }) {
                if active.status == "queued" {
                    active.status = "cancelled".to_owned();
                    active.execution_outcome = Some("cancelled".to_owned());
                    active.effect_disposition = "confirmed".into();
                    active.accounting_state = "settled".into();
                    active.usage = Some(TeamReservation::default());
                    active.usage_revision = increment(active.usage_revision, "usage revision")?;
                    if let Some(task) = current_team.tasks.iter_mut().find(|task| {
                        task.id == active.task_id
                            && task.assignee_member_id.as_deref() == Some(member_id)
                    }) {
                        task.status = "cancelled".to_owned();
                    }
                } else {
                    active.status = "cancellation_requested".to_owned();
                    if let Some(task) = current_team.tasks.iter_mut().find(|task| {
                        task.id == active.task_id
                            && task.assignee_member_id.as_deref() == Some(member_id)
                    }) {
                        task.status = "blocked".to_owned();
                        task.state_reason = Some(format!(
                            "Member revoked; awaiting original attempt completion: {}",
                            request.reason.trim(),
                        ));
                    }
                }
                active.state_reason = Some(request.reason.trim().to_owned());
                active.updated_at = now;
            }
            next.team_instances
                .insert(team_key(owner, workspace, team_id), current_team.clone());
            next.team_execution_command_receipts.insert(
                receipt_key.clone(),
                TeamExecutionCommandReceipt {
                    owner_id: owner.to_owned(),
                    workspace_id: workspace.to_owned(),
                    team_id: team_id.to_owned(),
                    command_id: request.command_id.clone(),
                    operation: operation.clone(),
                    request_digest: digest.clone(),
                    attempt_id: None,
                    recovery_attempt_ids: Vec::new(),
                    committed_at: now,
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(current_team);
            }
        }
        Err(CollaborationError::Conflict(
            "team membership changed during revocation".to_owned(),
        ))
    }
}

fn increment(current: u64, label: &str) -> Result<u64, CollaborationError> {
    current
        .checked_add(1)
        .ok_or_else(|| CollaborationError::Conflict(format!("{label} is exhausted")))
}
