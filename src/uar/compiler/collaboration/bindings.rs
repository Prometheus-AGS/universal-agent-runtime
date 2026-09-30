use std::collections::BTreeSet;

use chrono::Utc;
use serde_json::{Value, json};

use crate::uar::domain::collaboration::{
    BindingCommandRequest, BindingInstallResponse, BindingPreflightResponse,
    COLLABORATION_PROFILE_DRAFT_2, CollaborationCatalogState, CollaborationCommandReceipt,
    CollaborationDefinitionRecord, CollaborationKind, ConversionDiagnostic, ConversionDisposition,
    DeploymentBindingRecord, EffectiveBindingReceipt, FieldDiagnostic, ImmutableDefinitionRef,
    PrivateRevisionRef, RepresentationGrantRef, ResolvedSkill, SkillRef,
};

use super::service::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, receipt_key, validate_owner,
};
use super::validation::{
    canonical_digest, request_digest, validate_common_sections, validate_digest, validate_id,
    validate_semver,
};

impl CollaborationCatalogService {
    pub async fn preflight_binding(
        &self,
        owner_id: &str,
        workspace_id: &str,
        request: &BindingCommandRequest,
    ) -> Result<BindingPreflightResponse, CollaborationError> {
        let state = self.load_state().await?;
        let (preflight, _) = self
            .validate_binding(owner_id, workspace_id, request, &state)
            .await?;
        let storage_key = binding_key(owner_id, workspace_id, &preflight.binding_id);
        enforce_binding_revision(
            request.expected_revision,
            state
                .bindings
                .get(&storage_key)
                .map(|binding| binding.revision),
        )?;
        Ok(preflight)
    }

