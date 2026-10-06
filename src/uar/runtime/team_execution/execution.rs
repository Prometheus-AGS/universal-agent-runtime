use super::TeamExecutionRuntime;
use crate::uar::{
    compiler::collaboration::CollaborationError,
    domain::team_execution::{TeamExecutionAttempt, TeamReservation},
    runtime::{
        actor::messages::ActorOwner,
        thread::{
            AgentThreadResult, actor_host::ActorThreadSession, policy_intersection::ThreadBudgets,
        },
        turn::RunExecutionRequest,
    },
};
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Mutex, watch};
use tokio_util::sync::CancellationToken;

impl TeamExecutionRuntime {
    pub(super) async fn execute(
        &self,
        owner: ActorOwner,
        attempt: TeamExecutionAttempt,
        cancellation: CancellationToken,
    ) {
        let started = std::time::Instant::now();
        let entered = AtomicBool::new(false);
        let outcome = self
            .execute_turn(&owner, &attempt, cancellation.clone(), &entered)
            .await;
        let mut yielded = None;
        let mut output = None;
        let mut reason = None;
        let mut status = match outcome {
            Ok(record) => match record.thread.result {
                Some(AgentThreadResult::Completed { output: text }) => {
                    output = Some(Value::String(text));
                    "succeeded"
                }
                Some(AgentThreadResult::Yielded { control }) => {
                    yielded = Some(control);
                    "yielded"
                }
                Some(AgentThreadResult::Cancelled) => "cancelled",
                Some(AgentThreadResult::Failed { code, message }) => {
                    reason = Some(match message.split_once("; diagnostic reference ") {
                        Some((safe_code, reference))
                            if matches!(
                                safe_code,
                                "TEAM_PROVIDER_REQUEST_REJECTED" | "TEAM_PROVIDER_STREAM_FAILED"
                            ) && uuid::Uuid::parse_str(reference).is_ok() =>
                        {
                            message
                        }
                        _ => code,
                    });
                    "failed"
                }
                None => {
                    reason = Some("terminal_result_missing".into());
                    "uncertain"
                }
            },
            Err(error) => {
                tracing::error!(attempt_id = %attempt.id, %error, "Team actor host did not confirm terminal execution");
                if entered.load(Ordering::Acquire) {
                    reason = Some("team_host_completion_unconfirmed".into());
                    "uncertain"
                } else {
                    reason = Some("team_dispatch_denied_before_kernel_entry".into());
                    if cancellation.is_cancelled() {
                        "cancelled"
                    } else {
                        "failed"
                    }
                }
            }
        };
        if let Some(content) = &output {
            let contract_result = async {
                let team = self
                    .catalog
                    .get_team_instance(&attempt.owner_id, &attempt.workspace_id, &attempt.team_id)
                    .await?;
                let task = team
                    .tasks
                    .iter()
                    .find(|task| task.id == attempt.task_id)
                    .ok_or_else(|| CollaborationError::NotFound(attempt.task_id.clone()))?;
                let validator =
                    jsonschema::validator_for(&task.output_contract).map_err(|error| {
                        CollaborationError::Invalid(format!(
                            "Task output contract is invalid: {error}"
                        ))
                    })?;
                if validator.is_valid(content) {
                    return Ok(content.clone());
                }
                let parsed = content
                    .as_str()
                    .and_then(|text| serde_json::from_str::<Value>(text).ok());
                parsed
                    .filter(|parsed| validator.is_valid(parsed))
                    .ok_or_else(|| {
                        CollaborationError::Conflict(
                            "Member output does not satisfy its task output contract".into(),
                        )
                    })
            }
            .await;
            match contract_result {
                Ok(validated) => output = Some(validated),
                Err(_) => {
                    output = None;
                    status = "failed";
                    reason = Some("team_output_contract_rejected".into());
                }
            }
        }
        if let Some(content) = &output {
            if self
                .catalog
                .add_team_artifact(&attempt, content.clone())
                .await
                .is_err()
            {
                output = None;
                reason = Some("team_artifact_publication_unconfirmed".into());
                status = "failed";
            }
        }
        let snapshot = self.manager.run_usage(&attempt.run_id);
        let effects = self
            .manager
            .tool_admission_evidence_for_user(owner.user_id(), &attempt.run_id)
            .await;
        let effects_confirmed = effects.as_ref().is_ok_and(|records| {
            use crate::uar::persistence::tool_admission::ToolAdmissionEvidenceState;
            records
                .iter()
                .filter(|record| {
                    matches!(
                        record.state,
                        ToolAdmissionEvidenceState::ClaimIntent
                            | ToolAdmissionEvidenceState::OutcomeUnknown
                    )
                })
                .all(|claim| {
                    records.iter().any(|record| {
                        record.invocation_id == claim.invocation_id
                            && matches!(
                                record.state,
                                ToolAdmissionEvidenceState::Succeeded
                                    | ToolAdmissionEvidenceState::Failed
                            )
                    })
                })
        });
        // A model request without a provider usage report keeps its whole reservation.
        let usage_known = snapshot.usage_reports == snapshot.model_requests
            && snapshot.completed_usage_reports == snapshot.model_requests
            && !snapshot.unpriced_usage
            && effects_confirmed
            && status != "uncertain";
        let usage = usage_known.then(|| TeamReservation {
            tokens: snapshot.total_tokens,
            cost_microunits: (snapshot.cost_usd * 1_000_000.0).ceil() as u64,
            elapsed_seconds: started
                .elapsed()
                .as_secs()
                .saturating_add(u64::from(started.elapsed().subsec_nanos() > 0)),
        });
        let status = if effects_confirmed {
            status
        } else {
            "uncertain"
        };
        if !usage_known {
            reason.get_or_insert_with(|| "team_usage_or_effect_outcome_uncertain".into());
        }
        if let Some(control) = yielded.as_ref().filter(|_| effects_confirmed) {
            if let Err(error) = self
                .catalog
                .finalize_team_yield(&attempt, control, usage)
                .await
            {
                tracing::error!(attempt_id=%attempt.id,%error,"Team yield settlement remains unconfirmed");
            }
            return;
        }
        if let Err(error) = self
            .catalog
            .settle_team_attempt(&attempt, status, usage, output, reason)
            .await
        {
            tracing::error!(attempt_id = %attempt.id, %error, "Team attempt settlement remains unconfirmed");
        }
    }

