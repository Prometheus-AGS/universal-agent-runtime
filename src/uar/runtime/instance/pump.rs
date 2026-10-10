//! One bounded command pump per logical instance; no second model loop.

use std::{collections::HashSet, pin::Pin, sync::Arc};

use chrono::Utc;
use uuid::Uuid;

use crate::uar::{
    persistence::{
        agent_instances::{AgentInstanceAttempt, AgentInstanceRecord},
        tool_admission::ToolAdmissionEvidenceState,
    },
    runtime::{
        actor::messages::{ActorOwner, ActorRunError},
        thread::AgentThreadResult,
    },
};

use super::{
    AgentInstanceController, AgentInstanceError, InstanceActivationProfile, InstanceCommandKind,
    InstanceCommandStatus, InstanceLifecycle, InstanceRecovery,
    controller::{InstanceKey, append_event},
};

impl AgentInstanceController {
    pub(crate) fn kick(self: &Arc<Self>, owner: ActorOwner, workspace: &str, id: &str) {
        let key = InstanceKey::new(&owner, workspace, id);
        let controller = Arc::clone(self);
        let workspace = workspace.to_owned();
        let id = id.to_owned();
        let job: Pin<Box<dyn Future<Output = ()> + Send>> = Box::pin(async move {
            let mut pumps = controller.pumps.lock().await;
            if !pumps.insert(key.clone()) {
                return;
            }
            drop(pumps);
            loop {
                if let Err(error) = controller.run_pump(&owner, &workspace, &id).await {
                    tracing::error!(instance_id = %id, error = %error, "Logical instance pump stopped");
                    controller.pumps.lock().await.remove(&key);
                    return;
                }
                let mut pumps = controller.pumps.lock().await;
                pumps.remove(&key);
                // A submission may have committed after the last empty read.
                // Reserve this pump again only if there is actionable work.
                let actionable =
                    controller
                        .load(&owner, &workspace, &id)
                        .await
                        .is_ok_and(|record| {
                            record.recovery == InstanceRecovery::Ready
                                && record.active_attempt.is_none()
                                && matches!(
                                    record.lifecycle,
                                    InstanceLifecycle::Active | InstanceLifecycle::Draining
                                )
                                && record.inbox.iter().any(|command| {
                                    command.status == InstanceCommandStatus::Accepted
                                })
                        });
                if !actionable || !pumps.insert(key.clone()) {
                    return;
                }
            }
        });
        tokio::spawn(job);
    }