    pub async fn install_binding(
        &self,
        owner_id: &str,
        workspace_id: &str,
        request: BindingCommandRequest,
    ) -> Result<BindingInstallResponse, CollaborationError> {
        validate_owner(owner_id)?;
        let command_digest = request_digest(&request).map_err(CollaborationError::from)?;
        let receipt_key = receipt_key(owner_id, &request.command_id);

        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let (preflight, effective_binding_receipt) = self
                .validate_binding(owner_id, workspace_id, &request, &current)
                .await?;
            if let Some(receipt) = current.command_receipts.get(&receipt_key) {
                if receipt.operation != "install-binding"
                    || receipt.request_digest != command_digest
                {
                    return Err(CollaborationError::Conflict(format!(
                        "commandId '{}' was already used for a different operation or content",
                        request.command_id
                    )));
                }
                let receipt_revision = receipt.binding_revision.ok_or_else(|| {
                    CollaborationError::Storage(
                        "idempotent binding receipt has no binding revision".to_owned(),
                    )
                })?;
                let binding = current
                    .binding_history
                    .get(&binding_revision_key(
                        owner_id,
                        workspace_id,
                        &receipt.resource_id,
                        receipt_revision,
                    ))
                    .cloned()
                    .ok_or_else(|| {
                        CollaborationError::Storage(
                            "idempotent binding receipt has no matching record".to_owned(),
                        )
                    })?;
                return Ok(BindingInstallResponse {
                    preflight,
                    binding,
                    receipt: receipt.clone(),
                });
            }

            let binding_id = required_string(&request.binding, "id")?.to_owned();
            let storage_key = binding_key(owner_id, workspace_id, &binding_id);
            let existing_revision = current
                .bindings
                .get(&storage_key)
                .map(|binding| binding.revision);
            enforce_binding_revision(request.expected_revision, existing_revision)?;
            let revision = existing_revision.unwrap_or(0).saturating_add(1);
            let declared_revision = request
                .binding
                .get("revision")
                .and_then(Value::as_u64)
                .ok_or_else(|| {
                    CollaborationError::Invalid(
                        "binding revision must be a non-negative integer".to_owned(),
                    )
                })?;
            if declared_revision != revision {
                return Err(CollaborationError::Conflict(format!(
                    "binding document revision {declared_revision} must be next revision {revision}"
                )));
            }
            let package = parse_reference(request.binding.get("package").ok_or_else(|| {
                CollaborationError::Invalid("binding package is missing".to_owned())
            })?)?;
            let binding = DeploymentBindingRecord {
                id: binding_id.clone(),
                owner_id: owner_id.to_owned(),
                workspace_id: workspace_id.to_owned(),
                revision,
                document: request.binding.clone(),
                package,
                activation_supported: preflight.activation_supported,
                preflight_diagnostics: preflight.diagnostics.clone(),
                effective_binding_receipt: Some(effective_binding_receipt.clone()),
                updated_at: Utc::now(),
            };

            let mut next = current.clone();
            next.generation = current.generation.saturating_add(1);
            next.bindings.insert(storage_key, binding.clone());
            next.binding_history.insert(
                binding_revision_key(owner_id, workspace_id, &binding.id, revision),
                binding.clone(),
            );
            next.effective_binding_receipts.insert(
                effective_binding_receipt.content_digest.clone(),
                effective_binding_receipt,
            );
            let receipt = CollaborationCommandReceipt {
                command_id: request.command_id.clone(),
                owner_id: owner_id.to_owned(),
                request_digest: command_digest.clone(),
                operation: "install-binding".to_owned(),
                resource_id: binding_id,
                catalog_revision: next.catalog_revision,
                binding_revision: Some(revision),
                committed_at: Utc::now(),
            };
            next.command_receipts
                .insert(receipt_key.clone(), receipt.clone());
            if self.cas(current.generation, &next).await? {
                return Ok(BindingInstallResponse {
                    preflight,
                    binding,
                    receipt,
                });
            }
        }
        Err(CollaborationError::Conflict(
            "catalog changed repeatedly while installing the binding".to_owned(),
        ))
    }

    pub async fn list_bindings(
        &self,
        owner_id: &str,
        workspace_id: &str,
    ) -> Result<Vec<DeploymentBindingRecord>, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        Ok(self
            .load_state()
            .await?
            .bindings
            .into_values()
            .filter(|binding| binding.owner_id == owner_id && binding.workspace_id == workspace_id)
            .collect())
    }

    pub async fn get_binding(
        &self,
        owner_id: &str,
        workspace_id: &str,
        binding_id: &str,
    ) -> Result<DeploymentBindingRecord, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        validate_id(binding_id)?;
        self.load_state()
            .await?
            .bindings
            .get(&binding_key(owner_id, workspace_id, binding_id))
            .cloned()
            .ok_or_else(|| CollaborationError::NotFound(binding_id.to_owned()))
    }

    async fn validate_binding(
        &self,
        owner_id: &str,
        workspace_id: &str,
        request: &BindingCommandRequest,
        state: &CollaborationCatalogState,
    ) -> Result<(BindingPreflightResponse, EffectiveBindingReceipt), CollaborationError> {
        validate_binding(
            self.skill_service.as_deref(),
            self.provider_registry.as_deref(),
            self.service_instance.as_deref(),
            &self.execution_identity()?,
            owner_id,
            workspace_id,
            request,
            state,
        )
        .await
    }
}

