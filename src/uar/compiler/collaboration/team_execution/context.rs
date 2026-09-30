//! Attributed context selection under current attempt and directed-edge authority.

use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::{Value, json};

use crate::uar::domain::{
    collaboration::ResolvedSkill,
    team_context::{
        ContextDisposition, ContextSelection, ContextSourceKind, MAX_CONTEXT_ROSTER_MEMBERS,
        MAX_CONTEXT_SELECTIONS, MAX_ROSTER_CAPABILITIES, MAX_ROSTER_LABEL_BYTES,
        TEAM_INSTRUCTION_ORDER, TeamContextCountQuality, TeamContextReceipt, TeamOutcomeDataTrust,
    },
    team_execution::{TeamArtifact, TeamExecutionAttempt},
    team_wait::TargetOutcome,
};

use super::super::validation::resolve_team_instructions;
use super::{CollaborationCatalogService, CollaborationError, attempt, fence, key, peer, team};

/// Guidance is separate from the attributed data handed to the model.
pub struct SelectedTeamContext {
    pub receipt: TeamContextReceipt,
    pub data: Value,
}

impl CollaborationCatalogService {
    pub async fn selected_team_context(
        &self,
        expected: &TeamExecutionAttempt,
        resolved_skills: &[ResolvedSkill],
    ) -> Result<SelectedTeamContext, CollaborationError> {
        let state = self.load_state().await?;
        fence(&state, expected)?;
        let actual = attempt(&state, expected)?;
        let current_team = team(&state, &actual.owner_id, &actual.workspace_id, &actual.team_id)?;
        let task = current_team.tasks.iter().find(|task| task.id == actual.task_id)
            .ok_or_else(|| CollaborationError::NotFound(actual.task_id.clone()))?;
        let document = peer::team_document(&state, actual)?;
        let team_instructions = resolve_team_instructions(document)
            .map_err(|error| CollaborationError::Conflict(error.to_string()))?;
        let (roster, coordinator_member_id, authorization_revision) =
            peer::authorized_roster(&state, actual)?;
        if roster.len() > MAX_CONTEXT_ROSTER_MEMBERS {
            return Err(required_too_large("/roster"));
        }
        for member in &roster {
            profile_id(&member.member_id, "/roster/memberId")?;
            profile_id(&member.role, "/roster/role")?;
            if member.label.len() > MAX_ROSTER_LABEL_BYTES
                || member.safe_capabilities.len() > MAX_ROSTER_CAPABILITIES
            {
                return Err(required_too_large("/roster"));
            }
            let mut capabilities = BTreeSet::new();
            for capability in &member.safe_capabilities {
                profile_id(capability, "/roster/safeCapabilities")?;
                if !capabilities.insert(capability) {
                    return Err(required_unsupported("/roster/safeCapabilities"));
                }
            }
        }
        let self_member = roster.iter().find(|member| member.member_id == actual.member_id)
            .cloned().ok_or_else(|| peer::denied("TEAM_SCOPE_DENIED"))?;
        if actual.root_id.is_empty() || actual.approval_scope_id != actual.root_id {
            return Err(required_unsupported("/rootId"));
        }

        let messages = peer::authorized_inbox(&state, actual)?;
        let target_outcomes = continuation_outcomes(&state, actual)?;
        let mut selected_ids = BTreeSet::new();
        let mut artifacts = Vec::<TeamArtifact>::new();
        for id in actual.context_artifact_ids.iter()
            .chain(messages.iter().flat_map(|message| message.payload.artifact_ids.iter()))
            .chain(target_outcomes.iter().flat_map(|outcome| outcome.artifact_ids.iter()))
        {
            profile_id(id, "/contextArtifactIds")?;
            if !selected_ids.insert(id.clone()) {
                continue;
            }
            let artifact = state.team_artifacts
                .get(&key(&actual.owner_id, &actual.workspace_id, &actual.team_id, id))
                .filter(|artifact| artifact.owner_id == actual.owner_id
                    && artifact.workspace_id == actual.workspace_id
                    && artifact.team_id == actual.team_id && artifact.id == *id)
                .cloned().ok_or_else(|| peer::denied("TEAM_SCOPE_DENIED"))?;
            peer::require_artifact_edge(&state, actual, &artifact.member_id)?;
            artifacts.push(artifact);
        }
        let mut selections = vec![
            selection(&current_team.id, ContextSourceKind::TeamInput, &current_team.input)?,
            selection(&task.id, ContextSourceKind::TaskInput, &task.input)?,
        ];
        for artifact in &artifacts {
            selections.push(selection(&artifact.id, ContextSourceKind::Artifact, artifact)?);
        }
        for message in &messages {
            selections.push(selection(&message.message_id, ContextSourceKind::Message, message)?);
        }
        // These are exact installed references/configuration, not claims that
        // each skill body was activated or handed to the model.
        let skills = resolved_skills.iter().map(|resolved| &resolved.skill).collect::<Vec<_>>();
        for skill in &skills {
            selections.push(selection(&skill.id, ContextSourceKind::Skill, skill)?);
        }
        if selections.len() > MAX_CONTEXT_SELECTIONS {
            return Err(required_too_large("/selections"));
        }
        let authority = peer::authority(&state, actual)?;
        let receipt = TeamContextReceipt {
            authority,
            root_id: actual.root_id.clone(),
            approval_scope_id: actual.approval_scope_id.clone(),
            team_instructions,
            instruction_order: TEAM_INSTRUCTION_ORDER,
            self_member,
            coordinator_member_id,
            roster,
            authorization_revision,
            selections,
            target_outcomes,
            target_outcome_data_trust: TeamOutcomeDataTrust::UntrustedAttributedData,
            context_budget_tokens: actual.reservation.tokens,
            count_quality: TeamContextCountQuality::Unknown,
        };
        // Neither payload text nor artifact content is promoted into instruction authority.
        let data = json!({
            "self": receipt.self_member,
            "assignment": {"taskId": task.id, "role": task.role, "title": task.title,
                "teamRevision": current_team.revision},
            "mission": document["purpose"],
            "coordinatorMemberId": receipt.coordinator_member_id,
            "roster": receipt.roster,
            "teamInput": current_team.input,
            "taskInput": task.input,
            "taskOutputContract": task.output_contract,
            "artifacts": artifacts,
            "messages": messages,
            "skills": skills,
            "targetOutcomes": receipt.target_outcomes,
            "targetOutcomeDataTrust": receipt.target_outcome_data_trust,
        });
        Ok(SelectedTeamContext { receipt, data })
    }
}

