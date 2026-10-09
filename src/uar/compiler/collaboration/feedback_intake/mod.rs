//! Scoped feedback records reuse the existing workflow and C09 artifact authorities.
mod control;
mod explicit;
mod policy;
mod review;
use super::{CollaborationCatalogService, CollaborationError, service::MAX_CAS_ATTEMPTS};
use crate::uar::domain::{collaboration::CollaborationCatalogState, feedback_intake::*};
use chrono::Utc;
use uuid::Uuid;

fn key(owner: &str, workspace: &str, id: &str) -> String {
    format!("{owner}\u{1f}{workspace}\u{1f}{id}")
}
fn command_key(owner: &str, workspace: &str, operation: &str, id: &str) -> String {
    key(owner, workspace, &format!("{operation}:{id}"))
}
fn conflict(message: &str) -> CollaborationError {
    CollaborationError::Conflict(message.into())
}
fn invalid(message: &str) -> CollaborationError {
    CollaborationError::Invalid(message.into())
}
fn digest<T: serde::Serialize>(value: &T) -> Result<String, CollaborationError> {
    Ok(super::validation::request_digest(value)?)
}
fn validate_scope(owner: &str, workspace: &str) -> Result<(), CollaborationError> {
    super::service::validate_owner(owner)?;
    super::validation::validate_id(workspace)?;
    Ok(())
}
fn intake<'a>(
    state: &'a CollaborationCatalogState,
    owner: &str,
    workspace: &str,
    id: &str,
) -> Result<&'a FeedbackIntake, CollaborationError> {
    state
        .feedback_intakes
        .get(&key(owner, workspace, id))
        .ok_or_else(|| CollaborationError::NotFound(id.into()))
}
fn workflow<'a>(
    state: &'a CollaborationCatalogState,
    intake: &FeedbackIntake,
) -> Result<&'a crate::uar::domain::workflow_execution::WorkflowRun, CollaborationError> {
    let id = intake
        .workflow_run_id
        .as_ref()
        .ok_or_else(|| conflict("FEEDBACK_WORKFLOW_REQUIRED"))?;
    state
        .workflow_runs
        .get(&key(&intake.owner_id, &intake.workspace_id, id))
        .ok_or_else(|| conflict("FEEDBACK_WORKFLOW_UNAVAILABLE"))
}
fn receipt<'a>(
    state: &'a CollaborationCatalogState,
    command: &str,
    request_digest: &str,
) -> Result<Option<&'a FeedbackCommandReceipt>, CollaborationError> {
    let Some(value) = state.feedback_commands.get(command) else {
        return Ok(None);
    };
    if value.request_digest != request_digest {
        return Err(conflict("FEEDBACK_COMMAND_CONFLICT"));
    }
    Ok(Some(value))
}
fn mark_command(
    state: &mut CollaborationCatalogState,
    command: String,
    request_digest: String,
    id: String,
) {
    state.feedback_commands.insert(
        command,
        FeedbackCommandReceipt {
            request_digest,
            subject_id: id,
        },
    );
}
impl CollaborationCatalogService {
    pub async fn observe_feedback(
        &self,
        owner: &str,
        workspace: &str,
        request: ObserveFeedbackRequest,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::validation::validate_id(&request.command_id)?;
        super::validation::validate_id(&request.source_event_id)?;
        let content = request.feedback.trim();
        if content.is_empty() || content.len() > 16 * 1024 {
            return Err(invalid("FEEDBACK_CONTENT_INVALID"));
        }
        let fingerprint = digest(
            &content
                .split_whitespace()
                .map(str::to_lowercase)
                .collect::<Vec<_>>()
                .join(" "),
        )?;
        let feedback_digest = digest(&request.feedback)?;
        let request_digest = digest(&request)?;
        let command = command_key(owner, workspace, "observe", &request.command_id);
        let id = Uuid::new_v4().to_string();
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = receipt(&current, &command, &request_digest)? {
                return Ok(intake(&current, owner, workspace, &prior.subject_id)?.clone());
            }
            if let Some(prior) = current.feedback_intakes.values().find(|item| {
                item.owner_id == owner
                    && item.workspace_id == workspace
                    && item.source == request.source
                    && item.source_event_id == request.source_event_id
            }) {
                return if prior.feedback_digest == feedback_digest
                    && prior.requested_target == request.target
                {
                    Ok(prior.clone())
                } else {
                    Err(conflict("FEEDBACK_SOURCE_EVENT_CONFLICT"))
                };
            }
            let now = Utc::now();
            let record = FeedbackIntake {
                id: id.clone(),
                owner_id: owner.into(),
                workspace_id: workspace.into(),
                revision: 1,
                source: request.source,
                source_event_id: request.source_event_id.clone(),
                feedback: request.feedback.clone(),
                requested_target: request.target.clone(),
                feedback_digest: feedback_digest.clone(),
                fingerprint: fingerprint.clone(),
                status: "observed".into(),
                workflow_run_id: None,
                classification: None,
                duplicate_of: None,
                review_artifacts: Default::default(),
                issue_approval: None,
                issue_draft: None,
                connector_effect_id: None,
                external_issue_id: None,
                implementation: None,
                created_at: now,
                updated_at: now,
            };
            let mut next = current.clone();
            next.generation += 1;
            next.feedback_intakes
                .insert(key(owner, workspace, &id), record.clone());
            mark_command(
                &mut next,
                command.clone(),
                request_digest.clone(),
                id.clone(),
            );
            if self.cas(current.generation, &next).await? {
                return Ok(record);
            }
        }
        Err(conflict("FEEDBACK_CONCURRENT_CHANGE"))
    }

    pub async fn link_feedback_workflow(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: LinkFeedbackWorkflowRequest,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::validation::validate_id(&request.command_id)?;
        let request_digest = digest(&request)?;
        let command = command_key(owner, workspace, "link", &request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = receipt(&current, &command, &request_digest)? {
                return Ok(intake(&current, owner, workspace, &prior.subject_id)?.clone());
            }
            let selected = intake(&current, owner, workspace, id)?;
            if selected.revision != request.expected_revision || selected.workflow_run_id.is_some()
            {
                return Err(conflict("FEEDBACK_REVISION_CHANGED"));
            }
            let run = current
                .workflow_runs
                .get(&key(owner, workspace, &request.workflow_run_id))
                .ok_or_else(|| conflict("FEEDBACK_WORKFLOW_UNAVAILABLE"))?;
            if digest(&run.input["feedback"])? != selected.feedback_digest
                || current.feedback_intakes.values().any(|item| {
                    item.owner_id == owner
                        && item.workspace_id == workspace
                        && item.workflow_run_id.as_deref() == Some(&request.workflow_run_id)
                })
            {
                return Err(conflict("FEEDBACK_WORKFLOW_MISMATCH"));
            }
            let mut next = current.clone();
            let updated = next
                .feedback_intakes
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("FEEDBACK_UNAVAILABLE"))?;
            updated.workflow_run_id = Some(run.id.clone());
            updated.status = "classifying".into();
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

    pub async fn list_feedback_intakes(
        &self,
        owner: &str,
        workspace: &str,
    ) -> Result<Vec<FeedbackIntake>, CollaborationError> {
        validate_scope(owner, workspace)?;
        Ok(self
            .load_state()
            .await?
            .feedback_intakes
            .into_values()
            .filter(|item| item.owner_id == owner && item.workspace_id == workspace)
            .collect())
    }
    pub async fn get_feedback_intake(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        Ok(intake(&self.load_state().await?, owner, workspace, id)?.clone())
    }

    /// Classify the durable workflow output and choose one canonical issue candidate.
    pub async fn finalize_feedback(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        expected_revision: u64,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let selected = intake(&current, owner, workspace, id)?;
            if selected.revision != expected_revision {
                return Err(conflict("FEEDBACK_REVISION_CHANGED"));
            }
            if !matches!(selected.status.as_str(), "classifying" | "drafted") {
                return Err(conflict("FEEDBACK_NOT_FINALIZABLE"));
            }
            let run = workflow(&current, selected)?;
            if !matches!(run.status.as_str(), "awaiting_decision" | "accepted") {
                return Err(conflict("FEEDBACK_DRAFT_NOT_COMMITTED"));
            }
            if run
                .steps
                .get(1)
                .and_then(|step| step.artifact.as_ref())
                .is_none()
            {
                return Err(conflict("FEEDBACK_DRAFT_NOT_COMMITTED"));
            }
            let category = run
                .steps
                .first()
                .and_then(|s| s.artifact.as_ref())
                .and_then(|a| a.content["category"].as_str())
                .ok_or_else(|| conflict("FEEDBACK_CLASSIFICATION_UNAVAILABLE"))?
                .to_owned();
            let duplicate = current
                .feedback_intakes
                .values()
                .filter(|item| {
                    item.owner_id == owner
                        && item.workspace_id == workspace
                        && item.id != selected.id
                        && item.classification.as_deref() == Some(&category)
                        && item.fingerprint == selected.fingerprint
                        && item.duplicate_of.is_none()
                })
                .min_by_key(|item| item.created_at)
                .map(|item| item.id.clone());
            let mut next = current.clone();
            let updated = next
                .feedback_intakes
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("FEEDBACK_UNAVAILABLE"))?;
            updated.classification = Some(category);
            updated.duplicate_of = duplicate;
            updated.status = if updated.duplicate_of.is_some() {
                "duplicate"
            } else {
                "drafted"
            }
            .into();
            updated.revision += 1;
            updated.updated_at = Utc::now();
            let result = updated.clone();
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                return Ok(result);
            }
        }
        Err(conflict("FEEDBACK_CONCURRENT_CHANGE"))
    }
}