async fn validate_binding(
    skill_service: Option<&crate::uar::runtime::skills::SkillService>,
    provider_registry: Option<&crate::llm::ProviderRegistry>,
    service_instance: Option<&crate::uar::service_instance::ServiceInstanceAuthority>,
    execution_identity: &crate::uar::domain::team_execution::TeamExecutionFence,
    owner_id: &str,
    workspace_id: &str,
    request: &BindingCommandRequest,
    state: &CollaborationCatalogState,
) -> Result<(BindingPreflightResponse, EffectiveBindingReceipt), CollaborationError> {
    validate_owner(owner_id)?;
    validate_id(workspace_id)?;
    if request.command_id.trim().is_empty() {
        return Err(CollaborationError::Invalid(
            "commandId must not be empty".to_owned(),
        ));
    }
    let document = &request.binding;
    for field in [
        "profile",
        "kind",
        "id",
        "version",
        "contentDigest",
        "provenance",
        "requiredCapabilities",
        "extensions",
        "exportClass",
        "package",
        "ownerId",
        "workspaceId",
        "runtimeInstanceId",
        "revision",
        "modelBindings",
        "skillBindings",
        "storage",
        "policyRevision",
        "effectiveLimits",
        "effectiveBudget",
        "contextGrants",
        "representationGrantRefs",
        "status",
        "effectiveBindingReceiptRef",
    ] {
        if document.get(field).is_none() {
            return Err(CollaborationError::Invalid(format!(
                "binding required field '{field}' is missing"
            )));
        }
    }
    if required_string(document, "profile")?
        != crate::uar::domain::collaboration::COLLABORATION_PROFILE
        || required_string(document, "kind")? != "DeploymentBinding"
        || required_string(document, "exportClass")? != "private-installed-state"
    {
        return Err(CollaborationError::Invalid(
            "unsupported deployment binding profile, kind, or exportClass".to_owned(),
        ));
    }
    validate_id(required_string(document, "id")?)?;
    validate_semver(required_string(document, "version")?)?;
    validate_digest(required_string(document, "contentDigest")?)?;
    validate_common_sections(document)?;
    let observed_digest = canonical_digest(document)?;
    if observed_digest != required_string(document, "contentDigest")? {
        return Err(CollaborationError::Invalid(format!(
            "binding contentDigest mismatch: observed {observed_digest}"
        )));
    }
    if required_string(document, "ownerId")? != owner_id
        || required_string(document, "workspaceId")? != workspace_id
    {
        return Err(CollaborationError::Invalid(
            "binding ownerId/workspaceId must match authenticated request scope".to_owned(),
        ));
    }
    validate_private_references(document)?;
    reject_binding_secrets(document, "$")?;
    let package =
        parse_reference(document.get("package").ok_or_else(|| {
            CollaborationError::Invalid("binding package is missing".to_owned())
        })?)?;
    if !state.packages.contains_key(&package.storage_key()) {
        return Err(CollaborationError::NotFound(format!(
            "package {} {} {}",
            package.id, package.version, package.digest
        )));
    }
    if let Some(team_definition) = bound_team_definition(state, &package)? {
        let mut diagnostics = Vec::new();
        if state.execution_claim.as_ref().is_none_or(|claim| claim.state != "held" || claim.fence != *execution_identity) {
            diagnostics.push(FieldDiagnostic { pointer: "/runtimeInstanceId".into(), disposition: ConversionDisposition::RequiredUnsupported, reason_code: "TEAM_EXECUTION_OWNER_CONFLICT".into(), message: "The catalog executing authority belongs to another service or is draining.".into(), effective_binding_ref: None, source_kind: Some(CollaborationKind::DeploymentBinding), source_definition: None });
        }
        for (index, member) in team_definition.document["members"].as_array().into_iter().flatten().enumerate() {
            if member["kind"].as_str() == Some("team") {
                diagnostics.push(FieldDiagnostic { pointer: format!("/members/{index}/kind"), disposition: ConversionDisposition::RequiredUnsupported, reason_code: "TEAM_CAPABILITY_UNSUPPORTED".into(), message: "Nested team execution is not supported by this profile.".into(), effective_binding_ref: None, source_kind: Some(team_definition.kind.clone()), source_definition: Some(team_definition.identity.clone()) });
            } else if let Ok(reference) = serde_json::from_value::<ImmutableDefinitionRef>(member["definition"].clone()) {
                if let Some(definition) = state.definitions.get(&reference.storage_key()) {
                    let start = diagnostics.len();
                    super::runtime_semantics::resolve_legacy_team_context(definition, document, &mut diagnostics);
                    let models = resolve_models(document, definition, &mut diagnostics)?;
                    super::runtime_semantics::resolve_runtime_semantics(definition, &models, provider_registry, &mut diagnostics).await;
                    assign_diagnostic_sources(&mut diagnostics[start..], definition);
                }
            }
        }
        let execution_available = service_instance.is_some_and(|authority| {
            authority
                .descriptor()
                .capabilities
                .iter()
                .any(|capability| capability == "collaboration_team_execution_v1")
        });
        if !execution_available {
            diagnostics.push(FieldDiagnostic {
                pointer: "/package/entrypoints".to_owned(),
                disposition: ConversionDisposition::RequiredUnsupported,
                reason_code: "team.execution-unavailable".to_owned(),
                message:
                    "The team can be planned, but durable team execution is unavailable in this runtime."
                        .to_owned(),
                effective_binding_ref: None, source_kind: None, source_definition: None,
            });
        }
        let service_binding =
            resolve_service_binding(service_instance, document, &mut diagnostics)?;
        let representation_grants =
            super::grants::validate_binding_grants(owner_id, workspace_id, document, state)?;
        let activation_supported = !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.disposition == ConversionDisposition::RequiredUnsupported);
        let preflight = BindingPreflightResponse {
            binding_id: required_string(document, "id")?.to_owned(),
            package: package.clone(),
            diagnostics: diagnostics
                .iter()
                .map(|item| ConversionDiagnostic {
                    field: item.pointer.clone(),
                    disposition: item.disposition.clone(),
                    message: item.message.clone(),
                })
                .collect(),
            activation_supported,
            request_digest: request_digest(request).map_err(CollaborationError::from)?,
        };
        let receipt = effective_receipt(
            document,
            package,
            team_definition,
            Vec::new(),
            Vec::new(),
            representation_grants,
            service_binding,
            json!({"teamPlanning": true, "teamExecution": activation_supported}),
            diagnostics,
            activation_supported,
        )?;
        return Ok((preflight, receipt));
    }
    let definition = bound_agent_definition(state, &package)?;
    let (resolved_skills, mut field_diagnostics) =
        resolve_skills(skill_service, document, definition).await?;
    let resolved_models = resolve_models(document, definition, &mut field_diagnostics)?;
    let effective = super::runtime_semantics::resolve_runtime_semantics(
        definition,
        &resolved_models,
        provider_registry,
        &mut field_diagnostics,
    )
    .await;
    let service_binding =
        resolve_service_binding(service_instance, document, &mut field_diagnostics)?;
    let representation_grants =
        super::grants::validate_binding_grants(owner_id, workspace_id, document, state)?;
    let activation_supported = !field_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.disposition == ConversionDisposition::RequiredUnsupported);
    let diagnostics = field_diagnostics
        .iter()
        .map(|diagnostic| ConversionDiagnostic {
            field: diagnostic.pointer.clone(),
            disposition: diagnostic.disposition.clone(),
            message: diagnostic.message.clone(),
        })
        .collect::<Vec<_>>();
    let preflight = BindingPreflightResponse {
        binding_id: required_string(document, "id")?.to_owned(),
        package: package.clone(),
        diagnostics,
        activation_supported,
        request_digest: request_digest(request).map_err(CollaborationError::from)?,
    };
    let receipt = effective_receipt(
        document,
        package,
        definition,
        resolved_skills,
        resolved_models,
        representation_grants,
        service_binding,
        effective,
        field_diagnostics,
        activation_supported,
    )?;
    Ok((preflight, receipt))
}

