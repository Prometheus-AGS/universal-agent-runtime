//! Restriction-only representation profile. Cedar remains the tool policy owner.
use super::CollaborationRunBinding;
use crate::uar::domain::{
    collaboration::RepresentationGrant,
    policy::{EffectiveRunPolicy, SelectionMode},
};
use crate::uar::runtime::tool_admission::{
    AdmittedToolInvocation, LocalAdmissionDisposition, PreparedToolInvocation,
};
use std::collections::BTreeSet;

pub(crate) const CAPABILITY: &str = "collaboration_representation_execution_v1";

#[derive(Debug, Clone)]
pub(crate) struct RepresentationContext {
    pub grants: Vec<RepresentationGrant>,
}

impl RepresentationContext {
    fn prefixed(values: &[String], prefix: &str) -> anyhow::Result<BTreeSet<String>> {
        values
            .iter()
            .map(|value| {
                value
                    .strip_prefix(prefix)
                    .filter(|id| !id.is_empty() && *id != "*")
                    .map(str::to_owned)
                    .ok_or_else(|| anyhow::anyhow!("REPRESENTATION_SCOPE_REQUIRED_UNSUPPORTED"))
            })
            .collect()
    }

    pub fn validate(&self, owner: &str, workspace: &str) -> anyhow::Result<()> {
        for grant in &self.grants {
            Self::prefixed(&grant.action_scopes, "tool:")?;
            anyhow::ensure!(
                grant.resource_scopes == [format!("workspace:{workspace}")],
                "REPRESENTATION_RESOURCE_SCOPE_DENIED"
            );
            Self::prefixed(&grant.data_scopes, "knowledge-base:")?;
            anyhow::ensure!(
                grant.audience_scopes.iter().all(|scope| {
                    scope == &format!("user:{owner}")
                        || scope
                            .strip_prefix("team-member:")
                            .is_some_and(|id| !id.is_empty() && id != "*")
                }) && grant.audience_scopes.contains(&format!("user:{owner}")),
                "REPRESENTATION_AUDIENCE_SCOPE_DENIED"
            );
            anyhow::ensure!(
                grant
                    .approval_requirements
                    .iter()
                    .all(|requirement| matches!(
                        requirement.as_str(),
                        "current-policy" | "real-human"
                    )),
                "REPRESENTATION_APPROVAL_REQUIRED_UNSUPPORTED"
            );
            anyhow::ensure!(
                grant.disclosure_requirements == ["disclose-agent-assistance"],
                "REPRESENTATION_DISCLOSURE_REQUIRED_UNSUPPORTED"
            );
            anyhow::ensure!(
                grant
                    .restrictions
                    .forbidden_claims
                    .iter()
                    .all(|claim| matches!(claim.as_str(), "human-authorship" | "human-approval")),
                "REPRESENTATION_RESTRICTION_REQUIRED_UNSUPPORTED"
            );
            anyhow::ensure!(
                grant.offboarding.mode == "revoke-immediately"
                    && grant
                        .offboarding
                        .required_actions
                        .iter()
                        .all(|action| action == "disable-binding"),
                "REPRESENTATION_OFFBOARDING_REQUIRED_UNSUPPORTED"
            );
            anyhow::ensure!(
                grant.retention.policy == "retain-audit" && grant.retention.delete_after.is_none(),
                "REPRESENTATION_RETENTION_REQUIRED_UNSUPPORTED"
            );
        }
        Ok(())
    }

    pub fn narrow(&self, policy: &mut EffectiveRunPolicy) -> anyhow::Result<()> {
        let eligible_tools = policy.tools.ids.iter().cloned().collect::<BTreeSet<_>>();
        let eligible_data = policy
            .knowledge_bases
            .ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        for grant in &self.grants {
            let tools = Self::prefixed(&grant.action_scopes, "tool:")?;
            anyhow::ensure!(
                tools.is_subset(&eligible_tools),
                "REPRESENTATION_ACTION_RESOURCE_UNAVAILABLE"
            );
            policy.tools.ids.retain(|id| tools.contains(id));
            policy.tools.mode = SelectionMode::Selected;
            let data = Self::prefixed(&grant.data_scopes, "knowledge-base:")?;
            anyhow::ensure!(
                data.is_subset(&eligible_data),
                "REPRESENTATION_DATA_RESOURCE_UNAVAILABLE"
            );
            policy.knowledge_bases.ids.retain(|id| data.contains(id));
            policy.knowledge_bases.mode = SelectionMode::Selected;
            policy.memory_enabled = false;
        }
        Ok(())
    }

