//! Durable yield intents and uniquely linked fresh continuation admission.
use super::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, key, peer::*, team, team_key,
};
use crate::uar::domain::{
    collaboration::CollaborationCatalogState, team_execution::*, team_wait::*,
};
use chrono::Utc;
use serde_json::{Value, json};
use uuid::Uuid;

impl CollaborationCatalogService {
    pub async fn request_team_wait(
        &self,
        expected: &TeamExecutionAttempt,
        request: TeamWaitRequest,
    ) -> Result<Value, CollaborationError> {
        super::super::validation::validate_id(&request.command_id)?;
        if request.predicate != "all-terminal"
            || request.target_task_ids.is_empty()
            || request.target_task_ids.len() > 16
        {
            return Err(denied("TEAM_WAIT_CYCLE"));
        }
        let digest = super::super::validation::canonical_digest(&serde_json::to_value(&request)?)?;
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            current_actor(&current, expected)?;
            validate_payload(&current, expected, &request.continuation_input)?;
            let selected = team(
                &current,
                &expected.owner_id,
                &expected.workspace_id,
                &expected.team_id,
            )?;
            let member = selected
                .members
                .iter()
                .find(|m| m.id == expected.member_id)
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
            if team_document(&current, expected)?["coordinatorRole"].as_str()
                != Some(member.role.as_str())
            {
                return Err(denied("TEAM_SCOPE_DENIED"));
            }
            let mut seen = std::collections::BTreeSet::new();
            for target in &request.target_task_ids {
                if target == &expected.task_id
                    || !seen.insert(target)
                    || reaches(
                        &current,
                        expected,
                        target,
                        &expected.task_id,
                        &mut std::collections::BTreeSet::new(),
                    )?
                {
                    return Err(denied("TEAM_WAIT_CYCLE"));
                }
                let delegated = current
                    .team_peer_commands
                    .values()
                    .find(|r| {
                        r.scope.owner_id == expected.owner_id
                            && r.scope.workspace_id == expected.workspace_id
                            && r.scope.team_id == expected.team_id
                            && r.operation == "team_delegate"
                            && r.sender_member_id == expected.member_id
                            && r.sender_attempt_id == expected.id
                            && r.task_id.as_deref() == Some(target.as_str())
                    })
                    .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
                let task = selected
                    .tasks
                    .iter()
                    .find(|t| t.id == *target)
                    .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
                let recipient = task
                    .assignee_member_id
                    .as_deref()
                    .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
                require_result_edge(&current, expected, recipient)?;
                if delegated.scope.owner_id != expected.owner_id
                    || delegated.scope.workspace_id != expected.workspace_id
                    || delegated.scope.team_id != expected.team_id
                {
                    return Err(denied("TEAM_SCOPE_DENIED"));
                }
            }
            if let Some(receipt) = replay(
                &current,
                expected,
                &request.command_id,
                &digest,
                "team_wait",
            )? {
                return Ok(
                    json!({"receipt":receipt,"waitId":receipt.wait_id,"state":"yield_requested"}),
                );
            }
            if current
                .team_waits
                .values()
                .any(|w| w.authority.attempt_id == expected.id)
            {
                return Err(denied("TEAM_COMMAND_CONFLICT"));
            }
            if request.continuation_reservation.tokens == 0
                || request.continuation_reservation.elapsed_seconds == 0
            {
                return Err(denied("TEAM_BUDGET_EXHAUSTED"));
            }
            let mut next = current.clone();
            let wait_id = Uuid::new_v4().to_string();
            let wait = TeamWait {
                wait_id: wait_id.clone(),
                authority: authority(&current, expected)?,
                target_task_ids: request.target_task_ids.clone(),
                predicate: request.predicate.clone(),
                continuation_input: request.continuation_input.clone(),
                continuation_reservation: request.continuation_reservation.clone(),
                state: "yield_requested".into(),
                continuation_attempt_id: None,
                wake_outcomes: vec![],
                reason_code: None,
            };
            let mut receipt = command_receipt(expected, &request.command_id, &digest, "team_wait");
            receipt.wait_id = Some(wait_id.clone());
            receipt.task_id = Some(expected.task_id.clone());
            next.team_waits.insert(wait_id.clone(), wait);
            next.team_peer_commands
                .insert(command_key(expected, &request.command_id), receipt.clone());
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                return Ok(json!({"receipt":receipt,"waitId":wait_id,"state":"yield_requested"}));
            }
        }
        Err(denied("TEAM_REVISION_CONFLICT"))
    }
    pub async fn record_kernel_team_yield(
        &self,
        expected: &TeamExecutionAttempt,
        control: KernelTeamYield,
    ) -> Result<(), CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            current_actor(&current, expected)?;
            let wait = current
                .team_waits
                .get(&control.wait_id)
                .filter(|w| w.authority.attempt_id == expected.id && w.state == "yield_requested")
                .ok_or_else(|| denied("TEAM_WAIT_INVALIDATED"))?;
            if control.kind != "team-yield"
                || control.yielding_attempt_id != expected.id
                || !current.team_peer_commands.values().any(|r| {
                    r.wait_id.as_deref() == Some(wait.wait_id.as_str())
                        && r.command_id == control.durable_receipt_id
                })
            {
                return Err(denied("TEAM_SCOPE_DENIED"));
            }
            if let Some(prior) = current.team_yields.get(&expected.id) {
                return if prior == &control {
                    Ok(())
                } else {
                    Err(denied("TEAM_COMMAND_CONFLICT"))
                };
            }
            let mut next = current.clone();
            next.team_yields
                .insert(expected.id.clone(), control.clone());
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                self.team_execution_notify.notify_one();
                return Ok(());
            }
        }
        Err(denied("TEAM_REVISION_CONFLICT"))
    }
    /// Called only after the exact actor producer/children and effects are joined.
    pub async fn finalize_team_yield(
        &self,
        expected: &TeamExecutionAttempt,
        control: &KernelTeamYield,
        usage: Option<TeamReservation>,
    ) -> Result<(), CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, true)?;
            let mut next = current.clone();
            if !apply_confirmed_team_yield(&mut next, expected, control, usage.clone())? {
                return Ok(());
            }
            next.generation += 1;
            self.evaluate_team_waits(&mut next).await?;
            if self.cas(current.generation, &next).await? {
                self.team_execution_notify.notify_one();
                return Ok(());
            }
        }
        Err(denied("TEAM_REVISION_CONFLICT"))
    }
}