fn continuation_outcomes(
    state: &crate::uar::domain::collaboration::CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
) -> Result<Vec<TargetOutcome>, CollaborationError> {
    let Some(wait_id) = &attempt.continuation_of_wait_id else {
        return Ok(Vec::new());
    };
    let continuation = state.team_continuations.values()
        .find(|record| record.continuation_attempt_id == attempt.id)
        .ok_or_else(|| peer::denied("TEAM_WAIT_INVALIDATED"))?;
    let wait = state.team_waits.get(wait_id)
        .ok_or_else(|| peer::denied("TEAM_WAIT_INVALIDATED"))?;
    let authority = &continuation.authority;
    if continuation.wait_id != *wait_id
        || authority.owner_id != attempt.owner_id || authority.workspace_id != attempt.workspace_id
        || authority.team_id != attempt.team_id || authority.task_id != attempt.task_id
        || authority.member_id != attempt.member_id || authority.attempt_id != attempt.id
        || authority.run_id != attempt.run_id
        || authority.task_ownership_epoch != attempt.ownership_epoch
        || authority.member_revision != attempt.member_revision
        || authority.binding.revision != attempt.binding_revision
        || continuation.task_id != attempt.task_id || continuation.run_id != attempt.run_id
        || continuation.root_id != attempt.root_id
        || continuation.approval_scope_id != attempt.approval_scope_id
        || wait.authority.owner_id != attempt.owner_id
        || wait.authority.workspace_id != attempt.workspace_id
        || wait.authority.team_id != attempt.team_id
        || wait.authority.task_id != attempt.task_id
        || wait.authority.member_id != attempt.member_id
        || wait.authority.task_ownership_epoch != attempt.ownership_epoch
        || wait.continuation_attempt_id.as_deref() != Some(attempt.id.as_str())
        || wait.state != "resumed"
        || continuation.target_outcomes.len() != wait.target_task_ids.len()
        || continuation.target_outcomes.is_empty() || continuation.target_outcomes.len() > 16
        || serde_json::to_value(&continuation.target_outcomes)? != serde_json::to_value(&wait.wake_outcomes)?
    {
        return Err(peer::denied("TEAM_WAIT_INVALIDATED"));
    }
    for (target_id, outcome) in wait.target_task_ids.iter().zip(&continuation.target_outcomes) {
        if outcome.task_id != *target_id || outcome.effect_disposition != "confirmed"
            || !matches!(outcome.execution_outcome.as_str(), "succeeded" | "failed" | "cancelled")
        {
            return Err(peer::denied("TEAM_WAIT_INVALIDATED"));
        }
        peer::require_result_edge(state, attempt, &outcome.member_id)?;
        let target = state.team_execution_attempts
            .get(&key(&attempt.owner_id, &attempt.workspace_id, &attempt.team_id, &outcome.attempt_id))
            .filter(|target| target.task_id == outcome.task_id && target.member_id == outcome.member_id
                && target.execution_outcome.as_deref() == Some(outcome.execution_outcome.as_str())
                && target.effect_disposition == "confirmed")
            .ok_or_else(|| peer::denied("TEAM_WAIT_INVALIDATED"))?;
        let current_team = team(state, &attempt.owner_id, &attempt.workspace_id, &attempt.team_id)?;
        if !current_team.tasks.iter().any(|task| task.id == outcome.task_id
            && task.assignee_member_id.as_deref() == Some(outcome.member_id.as_str())
            && task.ownership_epoch == target.ownership_epoch)
        {
            return Err(peer::denied("TEAM_WAIT_INVALIDATED"));
        }
        for artifact_id in &outcome.artifact_ids {
            if !state.team_artifacts.get(&key(&attempt.owner_id, &attempt.workspace_id,
                &attempt.team_id, artifact_id)).is_some_and(|artifact| artifact.attempt_id == target.id)
            {
                return Err(peer::denied("TEAM_SCOPE_DENIED"));
            }
        }
    }
    Ok(continuation.target_outcomes.clone())
}

fn selection<T: Serialize>(
    source_id: &str,
    source_kind: ContextSourceKind,
    value: &T,
) -> Result<ContextSelection, CollaborationError> {
    profile_id(source_id, "/selections/sourceId")?;
    let bytes = serde_json::to_vec(value)?.len() as u64;
    Ok(ContextSelection {
        source_id: source_id.to_owned(), source_kind,
        disposition: ContextDisposition::Selected, reason_code: None,
        original_bytes: bytes, selected_bytes: bytes, truncated: false,
    })
}

fn profile_id(value: &str, field: &str) -> Result<(), CollaborationError> {
    if value.is_empty() || value.len() > 128 || !value.as_bytes()[0].is_ascii_alphanumeric()
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(required_unsupported(field));
    }
    Ok(())
}

fn required_too_large(field: &str) -> CollaborationError {
    CollaborationError::Conflict(format!("TEAM_CONTEXT_REQUIRED_TOO_LARGE {field}"))
}

fn required_unsupported(field: &str) -> CollaborationError {
    CollaborationError::Conflict(format!("TEAM_CONTEXT_REQUIRED_UNSUPPORTED {field}"))
}
