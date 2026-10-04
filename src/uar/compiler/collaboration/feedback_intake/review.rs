use super::*;

impl CollaborationCatalogService {
    /// Link an already committed C09 team output; this does not start another executor.
    pub async fn attach_feedback_review(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: AttachFeedbackReviewRequest,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        if !matches!(request.role.as_str(), "product" | "design" | "reviewer") {
            return Err(invalid("FEEDBACK_REVIEW_ROLE_INVALID"));
        }
        let request_digest = digest(&request)?;
        let command = command_key(owner, workspace, "review", &request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = receipt(&current, &command, &request_digest)? {
                return Ok(intake(&current, owner, workspace, &prior.subject_id)?.clone());
            }
            let selected = intake(&current, owner, workspace, id)?;
            if selected.revision != request.expected_revision
                || selected.duplicate_of.is_some()
                || selected.workflow_run_id.is_none()
                || selected.implementation.is_some()
                || selected.review_artifacts.contains_key(&request.role)
            {
                return Err(conflict("FEEDBACK_REVIEW_NOT_ELIGIBLE"));
            }
            let run = workflow(&current, selected)?;
            let artifact = current
                .team_artifacts
                .get(&key(owner, workspace, &run.team_id, &request.artifact_id))
                .ok_or_else(|| conflict("FEEDBACK_REVIEW_ARTIFACT_UNAVAILABLE"))?;
            let team = current
                .team_instances
                .get(&key(owner, workspace, &run.team_id))
                .ok_or_else(|| conflict("FEEDBACK_REVIEW_TEAM_UNAVAILABLE"))?;
            let task = team
                .tasks
                .iter()
                .find(|task| {
                    task.id == artifact.task_id
                        && task.role == request.role
                        && task.status == "succeeded"
                })
                .ok_or_else(|| conflict("FEEDBACK_REVIEW_TASK_UNAVAILABLE"))?;
            current
                .team_execution_attempts
                .get(&key(owner, workspace, &run.team_id, &artifact.attempt_id))
                .filter(|attempt| {
                    attempt.task_id == task.id
                        && attempt.member_id == artifact.member_id
                        && attempt.status == "succeeded"
                        && attempt.effect_disposition == "confirmed"
                        && task.assignee_member_id.as_deref() == Some(attempt.member_id.as_str())
                        && task.ownership_epoch == attempt.ownership_epoch
                })
                .ok_or_else(|| conflict("FEEDBACK_REVIEW_ATTEMPT_UNAVAILABLE"))?;
            let mut next = current.clone();
            let updated = next
                .feedback_intakes
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("FEEDBACK_UNAVAILABLE"))?;
            updated
                .review_artifacts
                .insert(request.role.clone(), artifact.id.clone());
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

    /// A separate operator decision is required before any implementation is admitted.
    pub async fn admit_feedback_implementation(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        operator_id: &str,
        request: AdmitFeedbackImplementationRequest,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(operator_id)?;
        super::super::validation::validate_id(&request.command_id)?;
        if !matches!(request.decision.as_str(), "admit" | "reject") {
            return Err(invalid("FEEDBACK_IMPLEMENTATION_DECISION_INVALID"));
        }
        let request_digest = digest(&(operator_id, &request))?;
        let command = command_key(owner, workspace, "implementation", &request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = receipt(&current, &command, &request_digest)? {
                return Ok(intake(&current, owner, workspace, &prior.subject_id)?.clone());
            }
            let selected = intake(&current, owner, workspace, id)?;
            if selected.revision != request.expected_revision
                || selected.duplicate_of.is_some()
                || selected.implementation.is_some()
                || !matches!(selected.status.as_str(), "drafted" | "issue_authorized")
            {
                return Err(conflict("FEEDBACK_IMPLEMENTATION_NOT_ELIGIBLE"));
            }
            let run = workflow(&current, selected)?;
            let draft = run
                .steps
                .get(1)
                .and_then(|step| step.artifact.as_ref())
                .ok_or_else(|| conflict("FEEDBACK_DRAFT_UNAVAILABLE"))?;
            let mut required = std::collections::BTreeSet::from([draft.id.clone()]);
            for role in ["product", "design", "reviewer"] {
                required.insert(
                    selected
                        .review_artifacts
                        .get(role)
                        .ok_or_else(|| conflict("FEEDBACK_REVIEW_REQUIRED"))?
                        .clone(),
                );
            }
            if request.artifact_ids != required {
                return Err(conflict("FEEDBACK_IMPLEMENTATION_ARTIFACT_MISMATCH"));
            }
            let mut next = current.clone();
            let updated = next
                .feedback_intakes
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("FEEDBACK_UNAVAILABLE"))?;
            updated.implementation = Some(FeedbackImplementationDecision {
                id: format!("implementation-decision-{id}"),
                operator_id: operator_id.into(),
                decision: request.decision.clone(),
                artifact_ids: required,
                decided_at: Utc::now(),
            });
            updated.status = if request.decision == "admit" {
                "implementation_admitted"
            } else {
                "implementation_rejected"
            }
            .into();
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