pub(super) fn resolve_service_binding(
    authority: Option<&crate::uar::service_instance::ServiceInstanceAuthority>,
    binding: &Value,
    diagnostics: &mut Vec<FieldDiagnostic>,
) -> Result<Option<crate::uar::service_instance::EffectiveServiceBinding>, CollaborationError> {
    let instance_id = required_string(binding, "runtimeInstanceId")?;
    let required_capabilities = binding
        .get("requiredCapabilities")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let expected_workspace_location = binding
        .get("workspaceLocation")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|_| CollaborationError::Invalid("workspaceLocation is invalid".to_owned()))?;
    let expected_endpoints = binding
        .get("endpointRoles")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|_| CollaborationError::Invalid("endpointRoles are invalid".to_owned()))?;
    let expectation = crate::uar::service_instance::ServicePlacementExpectation {
        intent: crate::uar::service_instance::PlacementIntent::New,
        expected_instance_id: instance_id.to_owned(),
        expected_profile: binding
            .get("serviceProfile")
            .and_then(Value::as_str)
            .unwrap_or(crate::uar::service_instance::SERVICE_PROFILE)
            .to_owned(),
        expected_workspace_location,
        required_capabilities,
        expected_endpoints,
        binding_id: Some(required_string(binding, "id")?.to_owned()),
        binding_revision: binding.get("revision").and_then(Value::as_u64),
        credential_ref: binding
            .get("runtimeCredentialRef")
            .and_then(Value::as_str)
            .map(str::to_owned),
    };
    let Some(authority) = authority else {
        return Ok(None);
    };
    let response = authority.evaluate(
        &expectation,
        crate::uar::service_instance::PlacementIntent::New,
    );
    for diagnostic in response.diagnostics {
        diagnostics.push(FieldDiagnostic {
            pointer: match diagnostic.field {
                "expectedInstanceId" => "/runtimeInstanceId",
                "expectedProfile" => "/serviceProfile",
                "expectedWorkspaceLocation" => "/workspaceLocation",
                "expectedEndpoints" => "/endpointRoles",
                "requiredCapabilities" => "/requiredCapabilities",
                _ => "/runtimeInstanceId",
            }
            .to_owned(),
            disposition: ConversionDisposition::RequiredUnsupported,
            reason_code: diagnostic.code.to_owned(),
            message: diagnostic.message,
            effective_binding_ref: None, source_kind: None, source_definition: None,
        });
    }
    Ok(response.effective_binding)
}

