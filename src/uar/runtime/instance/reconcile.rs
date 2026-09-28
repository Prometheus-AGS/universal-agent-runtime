//! Explicit operator settlement of an uncertain effect, without replaying its turn.

use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::uar::runtime::actor::messages::ActorOwner;

use super::{
    AgentInstanceController, AgentInstanceError, AgentInstanceView, InstanceCommandKind,
    InstanceCommandStatus, InstanceLifecycle, InstanceRecovery,
    controller::{append_event, require_name},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectDisposition {
    ConfirmedApplied,
    ConfirmedNotApplied,
}

impl EffectDisposition {
    fn as_str(self) -> &'static str {
        match self {
            Self::ConfirmedApplied => "confirmed_applied",
            Self::ConfirmedNotApplied => "confirmed_not_applied",
        }
    }

    fn event_kind(self) -> &'static str {
        match self {
            Self::ConfirmedApplied => "effect_confirmed_applied",
            Self::ConfirmedNotApplied => "effect_confirmed_not_applied",
        }
    }
}

impl AgentInstanceController {
    /// An authenticated operator attests to the disposition of one exact
    /// uncertain attempt. This closes the old turn as failed and admits new
    /// work only under a new epoch; it never repeats the old prompt or effect.
    pub async fn reconcile(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        target_command_id: &str,
        target_attempt_id: &str,
        disposition: EffectDisposition,
        receipt_id: &str,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        require_name(target_command_id)?;
        require_name(target_attempt_id)?;
        if receipt_id.is_empty()
            || receipt_id.len() > 128
            || !receipt_id.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
            })
        {
            return Err(AgentInstanceError::Invalid(
                "reconciliation receipt must be an opaque identifier",
            ));
        }
        let current = self.load(owner, workspace, id).await?;
        let matches_target =
            |command: &crate::uar::persistence::agent_instances::AgentInstanceCommand| {
                command.command_id == target_command_id
                    && command.kind == InstanceCommandKind::Turn
                    && command.attempt_id.as_deref() == Some(target_attempt_id)
            };
        if current.reconciliation_receipt.as_deref() == Some(receipt_id) {
            let same = current.inbox.iter().any(|command| {
                matches_target(command)
                    && command.status == InstanceCommandStatus::Failed
                    && command
                        .outcome
                        .as_ref()
                        .and_then(|outcome| outcome.get("effectDisposition"))
                        .and_then(serde_json::Value::as_str)
                        == Some(disposition.as_str())
            });
            return if same {
                Ok(AgentInstanceView::from(&current))
            } else {
                Err(AgentInstanceError::Conflict(
                    "reconciliation receipt belongs to another disposition",
                ))
            };
        }
        if current.recovery != InstanceRecovery::EffectUncertain
            || current.active_attempt.is_some()
            || !current.inbox.iter().any(|command| {
                matches_target(command) && command.status == InstanceCommandStatus::Uncertain
            })
        {
            return Err(AgentInstanceError::Conflict(
                "exact uncertain attempt is not awaiting reconciliation",
            ));
        }
        // A local actor with unconfirmed cleanup cannot be replaced safely.
        self.stop_actor(owner, workspace, id).await?;
        let record = self
            .update(owner, workspace, id, |next| {
                if next.recovery != InstanceRecovery::EffectUncertain
                    || next.active_attempt.is_some()
                {
                    return Err(AgentInstanceError::Conflict(
                        "instance recovery changed during reconciliation",
                    ));
                }
                let Some(index) = next.inbox.iter().position(|command| {
                    matches_target(command) && command.status == InstanceCommandStatus::Uncertain
                }) else {
                    return Err(AgentInstanceError::Conflict(
                        "exact uncertain attempt changed during reconciliation",
                    ));
                };
                let run_id = next.inbox[index].root_run_id.clone();
                next.inbox[index].status = InstanceCommandStatus::Failed;
                next.inbox[index].updated_at = Utc::now();
                next.inbox[index].outcome = Some(serde_json::json!({
                    "effectDisposition": disposition.as_str(),
                    "reconciliationReceipt": receipt_id,
                    "turnResult": "unknown",
                }));
                // No previously admitted work is replayed after an uncertain effect.
                for command in &mut next.inbox {
                    if command.command_id != target_command_id
                        && command.status == InstanceCommandStatus::Accepted
                    {
                        command.status = InstanceCommandStatus::Cancelled;
                        command.updated_at = Utc::now();
                    }
                }
                next.reconciliation_receipt = Some(receipt_id.to_owned());
                next.recovery = InstanceRecovery::Ready;
                next.lifecycle = InstanceLifecycle::Dormant;
                next.epoch = next
                    .epoch
                    .checked_add(1)
                    .ok_or(AgentInstanceError::Conflict("activation epoch exhausted"))?;
                next.last_error_code =
                    Some("turn_result_unknown_after_effect_reconciliation".into());
                append_event(
                    next,
                    disposition.event_kind(),
                    Some(target_command_id),
                    Some(target_attempt_id),
                    run_id.as_deref(),
                )?;
                Ok(true)
            })
            .await?;
        Ok(AgentInstanceView::from(&record))
    }
}
