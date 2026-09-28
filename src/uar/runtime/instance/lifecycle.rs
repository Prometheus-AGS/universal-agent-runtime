//! Revisioned lifecycle commands. Epoch handoff waits for a terminal turn.

use std::sync::Arc;

use chrono::Utc;

use crate::uar::{
    persistence::agent_instances::{AgentInstanceCommand, AgentInstanceRecord},
    runtime::actor::messages::ActorOwner,
};

use super::{
    AgentInstanceController, AgentInstanceError, AgentInstanceView, InstanceCommandKind,
    InstanceCommandStatus, InstanceLifecycle, InstanceRecovery,
    controller::{append_event, require_name, sha256_hex, trim_commands},
};

impl AgentInstanceController {
    pub async fn activate(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        require_name(command_id)?;
        let current = self.load(owner, workspace, id).await?;
        if let Some(existing) = current
            .inbox
            .iter()
            .find(|command| command.command_id == command_id)
        {
            return if existing.kind == InstanceCommandKind::Activate {
                Ok(AgentInstanceView::from(&current))
            } else {
                Err(AgentInstanceError::Conflict(
                    "command ID was already used for another request",
                ))
            };
        }
        let warm = self.has_actor(owner, workspace, id, current.epoch).await;
        let record = self
            .update(owner, workspace, id, |next| {
                if !insert_command(next, InstanceCommandKind::Activate, command_id)? {
                    return Ok(false);
                }
                if next.recovery != InstanceRecovery::Ready {
                    return Err(AgentInstanceError::Conflict(
                        "instance requires effect reconciliation",
                    ));
                }
                if !warm && next.restart_attempts >= next.limits.max_restart_attempts {
                    return Err(AgentInstanceError::Conflict(
                        "activation retry budget is exhausted",
                    ));
                }
                if next.active_attempt.is_some() && !warm {
                    next.recovery = InstanceRecovery::EffectUncertain;
                    next.lifecycle = InstanceLifecycle::Failed;
                    next.last_error_code = Some("orphaned_attempt".into());
                    set_command(next, command_id, InstanceCommandStatus::Failed);
                    append_event(
                        next,
                        "activation_reconciliation_required",
                        Some(command_id),
                        None,
                        None,
                    )?;
                    return Ok(true);
                }
                if matches!(
                    next.lifecycle,
                    InstanceLifecycle::Disabled | InstanceLifecycle::Draining
                ) {
                    return Err(AgentInstanceError::Conflict(
                        "instance cannot activate in its current lifecycle",
                    ));
                }
                if next.lifecycle != InstanceLifecycle::Active || !warm {
                    next.epoch = next
                        .epoch
                        .checked_add(1)
                        .ok_or(AgentInstanceError::Conflict("activation epoch exhausted"))?;
                    next.lifecycle = InstanceLifecycle::Active;
                    append_event(next, "activation_starting", Some(command_id), None, None)?;
                }
                Ok(true)
            })
            .await?;
        if record.lifecycle == InstanceLifecycle::Active
            && record.recovery == InstanceRecovery::Ready
        {
            if let Err(error) = self.actor(owner, &record).await {
                self.activation_failed(owner, workspace, id, command_id, record.epoch)
                    .await?;
                return Err(error);
            }
            let completed = self
                .update(owner, workspace, id, |next| {
                    if next.epoch != record.epoch || next.lifecycle != InstanceLifecycle::Active {
                        return Err(AgentInstanceError::Conflict(
                            "activation changed before startup completed",
                        ));
                    }
                    let Some(command) = next
                        .inbox
                        .iter()
                        .find(|command| command.command_id == command_id)
                    else {
                        return Err(AgentInstanceError::Conflict(
                            "activation receipt disappeared",
                        ));
                    };
                    if command.status != InstanceCommandStatus::Accepted {
                        return Ok(false);
                    }
                    set_command(next, command_id, InstanceCommandStatus::Completed);
                    next.last_error_code = None;
                    append_event(next, "activated", Some(command_id), None, None)?;
                    Ok(true)
                })
                .await?;
            self.kick(owner.clone(), workspace, id);
            return Ok(AgentInstanceView::from(&completed));
        }
        Ok(AgentInstanceView::from(&record))
    }

