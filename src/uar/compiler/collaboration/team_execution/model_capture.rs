//! Resolve endpoint settings before admission; subsequent turns compare immutable captures.
use super::{CollaborationCatalogService, CollaborationError};
use crate::uar::domain::{collaboration::{CollaborationCatalogState, CollaborationKind, ConversionDisposition}, team_execution::{EffectiveTeamModelReceipt, TeamModelRoute, TeamModelSettingsRequest}, team_planning::{TeamInstance, TeamMember}};

impl CollaborationCatalogService {
    pub(super) async fn capture_team_models(&self, state: &CollaborationCatalogState, team: &TeamInstance, member: &TeamMember) -> Result<Vec<EffectiveTeamModelReceipt>, CollaborationError> {
        if member.kind != "agent" { return Err(CollaborationError::Conflict("TEAM_CAPABILITY_UNSUPPORTED".into())); }
        let definition = state.definitions.get(&member.definition.storage_key()).filter(|d| d.kind == CollaborationKind::AgentDefinition).ok_or_else(|| CollaborationError::NotFound(member.definition.id.clone()))?;
        let binding = state.bindings.get(&super::super::bindings::binding_key(&team.owner_id, &team.workspace_id, &team.binding.id)).ok_or_else(|| CollaborationError::NotFound(team.binding.id.clone()))?;
        let mut diagnostics = Vec::new();
        super::super::runtime_semantics::resolve_legacy_team_context(definition, &binding.document, &mut diagnostics);
        let models = super::super::bindings::resolve_models(&binding.document, definition, &mut diagnostics)?;
        super::super::runtime_semantics::resolve_runtime_semantics(definition, &models, self.provider_registry.as_deref(), &mut diagnostics).await;
        if let Some(unsupported) = diagnostics.iter().find(|d| d.disposition == ConversionDisposition::RequiredUnsupported) {
            return Err(CollaborationError::Conflict(if unsupported.reason_code.starts_with("TEAM_") { unsupported.reason_code.clone() } else { "TEAM_CAPABILITY_UNSUPPORTED".into() }));
        }
        let registry = self.provider_registry.as_deref().ok_or_else(|| CollaborationError::Conflict("TEAM_PROFILE_UNSUPPORTED".into()))?;
        let mut result = Vec::new();
        for model in models {
            let provider = model["providerId"].as_str().ok_or_else(|| CollaborationError::Invalid("TEAM_ROUTE_PROFILE_MISMATCH".into()))?;
            let model_id = model["modelId"].as_str().ok_or_else(|| CollaborationError::Invalid("TEAM_ROUTE_PROFILE_MISMATCH".into()))?;
            let mut request = registry.team_model_settings_request(provider, model_id).await.map_err(|e| CollaborationError::Conflict(e.to_string()))?;
            if !model["profile"].is_null() || !model["settingsRevision"].is_null() {
                request = TeamModelSettingsRequest {
                    route: TeamModelRoute { provider_id: provider.into(), model_id: model_id.into() },
                    profile: serde_json::from_value(model["profile"].clone())?,
                    expected_settings_revision: model["settingsRevision"].as_u64().ok_or_else(|| CollaborationError::Invalid("TEAM_REVISION_CONFLICT".into()))?,
                    reasoning: request.reasoning,
                };
            }
            result.push(registry.resolve_team_model_settings(&request).await.map_err(|e| CollaborationError::Conflict(e.to_string()))?);
        }
        if result.is_empty() { return Err(CollaborationError::Conflict("TEAM_PROFILE_UNSUPPORTED".into())); }
        Ok(result)
    }
}
