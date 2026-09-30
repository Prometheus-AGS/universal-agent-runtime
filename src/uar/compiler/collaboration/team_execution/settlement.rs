use super::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, attempt, fence, key, team,
    team_key,
};
use crate::uar::domain::team_execution::{
    TeamControlRequest, TeamExecutionAttempt, TeamExecutionCommandReceipt, TeamExecutionDiagnostic,
    TeamReservation,
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
            self.require_execution_owner(&current, false)?;
            fence(&current, expected)?;
            let mut item = attempt(&current, expected)?.clone();
            if item.status != "queued" {
                return Err(CollaborationError::Conflict(
                    "team dispatch was already claimed".to_owned(),
                ));
            }
            let selected = team(&current, &item.owner_id, &item.workspace_id, &item.team_id)?;
            let task = selected
                .tasks
                .iter()
                .find(|t| t.id == item.task_id)
                .ok_or_else(|| CollaborationError::NotFound(item.task_id.clone()))?;
            if !task.depends_on.iter().all(|id| {
                selected
                    .tasks
                    .iter()
                    .any(|t| &t.id == id && t.status == "succeeded")
            }) {
                return Err(CollaborationError::Conflict("TEAM_PENDING_LIMIT".into()));
            }
            let definition = current
                .definitions
                .get(&selected.definition.storage_key())
                .ok_or_else(|| CollaborationError::NotFound(selected.definition.id.clone()))?;
            let binding = current
                .bindings
                .get(&super::super::bindings::binding_key(
                    &item.owner_id,
                    &item.workspace_id,
                    &selected.binding.id,
                ))
                .ok_or_else(|| CollaborationError::NotFound(selected.binding.id.clone()))?;
            let limit = definition.document["limits"]["concurrentTurns"]
                .as_u64()
                .unwrap_or(0)
                .min(
                    binding.document["effectiveLimits"]["concurrentTurns"]
                        .as_u64()
                        .unwrap_or(u64::MAX),
                );
            let active = |a: &&TeamExecutionAttempt| {
                matches!(a.status.as_str(), "running" | "cancellation_requested")
                    || (a.status == "uncertain" && a.effect_disposition != "confirmed")
            };
            let global = std::env::var("UAR_TEAM_EXECUTION_MAX_ACTIVE")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|v| *v > 0)
                .unwrap_or(4);
            if current
                .team_execution_attempts
                .values()
                .filter(active)
                .count() as u64
                >= global
                || current
                    .team_execution_attempts
                    .values()
                    .filter(|a| {
                        a.owner_id == item.owner_id
                            && a.workspace_id == item.workspace_id
                            && a.team_id == item.team_id
                    })
                    .filter(active)
                    .count() as u64
                    >= limit
            {
                return Err(CollaborationError::Conflict("TEAM_PENDING_LIMIT".into()));
            }
            item.status = "running".to_owned();
            item.updated_at = Utc::now();
            let mut next = current.clone();
            next.generation += 1;
            next.team_execution_attempts.insert(
                key(&item.owner_id, &item.workspace_id, &item.team_id, &item.id),
                item.clone(),
            );
            if item.status == "running" {
                if let Some(selected) = next.team_instances.get_mut(&team_key(
                    &item.owner_id,
                    &item.workspace_id,
                    &item.team_id,
                )) {
                    if let Some(task) = selected.tasks.iter_mut().find(|t| t.id == item.task_id) {
                        task.status = "running".into();
                        task.revision += 1;
                    }
                    selected.revision += 1;
                }
            }
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
            self.require_execution_owner(&current, true)?;
            super::ownership::require_attempt_fence(&current, expected, true)?;
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
            if matches!(
                item.execution_outcome.as_deref(),
                Some("succeeded" | "failed" | "cancelled")
            ) && (item.execution_outcome.as_deref() != Some(status) || item.output != output)
            {
                return Err(CollaborationError::Conflict(
                    "settled execution outcome is immutable even when accounting is unknown".into(),
                ));
            }
            let now = Utc::now();
            item.status = status.to_owned();
            item.effect_disposition = if status == "uncertain" {
                "uncertain"
            } else {
                "confirmed"
            }
            .into();
            item.accounting_state = if usage.is_some() {
                "settled"
            } else {
                "reserved-unknown"
            }
            .into();
            item.execution_outcome = Some(status.to_owned());
            item.usage = usage.clone();
            item.usage_revision += 1;
            item.output = output.clone();
            item.state_reason = reason.clone().or_else(|| {
                usage.is_none().then(|| {
                    "Provider usage or effects are uncertain; reservation retained".to_owned()
                })
            });
            item.diagnostic = reason.as_deref().and_then(provider_diagnostic);
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
            if matches!(status, "succeeded" | "failed" | "cancelled") {
                for wait in next
                    .team_waits
                    .values_mut()
                    .filter(|w| w.authority.attempt_id == item.id && w.state == "yield_requested")
                {
                    wait.state = "invalidated".into();
                    wait.reason_code = Some("TEAM_WAIT_INVALIDATED".into());
                }
            }
            next.team_execution_attempts.insert(
                key(&item.owner_id, &item.workspace_id, &item.team_id, &item.id),
                item.clone(),
            );
            self.evaluate_team_waits(&mut next).await?;
            if self.cas(current.generation, &next).await? {
                self.team_execution_notify.notify_one();
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
            self.require_execution_owner(&current, false)?;
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
                "queued" => { item.status = "cancelled".to_owned(); item.execution_outcome = Some("cancelled".to_owned()); item.effect_disposition = "confirmed".into(); item.accounting_state = "settled".into(); item.usage = Some(TeamReservation::default()); item.usage_revision += 1; }
                "running" | "cancellation_requested" => item.status = "cancellation_requested".to_owned(),
                "yielded" if current.team_waits.values().any(|w|w.authority.attempt_id==item.id&&matches!(w.state.as_str(),"waiting"|"blocked")) => {},
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
            if matches!(item.status.as_str(), "cancelled" | "yielded")
                && let Some(task) = updated.tasks.iter_mut().find(|t| t.id == item.task_id)
            {
                task.status = "cancelled".to_owned();
                task.state_reason = item.state_reason.clone();
                task.revision += 1;
                task.updated_at = now;
            }
            for wait in next.team_waits.values_mut().filter(|w| {
                w.authority.attempt_id == item.id
                    && matches!(w.state.as_str(), "yield_requested" | "waiting" | "blocked")
            }) {
                wait.state = "invalidated".into();
                wait.reason_code = Some("TEAM_WAIT_INVALIDATED".into());
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
            self.require_execution_owner(&current, false)?;
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
                        item.effect_disposition = "uncertain".into();
                        item.accounting_state = "reserved-unknown".into();
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

fn provider_diagnostic(reason: &str) -> Option<TeamExecutionDiagnostic> {
    let (code, reference) = reason.split_once("; diagnostic reference ")?;
    if !matches!(
        code,
        "TEAM_PROVIDER_REQUEST_REJECTED" | "TEAM_PROVIDER_STREAM_FAILED"
    ) {
        return None;
    }
    let reference = uuid::Uuid::parse_str(reference).ok()?.to_string();
    Some(TeamExecutionDiagnostic {
        code: code.into(),
        field: None,
        retryable: false,
        action: "change-settings".into(),
        protected_diagnostic_ref: Some(reference),
    })
}
