//! Explicit durable-intent dispatch uses the ordinary controller, never another queue.
use super::{CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, fence, key, team};
use crate::uar::domain::team_execution::{DispatchTeamAttemptRequest, TeamExecutionAttempt, TeamExecutionCommandReceipt};
use chrono::Utc;
use serde_json::json;

impl CollaborationCatalogService {
    pub async fn record_team_dispatch_request(
        &self, owner: &str, workspace: &str, team_id: &str, attempt_id: &str,
        request: DispatchTeamAttemptRequest,
    ) -> Result<TeamExecutionAttempt, CollaborationError> {
        super::super::validation::validate_id(&request.command_id)?;
        super::super::validation::validate_id(attempt_id)?;
        let digest = super::super::validation::canonical_digest(&json!({"teamId":team_id,"attemptId":attempt_id,"request":request}))?;
        let receipt_key = format!("{owner}\u{1f}{workspace}\u{1f}{}", request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            let selected = team(&current, owner, workspace, team_id)?;
            let attempt = current.team_execution_attempts.get(&key(owner, workspace, team_id, attempt_id))
                .cloned().ok_or_else(|| CollaborationError::NotFound(attempt_id.into()))?;
            if let Some(receipt) = current.team_execution_command_receipts.get(&receipt_key) {
                if receipt.operation != "dispatch-attempt" || receipt.team_id != team_id || receipt.request_digest != digest {
                    return Err(CollaborationError::Conflict("TEAM_COMMAND_CONFLICT".into()));
                }
                return Ok(attempt);
            }
            if current.team_command_receipts.contains_key(&receipt_key) || current.team_mailbox_command_receipts.contains_key(&receipt_key) {
                return Err(CollaborationError::Conflict("TEAM_COMMAND_CONFLICT".into()));
            }
            if selected.revision != request.expected_team_revision || attempt.status != "queued" {
                return Err(CollaborationError::Conflict("TEAM_REVISION_CONFLICT".into()));
            }
            fence(&current, &attempt)?;
            let mut next = current.clone(); next.generation += 1;
            next.team_execution_command_receipts.insert(receipt_key.clone(), TeamExecutionCommandReceipt {
                owner_id: owner.into(), workspace_id: workspace.into(), team_id: team_id.into(),
                command_id: request.command_id.clone(), operation: "dispatch-attempt".into(), request_digest: digest.clone(),
                attempt_id: Some(attempt_id.into()), recovery_attempt_ids: Vec::new(), committed_at: Utc::now(),
            });
            if self.cas(current.generation, &next).await? { return Ok(attempt); }
        }
        Err(CollaborationError::Conflict("TEAM_REVISION_CONFLICT".into()))
    }
}
