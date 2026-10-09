//! A recorded rejection permits a new explicit decision without rewriting the old effect.
use super::*;
use crate::uar::domain::connector_effect::{ConnectorAction, ConnectorProvider};

impl CollaborationCatalogService {
    /// Approve the same issue bytes again after a definitively rejected dispatch.
    pub async fn retry_feedback_issue_approval(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: RetryFeedbackIssueApprovalRequest,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        explicit::validate_issue(&request.sanitized_issue)?;
        let request_digest = digest(&(id, &request))?;
        let command = command_key(owner, workspace, "retry-issue", &request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            // Replays remain valid after the host has prepared or dispatched the new effect.
            if let Some(prior) = receipt(&current, &command, &request_digest)? {
                return Ok(intake(&current, owner, workspace, &prior.subject_id)?.clone());
            }
            let selected = intake(&current, owner, workspace, id)?;
            if selected.revision != request.expected_revision
                || selected.status != "issue_authorized"
                || selected.duplicate_of.is_some()
                || selected.external_issue_id.is_some()
                || selected.connector_effect_id.as_deref() != Some(&request.expected_effect_id)
            {
                return Err(conflict("FEEDBACK_RETRY_NOT_ELIGIBLE"));
            }
            let draft = selected
                .issue_draft
                .as_ref()
                .ok_or_else(|| conflict("FEEDBACK_DRAFT_UNAVAILABLE"))?;
            let prior_approval = selected
                .issue_approval
                .as_ref()
                .ok_or_else(|| conflict("FEEDBACK_EXPLICIT_APPROVAL_REQUIRED"))?;
            let effect = current
                .connector_effects
                .get(&key(owner, workspace, &request.expected_effect_id))
                .ok_or_else(|| conflict("FEEDBACK_REJECTED_EFFECT_UNAVAILABLE"))?;
            if effect.owner_id != owner
                || effect.workspace_id != workspace
                || effect.status != "rejected"
                || effect.dispatch_id.as_deref() != Some(&request.expected_dispatch_id)
                || effect.receipt.as_ref().is_none_or(|receipt| {
                    receipt.disposition != "rejected" || receipt.external_id.is_some()
                })
                || effect.provider != ConnectorProvider::Github
                || effect.action != ConnectorAction::Publish
                || effect.decision_ref.as_deref() != Some(&prior_approval.id)
                || effect.binding_id != draft.connector_binding_id
                || effect.binding_revision != draft.connector_binding_revision
                || effect.target != draft.target
                || effect.payload_digest != draft.payload_digest
                || effect.payload != draft.sanitized_issue
                || effect.egress_labels.len() != 1
                || !effect.egress_labels.contains(&draft.egress_label)
            {
                return Err(conflict("FEEDBACK_REJECTED_EFFECT_MISMATCH"));
            }
            let run = workflow(&current, selected)?;
            if !matches!(run.status.as_str(), "awaiting_decision" | "accepted")
                || run
                    .steps
                    .get(1)
                    .and_then(|step| step.artifact.as_ref())
                    .is_none_or(|artifact| {
                        artifact.id != draft.artifact_id || artifact.digest != draft.artifact_digest
                    })
            {
                return Err(conflict("FEEDBACK_DRAFT_NOT_COMMITTED"));
            }
            let grant = current
                .connector_bindings
                .get(&key(owner, workspace, &request.connector_binding_id))
                .ok_or_else(|| conflict("FEEDBACK_CONNECTOR_BINDING_UNAVAILABLE"))?;
            if grant.revoked
                || grant.provider != ConnectorProvider::Github
                || grant.revision != request.expected_binding_revision
                || grant.target != request.target
                || !grant.allowed_actions.contains(&ConnectorAction::Publish)
                || !grant.allowed_egress_labels.contains(&request.egress_label)
                || draft.connector_binding_id != grant.id
                || draft.connector_binding_revision != grant.revision
                || draft.target != grant.target
                || draft.egress_label != request.egress_label
                || draft.artifact_id != request.artifact_id
                || draft.artifact_digest != request.artifact_digest
                || draft.payload_digest != request.payload_digest
                || draft.payload_digest != digest(&request.sanitized_issue)?
                || draft.sanitized_issue != request.sanitized_issue
                || draft.sanitization_ref != request.sanitization_ref
                || prior_approval.authority != "explicit-customer"
                || prior_approval.operator_id.as_deref() != Some(owner)
                || prior_approval.action != Some(ConnectorAction::Publish)
                || prior_approval.connector_binding_id.as_deref() != Some(&grant.id)
                || prior_approval.connector_binding_revision != Some(grant.revision)
                || prior_approval.target.as_deref() != Some(&grant.target)
                || prior_approval.egress_label.as_deref() != Some(&draft.egress_label)
                || prior_approval.artifact_id != draft.artifact_id
                || prior_approval.artifact_digest != draft.artifact_digest
                || prior_approval.sanitized_payload_digest != draft.payload_digest
                || prior_approval.sanitization_ref != draft.sanitization_ref
            {
                return Err(conflict("FEEDBACK_EXPLICIT_APPROVAL_MISMATCH"));
            }
            let approval = FeedbackApproval {
                id: format!("feedback-explicit-retry-{id}-{}", request.command_id),
                operator_id: Some(owner.into()),
                connector_binding_revision: Some(grant.revision),
                target: Some(grant.target.clone()),
                action: Some(ConnectorAction::Publish),
                authority: "explicit-customer".into(),
                policy_id: None,
                policy_revision: None,
                connector_binding_id: Some(grant.id.clone()),
                egress_label: Some(draft.egress_label.clone()),
                artifact_id: draft.artifact_id.clone(),
                artifact_digest: draft.artifact_digest.clone(),
                sanitized_payload_digest: draft.payload_digest.clone(),
                sanitization_ref: draft.sanitization_ref.clone(),
                decided_at: Utc::now(),
            };
            let mut next = current.clone();
            let updated = next
                .feedback_intakes
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("FEEDBACK_UNAVAILABLE"))?;
            updated.issue_approval_history.push(prior_approval.clone());
            updated.issue_approval = Some(approval);
            updated.connector_effect_id = None;
            updated.revision += 1;
            updated.updated_at = Utc::now();
            let result = updated.clone();
            next.generation += 1;
            mark_command(
                &mut next,
                command.clone(),
                request_digest.clone(),
                id.into(),
            );
            if self.cas(current.generation, &next).await? {
                return Ok(result);
            }
        }
        Err(conflict("FEEDBACK_CONCURRENT_CHANGE"))
    }
}
