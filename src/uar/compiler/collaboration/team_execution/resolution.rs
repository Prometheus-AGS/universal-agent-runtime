//! Exact member projection through the existing ordinary-agent resource resolver.

use super::super::{
    BoundAgentRun, CollaborationCatalogService, CollaborationError, bindings, grants,
    runtime_semantics,
};
use crate::uar::domain::{
    collaboration::{CollaborationKind, ConversionDisposition},
    team_execution::TeamExecutionAttempt,
};

impl CollaborationCatalogService {
    pub async fn resolve_team_member_run(
        &self,
        attempt: &TeamExecutionAttempt,
    ) -> Result<BoundAgentRun, CollaborationError> {
        self.revalidate_team_attempt(attempt).await?;
        let state = self.load_state().await?;
        let team = self
            .get_team_instance(&attempt.owner_id, &attempt.workspace_id, &attempt.team_id)
            .await?;
        let member = team
            .members
            .iter()
            .find(|member| member.id == attempt.member_id)
            .ok_or_else(|| CollaborationError::NotFound(attempt.member_id.clone()))?;
        let definition = state
            .definitions
            .get(&member.definition.storage_key())
            .filter(|definition| definition.kind == CollaborationKind::AgentDefinition)
            .ok_or_else(|| {
                CollaborationError::Conflict("Member has no exact installed AgentDefinition".into())
            })?;
        let binding = state
            .bindings
            .get(&bindings::binding_key(
                &attempt.owner_id,
                &attempt.workspace_id,
                &team.binding.id,
            ))
            .filter(|binding| {
                binding.revision == attempt.binding_revision && binding.package == team.package
            })
            .cloned()
            .ok_or_else(|| {
                CollaborationError::Conflict("Team deployment binding changed".into())
            })?;
        if !state
            .packages
            .get(&binding.package.storage_key())
            .is_some_and(|package| {
                package
                    .definition_keys
                    .contains(&member.definition.storage_key())
            })
        {
            return Err(CollaborationError::Conflict(
                "Member definition is outside the installed team package".into(),
            ));
        }
        let (skills, mut diagnostics) =
            bindings::resolve_skills(self.skill_service.as_deref(), &binding.document, definition)
                .await?;
        let mut models = bindings::resolve_models(&binding.document, definition, &mut diagnostics)?;
        if let Some(registry) = self.provider_registry.as_deref() {
            for model in &mut models {
                let provider_id = model["providerId"].as_str().unwrap_or_default();
                let model_id = model["modelId"].as_str().unwrap_or_default();
                if let Some(provider) = registry.get(provider_id).await {
                    if let Some(identity) = provider
                        .models
                        .iter()
                        .find(|configured| configured.id == model_id)
                        .and_then(|configured| configured.pricing_identity.as_ref())
                    {
                        identity
                            .qualified_model()
                            .map_err(|error| CollaborationError::Conflict(error.to_string()))?;
                        model["pricingIdentity"] = serde_json::to_value(identity)
                            .map_err(|error| CollaborationError::Storage(error.to_string()))?;
                    }
                }
            }
        }
        let effective = runtime_semantics::resolve_runtime_semantics(
            definition,
            &models,
            self.provider_registry.as_deref(),
            &mut diagnostics,
        )
        .await;
        let service_binding = bindings::resolve_service_binding(
            self.service_instance.as_deref(),
            &binding.document,
            &mut diagnostics,
        )?;
        let representation_grants = grants::validate_binding_grants(
            &attempt.owner_id,
            &attempt.workspace_id,
            &binding.document,
            &state,
        )?;
        if diagnostics
            .iter()
            .any(|item| item.disposition == ConversionDisposition::RequiredUnsupported)
        {
            return Err(CollaborationError::Conflict(
                "Member resources cannot be executed by the current runtime".into(),
            ));
        }
        let receipt = bindings::effective_receipt(
            &binding.document,
            binding.package.clone(),
            definition,
            skills,
            models,
            representation_grants,
            service_binding,
            effective,
            diagnostics,
            true,
        )?;
        let artifact = definition.compatibility_agent.clone().ok_or_else(|| {
            CollaborationError::Conflict(
                "Member definition has no ordinary-agent projection".into(),
            )
        })?;
        self.revalidate_team_attempt(attempt).await?;
        Ok(BoundAgentRun {
            artifact,
            binding,
            effective_binding_receipt: receipt,
        })
    }
}
