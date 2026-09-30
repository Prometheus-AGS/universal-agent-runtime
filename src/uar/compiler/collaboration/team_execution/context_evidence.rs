//! Durable selection is distinct from an observed model-input handoff.
use super::{CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, peer::*, team};
use crate::uar::domain::{
    team_context::{ContextDisposition, ContextSourceKind, TeamContextReceipt},
    team_execution::TeamExecutionAttempt,
    team_wait::MessageDelivery,
};
use serde_json::{Value, json};
impl CollaborationCatalogService {
    pub async fn record_team_context_selection(
        &self,
        a: &TeamExecutionAttempt,
        receipt: &TeamContextReceipt,
    ) -> Result<(), CollaborationError> {
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            current_actor(&current, a)?;
            if receipt.authority.attempt_id != a.id
                || receipt.root_id != a.root_id
                || receipt.approval_scope_id != a.approval_scope_id
            {
                return Err(denied("TEAM_SCOPE_DENIED"));
            }
            if let Some(prior) = current.team_context_receipts.get(&a.id) {
                return if serde_json::to_value(prior)? == serde_json::to_value(receipt)? {
                    Ok(())
                } else {
                    Err(denied("TEAM_REVISION_CONFLICT"))
                };
            }
            let authorized = authorized_inbox(&current, a)?;
            let mut next = current.clone();
            for selection in receipt.selections.iter().filter(|s| {
                s.source_kind == ContextSourceKind::Message
                    && s.disposition == ContextDisposition::Selected
            }) {
                let message = authorized
                    .iter()
                    .find(|m| m.message_id == selection.source_id)
                    .ok_or_else(|| denied("TEAM_EDGE_DENIED"))?;
                next.team_message_deliveries.insert(
                    format!("{}:{}", message.message_id, a.id),
                    MessageDelivery {
                        message_id: message.message_id.clone(),
                        recipient_member_id: a.member_id.clone(),
                        recipient_task_id: Some(a.task_id.clone()),
                        status: "delivered".into(),
                        selected_attempt_id: Some(a.id.clone()),
                        selected_at: Some(chrono::Utc::now()),
                        consumed_at: None,
                        rejection_code: None,
                    },
                );
            }
            next.team_context_receipts
                .insert(a.id.clone(), receipt.clone());
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                return Ok(());
            }
        }
        Err(denied("TEAM_REVISION_CONFLICT"))
    }
    pub async fn validate_team_model_handoff(
        &self,
        a: &TeamExecutionAttempt,
    ) -> Result<(), CollaborationError> {
        {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            current_actor(&current, a)?;
            let authorized = authorized_inbox(&current, a)?;
            let receipt = current
                .team_context_receipts
                .get(&a.id)
                .ok_or_else(|| denied("TEAM_CONTEXT_REQUIRED_UNSUPPORTED"))?;
            for selection in receipt.selections.iter().filter(|s| {
                s.source_kind == ContextSourceKind::Message
                    && s.disposition == ContextDisposition::Selected
            }) {
                if !authorized
                    .iter()
                    .any(|m| m.message_id == selection.source_id)
                {
                    return Err(denied("TEAM_EDGE_DENIED"));
                }
            }
            for selection in receipt.selections.iter().filter(|s| {
                s.source_kind == ContextSourceKind::Artifact
                    && s.disposition == ContextDisposition::Selected
            }) {
                let artifact = current
                    .team_artifacts
                    .values()
                    .find(|v| {
                        v.id == selection.source_id
                            && v.owner_id == a.owner_id
                            && v.workspace_id == a.workspace_id
                            && v.team_id == a.team_id
                    })
                    .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
                require_artifact_edge(&current, a, &artifact.member_id)?;
            }
            let (visible, _, _) = authorized_roster(&current, a)?;
            if receipt
                .roster
                .iter()
                .any(|m| !visible.iter().any(|v| v.member_id == m.member_id))
            {
                return Err(denied("TEAM_EDGE_DENIED"));
            }
            for outcome in &receipt.target_outcomes {
                require_result_edge(&current, a, &outcome.member_id)?;
            }
            Ok(())
        }
    }
    pub async fn record_team_model_handoff(
        &self,
        a: &TeamExecutionAttempt,
    ) -> Result<(), CollaborationError> {
        self.validate_team_model_handoff(a).await?;
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            current_actor(&current, a)?;
            let mut next = current.clone();
            let mut changed = false;
            for delivery in next.team_message_deliveries.values_mut().filter(|d| {
                d.selected_attempt_id.as_deref() == Some(a.id.as_str()) && d.status == "delivered"
            }) {
                delivery.status = "consumed".into();
                delivery.consumed_at = Some(chrono::Utc::now());
                changed = true;
            }
            if !changed {
                return Ok(());
            }
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                return Ok(());
            }
        }
        Err(denied("TEAM_REVISION_CONFLICT"))
    }
    pub async fn read_team_context(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
        id: &str,
    ) -> Result<TeamContextReceipt, CollaborationError> {
        let state = self.load_state().await?;
        team(&state, owner, workspace, team_id)?;
        let a = self.get_team_attempt(owner, workspace, team_id, id).await?;
        let receipt = state
            .team_context_receipts
            .get(id)
            .cloned()
            .ok_or_else(|| denied("TEAM_CONTEXT_REQUIRED_UNSUPPORTED"))?;
        for selection in receipt.selections.iter().filter(|s| {
            s.source_kind == ContextSourceKind::Artifact
                && s.disposition == ContextDisposition::Selected
        }) {
            let artifact = state
                .team_artifacts
                .values()
                .find(|v| {
                    v.id == selection.source_id
                        && v.owner_id == a.owner_id
                        && v.workspace_id == a.workspace_id
                        && v.team_id == a.team_id
                })
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
            require_artifact_edge(&state, &a, &artifact.member_id)?;
        }
        let (visible, _, _) = authorized_roster(&state, &a)?;
        if receipt
            .roster
            .iter()
            .any(|m| !visible.iter().any(|v| v.member_id == m.member_id))
        {
            return Err(denied("TEAM_EDGE_DENIED"));
        }
        for outcome in &receipt.target_outcomes {
            require_result_edge(&state, &a, &outcome.member_id)?;
        }
        Ok(receipt)
    }
    pub async fn read_team_peer_messages(
        &self,
        owner: &str,
        workspace: &str,
        team_id: &str,
    ) -> Result<Value, CollaborationError> {
        let state = self.load_state().await?;
        team(&state, owner, workspace, team_id)?;
        let mut messages = Vec::new();
        for message in state
            .team_peer_messages
            .values()
            .filter(|m| m.owner_id == owner && m.workspace_id == workspace && m.team_id == team_id)
        {
            let Some(sender) = state.team_execution_attempts.values().find(|a| {
                a.id == message.sender_attempt_id
                    && a.owner_id == owner
                    && a.workspace_id == workspace
                    && a.team_id == team_id
            }) else {
                continue;
            };
            if require_edge(&state, sender, &message.recipient_member_id, &message.mode).is_err() {
                continue;
            }
            let delivery = state
                .team_message_deliveries
                .values()
                .filter(|d| d.message_id == message.message_id)
                .max_by_key(|d| d.consumed_at.or(d.selected_at));
            messages.push(json!({"message":message,"delivery":delivery}));
        }
        Ok(json!({"messages":messages}))
    }
}
