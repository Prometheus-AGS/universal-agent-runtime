use super::*;

impl CollaborationCatalogService {
    /// Reject or cancel feedback before an external dispatch; dispatched effects remain inspectable.
    pub async fn control_feedback(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: FeedbackControlRequest,
    ) -> Result<FeedbackIntake, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        if !matches!(request.decision.as_str(), "reject" | "cancel") {
            return Err(invalid("FEEDBACK_CONTROL_INVALID"));
        }
        let request_digest = digest(&(id, &request))?;
        let command = command_key(owner, workspace, "control", &request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = receipt(&current, &command, &request_digest)? {
                return Ok(intake(&current, owner, workspace, &prior.subject_id)?.clone());
            }
            let selected = intake(&current, owner, workspace, id)?;
            if selected.revision != request.expected_revision
                || selected.implementation.is_some()
                || selected
                    .connector_effect_id
                    .as_ref()
                    .is_some_and(|effect_id| {
                        current
                            .connector_effects
                            .get(&key(owner, workspace, effect_id))
                            .is_none_or(|effect| effect.status != "cancelled")
                    })
            {
                return Err(conflict("FEEDBACK_CONTROL_NOT_ELIGIBLE"));
            }
            let mut next = current.clone();
            let updated = next
                .feedback_intakes
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("FEEDBACK_UNAVAILABLE"))?;
            updated.status = if request.decision == "reject" {
                "rejected"
            } else {
                "cancelled"
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