    async fn run_pump(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
    ) -> Result<(), AgentInstanceError> {
        loop {
            self.finish_lifecycle(owner, workspace, id).await?;
            let record = self.load(owner, workspace, id).await?;
            if record.recovery != InstanceRecovery::Ready || record.active_attempt.is_some() {
                return Ok(());
            }
            if !matches!(
                record.lifecycle,
                InstanceLifecycle::Active | InstanceLifecycle::Draining
            ) {
                return Ok(());
            }
            let pending_lifecycle = record.inbox.iter().any(|command| {
                command.kind != InstanceCommandKind::Turn
                    && command.status == InstanceCommandStatus::Accepted
                    && command.kind != InstanceCommandKind::Drain
            });
            if pending_lifecycle {
                return Ok(());
            }
            let Some(command) = record.inbox.iter().find(|command| {
                command.kind == InstanceCommandKind::Turn
                    && command.status == InstanceCommandStatus::Accepted
            }) else {
                self.maybe_idle_passivate(owner, workspace, id, &record)
                    .await;
                return Ok(());
            };
            let command_id = command.command_id.clone();
            let prompt = command
                .payload
                .as_ref()
                .and_then(|payload| payload.get("prompt"))
                .and_then(serde_json::Value::as_str)
                .ok_or(AgentInstanceError::Conflict(
                    "accepted turn payload is unavailable",
                ))?
                .to_owned();
            let run_id = Uuid::new_v4().to_string();
            let attempt_id = Uuid::new_v4().to_string();
            let claimed = self
                .update(owner, workspace, id, |next| {
                    if next.active_attempt.is_some()
                        || next.recovery != InstanceRecovery::Ready
                        || !matches!(
                            next.lifecycle,
                            InstanceLifecycle::Active | InstanceLifecycle::Draining
                        )
                    {
                        return Ok(false);
                    }
                    let Some(command) = next
                        .inbox
                        .iter_mut()
                        .find(|command| command.command_id == command_id)
                    else {
                        return Ok(false);
                    };
                    if command.status != InstanceCommandStatus::Accepted {
                        return Ok(false);
                    }
                    command.status = InstanceCommandStatus::Running;
                    command.attempt_id = Some(attempt_id.clone());
                    command.root_run_id = Some(run_id.clone());
                    command.updated_at = Utc::now();
                    next.active_attempt = Some(AgentInstanceAttempt {
                        command_id: command_id.clone(),
                        attempt_id: attempt_id.clone(),
                        root_run_id: run_id.clone(),
                        epoch: next.epoch,
                        started_at: Utc::now(),
                    });
                    append_event(
                        next,
                        "turn_started",
                        Some(&command_id),
                        Some(&attempt_id),
                        Some(&run_id),
                    )?;
                    Ok(true)
                })
                .await?;
            if claimed
                .active_attempt
                .as_ref()
                .is_none_or(|active| active.root_run_id != run_id)
            {
                continue;
            }
            let before_dispatch = self.load(owner, workspace, id).await?;
            if before_dispatch.inbox.iter().any(|receipt| {
                receipt.status == InstanceCommandStatus::Accepted
                    && matches!(
                        receipt.kind,
                        InstanceCommandKind::Cancel
                            | InstanceCommandKind::Passivate
                            | InstanceCommandKind::Disable
                            | InstanceCommandKind::Restart
                    )
            }) {
                self.settle_turn(
                    owner,
                    workspace,
                    id,
                    &command_id,
                    &attempt_id,
                    &run_id,
                    TurnSettlement::Cancelled,
                    None,
                )
                .await?;
                continue;
            }
            if self.bound(&before_dispatch).await.is_err() {
                self.settle_turn(
                    owner,
                    workspace,
                    id,
                    &command_id,
                    &attempt_id,
                    &run_id,
                    TurnSettlement::BindingUnavailable,
                    None,
                )
                .await?;
                continue;
            }
            let mut failure_diagnostic = None;
            let outcome = match self.actor(owner, &claimed).await {
                Ok(actor) => match actor.submit_reserved_prompt(run_id.clone(), prompt) {
                    Ok(turn) => match turn.completion.await {
                        Ok(Ok(thread)) => match thread.thread.result {
                            Some(AgentThreadResult::Completed { .. }) => TurnSettlement::Completed,
                            Some(AgentThreadResult::Cancelled) => TurnSettlement::Cancelled,
                            Some(AgentThreadResult::Failed { code, .. })
                                if code == "kernel_not_started" =>
                            {
                                TurnSettlement::Failed
                            }
                            Some(AgentThreadResult::Failed { code, message }) => {
                                failure_diagnostic = Some((
                                    "run_kernel",
                                    kernel_failure_code(&code, &message),
                                ));
                                TurnSettlement::Uncertain
                            }
                            Some(AgentThreadResult::Yielded { .. }) | None => {
                                TurnSettlement::Uncertain
                            }
                        },
                        Ok(Err(ActorRunError::Stopped)) => TurnSettlement::Cancelled,
                        Ok(Err(ActorRunError::Host(error))) => {
                            failure_diagnostic = Some(("actor_host", actor_host_failure_code(&error)));
                            TurnSettlement::Uncertain
                        }
                        Err(_) => TurnSettlement::Uncertain,
                    },
                    Err(_) => TurnSettlement::Failed,
                },
                Err(_) => TurnSettlement::Failed,
            };
            let outcome = self
                .settle_turn(
                    owner,
                    workspace,
                    id,
                    &command_id,
                    &attempt_id,
                    &run_id,
                    outcome,
                    failure_diagnostic,
                )
                .await?;
            if outcome == TurnSettlement::Uncertain {
                return Ok(());
            }
        }
    }