pub(super) fn bound_agent_definition<'a>(
    state: &'a CollaborationCatalogState,
    package: &ImmutableDefinitionRef,
) -> Result<&'a CollaborationDefinitionRecord, CollaborationError> {
    let package_record = state
        .packages
        .get(&package.storage_key())
        .ok_or_else(|| CollaborationError::NotFound(package.id.clone()))?;
    let entrypoints = package_record
        .manifest
        .get("entrypoints")
        .and_then(Value::as_array)
        .ok_or_else(|| CollaborationError::Invalid("package entrypoints are missing".to_owned()))?;
    let mut agents = Vec::new();
    for entrypoint in entrypoints {
        let reference = parse_reference(entrypoint)?;
        if let Some(definition) = state.definitions.get(&reference.storage_key())
            && definition.kind == CollaborationKind::AgentDefinition
        {
            agents.push(definition);
        }
    }
    if agents.len() != 1 {
        return Err(CollaborationError::Invalid(
            "ordinary binding requires exactly one AgentDefinition entrypoint".to_owned(),
        ));
    }
    Ok(agents[0])
}

fn bound_team_definition<'a>(
    state: &'a CollaborationCatalogState,
    package: &ImmutableDefinitionRef,
) -> Result<Option<&'a CollaborationDefinitionRecord>, CollaborationError> {
    let package_record = state
        .packages
        .get(&package.storage_key())
        .ok_or_else(|| CollaborationError::NotFound(package.id.clone()))?;
    let entrypoints = package_record.manifest["entrypoints"]
        .as_array()
        .ok_or_else(|| CollaborationError::Invalid("package entrypoints are missing".to_owned()))?;
    let mut teams = Vec::new();
    let mut has_agent = false;
    for entrypoint in entrypoints {
        let reference = parse_reference(entrypoint)?;
        if let Some(definition) = state.definitions.get(&reference.storage_key()) {
            match &definition.kind {
                CollaborationKind::TeamDefinition => teams.push(definition),
                CollaborationKind::AgentDefinition => has_agent = true,
                _ => {}
            }
        }
    }
    if has_agent || teams.is_empty() {
        return Ok(None);
    }
    if teams.len() != 1 {
        return Err(CollaborationError::Invalid(
            "team planning binding requires exactly one TeamDefinition entrypoint".to_owned(),
        ));
    }
    Ok(Some(teams[0]))
}