fn reaches(
    state: &CollaborationCatalogState,
    a: &TeamExecutionAttempt,
    from: &str,
    target: &str,
    seen: &mut std::collections::BTreeSet<String>,
) -> Result<bool, CollaborationError> {
    if from == target {
        return Ok(true);
    }
    if !seen.insert(from.into()) {
        return Ok(false);
    }
    let selected = team(state, &a.owner_id, &a.workspace_id, &a.team_id)?;
    let task = selected
        .tasks
        .iter()
        .find(|t| t.id == from)
        .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
    for next in task.depends_on.iter().chain(
        state
            .team_waits
            .values()
            .filter(|w| {
                w.authority.team_id == a.team_id
                    && w.authority.owner_id == a.owner_id
                    && w.authority.workspace_id == a.workspace_id
                    && w.authority.task_id == from
                    && matches!(w.state.as_str(), "yield_requested" | "waiting" | "blocked")
            })
            .flat_map(|w| w.target_task_ids.iter()),
    ) {
        if reaches(state, a, next, target, seen)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Private host-only transition; caller has verified exact kernel/effect completion.
pub(super) fn apply_confirmed_team_yield(
    state: &mut CollaborationCatalogState,
    expected: &TeamExecutionAttempt,
    control: &KernelTeamYield,
    usage: Option<TeamReservation>,
) -> Result<bool, CollaborationError> {
    super::ownership::require_attempt_fence(state, expected, true)?;
    if state.team_yields.get(&expected.id) != Some(control) {
        return Err(denied("TEAM_EFFECTS_UNCERTAIN"));
    }
    let wait = state
        .team_waits
        .get(&control.wait_id)
        .filter(|w| w.authority.attempt_id == expected.id)
        .ok_or_else(|| denied("TEAM_WAIT_INVALIDATED"))?;
    if matches!(wait.state.as_str(), "waiting" | "blocked" | "resumed") {
        return Ok(false);
    }
    if !matches!(wait.state.as_str(), "yield_requested" | "invalidated") {
        return Err(denied("TEAM_WAIT_INVALIDATED"));
    }
    let selected = team(
        state,
        &expected.owner_id,
        &expected.workspace_id,
        &expected.team_id,
    )?;
    let task = selected
        .tasks
        .iter()
        .find(|t| t.id == expected.task_id)
        .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
    let still_owned = task.ownership_epoch == expected.ownership_epoch
        && task.assignee_member_id.as_deref() == Some(expected.member_id.as_str());
    let may_wait = wait.state == "yield_requested" && still_owned;
    let attempt = state
        .team_execution_attempts
        .get_mut(&key(
            &expected.owner_id,
            &expected.workspace_id,
            &expected.team_id,
            &expected.id,
        ))
        .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
    if attempt.run_id != expected.run_id || attempt.execution_epoch != expected.execution_epoch {
        return Err(denied("TEAM_SCOPE_DENIED"));
    }
    if attempt.status == "yielded" {
        return Ok(false);
    }
    if !matches!(
        attempt.status.as_str(),
        "running" | "uncertain" | "cancellation_requested"
    ) {
        return Err(denied("TEAM_WAIT_INVALIDATED"));
    }
    let now = Utc::now();
    attempt.status = "yielded".into();
    attempt.execution_outcome = Some("yielded".into());
    attempt.effect_disposition = "confirmed".into();
    attempt.accounting_state = if usage.is_some() {
        "settled"
    } else {
        "reserved-unknown"
    }
    .into();
    attempt.usage = usage;
    attempt.usage_revision += 1;
    attempt.updated_at = now;
    let wait = state
        .team_waits
        .get_mut(&control.wait_id)
        .ok_or_else(|| denied("TEAM_WAIT_INVALIDATED"))?;
    wait.state = if may_wait { "waiting" } else { "invalidated" }.into();
    if !may_wait {
        wait.reason_code = Some("TEAM_WAIT_INVALIDATED".into());
    }
    let selected = state
        .team_instances
        .get_mut(&team_key(
            &expected.owner_id,
            &expected.workspace_id,
            &expected.team_id,
        ))
        .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
    if still_owned {
        let task = selected
            .tasks
            .iter_mut()
            .find(|t| t.id == expected.task_id)
            .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
        task.status = if may_wait { "waiting" } else { "cancelled" }.into();
        task.revision += 1;
        task.updated_at = now;
    }
    selected.revision += 1;
    selected.updated_at = now;
    Ok(true)
}