    async fn settle_turn(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
        attempt_id: &str,
        run_id: &str,
        outcome: TurnSettlement,
        failure_diagnostic: Option<(&'static str, &'static str)>,
    ) -> Result<TurnSettlement, AgentInstanceError> {
        let evidence = if outcome == TurnSettlement::Uncertain {
            None
        } else {
            Some(self.has_unsettled_effect_claim(owner, run_id).await)
        };
        let evidence_unavailable = matches!(evidence.as_ref(), Some(Err(_)));
        let outcome = if matches!(evidence.as_ref(), Some(Ok(true) | Err(_))) {
            TurnSettlement::Uncertain
        } else {
            outcome
        };
        self.update(owner, workspace, id, |next| {
            let Some(active) = &next.active_attempt else {
                return Ok(false);
            };
            if active.command_id != command_id
                || active.attempt_id != attempt_id
                || active.root_run_id != run_id
                || active.epoch != next.epoch
            {
                return Ok(false);
            }
            let status = match outcome {
                TurnSettlement::Completed => InstanceCommandStatus::Completed,
                TurnSettlement::Cancelled => InstanceCommandStatus::Cancelled,
                TurnSettlement::Failed | TurnSettlement::BindingUnavailable => {
                    InstanceCommandStatus::Failed
                }
                TurnSettlement::Uncertain => InstanceCommandStatus::Uncertain,
            };
            let command = next
                .inbox
                .iter_mut()
                .find(|command| command.command_id == command_id)
                .ok_or(AgentInstanceError::Conflict(
                    "active turn receipt disappeared",
                ))?;
            command.status = status;
            command.updated_at = Utc::now();
            if let Some((stage, code)) = failure_diagnostic {
                // Persist only trusted static stages, never the host's raw
                // error chain, which may contain paths or credentials.
                command.outcome = Some(serde_json::json!({
                    "sourceStage": stage,
                    "errorCode": code,
                }));
            }
            next.active_attempt = None;
            if outcome == TurnSettlement::Uncertain {
                next.recovery = InstanceRecovery::EffectUncertain;
                next.last_error_code = Some(
                    if evidence_unavailable {
                        "tool_admission_evidence_unavailable"
                    } else {
                        "turn_outcome_uncertain"
                    }
                    .into(),
                );
            } else if outcome == TurnSettlement::BindingUnavailable {
                next.last_error_code = Some("required_binding_unavailable".into());
            }
            append_event(
                next,
                match outcome {
                    TurnSettlement::Completed => "turn_completed",
                    TurnSettlement::Cancelled => "turn_cancelled",
                    TurnSettlement::Failed => "turn_failed",
                    TurnSettlement::BindingUnavailable => "turn_binding_unavailable",
                    TurnSettlement::Uncertain => "turn_outcome_uncertain",
                },
                Some(command_id),
                Some(attempt_id),
                Some(run_id),
            )?;
            Ok(true)
        })
        .await?;
        Ok(outcome)
    }

    async fn has_unsettled_effect_claim(
        &self,
        owner: &ActorOwner,
        run_id: &str,
    ) -> Result<bool, AgentInstanceError> {
        let evidence = self
            .store
            .list_tool_admission_evidence(owner.user_id())
            .await
            .map_err(super::controller::store_error)?;
        let mut claimed = HashSet::new();
        let mut confirmed = HashSet::new();
        for record in evidence
            .into_iter()
            .filter(|record| record.root_run_id == run_id)
        {
            match record.state {
                ToolAdmissionEvidenceState::ClaimIntent
                | ToolAdmissionEvidenceState::OutcomeUnknown => {
                    claimed.insert(record.invocation_id);
                }
                ToolAdmissionEvidenceState::Succeeded | ToolAdmissionEvidenceState::Failed => {
                    confirmed.insert(record.invocation_id);
                }
                _ => {}
            }
        }
        Ok(claimed.difference(&confirmed).next().is_some())
    }