pub(super) async fn resolve_skills(
    skill_service: Option<&crate::uar::runtime::skills::SkillService>,
    binding: &Value,
    definition: &CollaborationDefinitionRecord,
) -> Result<(Vec<ResolvedSkill>, Vec<FieldDiagnostic>), CollaborationError> {
    let declared = definition
        .document
        .get("skills")
        .and_then(Value::as_array)
        .ok_or_else(|| CollaborationError::Invalid("definition skills are missing".to_owned()))?;
    let binding_skills = binding
        .get("skillBindings")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CollaborationError::Invalid("binding skillBindings are missing".to_owned())
        })?;
    let installed = match skill_service {
        Some(service) => service.get_skills().await,
        None => Vec::new(),
    };
    let mut resolved = Vec::new();
    let mut diagnostics = Vec::new();
    for (index, value) in declared.iter().enumerate() {
        let requested: SkillRef = serde_json::from_value(value.clone()).map_err(|_| {
            CollaborationError::Invalid("definition SkillRef is invalid".to_owned())
        })?;
        let pointer = format!("/skills/{index}");
        let Some(bound_value) = binding_skills.iter().find(|candidate| {
            candidate.get("id").and_then(Value::as_str) == Some(requested.id.as_str())
        }) else {
            diagnostics.push(binding_diagnostic(
                pointer,
                &requested,
                "skill.binding-missing",
            ));
            continue;
        };
        let location = required_string(bound_value, "installedLocation")?.to_owned();
        let mut bound_skill_value = bound_value.clone();
        bound_skill_value
            .as_object_mut()
            .ok_or_else(|| {
                CollaborationError::Invalid("skill binding must be an object".to_owned())
            })?
            .remove("installedLocation");
        let bound: SkillRef = serde_json::from_value(bound_skill_value)
            .map_err(|_| CollaborationError::Invalid("bound SkillRef is invalid".to_owned()))?;
        if bound != requested {
            return Err(CollaborationError::Invalid(format!(
                "skill binding at {pointer} does not exactly match the immutable definition"
            )));
        }
        let installed_skill = installed
            .iter()
            .find(|skill| skill.skill_id == requested.id);
        let exact = installed_skill.is_some_and(|skill| {
            skill.version == requested.version
                && skill.artifact_digest.as_deref() == Some(requested.digest.as_str())
                && skill.installed_location.as_deref() == Some(location.as_str())
                && skill.entrypoint == requested.entrypoint
                && skill.required_tools.iter().collect::<BTreeSet<_>>()
                    == requested.required_tools.iter().collect::<BTreeSet<_>>()
                && skill.enabled
                && !skill.tombstoned
        });
        if !exact {
            diagnostics.push(binding_diagnostic(
                pointer,
                &requested,
                "skill.installed-artifact-mismatch",
            ));
            continue;
        }
        diagnostics.push(FieldDiagnostic {
            pointer,
            disposition: ConversionDisposition::Exact,
            reason_code: "skill.bound-exactly".to_owned(),
            message: "The complete SkillRef resolves to the exact enabled installed artifact."
                .to_owned(),
            effective_binding_ref: None, source_kind: None, source_definition: None,
        });
        resolved.push(ResolvedSkill {
            skill: requested,
            installed_location: location,
        });
    }
    Ok((resolved, diagnostics))
}

pub(super) async fn revalidate_resolved_skills(
    skill_service: Option<&crate::uar::runtime::skills::SkillService>,
    receipt: &EffectiveBindingReceipt,
) -> Result<(), CollaborationError> {
    let installed = match skill_service {
        Some(service) => service.get_skills().await,
        None if receipt.resolved_skills.is_empty() => return Ok(()),
        None => {
            return Err(CollaborationError::Conflict(
                "installed skill catalog is unavailable".to_owned(),
            ));
        }
    };
    for resolved in &receipt.resolved_skills {
        let requested = &resolved.skill;
        let exact = installed.iter().any(|skill| {
            skill.skill_id == requested.id
                && skill.version == requested.version
                && skill.artifact_digest.as_deref() == Some(requested.digest.as_str())
                && skill.installed_location.as_deref() == Some(resolved.installed_location.as_str())
                && skill.entrypoint == requested.entrypoint
                && skill.required_tools.iter().collect::<BTreeSet<_>>()
                    == requested.required_tools.iter().collect::<BTreeSet<_>>()
                && skill.enabled
                && !skill.tombstoned
        });
        if !exact {
            return Err(CollaborationError::Conflict(format!(
                "resolved skill '{}' changed after binding",
                requested.id
            )));
        }
    }
    Ok(())
}

fn binding_diagnostic(pointer: String, skill: &SkillRef, reason: &str) -> FieldDiagnostic {
    FieldDiagnostic {
        pointer,
        disposition: if skill.required {
            ConversionDisposition::RequiredUnsupported
        } else {
            ConversionDisposition::OptionalUnsupported
        },
        reason_code: reason.to_owned(),
        message: if skill.required {
            "The required skill does not resolve to the exact installed artifact."
        } else {
            "The optional skill is preserved but does not resolve to an installed artifact."
        }
        .to_owned(),
        effective_binding_ref: None, source_kind: None, source_definition: None,
    }
}

