//! The customer GitHub profile requires an explicit decision on the exact draft.
use super::*;
use crate::uar::domain::connector_effect::{ConnectorAction, ConnectorProvider};

pub(super) fn validate_issue(value: &serde_json::Value) -> Result<(), CollaborationError> {
    if value.as_object().is_none_or(|object| object.len() != 2)
        || ["title", "body"].iter().any(|field| {
            value[*field]
                .as_str()
                .is_none_or(|text| text.trim().is_empty())
        })
        || serde_json::to_vec(value)?.len() > 64 * 1024
    {
        return Err(invalid("FEEDBACK_SANITIZED_ISSUE_INVALID"));
    }
    Ok(())
}

impl CollaborationCatalogService {
    /// Materialize a committed team workflow draft for customer approval.
    pub async fn draft_feedback_issue(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: DraftFeedbackIssueRequest,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        super::super::validation::validate_id(&request.sanitization_ref)?;
        validate_issue(&request.sanitized_issue)?;
        let request_digest = digest(&(id, &request))?;
        let command = command_key(owner, workspace, "issue-draft", &request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = receipt(&current, &command, &request_digest)? {
                return Ok(intake(&current, owner, workspace, &prior.subject_id)?.clone());
            }
            let selected = intake(&current, owner, workspace, id)?;
            if selected.revision != request.expected_revision
                || selected.issue_draft.is_some()
                || selected.issue_approval.is_some()
                || selected.duplicate_of.is_some()
                || selected.status != "drafted"
            {
                return Err(conflict("FEEDBACK_DRAFT_NOT_ELIGIBLE"));
            }
            let run = workflow(&current, selected)?;
            if !matches!(run.status.as_str(), "awaiting_decision" | "accepted") {
                return Err(conflict("FEEDBACK_DRAFT_NOT_COMMITTED"));
            }
            let artifact = run
                .steps
                .get(1)
                .and_then(|step| step.artifact.as_ref())
                .ok_or_else(|| conflict("FEEDBACK_DRAFT_UNAVAILABLE"))?;
            if artifact.content != request.sanitized_issue {
                return Err(conflict("FEEDBACK_WORKFLOW_DRAFT_MISMATCH"));
            }
            let grant = current
                .connector_bindings
                .get(&key(owner, workspace, &request.connector_binding_id))
                .ok_or_else(|| conflict("FEEDBACK_CONNECTOR_BINDING_UNAVAILABLE"))?;
            if grant.revoked
                || grant.provider != ConnectorProvider::Github
                || selected
                    .requested_target
                    .as_ref()
                    .is_some_and(|target| target != &grant.target)
                || grant.revision != request.expected_binding_revision
                || !grant.allowed_actions.contains(&ConnectorAction::Publish)
                || !grant.allowed_egress_labels.contains(&request.egress_label)
            {
                return Err(conflict("FEEDBACK_CONNECTOR_AUTHORITY_CHANGED"));
            }
            let duplicate = current
                .feedback_intakes
                .values()
                .filter(|item| {
                    item.owner_id == owner
                        && item.workspace_id == workspace
                        && item.id != id
                        && item.fingerprint == selected.fingerprint
                        && item.duplicate_of.is_none()
                        && item
                            .issue_draft
                            .as_ref()
                            .is_some_and(|draft| draft.target == grant.target)
                })
                .min_by_key(|item| item.created_at)
                .map(|item| item.id.clone());
            let draft = FeedbackIssueDraft {
                artifact_id: artifact.id.clone(),
                artifact_digest: artifact.digest.clone(),
                payload_digest: digest(&issue_with_marker(&request.sanitized_issue, id))?,
                connector_binding_id: grant.id.clone(),
                connector_binding_revision: grant.revision,
                target: grant.target.clone(),
                egress_label: request.egress_label.clone(),
                sanitized_issue: issue_with_marker(&request.sanitized_issue, id),
                sanitization_ref: request.sanitization_ref.clone(),
            };
            let mut next = current.clone();
            let updated = next
                .feedback_intakes
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("FEEDBACK_UNAVAILABLE"))?;
            updated.duplicate_of = duplicate;
            updated.status = if updated.duplicate_of.is_some() {
                "duplicate"
            } else {
                "drafted"
            }
            .into();
            updated.issue_draft = Some(draft);
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

    /// The authenticated customer approves repository, action, artifact, and payload together.
    pub async fn explicitly_approve_feedback_issue(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: ExplicitFeedbackIssueApprovalRequest,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        validate_issue(&request.sanitized_issue)?;
        let request_digest = digest(&(id, &request))?;
        let command = command_key(owner, workspace, "explicit-issue", &request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = receipt(&current, &command, &request_digest)? {
                return Ok(intake(&current, owner, workspace, &prior.subject_id)?.clone());
            }
            let selected = intake(&current, owner, workspace, id)?;
            if selected.revision != request.expected_revision
                || selected.status != "drafted"
                || selected.duplicate_of.is_some()
                || selected.issue_approval.is_some()
            {
                return Err(conflict("FEEDBACK_ISSUE_NOT_ELIGIBLE"));
            }
            let draft = selected
                .issue_draft
                .as_ref()
                .ok_or_else(|| conflict("FEEDBACK_DRAFT_UNAVAILABLE"))?;
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
            {
                return Err(conflict("FEEDBACK_EXPLICIT_APPROVAL_MISMATCH"));
            }
            let approval = FeedbackApproval {
                id: format!("feedback-explicit-approval-{id}"),
                operator_id: Some(owner.into()),
                connector_binding_revision: Some(grant.revision),
                target: Some(grant.target.clone()),
                action: Some(ConnectorAction::Publish),
                authority: "explicit-customer".into(),
                policy_id: None,
                policy_revision: None,
                connector_binding_id: Some(grant.id.clone()),
                egress_label: Some(request.egress_label.clone()),
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
            updated.issue_approval = Some(approval);
            updated.status = "issue_authorized".into();
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

fn issue_with_marker(issue: &serde_json::Value, id: &str) -> serde_json::Value {
    let mut payload = issue.clone();
    payload["body"] = serde_json::Value::String(format!(
        "{}\n\n<!-- prometheus-feedback-intake:{id} -->",
        issue["body"].as_str().unwrap_or_default()
    ));
    payload
}