    async fn maybe_idle_passivate(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        record: &AgentInstanceRecord,
    ) {
        if record.lifecycle != InstanceLifecycle::Active {
            return;
        }
        match record.profile {
            InstanceActivationProfile::Resident => {}
            InstanceActivationProfile::Request => {
                let controller = Arc::clone(self);
                let owner = owner.clone();
                let workspace = workspace.to_owned();
                let id = id.to_owned();
                let job: Pin<Box<dyn Future<Output = ()> + Send>> = Box::pin(async move {
                    let _ = controller
                        .passivate(&owner, &workspace, &id, &Uuid::new_v4().to_string())
                        .await;
                });
                tokio::spawn(job);
            }
            InstanceActivationProfile::OnDemand => {
                // An idle timer must not keep the controller (and its embedded
                // SurrealKV handle) alive after the server has shut down.
                let controller = Arc::downgrade(self);
                let owner = owner.clone();
                let workspace = workspace.to_owned();
                let id = id.to_owned();
                let epoch = record.epoch;
                let revision = record.revision;
                let delay = record.limits.idle_timeout_secs;
                let job: Pin<Box<dyn Future<Output = ()> + Send>> = Box::pin(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                    let Some(controller) = controller.upgrade() else {
                        return;
                    };
                    if let Ok(current) = controller.load(&owner, &workspace, &id).await
                        && current.epoch == epoch
                        && current.revision == revision
                        && current.lifecycle == InstanceLifecycle::Active
                        && current.active_attempt.is_none()
                        && !current
                            .inbox
                            .iter()
                            .any(|command| command.status == InstanceCommandStatus::Accepted)
                    {
                        let _ = controller
                            .passivate(&owner, &workspace, &id, &Uuid::new_v4().to_string())
                            .await;
                    }
                });
                tokio::spawn(job);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TurnSettlement {
    Completed,
    Cancelled,
    Failed,
    BindingUnavailable,
    Uncertain,
}

fn actor_host_failure_code(error: &anyhow::Error) -> &'static str {
    const CODES: &[&str] = &[
        "actor_root_request_scope_mismatch",
        "actor_root_artifact_scope_failed",
        "actor_root_catalog_binding_failed",
        "actor_root_instance_epoch_failed",
        "actor_root_previous_recovery_failed",
        "actor_root_identity_failed",
        "actor_root_registration_failed",
        "actor_root_terminal_epoch_failed",
        "actor_root_terminal_persistence_failed",
    ];
    error
        .chain()
        .find_map(|cause| {
            let message = cause.to_string();
            CODES.iter().copied().find(|code| message == *code)
        })
        .unwrap_or("actor_host_failed")
}

fn kernel_failure_code(code: &str, message: &str) -> &'static str {
    if code == "representation_admission_denied" {
        return match message {
            "REPRESENTATION_CEDAR_REQUIRED" => "representation_cedar_required",
            "REPRESENTATION_HISTORY_SCOPE_UNSUPPORTED" => "representation_history_scope_unsupported",
            "REPRESENTATION_INSTANCE_SCOPE_DENIED" => "representation_instance_scope_denied",
            _ => "representation_admission_denied",
        };
    }
    const CODES: &[&str] = &[
        "actor_root_mismatch", "run_owner_mismatch", "mcp_catalog_unavailable",
        "mcp_capture_mismatch", "child_bindings_unavailable", "sandbox_binding_unavailable",
        "mcp_server_not_run_scoped", "mcp_preflight_failed", "approval_channel_unavailable",
        "world_state_load_failed", "tool_admission_context_failed", "provider_model_unavailable",
        "thread_attachment_failed", "turn_assembly_rejected", "root_resource_binding_conflict",
        "kernel_completion_closed", "kernel_panicked", "thread_cleanup_unconfirmed",
        "session_persistence_unconfirmed",
    ];
    CODES
        .iter()
        .copied()
        .find(|known| code == *known)
        .unwrap_or("actor_kernel_failed")
}
