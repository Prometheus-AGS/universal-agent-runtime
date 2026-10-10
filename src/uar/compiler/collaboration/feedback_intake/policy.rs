use super::*;
use crate::uar::domain::connector_effect::{ConnectorAction, ConnectorApprovalMode, ConnectorProvider};

impl CollaborationCatalogService {
    pub async fn install_feedback_policy(
        &self,
        owner: &str,
        workspace: &str,
        request: FeedbackPolicyRequest,
    ) -> Result<FeedbackStandingPolicy, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        super::super::validation::validate_id(&request.id)?;
        super::super::validation::validate_id(&request.connector_binding_id)?;
        if request.egress_label.is_empty() || request.egress_label.len() > 100 {
            return Err(invalid("FEEDBACK_EGRESS_LABEL_INVALID"));
        }
        let request_digest = digest(&request)?;
        let command = command_key(owner, workspace, "policy", &request.command_id);
        let storage_key = key(owner, workspace, &request.id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = receipt(&current, &command, &request_digest)? {
                return current
                    .feedback_policies
                    .get(&key(owner, workspace, &prior.subject_id))
                    .cloned()
                    .ok_or_else(|| conflict("FEEDBACK_POLICY_UNAVAILABLE"));
            }
            let grant = current
                .connector_bindings
                .get(&key(owner, workspace, &request.connector_binding_id))
                .ok_or_else(|| conflict("FEEDBACK_CONNECTOR_BINDING_UNAVAILABLE"))?;
            if grant.revoked
                || grant.provider != ConnectorProvider::Github
                || grant.approval_mode != ConnectorApprovalMode::StandingPolicy
                || !grant.allowed_actions.contains(&ConnectorAction::Publish)
                || !grant.allowed_egress_labels.contains(&request.egress_label)
            {
                return Err(conflict("FEEDBACK_POLICY_CONNECTOR_DENIED"));
            }
            let previous = current.feedback_policies.get(&storage_key);
            if previous.map(|policy| policy.revision) != request.expected_revision {
                return Err(conflict("FEEDBACK_POLICY_REVISION_CHANGED"));
            }
            let policy = FeedbackStandingPolicy {
                id: request.id.clone(),
                owner_id: owner.into(),
                workspace_id: workspace.into(),
                revision: previous.map_or(1, |value| value.revision + 1),
                source: request.source,
                connector_binding_id: request.connector_binding_id.clone(),
                egress_label: request.egress_label.clone(),
                enabled: request.enabled,
                updated_at: Utc::now(),
            };
            let mut next = current.clone();
            next.generation += 1;
            next.feedback_policies
                .insert(storage_key.clone(), policy.clone());
            mark_command(
                &mut next,
                command.clone(),
                request_digest.clone(),
                request.id.clone(),
            );
            if self.cas(current.generation, &next).await? {
                return Ok(policy);
            }
        }
        Err(conflict("FEEDBACK_CONCURRENT_CHANGE"))
    }

    pub async fn list_feedback_policies(
        &self,
        owner: &str,
        workspace: &str,
    ) -> Result<Vec<FeedbackStandingPolicy>, CollaborationError> {
        validate_scope(owner, workspace)?;
        Ok(self
            .load_state()
            .await?
            .feedback_policies
            .into_values()
            .filter(|item| item.owner_id == owner && item.workspace_id == workspace)
            .collect())
    }

    /// A scoped standing policy authorizes only the exact drafted artifact, once.
    pub async fn authorize_feedback_issue(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: AuthorizeFeedbackIssueRequest,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        super::super::validation::validate_id(&request.sanitization_ref)?;
        super::explicit::validate_issue(&request.sanitized_issue)?;
        let request_digest = digest(&(id, &request))?;
        let command = command_key(owner, workspace, "issue", &request.command_id);
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
            let policy = current
                .feedback_policies
                .get(&key(owner, workspace, &request.policy_id))
                .ok_or_else(|| conflict("FEEDBACK_POLICY_UNAVAILABLE"))?;
            if !policy.enabled
                || policy.revision != request.expected_policy_revision
                || policy.source != selected.source
            {
                return Err(conflict("FEEDBACK_POLICY_CHANGED"));
            }
            let grant = current
                .connector_bindings
                .get(&key(owner, workspace, &policy.connector_binding_id))
                .ok_or_else(|| conflict("FEEDBACK_CONNECTOR_BINDING_UNAVAILABLE"))?;
            if grant.revoked
                || grant.provider != ConnectorProvider::Github
                || grant.approval_mode != ConnectorApprovalMode::StandingPolicy
                || !grant.allowed_actions.contains(&ConnectorAction::Publish)
                || !grant.allowed_egress_labels.contains(&policy.egress_label)
            {
                return Err(conflict("FEEDBACK_POLICY_CONNECTOR_DENIED"));
            }
            let run = workflow(&current, selected)?;
            if !matches!(run.status.as_str(), "awaiting_decision" | "accepted") {
                return Err(conflict("FEEDBACK_DRAFT_NOT_COMMITTED"));
            }
            let draft = run
                .steps
                .get(1)
                .and_then(|step| step.artifact.as_ref())
                .ok_or_else(|| conflict("FEEDBACK_DRAFT_UNAVAILABLE"))?;
            let materialized = selected
                .issue_draft
                .as_ref()
                .ok_or_else(|| conflict("FEEDBACK_DRAFT_UNAVAILABLE"))?;
            if materialized.artifact_id != draft.id
                || materialized.artifact_digest != draft.digest
                || materialized.connector_binding_id != grant.id
                || materialized.connector_binding_revision != grant.revision
                || materialized.target != grant.target
                || materialized.egress_label != policy.egress_label
                || materialized.sanitized_issue != request.sanitized_issue
                || materialized.payload_digest != digest(&request.sanitized_issue)?
                || materialized.sanitization_ref != request.sanitization_ref
            {
                return Err(conflict("FEEDBACK_STANDING_APPROVAL_MISMATCH"));
            }
            let approval = FeedbackApproval {
                operator_id: None,
                connector_binding_revision: Some(grant.revision),
                target: Some(grant.target.clone()),
                action: Some(ConnectorAction::Publish),
                id: format!("feedback-approval-{id}"),
                authority: "standing-policy".into(),
                policy_id: Some(policy.id.clone()),
                policy_revision: Some(policy.revision),
                connector_binding_id: Some(grant.id.clone()),
                egress_label: Some(policy.egress_label.clone()),
                artifact_id: draft.id.clone(),
                artifact_digest: draft.digest.clone(),
                sanitized_payload_digest: digest(&request.sanitized_issue)?,
                sanitization_ref: request.sanitization_ref.clone(),
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
