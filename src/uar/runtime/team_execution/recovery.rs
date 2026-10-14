//! A saved wait never restores a future. Confirm the old kernel joined, then CAS admission.
use super::TeamExecutionRuntime;
use crate::uar::{
    compiler::collaboration::CollaborationError,
    persistence::tool_admission::ToolAdmissionEvidenceState,
    runtime::{actor::messages::ActorOwner, thread::AgentThreadResult},
};
impl TeamExecutionRuntime {
    pub(super) async fn confirmed_joined_yields(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        team: &str,
    ) -> Result<
        Vec<(
            crate::uar::domain::team_execution::TeamExecutionAttempt,
            crate::uar::domain::team_wait::KernelTeamYield,
        )>,
        CollaborationError,
    > {
        let candidates = self
            .catalog
            .pending_team_yield_recoveries(&owner.presentation_owner_key(), workspace, team)
            .await?;
        let mut confirmed = Vec::new();
        for (attempt, control) in candidates {
            let Some(root) = self
                .persistence
                .load_agent_thread(owner.user_id(), &attempt.root_id)
                .await
                .map_err(|_| {
                    CollaborationError::Storage("Team recovery root could not be read".into())
                })?
            else {
                continue;
            };
            if root.thread.run_id.as_deref() != Some(attempt.run_id.as_str())
                || root.thread.root_thread_id != attempt.root_id
                || !matches!(root.thread.result.as_ref(),Some(AgentThreadResult::Yielded{control:stored}) if stored==&control)
            {
                continue;
            }
            let children = self
                .persistence
                .list_agent_threads(owner.user_id(), &attempt.run_id)
                .await
                .map_err(|_| {
                    CollaborationError::Storage("Team recovery tree could not be read".into())
                })?;
            if children.is_empty()
                || children
                    .iter()
                    .any(|record| !record.thread.status.is_terminal())
            {
                continue;
            }
            let effects = self
                .manager
                .tool_admission_evidence_for_user(&owner.presentation_owner_key(), &attempt.run_id)
                .await
                .map_err(|_| {
                    CollaborationError::Storage(
                        "Team recovery effect evidence could not be read".into(),
                    )
                })?;
            let mut runs = std::collections::BTreeSet::from([attempt.run_id.clone()]);
            runs.extend(effects.iter().map(|record| record.run_id.clone()));
            let mut canonical = Vec::new();
            for run in runs {
                canonical.extend(self.persistence.list_canonical_tool_receipts(owner.user_id(), &run)
                    .await.map_err(|_| CollaborationError::Storage("Team recovery canonical outcomes could not be read".into()))?);
            }
            if crate::uar::persistence::agent_threads::has_ambiguous_legacy_tool_outcomes(&canonical, &effects) {
                continue;
            }
            if effects
                .iter()
                .filter(|e| {
                    matches!(
                        e.state,
                        ToolAdmissionEvidenceState::ClaimIntent
                            | ToolAdmissionEvidenceState::OutcomeUnknown
                    )
                })
                .any(|claim| {
                    !effects.iter().any(|e| {
                        e.invocation_id == claim.invocation_id
                            && matches!(
                                e.state,
                                ToolAdmissionEvidenceState::Succeeded
                                    | ToolAdmissionEvidenceState::Failed
                                    | ToolAdmissionEvidenceState::ApplicationFailed
                                    | ToolAdmissionEvidenceState::TransportFailed
                            )
                    })
                })
            {
                continue;
            }
            // Provider accounting was not restored from a future; keep its reservation unknown.
            confirmed.push((attempt, control));
        }
        Ok(confirmed)
    }
}