    pub async fn passivate(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        self.lifecycle_command(
            owner,
            workspace,
            id,
            command_id,
            InstanceCommandKind::Passivate,
        )
        .await
    }

    pub async fn drain(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        self.lifecycle_command(owner, workspace, id, command_id, InstanceCommandKind::Drain)
            .await
    }

    pub async fn disable(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        self.lifecycle_command(
            owner,
            workspace,
            id,
            command_id,
            InstanceCommandKind::Disable,
        )
        .await
    }

    pub async fn restart(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        self.lifecycle_command(
            owner,
            workspace,
            id,
            command_id,
            InstanceCommandKind::Restart,
        )
        .await
    }

    pub async fn cancel(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        self.lifecycle_command(
            owner,
            workspace,
            id,
            command_id,
            InstanceCommandKind::Cancel,
        )
        .await
    }

    async fn lifecycle_command(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
        kind: InstanceCommandKind,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        require_name(command_id)?;
        let current = self.load(owner, workspace, id).await?;
        if let Some(existing) = current
            .inbox
            .iter()
            .find(|command| command.command_id == command_id)
        {
            return if existing.kind == kind {
                Ok(AgentInstanceView::from(&current))
            } else {
                Err(AgentInstanceError::Conflict(
                    "command ID was already used for another request",
                ))
            };
        }
        let record = self
            .update(owner, workspace, id, |next| {
                if !insert_command(next, kind, command_id)? {
                    return Ok(false);
                }
                if next.recovery != InstanceRecovery::Ready {
                    return Err(AgentInstanceError::Conflict(
                        "instance requires effect reconciliation",
                    ));
                }
                if next.inbox.iter().any(|command| {
                    command.command_id != command_id
                        && command.kind == InstanceCommandKind::Cancel
                        && command.status == InstanceCommandStatus::Accepted
                }) {
                    return Err(AgentInstanceError::Conflict(
                        "cancellation is already pending",
                    ));
                }
                let pending_control = next.inbox.iter().position(|command| {
                    command.command_id != command_id
                        && command.kind != InstanceCommandKind::Turn
                        && command.kind != InstanceCommandKind::Cancel
                        && command.status == InstanceCommandStatus::Accepted
                });
                if kind != InstanceCommandKind::Cancel && pending_control.is_some() {
                    return Err(AgentInstanceError::Conflict(
                        "another lifecycle change is already pending",
                    ));
                }
                if let Some(index) = pending_control {
                    next.inbox[index].status = InstanceCommandStatus::Cancelled;
                    next.inbox[index].updated_at = Utc::now();
                    if next.lifecycle == InstanceLifecycle::Draining {
                        next.lifecycle = InstanceLifecycle::Active;
                    }
                    let superseded_id = next.inbox[index].command_id.clone();
                    append_event(
                        next,
                        "lifecycle_superseded",
                        Some(&superseded_id),
                        None,
                        None,
                    )?;
                }
                if kind == InstanceCommandKind::Restart
                    && next.restart_attempts >= next.limits.max_restart_attempts
                {
                    next.lifecycle = InstanceLifecycle::Failed;
                    next.last_error_code = Some("restart_budget_exhausted".into());
                    set_command(next, command_id, InstanceCommandStatus::Failed);
                    append_event(
                        next,
                        "restart_budget_exhausted",
                        Some(command_id),
                        None,
                        None,
                    )?;
                    return Ok(true);
                }
                if matches!(
                    kind,
                    InstanceCommandKind::Passivate | InstanceCommandKind::Disable
                ) {
                    for command in &mut next.inbox {
                        if command.kind == InstanceCommandKind::Turn
                            && command.status == InstanceCommandStatus::Accepted
                        {
                            command.status = InstanceCommandStatus::Cancelled;
                            command.updated_at = Utc::now();
                        }
                    }
                }
                if kind == InstanceCommandKind::Cancel && next.active_attempt.is_none() {
                    set_command(next, command_id, InstanceCommandStatus::Completed);
                    append_event(next, "cancel_no_active_turn", Some(command_id), None, None)?;
                    return Ok(true);
                }
                if next.active_attempt.is_some()
                    || (kind == InstanceCommandKind::Drain && has_accepted_turn(next))
                {
                    if kind != InstanceCommandKind::Cancel {
                        next.lifecycle = InstanceLifecycle::Draining;
                    }
                    append_event(next, "lifecycle_pending", Some(command_id), None, None)?;
                } else {
                    append_event(next, "lifecycle_requested", Some(command_id), None, None)?;
                }
                Ok(true)
            })
            .await?;
        if let Some(active) = &record.active_attempt
            && matches!(
                kind,
                InstanceCommandKind::Passivate
                    | InstanceCommandKind::Disable
                    | InstanceCommandKind::Restart
                    | InstanceCommandKind::Cancel
            )
        {
            if !self
                .manager
                .cancel_run_for_user(owner.user_id(), &active.root_run_id)
                .await
            {
                // The attempt can be durably claimed before RunManager registers
                // its cancellation token. Keep the same cancellation intent and
                // retry until that exact attempt is running or has settled.
                let controller = Arc::clone(self);
                let owner = owner.clone();
                let workspace = workspace.to_owned();
                let id = id.to_owned();
                let run_id = active.root_run_id.clone();
                tokio::spawn(async move {
                    let mut delay = 10_u64;
                    loop {
                        let Ok(current) = controller.load(&owner, &workspace, &id).await else {
                            return;
                        };
                        if current
                            .active_attempt
                            .as_ref()
                            .is_none_or(|attempt| attempt.root_run_id != run_id)
                        {
                            return;
                        }
                        if controller
                            .manager
                            .cancel_run_for_user(owner.user_id(), &run_id)
                            .await
                        {
                            return;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                        delay = (delay * 2).min(500);
                    }
                });
            }
        }
        self.kick(owner.clone(), workspace, id);
        Ok(AgentInstanceView::from(
            &self.load(owner, workspace, id).await?,
        ))
    }

    pub(crate) async fn finish_lifecycle(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
    ) -> Result<(), AgentInstanceError> {
        let before = self.load(owner, workspace, id).await?;
        if before.recovery != InstanceRecovery::Ready || before.active_attempt.is_some() {
            return Ok(());
        }
        let pending = before.inbox.iter().find(|command| {
            command.status == InstanceCommandStatus::Accepted
                && command.kind != InstanceCommandKind::Turn
        });
        let Some(pending) = pending else {
            return Ok(());
        };
        if pending.kind == InstanceCommandKind::Drain && has_accepted_turn(&before) {
            return Ok(());
        }
        let command_id = pending.command_id.clone();
        let kind = pending.kind;
        if matches!(
            kind,
            InstanceCommandKind::Passivate
                | InstanceCommandKind::Drain
                | InstanceCommandKind::Disable
                | InstanceCommandKind::Restart
        ) {
            self.stop_actor(owner, workspace, id).await?;
        }
        let record = self
            .update(owner, workspace, id, |next| {
                if next.recovery != InstanceRecovery::Ready || next.active_attempt.is_some() {
                    return Ok(false);
                }
                if kind == InstanceCommandKind::Drain && has_accepted_turn(next) {
                    return Ok(false);
                }
                let Some(command) = next
                    .inbox
                    .iter()
                    .find(|command| command.command_id == command_id)
                else {
                    return Ok(false);
                };
                if command.status != InstanceCommandStatus::Accepted {
                    return Ok(false);
                }
                match kind {
                    InstanceCommandKind::Passivate | InstanceCommandKind::Drain => {
                        next.lifecycle = InstanceLifecycle::Dormant;
                        next.epoch = next
                            .epoch
                            .checked_add(1)
                            .ok_or(AgentInstanceError::Conflict("activation epoch exhausted"))?;
                    }
                    InstanceCommandKind::Disable => {
                        next.lifecycle = InstanceLifecycle::Disabled;
                        next.epoch = next
                            .epoch
                            .checked_add(1)
                            .ok_or(AgentInstanceError::Conflict("activation epoch exhausted"))?;
                    }
                    InstanceCommandKind::Restart => {
                        next.restart_attempts += 1;
                        next.lifecycle = InstanceLifecycle::Active;
                        next.epoch = next
                            .epoch
                            .checked_add(1)
                            .ok_or(AgentInstanceError::Conflict("activation epoch exhausted"))?;
                    }
                    InstanceCommandKind::Cancel => {}
                    InstanceCommandKind::Activate | InstanceCommandKind::Turn => return Ok(false),
                }
                if kind == InstanceCommandKind::Restart {
                    append_event(next, "restart_starting", Some(&command_id), None, None)?;
                } else {
                    set_command(next, &command_id, InstanceCommandStatus::Completed);
                    append_event(next, "lifecycle_completed", Some(&command_id), None, None)?;
                }
                Ok(true)
            })
            .await?;
        if kind == InstanceCommandKind::Restart && record.lifecycle == InstanceLifecycle::Active {
            if let Err(error) = self.actor(owner, &record).await {
                self.update(owner, workspace, id, |next| {
                    if next.epoch != record.epoch
                        || !next.inbox.iter().any(|command| {
                            command.command_id == command_id
                                && command.status == InstanceCommandStatus::Accepted
                        })
                    {
                        return Ok(false);
                    }
                    next.lifecycle = if next.restart_attempts >= next.limits.max_restart_attempts {
                        InstanceLifecycle::Failed
                    } else {
                        InstanceLifecycle::Dormant
                    };
                    next.last_error_code = Some("restart_activation_failed".into());
                    set_command(next, &command_id, InstanceCommandStatus::Failed);
                    append_event(next, "restart_failed", Some(&command_id), None, None)?;
                    Ok(true)
                })
                .await?;
                return Err(error);
            }
            self.update(owner, workspace, id, |next| {
                if next.epoch != record.epoch || next.lifecycle != InstanceLifecycle::Active {
                    return Err(AgentInstanceError::Conflict(
                        "restart changed before startup completed",
                    ));
                }
                set_command(next, &command_id, InstanceCommandStatus::Completed);
                next.last_error_code = None;
                append_event(next, "restart_completed", Some(&command_id), None, None)?;
                Ok(true)
            })
            .await?;
        }
        Ok(())
    }

    async fn activation_failed(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
        expected_epoch: u64,
    ) -> Result<(), AgentInstanceError> {
        self.update(owner, workspace, id, |next| {
            if next.epoch != expected_epoch
                || !next.inbox.iter().any(|command| {
                    command.command_id == command_id
                        && command.status == InstanceCommandStatus::Accepted
                })
            {
                return Ok(false);
            }
            next.restart_attempts = next.restart_attempts.saturating_add(1);
            next.lifecycle = if next.restart_attempts >= next.limits.max_restart_attempts {
                InstanceLifecycle::Failed
            } else {
                InstanceLifecycle::Dormant
            };
            next.last_error_code = Some("activation_failed".into());
            set_command(next, command_id, InstanceCommandStatus::Failed);
            append_event(next, "activation_failed", Some(command_id), None, None)?;
            Ok(true)
        })
        .await?;
        Ok(())
    }
}

fn insert_command(
    record: &mut AgentInstanceRecord,
    kind: InstanceCommandKind,
    command_id: &str,
) -> Result<bool, AgentInstanceError> {
    let digest = sha256_hex(format!("lifecycle\0{kind:?}").as_bytes());
    if let Some(existing) = record
        .inbox
        .iter()
        .find(|command| command.command_id == command_id)
    {
        return if existing.kind == kind && existing.request_digest == digest {
            Ok(false)
        } else {
            Err(AgentInstanceError::Conflict(
                "command ID was already used for another request",
            ))
        };
    }
    trim_commands(record)?;
    let now = Utc::now();
    record.inbox.push(AgentInstanceCommand {
        command_id: command_id.to_owned(),
        request_digest: digest,
        kind,
        payload: None,
        payload_ref: None,
        status: InstanceCommandStatus::Accepted,
        attempt_id: None,
        root_run_id: None,
        accepted_at: now,
        updated_at: now,
        outcome: None,
    });
    Ok(true)
}

fn set_command(record: &mut AgentInstanceRecord, id: &str, status: InstanceCommandStatus) {
    if let Some(command) = record
        .inbox
        .iter_mut()
        .find(|command| command.command_id == id)
    {
        command.status = status;
        command.updated_at = Utc::now();
    }
}

fn has_accepted_turn(record: &AgentInstanceRecord) -> bool {
    record.inbox.iter().any(|command| {
        command.kind == InstanceCommandKind::Turn
            && command.status == InstanceCommandStatus::Accepted
    })
}