    pub fn disclosure(&self) -> String {
        let grants = self
            .grants
            .iter()
            .map(|grant| {
                format!(
                    "{}@{} (subject {}, office {})",
                    grant.grant_id, grant.revision, grant.subject_principal_id, grant.office
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "AI-assisted representation under scoped grants {grants}. This is agent assistance, not human authorship or human approval."
        )
    }

    fn authorize_tool(&self, invocation: &PreparedToolInvocation) -> anyhow::Result<()> {
        for grant in &self.grants {
            anyhow::ensure!(
                invocation.representation_effect
                    == crate::uar::tools::descriptor::ToolEffect::ReadOnly
                    || (invocation.admission_owner
                        == crate::uar::persistence::tool_admission::AdmissionOwner::UarRuntime
                        && matches!(
                            invocation.provider_tool_name.as_str(),
                            "team_send" | "team_wait"
                        )),
                "REPRESENTATION_EFFECT_REQUIRED_UNSUPPORTED"
            );
            anyhow::ensure!(
                grant
                    .action_scopes
                    .contains(&format!("tool:{}", invocation.provider_tool_name)),
                "REPRESENTATION_ACTION_SCOPE_DENIED"
            );
            if invocation.provider_tool_name == "team_send" {
                let recipient = invocation
                    .validated_arguments
                    .pointer("/recipient/memberId")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("REPRESENTATION_COMMUNICATION_SCOPE_DENIED"))?;
                anyhow::ensure!(
                    grant
                        .audience_scopes
                        .contains(&format!("team-member:{recipient}")),
                    "REPRESENTATION_COMMUNICATION_SCOPE_DENIED"
                );
                let text = invocation
                    .validated_arguments
                    .pointer("/payload/text")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                anyhow::ensure!(
                    text.contains(&self.disclosure()),
                    "REPRESENTATION_COMMUNICATION_DISCLOSURE_REQUIRED"
                );
            }
            // Delegation would assign another agent authority not granted to this grantee.
            anyhow::ensure!(
                invocation.provider_tool_name != "team_delegate",
                "REPRESENTATION_SUBDELEGATION_UNSUPPORTED"
            );
        }
        Ok(())
    }
}

impl CollaborationRunBinding {
    pub(crate) fn representation_grants(
        &self,
    ) -> Vec<crate::uar::domain::collaboration::RepresentationGrantRef> {
        let mut references = self.receipt.representation_grants.clone();
        if let Some(instance) = &self.instance_authority {
            for reference in &instance.representation_grants {
                if !references.contains(reference) {
                    references.push(reference.clone());
                }
            }
        }
        references
    }

    pub(crate) fn has_representation(&self) -> bool {
        !self.receipt.representation_grants.is_empty()
            || self.instance_authority.as_ref().is_some_and(|instance| {
                !instance.representation_grants.is_empty()
            })
    }

    pub(crate) async fn representation_context(&self) -> anyhow::Result<RepresentationContext> {
        self.revalidate_authority().await?;
        let references = self.representation_grants();
        let context = RepresentationContext {
            grants: self
                .service
                .execution_representation_grants(
                    &self.owner_id,
                    &self.workspace_id,
                    &references,
                )
                .await?,
        };
        let principal = if let Some(instance) = &self.instance_authority {
            anyhow::ensure!(
                context.grants.iter().all(|grant|
                    grant.grantee_agent_instance_id == instance.instance_id()
                        && grant.issuer_principal_id == instance.principal_id),
                "REPRESENTATION_INSTANCE_SCOPE_DENIED"
            );
            instance.principal_id.as_str()
        } else {
            self.principal_id.as_deref().unwrap_or(&self.owner_id)
        };
        context.validate(principal, &self.workspace_id)?;
        Ok(context)
    }

    pub(crate) async fn prepare_representation_tool(
        &self,
        invocation: &PreparedToolInvocation,
    ) -> anyhow::Result<bool> {
        let context = self.representation_context().await?;
        context.authorize_tool(invocation)?;
        Ok(context.grants.iter().any(|grant| {
            grant
                .approval_requirements
                .iter()
                .any(|requirement| requirement == "real-human")
        }))
    }

    pub(crate) async fn claim_representation_tool(
        &self,
        admitted: &AdmittedToolInvocation,
    ) -> anyhow::Result<()> {
        let requires_human = self.prepare_representation_tool(&admitted.prepared).await?;
        anyhow::ensure!(
            !self.has_representation()
                || admitted.local_disposition != LocalAdmissionDisposition::GovernanceBypassed,
            "REPRESENTATION_CEDAR_REQUIRED"
        );
        anyhow::ensure!(
            !requires_human || admitted.local_disposition == LocalAdmissionDisposition::Approved,
            "REPRESENTATION_REAL_HUMAN_APPROVAL_REQUIRED"
        );
        Ok(())
    }
}
