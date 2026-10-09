//! Exact-artifact publication authority, rechecked before persistence and dispatch.
use super::*;

pub(super) fn feedback_approval<'a>(
    state: &'a crate::uar::domain::collaboration::CollaborationCatalogState,
    owner: &str,
    workspace: &str,
    grant: &ConnectorBinding,
    decision_ref: Option<&str>,
    payload_digest: &str,
    labels: &std::collections::BTreeSet<String>,
) -> Result<&'a crate::uar::domain::feedback_intake::FeedbackIntake, CollaborationError> {
    state
        .feedback_intakes
        .values()
        .find(|intake| {
            intake.owner_id == owner
                && intake.workspace_id == workspace
                && intake.duplicate_of.is_none()
                && intake.status != "cancelled"
                && intake.status != "rejected"
                && intake.issue_approval.as_ref().is_some_and(|approval| {
                    (match grant.approval_mode {
                        ConnectorApprovalMode::ExplicitCustomer => {
                            approval.authority == "explicit-customer"
                                && approval.operator_id.as_deref() == Some(owner)
                        }
                        ConnectorApprovalMode::StandingPolicy => {
                            approval.authority == "standing-policy"
                                && approval.policy_id.as_ref().is_some_and(|policy_id| {
                                    state
                                        .feedback_policies
                                        .get(&key(owner, workspace, policy_id))
                                        .is_some_and(|policy| {
                                            policy.enabled
                                                && Some(policy.revision) == approval.policy_revision
                                                && policy.source == intake.source
                                                && policy.connector_binding_id == grant.id
                                                && approval.egress_label.as_deref()
                                                    == Some(policy.egress_label.as_str())
                                        })
                                })
                        }
                    })
                        && Some(approval.id.as_str()) == decision_ref
                        && approval.sanitized_payload_digest == payload_digest
                        && approval.connector_binding_id.as_deref() == Some(grant.id.as_str())
                        && approval.connector_binding_revision == Some(grant.revision)
                        && approval.target.as_deref() == Some(grant.target.as_str())
                        && approval.action == Some(ConnectorAction::Publish)
                        && approval
                            .egress_label
                            .as_ref()
                            .is_some_and(|label| labels.len() == 1 && labels.contains(label))
                        && intake.issue_draft.as_ref().is_some_and(|draft| {
                            draft.artifact_id == approval.artifact_id
                                && draft.artifact_digest == approval.artifact_digest
                                && draft.payload_digest == payload_digest
                                && intake.workflow_run_id.as_ref().is_some_and(|run_id| {
                                    state
                                        .workflow_runs
                                        .get(&key(owner, workspace, run_id))
                                        .is_some_and(|run| {
                                            matches!(
                                                run.status.as_str(),
                                                "awaiting_decision" | "accepted"
                                            ) && run
                                                .steps
                                                .get(1)
                                                .and_then(|step| step.artifact.as_ref())
                                                .is_some_and(|artifact| {
                                                    artifact.id == draft.artifact_id
                                                        && artifact.digest == draft.artifact_digest
                                                })
                                        })
                                })
                        })
                })
        })
        .ok_or_else(|| {
            conflict(match grant.approval_mode {
                ConnectorApprovalMode::ExplicitCustomer => {
                    "CONNECTOR_EXPLICIT_CUSTOMER_APPROVAL_REQUIRED"
                }
                ConnectorApprovalMode::StandingPolicy => "CONNECTOR_STANDING_POLICY_APPROVAL_REQUIRED",
            })
        })
}