pub(super) fn resolve_models(
    binding: &Value,
    definition: &CollaborationDefinitionRecord,
    diagnostics: &mut Vec<FieldDiagnostic>,
) -> Result<Vec<Value>, CollaborationError> {
    let model_bindings = binding
        .get("modelBindings")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CollaborationError::Invalid("binding modelBindings are missing".to_owned())
        })?;
    let requirements = definition
        .document
        .get("models")
        .and_then(Value::as_array)
        .ok_or_else(|| CollaborationError::Invalid("definition models are missing".to_owned()))?;
    let mut resolved = Vec::new();
    for (index, requirement) in requirements.iter().enumerate() {
        let aliases = requirement
            .get("preferredAliases")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>();
        let selected = model_bindings.iter().find(|candidate| {
            candidate
                .get("requestedAlias")
                .and_then(Value::as_str)
                .is_some_and(|alias| aliases.contains(&alias))
        });
        let Some(selected) = selected else {
            diagnostics.push(FieldDiagnostic {
                pointer: format!("/models/{index}"),
                disposition: ConversionDisposition::RequiredUnsupported,
                reason_code: "model.binding-missing".to_owned(),
                message: "No model binding resolves a preferred immutable definition alias."
                    .to_owned(),
                effective_binding_ref: None, source_kind: None, source_definition: None,
            });
            continue;
        };
        resolved.push(json!({
            "role": requirement.get("role"),
            "requestedAlias": selected.get("requestedAlias"),
            "providerId": selected.get("providerId"),
            "modelId": selected.get("modelId"),
            "profile": selected.get("profile"),
            "settingsRevision": selected.get("settingsRevision"),
        }));
        diagnostics.push(FieldDiagnostic {
            pointer: format!("/models/{index}"),
            disposition: ConversionDisposition::Exact,
            reason_code: "model.bound-exactly".to_owned(),
            message: "The model role resolves through a declared preferred alias.".to_owned(),
            effective_binding_ref: None, source_kind: None, source_definition: None,
        });
    }
    Ok(resolved)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn effective_receipt(
    binding: &Value,
    package: ImmutableDefinitionRef,
    definition: &CollaborationDefinitionRecord,
    resolved_skills: Vec<ResolvedSkill>,
    resolved_models: Vec<Value>,
    representation_grants: Vec<RepresentationGrantRef>,
    service_binding: Option<crate::uar::service_instance::EffectiveServiceBinding>,
    effective: Value,
    mut diagnostics: Vec<FieldDiagnostic>,
    admitted: bool,
) -> Result<EffectiveBindingReceipt, CollaborationError> {
    let binding_ref = PrivateRevisionRef {
        id: required_string(binding, "id")?.to_owned(),
        revision: binding
            .get("revision")
            .and_then(Value::as_u64)
            .ok_or_else(|| CollaborationError::Invalid("binding revision is missing".to_owned()))?,
        digest: required_string(binding, "contentDigest")?.to_owned(),
    };
    for diagnostic in &mut diagnostics {
        diagnostic.effective_binding_ref = Some(binding_ref.clone());
    }
    assign_diagnostic_sources(&mut diagnostics, definition);
    let requested = json!({
        "definition": definition.identity,
        "skills": definition.document.get("skills").cloned().unwrap_or_else(|| json!([])),
        "modelRequirements": definition.document.get("modelRequirements").cloned(),
        "promptDialect": definition.document.get("promptDialect").cloned(),
        "ragConfiguration": definition.document.get("ragConfiguration").cloned(),
        "contextStrategy": definition.document.get("contextStrategy").cloned(),
        "apiHarness": definition.document.get("apiHarness").cloned(),
    });
    let runtime_capabilities = service_binding
        .as_ref()
        .map(|binding| binding.capabilities.clone())
        .unwrap_or_else(|| {
            vec![
                "collaboration_definition_packages_v2".to_owned(),
                "collaboration_deployment_bindings_v2".to_owned(),
                "collaboration_conversion_reports_v1".to_owned(),
                "collaboration_representation_grant_refs_v1".to_owned(),
            ]
        });
    let mut receipt = EffectiveBindingReceipt {
        profile: COLLABORATION_PROFILE_DRAFT_2.to_owned(),
        kind: CollaborationKind::EffectiveBindingReceipt,
        export_class: "private-binding-evidence".to_owned(),
        id: format!("{}/effective", binding_ref.id),
        revision: binding_ref.revision,
        content_digest: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .to_owned(),
        binding_ref,
        package,
        requested,
        effective,
        resolved_skills,
        resolved_models,
        policy_revision: required_string(binding, "policyRevision")?.to_owned(),
        representation_grants,
        runtime_capabilities,
        service_binding,
        diagnostics,
        admitted,
        created_at: Utc::now(),
    };
    receipt.content_digest = canonical_digest(&serde_json::to_value(&receipt)?)?;
    Ok(receipt)
}

