//! Trusted settings-only team profiles. Pricing and capacity do not qualify wire fit.

use serde::{Deserialize, Serialize};
use crate::uar::domain::team_execution::{
    EffectiveTeamModelReceipt, TeamFitDisposition, TeamLimitMetadata,
    TeamModelRoute, TeamModelSettingsRequest, TeamProfileRef, TeamReasoningRequest, TeamPricingIdentity,
};
use super::{EndpointRequestProfile, EndpointRequestTransform};
use super::registry::{ProviderRegistry, ProtocolSetting};

pub const SETTINGS_PROFILE_ID: &str = "uar.openai-compatible-chat.settings-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamModelExecutionSettings {
    pub profile: TeamProfileRef,
    pub settings_revision: u64,
    #[serde(default)]
    pub reasoning: TeamReasoningRequest,
}

impl Default for TeamModelExecutionSettings {
    fn default() -> Self {
        Self {
            profile: TeamProfileRef { id: SETTINGS_PROFILE_ID.into(), revision: 1 },
            settings_revision: 1,
            reasoning: TeamReasoningRequest::Off,
        }
    }
}

impl TeamModelExecutionSettings {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.profile.id == SETTINGS_PROFILE_ID && self.profile.revision == 1,
            "TEAM_PROFILE_UNSUPPORTED");
        anyhow::ensure!(self.settings_revision > 0 && self.settings_revision <= 9_007_199_254_740_991,
            "TEAM_REVISION_CONFLICT");
        // No explicit reasoning mapping is qualified in this registered endpoint profile.
        anyhow::ensure!(matches!(self.reasoning, TeamReasoningRequest::Off),
            "TEAM_REASONING_UNSUPPORTED");
        Ok(())
    }
}

impl ProviderRegistry {
    pub async fn team_model_settings_request(
        &self, provider_id: &str, model_id: &str,
    ) -> anyhow::Result<TeamModelSettingsRequest> {
        let provider = self.get(provider_id).await
            .ok_or_else(|| anyhow::anyhow!("TEAM_ROUTE_PROFILE_MISMATCH"))?;
        let model = provider.models.iter().find(|model| model.id == model_id)
            .ok_or_else(|| anyhow::anyhow!("TEAM_ROUTE_PROFILE_MISMATCH"))?;
        let settings = model.execution_profile.clone().unwrap_or_default();
        Ok(TeamModelSettingsRequest {
            route: TeamModelRoute { provider_id: provider_id.into(), model_id: model_id.into() },
            profile: settings.profile,
            expected_settings_revision: settings.settings_revision,
            reasoning: settings.reasoning,
        })
    }

