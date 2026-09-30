//! Exact member projection through the existing ordinary-agent resource resolver.

use super::super::{
    BoundAgentRun, CollaborationCatalogService, CollaborationError, bindings, grants,
    runtime_semantics,
};
use crate::uar::domain::{
    collaboration::{CollaborationKind, ConversionDisposition},
    policy::{ResourceSelection, RunPolicy, SelectionMode},
    team_execution::{TeamExecutionAttempt, TeamModelSettingsRequest},
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
        runtime_semantics::resolve_legacy_team_context(definition, &binding.document, &mut diagnostics);
        let mut models = bindings::resolve_models(&binding.document, definition, &mut diagnostics)?;
        let registry = self.provider_registry.as_deref()
            .ok_or_else(|| CollaborationError::Conflict("TEAM_PROFILE_UNSUPPORTED".into()))?;
        for model in &mut models {
            let provider_id = model["providerId"].as_str().unwrap_or_default().to_owned();
            let model_id = model["modelId"].as_str().unwrap_or_default().to_owned();
            let captured = attempt.effective_models.iter().find(|captured|
                captured.route.provider_id == provider_id && captured.route.model_id == model_id)
                .ok_or_else(|| CollaborationError::Conflict("TEAM_ROUTE_PROFILE_MISMATCH".into()))?;
            let current = registry.resolve_team_model_settings(&TeamModelSettingsRequest {
                route: captured.route.clone(), profile: captured.profile.clone(),
                expected_settings_revision: captured.settings_revision,
                reasoning: captured.requested_reasoning.clone(),
            }).await.map_err(|error| CollaborationError::Conflict(error.to_string()))?;
            if serde_json::to_value(&current)? != serde_json::to_value(captured)? {
                return Err(CollaborationError::Conflict("TEAM_ROUTE_PROFILE_MISMATCH".into()));
            }
            model["effectiveModel"] = serde_json::to_value(captured)?;
            if let Some(provider) = registry.get(&provider_id).await {
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
        let mut effective = runtime_semantics::resolve_runtime_semantics(
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
        // Team attempts admit the immutable member's resources, rather than
        // inheriting the host's complete skill/tool/context inventory.
        let selected = |ids: Vec<String>| {
            if ids.is_empty() {
                ResourceSelection {
                    mode: SelectionMode::None,
                    ..Default::default()
                }
            } else {
                ResourceSelection::selected(ids)
            }
        };
        let skill_ids = skills.iter().map(|skill| skill.skill.id.clone()).collect();
        let mut tool_ids = skills
            .iter()
            .flat_map(|skill| skill.skill.required_tools.iter().cloned())
            .collect::<Vec<_>>();
        if tool_ids.iter().any(|tool| tool == "spawn_agent") {
            return Err(CollaborationError::Conflict("TEAM_CAPABILITY_UNSUPPORTED".into()));
        }
        // These factories remain bound to the ordinary host's control policy.
        tool_ids.extend(
            crate::uar::runtime::thread::control::AGENT_TOOL_NAMES
                .into_iter()
                .filter(|tool| *tool != "spawn_agent")
                .map(str::to_owned),
        );
        if crate::uar::api::capabilities::team_execution_b_enabled(){tool_ids.extend(["team_roster","team_send","team_delegate","team_wait"].into_iter().map(str::to_owned));}
        tool_ids.push("activate_skill".into());
        tool_ids.push(crate::uar::runtime::native_skills::search_tools::SEARCH_TOOLS_NAME.into());
        let installed = match self.skill_service.as_deref() {
            Some(service) => service.get_skills().await,
            None => Vec::new(),
        };
        let server_ids = installed
            .iter()
            .filter(|installed| skills.iter().any(|skill| skill.skill.id == installed.skill_id))
            .filter_map(|skill| skill.mcp_config.as_ref())
            .flat_map(|config| config.mcp_servers.keys().cloned())
            .collect();
        let team_policy = RunPolicy {
            skills: selected(skill_ids),
            tools: selected(tool_ids),
            mcp_servers: selected(server_ids),
            knowledge_bases: selected(Vec::new()),
            presentations: selected(Vec::new()),
            memory_enabled: Some(false),
            context_strategy: effective
                .get("contextStrategy")
                .cloned()
                .and_then(|value| serde_json::from_value(value).ok()),
            ..Default::default()
        };
        effective["teamRunPolicy"] = serde_json::to_value(team_policy)?;
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