fn assign_diagnostic_sources(diagnostics: &mut [FieldDiagnostic], definition: &CollaborationDefinitionRecord) {
    for diagnostic in diagnostics {
        if diagnostic.source_kind.is_some() { continue; }
        if ["/modelBindings", "/skillBindings", "/contextGrants", "/runtimeInstanceId", "/requiredCapabilities", "/workspaceLocation", "/endpointRoles"].iter().any(|prefix| diagnostic.pointer.starts_with(prefix)) {
            diagnostic.source_kind = Some(CollaborationKind::DeploymentBinding);
        } else {
            diagnostic.source_kind = Some(definition.kind.clone());
            diagnostic.source_definition = Some(definition.identity.clone());
        }
    }
}

fn enforce_binding_revision(
    expected: Option<u64>,
    actual: Option<u64>,
) -> Result<(), CollaborationError> {
    match (expected, actual) {
        (None | Some(0), None) => Ok(()),
        (Some(expected), Some(actual)) if expected == actual => Ok(()),
        (expected, actual) => Err(CollaborationError::Conflict(format!(
            "binding expectedRevision {expected:?} does not match {actual:?}"
        ))),
    }
}

fn validate_private_references(document: &Value) -> Result<(), CollaborationError> {
    for field in ["runtimeCredentialRef", "workspaceRef"] {
        if let Some(reference) = document.get(field).and_then(Value::as_str)
            && (reference.chars().any(char::is_whitespace) || !reference.contains("://"))
        {
            return Err(CollaborationError::Invalid(format!(
                "{field} must be an opaque protected-store reference"
            )));
        }
    }
    for binding in document
        .get("modelBindings")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let reference = required_string(binding, "credentialRef")?;
        if reference.chars().any(char::is_whitespace) || !reference.contains("://") {
            return Err(CollaborationError::Invalid(
                "credentialRef must be an opaque protected-store reference".to_owned(),
            ));
        }
    }
    let storage = document
        .get("storage")
        .ok_or_else(|| CollaborationError::Invalid("binding storage is missing".to_owned()))?;
    let connection = required_string(storage, "connectionRef")?;
    if connection.chars().any(char::is_whitespace) || !connection.contains("://") {
        return Err(CollaborationError::Invalid(
            "connectionRef must be an opaque protected-store reference".to_owned(),
        ));
    }
    if storage.get("durableTransactions").and_then(Value::as_bool) != Some(true) {
        return Err(CollaborationError::Invalid(
            "binding storage must provide durableTransactions".to_owned(),
        ));
    }
    Ok(())
}

fn reject_binding_secrets(value: &Value, path: &str) -> Result<(), CollaborationError> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
                if matches!(
                    normalized.as_str(),
                    "apikey" | "password" | "secret" | "token" | "credential"
                ) {
                    return Err(CollaborationError::Invalid(format!(
                        "binding contains secret value field '{path}.{key}'; store it behind credentialRef"
                    )));
                }
                reject_binding_secrets(child, &format!("{path}.{key}"))?;
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                reject_binding_secrets(child, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_reference(value: &Value) -> Result<ImmutableDefinitionRef, CollaborationError> {
    let reference: ImmutableDefinitionRef = serde_json::from_value(value.clone())
        .map_err(|error| CollaborationError::Invalid(error.to_string()))?;
    validate_id(&reference.id)?;
    validate_semver(&reference.version)?;
    validate_digest(&reference.digest)?;
    Ok(reference)
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, CollaborationError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| CollaborationError::Invalid(format!("'{field}' must be a non-empty string")))
}

pub(super) fn binding_key(owner_id: &str, workspace_id: &str, binding_id: &str) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{binding_id}")
}

fn binding_revision_key(
    owner_id: &str,
    workspace_id: &str,
    binding_id: &str,
    revision: u64,
) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{binding_id}\u{1f}{revision}")
}