    pub async fn resolve_team_model_settings(
        &self, request: &TeamModelSettingsRequest,
    ) -> anyhow::Result<EffectiveTeamModelReceipt> {
        let provider = self.get(&request.route.provider_id).await
            .ok_or_else(|| anyhow::anyhow!("TEAM_ROUTE_PROFILE_MISMATCH"))?;
        let model = provider.models.iter().find(|model| model.id == request.route.model_id)
            .filter(|model| model.enabled && provider.enabled)
            .ok_or_else(|| anyhow::anyhow!("TEAM_ROUTE_PROFILE_MISMATCH"))?;
        let settings = model.execution_profile.clone().unwrap_or_default();
        settings.validate()?;
        anyhow::ensure!(request.profile == settings.profile, "TEAM_ROUTE_PROFILE_MISMATCH");
        anyhow::ensure!(request.expected_settings_revision == settings.settings_revision,
            "TEAM_REVISION_CONFLICT");
        anyhow::ensure!(request.reasoning == settings.reasoning, "TEAM_REASONING_UNSUPPORTED");
        anyhow::ensure!(!provider.base_url.trim().is_empty()
            && !matches!(provider.protocol, ProtocolSetting::Responses), "TEAM_PROFILE_UNSUPPORTED");
        let pricing_identity = match &model.pricing_identity {
            Some(identity) => {
                identity.qualified_model()?;
                let catalog = super::catalog::ModelCatalog::global().model(&identity.provider_id, &identity.model_id)
                    .ok_or_else(|| anyhow::anyhow!("TEAM_ROUTE_PROFILE_MISMATCH"))?;
                use sha2::{Digest, Sha256};
                let digest = Sha256::digest(serde_json::to_vec(&serde_json::json!({
                    "provider": identity.provider_id, "model": identity.model_id, "price": catalog.cost,
                }))?);
                let revision = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                Some(TeamPricingIdentity { provider_id: identity.provider_id.clone(), model_id: identity.model_id.clone(), catalog_revision: format!("sha256:{revision}") })
            }
            None => None,
        };
        let fingerprint = crate::uar::runtime::context::budget::endpoint_fingerprint(&provider.base_url);
        use sha2::{Digest, Sha256};
        let evidence = serde_json::to_vec(&serde_json::json!({
            "route": request.route, "endpoint": fingerprint, "settings": settings,
            "wireAlias": model.id, "profileSource": "uar.team-settings-profile.v1"
        }))?;
        let digest = Sha256::digest(evidence);
        let evidence_id = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        Ok(EffectiveTeamModelReceipt {
            route: request.route.clone(), wire_model_alias: model.id.clone(),
            pricing_identity,
            endpoint_kind: "openai-compatible-chat".into(), profile: settings.profile,
            settings_revision: settings.settings_revision,
            requested_reasoning: request.reasoning.clone(), effective_reasoning: settings.reasoning,
            support: "validated".into(), support_evidence_ref: format!("team-profile-{evidence_id}"),
            // Operator/catalog metadata cannot prove the gateway's actual capacity.
            limits: TeamLimitMetadata { context_tokens: None, output_tokens: None,
                source: "unknown".into(), source_revision: None },
            fit: TeamFitDisposition::SettingsOnly { guaranteed_fit: false,
                reason: "count-or-framing-unqualified".into() },
        })
    }

    /// Capture the endpoint fingerprint with the already admitted settings receipt.
    pub async fn capture_team_endpoint_profile(
        &self, effective: &EffectiveTeamModelReceipt,
    ) -> anyhow::Result<EndpointRequestProfile> {
        let _administration = self.administration_guard().await;
        let current = self.resolve_team_model_settings(&TeamModelSettingsRequest {
            route: effective.route.clone(), profile: effective.profile.clone(),
            expected_settings_revision: effective.settings_revision,
            reasoning: effective.requested_reasoning.clone(),
        }).await?;
        anyhow::ensure!(serde_json::to_value(&current)? == serde_json::to_value(effective)?,
            "TEAM_ROUTE_PROFILE_MISMATCH");
        let provider = self.get(&effective.route.provider_id).await
            .ok_or_else(|| anyhow::anyhow!("TEAM_ROUTE_PROFILE_MISMATCH"))?;
        Ok(EndpointRequestProfile {
            profile_id: effective.profile.id.clone(), profile_revision: effective.profile.revision.to_string(),
            provider_id: effective.route.provider_id.clone(), endpoint_kind: effective.endpoint_kind.clone(),
            endpoint_fingerprint: crate::uar::runtime::context::budget::endpoint_fingerprint(&provider.base_url),
            qualified_model: format!("{}/{}", effective.route.provider_id, effective.route.model_id),
            wire_model: effective.wire_model_alias.clone(), model_revision: effective.settings_revision.to_string(),
            settings_revision: effective.settings_revision.to_string(),
            transform: EndpointRequestTransform::OpenAiCompatibleChatV1,
            allowed_request_fields: ["model", "messages", "stream", "stream_options", "tools",
                "tool_choice", "parallel_tool_calls"].into_iter().map(str::to_owned).collect(),
            output_ceiling: None,
        })
    }
}