    async fn execute_turn(
        &self,
        owner: &ActorOwner,
        attempt: &TeamExecutionAttempt,
        cancellation: CancellationToken,
        entered: &AtomicBool,
    ) -> anyhow::Result<crate::uar::persistence::agent_threads::PersistedAgentThread> {
        anyhow::ensure!(
            !cancellation.is_cancelled(),
            "Team attempt cancelled before actor entry"
        );
        let bound = self.catalog.resolve_team_member_run(attempt).await?;
        let context = self
            .catalog
            .selected_team_context(attempt, &bound.effective_binding_receipt.resolved_skills)
            .await?;
        self.catalog
            .record_team_context_selection(attempt, &context.receipt)
            .await?;
        let guidance = context.receipt.team_instructions.clone();
        let mut request = RunExecutionRequest::from_bound_agent(
            bound,
            serde_json::to_string(&context.data)?,
            attempt.owner_id.clone(),
            attempt.workspace_id.clone(),
            Arc::clone(&self.catalog),
        )
        .with_verified_owner(owner.clone())
        .with_team_attempt(attempt.clone())?;
        self.apply_host_context(attempt, &mut request).await?;
        if let Some(binding) = request.collaboration_binding.as_mut() {
            binding.team_instructions = guidance;
        }
        request.session_id = Some(format!("team-attempt:{}", attempt.id));
        request.host_budget_constraint = Some(ThreadBudgets {
            max_tokens_per_turn: Some(attempt.reservation.tokens),
            max_tokens_per_session: Some(attempt.reservation.tokens),
            max_cost_per_session_usd: Some(
                attempt.reservation.cost_microunits as f64 / 1_000_000.0,
            ),
            timeout_seconds: Some(attempt.reservation.elapsed_seconds),
            ..ThreadBudgets::default()
        });
        request.host_usage_grant =
            Some(crate::uar::runtime::cost_budget::RemoteUsageGrantBinding {
                accounting_id: format!("team-budget:{}", attempt.id),
                grant: crate::uar::api::a2a::contract::UarUsageGrant {
                    max_total_tokens: Some(attempt.reservation.tokens),
                    max_total_cost_usd: Some(
                        attempt.reservation.cost_microunits as f64 / 1_000_000.0,
                    ),
                    expires_after_seconds: Some(attempt.reservation.elapsed_seconds),
                    ..Default::default()
                },
                started_at: std::time::Instant::now(),
            });
        let (state, _observer) = watch::channel(None);
        let mut host = ActorThreadSession::new(
            owner.clone(),
            request.artifact.clone(),
            request
                .session_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Team attempt session is missing"))?,
            Arc::clone(&self.manager),
            Arc::clone(&self.persistence),
            cancellation,
            state,
            Arc::new(Mutex::new(None)),
        );
        entered.store(true, Ordering::Release);
        let result = host
            .execute_request(request, attempt.run_id.clone(), None, None)
            .await;
        // Failed starts can still own a persisted root or child cleanup; settle them.
        host.finish_abandoned().await?;
        result
    }
}
