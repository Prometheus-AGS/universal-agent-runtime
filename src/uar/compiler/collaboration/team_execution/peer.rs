//! Current directed communication authority, derived from an admitted attempt.
use super::{CollaborationError, fence, team};
use crate::uar::domain::{
    collaboration::CollaborationCatalogState, team_context::RosterMember, team_execution::*,
    team_wait::*,
};
use serde_json::Value;

pub(crate) fn denied(code: &str) -> CollaborationError {
    CollaborationError::Conflict(code.into())
}

pub(crate) fn team_document<'a>(
    state: &'a CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
) -> Result<&'a Value, CollaborationError> {
    let team = team(
        state,
        &attempt.owner_id,
        &attempt.workspace_id,
        &attempt.team_id,
    )?;
    state
        .definitions
        .get(&team.definition.storage_key())
        .map(|d| &d.document)
        .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))
}
pub(crate) fn require_edge(
    state: &CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
    recipient: &str,
    mode: &str,
) -> Result<(), CollaborationError> {
    let current = team(
        state,
        &attempt.owner_id,
        &attempt.workspace_id,
        &attempt.team_id,
    )?;
    let sender = current
        .members
        .iter()
        .find(|m| m.id == attempt.member_id && !matches!(m.status.as_str(), "revoked" | "stopped"))
        .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
    let target = current
        .members
        .iter()
        .find(|m| m.id == recipient && !matches!(m.status.as_str(), "revoked" | "stopped"))
        .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
    let doc = team_document(state, attempt)?;
    if doc["communication"].as_array().is_some_and(|edges| {
        edges.iter().any(|edge| {
            edge["fromRole"].as_str() == Some(sender.role.as_str())
                && edge["toRole"].as_str() == Some(target.role.as_str())
                && edge["modes"]
                    .as_array()
                    .is_some_and(|modes| modes.iter().any(|v| v.as_str() == Some(mode)))
        })
    }) {
        Ok(())
    } else {
        Err(denied("TEAM_EDGE_DENIED"))
    }
}
pub(crate) fn require_result_edge(
    state: &CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
    target: &str,
) -> Result<(), CollaborationError> {
    require_edge(state, attempt, target, "trigger-turn")?;
    let mut reverse = attempt.clone();
    reverse.member_id = target.into();
    require_edge(state, &reverse, &attempt.member_id, "queue-only")
}
pub(crate) fn authorized_roster(
    state: &CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
) -> Result<(Vec<RosterMember>, String, u64), CollaborationError> {
    let current = team(
        state,
        &attempt.owner_id,
        &attempt.workspace_id,
        &attempt.team_id,
    )?;
    let coordinator = team_document(state, attempt)?["coordinatorRole"]
        .as_str()
        .ok_or_else(|| denied("TEAM_CONTEXT_REQUIRED_UNSUPPORTED"))?;
    let mut roster = Vec::new();
    let mut coordinator_id = None;
    for member in current
        .members
        .iter()
        .filter(|m| !matches!(m.status.as_str(), "revoked" | "stopped"))
    {
        let visible = member.id == attempt.member_id
            || require_edge(state, attempt, &member.id, "queue-only").is_ok()
            || require_edge(state, attempt, &member.id, "trigger-turn").is_ok();
        if !visible {
            continue;
        }
        if member.role == coordinator {
            coordinator_id = Some(member.id.clone());
        }
        if member.role.len() > 256 {
            return Err(denied("TEAM_CONTEXT_REQUIRED_TOO_LARGE"));
        }
        roster.push(RosterMember {
            member_id: member.id.clone(),
            role: member.role.clone(),
            label: member.role.clone(),
            safe_capabilities: {
                let mut capabilities = vec!["team_roster".into()];
                let mut source = attempt.clone();
                source.member_id = member.id.clone();
                if current.members.iter().any(|target| {
                    target.id != member.id
                        && require_edge(state, &source, &target.id, "queue-only").is_ok()
                }) {
                    capabilities.push("team_send".into());
                }
                if member.role == coordinator
                    && team_document(state, attempt)?["taskAcceptance"]["mode"].as_str()
                        == Some("coordinator-within-binding")
                {
                    capabilities.extend(["team_delegate".into(), "team_wait".into()]);
                }
                capabilities
            },
        });
    }
    roster.sort_by(|a, b| a.member_id.cmp(&b.member_id));
    Ok((
        roster,
        coordinator_id.ok_or_else(|| denied("TEAM_CONTEXT_REQUIRED_UNSUPPORTED"))?,
        state.generation,
    ))
}
pub(crate) fn authorized_inbox(
    state: &CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
) -> Result<Vec<TeamPeerMessage>, CollaborationError> {
    let mut result = Vec::new();
    for message in state.team_peer_messages.values().filter(|m| {
        m.owner_id == attempt.owner_id
            && m.workspace_id == attempt.workspace_id
            && m.team_id == attempt.team_id
            && m.recipient_member_id == attempt.member_id
            && m.recipient_task_id
                .as_ref()
                .is_none_or(|id| id == &attempt.task_id)
    }) {
        let mut sender = attempt.clone();
        sender.member_id = message.sender_member_id.clone();
        if require_edge(state, &sender, &attempt.member_id, &message.mode).is_err() {
            continue;
        }
        // Consumed envelopes do not become another turn's implicit history.
        if state.team_message_deliveries.values().any(|d| {
            d.message_id == message.message_id
                && d.status == "consumed"
                && d.selected_attempt_id.as_deref() != Some(attempt.id.as_str())
        }) {
            continue;
        }
        result.push(message.clone());
    }
    result.sort_by(|a, b| {
        a.accepted_at
            .cmp(&b.accepted_at)
            .then_with(|| a.message_id.cmp(&b.message_id))
    });
    Ok(result)
}
pub(crate) fn authority(
    state: &CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
) -> Result<AttemptAuthority, CollaborationError> {
    let current = team(
        state,
        &attempt.owner_id,
        &attempt.workspace_id,
        &attempt.team_id,
    )?;
    let effective = state
        .execution_transfers
        .get(&attempt.id)
        .or(attempt.execution_fence.as_ref())
        .ok_or_else(|| denied("TEAM_EXECUTION_EPOCH_STALE"))?;
    Ok(AttemptAuthority {
        owner_id: attempt.owner_id.clone(),
        workspace_id: attempt.workspace_id.clone(),
        team_id: attempt.team_id.clone(),
        task_id: attempt.task_id.clone(),
        member_id: attempt.member_id.clone(),
        attempt_id: attempt.id.clone(),
        run_id: attempt.run_id.clone(),
        task_ownership_epoch: attempt.ownership_epoch,
        member_revision: attempt.member_revision,
        binding: TeamProfileRef {
            id: current.binding.id.clone(),
            revision: attempt.binding_revision,
        },
        execution_fence: effective.clone(),
    })
}
pub(crate) fn require_artifact_edge(
    state: &CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
    producer: &str,
) -> Result<(), CollaborationError> {
    if producer == attempt.member_id {
        return Ok(());
    }
    let mut source = attempt.clone();
    source.member_id = producer.into();
    if require_edge(state, &source, &attempt.member_id, "queue-only").is_ok()
        || require_edge(state, &source, &attempt.member_id, "trigger-turn").is_ok()
    {
        Ok(())
    } else {
        Err(denied("TEAM_EDGE_DENIED"))
    }
}
pub(crate) fn validate_payload(
    state: &CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
    payload: &AttributedPayload,
) -> Result<(), CollaborationError> {
    if payload.text.len() > 8192 || payload.artifact_ids.len() > 16 {
        return Err(denied("TEAM_CONTEXT_REQUIRED_TOO_LARGE"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for id in &payload.artifact_ids {
        if !seen.insert(id) {
            return Err(denied("TEAM_SCOPE_DENIED"));
        }
        let artifact = state
            .team_artifacts
            .values()
            .find(|a| {
                a.id == *id
                    && a.owner_id == attempt.owner_id
                    && a.workspace_id == attempt.workspace_id
                    && a.team_id == attempt.team_id
            })
            .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
        require_artifact_edge(state, attempt, &artifact.member_id)?;
    }
    Ok(())
}
pub(crate) fn command_key(attempt: &TeamExecutionAttempt, id: &str) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{id}",
        attempt.owner_id, attempt.workspace_id, attempt.team_id, attempt.id
    )
}
pub(crate) fn command_receipt(
    attempt: &TeamExecutionAttempt,
    command_id: &str,
    digest: &str,
    operation: &str,
) -> TeamPeerCommandReceipt {
    TeamPeerCommandReceipt {
        command_id: command_id.into(),
        request_digest: digest.into(),
        scope: TeamPeerScope {
            owner_id: attempt.owner_id.clone(),
            workspace_id: attempt.workspace_id.clone(),
            team_id: attempt.team_id.clone(),
        },
        operation: operation.into(),
        sender_member_id: attempt.member_id.clone(),
        sender_attempt_id: attempt.id.clone(),
        accepted_at: chrono::Utc::now(),
        message_id: None,
        task_id: None,
        attempt_id: None,
        wait_id: None,
    }
}
pub(crate) fn replay<'a>(
    state: &'a CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
    id: &str,
    digest: &str,
    operation: &str,
) -> Result<Option<&'a TeamPeerCommandReceipt>, CollaborationError> {
    let receipt = state.team_peer_commands.get(&command_key(attempt, id));
    if receipt.is_some_and(|r| r.request_digest != digest || r.operation != operation) {
        return Err(denied("TEAM_COMMAND_CONFLICT"));
    }
    Ok(receipt)
}
pub(crate) fn reserve(
    state: &CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
    reservation: &TeamReservation,
) -> Result<(), CollaborationError> {
    if reservation.tokens == 0 || reservation.elapsed_seconds == 0 {
        return Err(denied("TEAM_BUDGET_EXHAUSTED"));
    }
    let current = team(
        state,
        &attempt.owner_id,
        &attempt.workspace_id,
        &attempt.team_id,
    )?;
    let doc = team_document(state, attempt)?;
    let binding = state
        .bindings
        .get(&super::super::bindings::binding_key(
            &attempt.owner_id,
            &attempt.workspace_id,
            &current.binding.id,
        ))
        .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
    let mut total = reservation.clone();
    for prior in state.team_execution_attempts.values().filter(|a| {
        a.owner_id == attempt.owner_id
            && a.workspace_id == attempt.workspace_id
            && a.team_id == attempt.team_id
    }) {
        super::add(
            &mut total,
            prior.usage.as_ref().unwrap_or(&prior.reservation),
        )?;
    }
    for (name, value) in [
        ("maxTokens", total.tokens),
        ("maxCostMicrounits", total.cost_microunits),
        ("maxElapsedSeconds", total.elapsed_seconds),
    ] {
        let cap = doc["budget"][name].as_u64().unwrap_or(0).min(
            binding.document["effectiveBudget"][name]
                .as_u64()
                .unwrap_or(u64::MAX),
        );
        if value > cap {
            return Err(denied("TEAM_BUDGET_EXHAUSTED"));
        }
    }
    Ok(())
}
pub(crate) fn current_actor(
    state: &CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
) -> Result<(), CollaborationError> {
    fence(state, attempt)?;
    if super::attempt(state, attempt)?.status != "running" {
        return Err(denied("TEAM_SCOPE_DENIED"));
    }
    Ok(())
}
