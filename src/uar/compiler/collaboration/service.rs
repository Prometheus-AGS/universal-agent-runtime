use std::sync::Arc;

use chrono::Utc;
use thiserror::Error;

use crate::uar::domain::artifact::AgentArtifact;
use crate::uar::domain::collaboration::{
    CollaborationCatalogState, CollaborationCommandReceipt, CollaborationDefinitionRecord,
    CollaborationPackageRecord, DeploymentBindingRecord, EffectiveBindingReceipt,
    ImmutableDefinitionRef, PackageInstallResponse, PackagePreflightResponse, PackageSourceRequest,
};

use super::storage::{CollaborationStorage, InMemoryCollaborationStorage};
use super::validation::{
    preflight_response, prepare_package, validate_digest, validate_id, validate_semver,
};

pub(super) const MAX_CAS_ATTEMPTS: usize = 8;

#[derive(Debug, Error)]
pub enum CollaborationError {
    #[error("invalid collaboration document: {0}")]
    Invalid(String),
    #[error("collaboration resource not found: {0}")]
    NotFound(String),
    #[error("collaboration revision conflict: {0}")]
    Conflict(String),
    #[error("collaboration persistence unavailable: {0}")]
    Storage(String),
}

#[derive(Debug, Clone)]
pub struct BoundAgentRun {
    pub artifact: AgentArtifact,
    pub binding: DeploymentBindingRecord,
    pub effective_binding_receipt: EffectiveBindingReceipt,
}

impl From<anyhow::Error> for CollaborationError {
    fn from(error: anyhow::Error) -> Self {
        Self::Invalid(error.to_string())
    }
}

impl From<serde_json::Error> for CollaborationError {
    fn from(error: serde_json::Error) -> Self {
        Self::Invalid(error.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct CollaborationCatalogService {
    pub(super) storage: Arc<dyn CollaborationStorage>,
    pub(crate) team_execution_notify: Arc<tokio::sync::Notify>,
    pub(super) execution_identity: Arc<std::sync::RwLock<crate::uar::domain::team_execution::TeamExecutionFence>>,
    pub(super) skill_service: Option<Arc<crate::uar::runtime::skills::SkillService>>,
    pub(super) provider_registry: Option<Arc<crate::llm::ProviderRegistry>>,
    pub(super) service_instance:
        Option<Arc<crate::uar::service_instance::ServiceInstanceAuthority>>,
}

impl CollaborationCatalogService {
    #[must_use]
    pub fn new(storage: Arc<dyn CollaborationStorage>) -> Self {
        Self {
            storage,
            team_execution_notify: Arc::new(tokio::sync::Notify::new()),
            execution_identity: Arc::new(std::sync::RwLock::new(crate::uar::domain::team_execution::TeamExecutionFence { catalog_id: "uar-collaboration-catalog".into(), service_instance_id: "unconfigured".into(), incarnation_id: uuid::Uuid::new_v4().to_string(), epoch: 0 })),
            skill_service: None,
            provider_registry: None,
            service_instance: None,
        }
    }

    #[must_use]
    pub fn in_memory() -> Self {
        Self::new(Arc::new(InMemoryCollaborationStorage::new()))
    }

    #[must_use]
    pub fn with_skill_service(
        mut self,
        skill_service: Arc<crate::uar::runtime::skills::SkillService>,
    ) -> Self {
        self.skill_service = Some(skill_service);
        self
    }

    #[must_use]
    pub fn with_provider_registry(
        mut self,
        provider_registry: Arc<crate::llm::ProviderRegistry>,
    ) -> Self {
        self.provider_registry = Some(provider_registry);
        self
    }

    #[must_use]
    pub fn with_service_instance(
        mut self,
        service_instance: Arc<crate::uar::service_instance::ServiceInstanceAuthority>,
    ) -> Self {
        if let Ok(mut identity) = self.execution_identity.write() {
            identity.service_instance_id = service_instance.descriptor().instance.id.clone();
        }
        self.service_instance = Some(service_instance);
        self
    }

    pub async fn resolve_bound_agent_run(
        &self,
        owner_id: &str,
        workspace_id: &str,
        binding_id: &str,
    ) -> Result<BoundAgentRun, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        validate_id(binding_id)?;
        let state = self.load_state().await?;
        let binding = state
            .bindings
            .get(&super::bindings::binding_key(
                owner_id,
                workspace_id,
                binding_id,
            ))
            .cloned()
            .ok_or_else(|| CollaborationError::NotFound(binding_id.to_owned()))?;
        let receipt = binding.effective_binding_receipt.clone().ok_or_else(|| {
            CollaborationError::Conflict("binding has no effective binding receipt".to_owned())
        })?;
        self.revalidate_effective_binding(owner_id, workspace_id, &receipt)
            .await?;
        let definition = super::bindings::bound_agent_definition(&state, &binding.package)?;
        let artifact = definition.compatibility_agent.clone().ok_or_else(|| {
            CollaborationError::Invalid(
                "bound definition has no ordinary-agent projection".to_owned(),
            )
        })?;
        Ok(BoundAgentRun {
            artifact,
            binding,
            effective_binding_receipt: receipt,
        })
    }

    pub async fn revalidate_effective_binding(
        &self,
        owner_id: &str,
        workspace_id: &str,
        receipt: &EffectiveBindingReceipt,
    ) -> Result<(), CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        if !receipt.admitted {
            return Err(CollaborationError::Conflict(
                "effective binding receipt does not admit activation".to_owned(),
            ));
        }
        if let Some(authority) = &self.service_instance {
            let service_binding = receipt.service_binding.as_ref().ok_or_else(|| {
                CollaborationError::Conflict(
                    "effective binding receipt has no live service binding".to_owned(),
                )
            })?;
            authority.revalidate(service_binding).map_err(|response| {
                CollaborationError::Conflict(
                    response
                        .diagnostics
                        .into_iter()
                        .map(|diagnostic| diagnostic.message)
                        .collect::<Vec<_>>()
                        .join("; "),
                )
            })?;
        }
        let state = self.load_state().await?;
        let binding = state
            .bindings
            .get(&super::bindings::binding_key(
                owner_id,
                workspace_id,
                &receipt.binding_ref.id,
            ))
            .ok_or_else(|| CollaborationError::NotFound(receipt.binding_ref.id.clone()))?;
        if binding.revision != receipt.binding_ref.revision
            || binding
                .document
                .get("contentDigest")
                .and_then(serde_json::Value::as_str)
                != Some(receipt.binding_ref.digest.as_str())
            || binding
                .document
                .get("policyRevision")
                .and_then(serde_json::Value::as_str)
                != Some(receipt.policy_revision.as_str())
        {
            return Err(CollaborationError::Conflict(
                "binding or policy revision changed after effective resolution".to_owned(),
            ));
        }
        let stored_receipt = binding
            .effective_binding_receipt
            .as_ref()
            .filter(|stored| stored.content_digest == receipt.content_digest)
            .and_then(|stored| state.effective_binding_receipts.get(&stored.content_digest))
            .ok_or_else(|| {
                CollaborationError::Conflict(
                    "effective binding receipt is not the current persisted receipt".to_owned(),
                )
            })?;
        if super::validation::canonical_digest(&serde_json::to_value(stored_receipt)?)?
            != stored_receipt.content_digest
        {
            return Err(CollaborationError::Storage(
                "persisted effective binding receipt digest is invalid".to_owned(),
            ));
        }
        let current_grants = super::grants::validate_binding_grants(
            owner_id,
            workspace_id,
            &binding.document,
            &state,
        )?;
        if current_grants != receipt.representation_grants {
            return Err(CollaborationError::Conflict(
                "representation grant set changed after effective resolution".to_owned(),
            ));
        }
        super::bindings::revalidate_resolved_skills(self.skill_service.as_deref(), receipt).await?;
        Ok(())
    }

    pub async fn get_effective_binding_receipt(
        &self,
        owner_id: &str,
        workspace_id: &str,
        binding_id: &str,
    ) -> Result<EffectiveBindingReceipt, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        validate_id(binding_id)?;
        self.load_state()
            .await?
            .bindings
            .get(&super::bindings::binding_key(
                owner_id,
                workspace_id,
                binding_id,
            ))
            .and_then(|binding| binding.effective_binding_receipt.clone())
            .ok_or_else(|| CollaborationError::NotFound(binding_id.to_owned()))
    }

    pub async fn preflight_package(
        &self,
        request: &PackageSourceRequest,
    ) -> Result<PackagePreflightResponse, CollaborationError> {
        let prepared = prepare_package(request)?;
        let current = self.load_state().await?;
        enforce_catalog_revision(request.expected_catalog_revision, current.catalog_revision)?;
        Ok(preflight_response(&prepared))
    }

    pub async fn install_package(
        &self,
        owner_id: &str,
        request: PackageSourceRequest,
    ) -> Result<PackageInstallResponse, CollaborationError> {
        validate_owner(owner_id)?;
        let prepared = prepare_package(&request)?;
        let preflight = preflight_response(&prepared);
        let package_identity = prepared.manifest.definition_ref();
        let package_key = package_identity.storage_key();
        let receipt_key = receipt_key(owner_id, &request.command_id);

        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.command_receipts.get(&receipt_key) {
                if receipt.operation != "install-package"
                    || receipt.request_digest != prepared.request_digest
                {
                    return Err(CollaborationError::Conflict(format!(
                        "commandId '{}' was already used for a different operation or content",
                        request.command_id
                    )));
                }
                return Ok(PackageInstallResponse {
                    preflight,
                    receipt: receipt.clone(),
                });
            }
            enforce_catalog_revision(request.expected_catalog_revision, current.catalog_revision)?;
            reject_immutable_conflicts(&current, &package_identity, &prepared.definitions)?;

            let mut next = current.clone();
            next.generation = current.generation.saturating_add(1);
            let changes_catalog = !current.packages.contains_key(&package_key)
                || prepared.definitions.iter().any(|definition| {
                    !current
                        .definitions
                        .contains_key(&definition.identity.storage_key())
                });
            if changes_catalog {
                next.catalog_revision = current.catalog_revision.saturating_add(1);
            }
            for definition in &prepared.definitions {
                next.definitions
                    .entry(definition.identity.storage_key())
                    .or_insert_with(|| definition.clone());
            }
            next.packages.entry(package_key.clone()).or_insert_with(|| {
                CollaborationPackageRecord {
                    identity: package_identity.clone(),
                    manifest: prepared.manifest_document.clone(),
                    source_json: prepared.manifest_json.clone(),
                    definition_keys: prepared
                        .definitions
                        .iter()
                        .map(|record| record.identity.storage_key())
                        .collect(),
                    installed_at: Utc::now(),
                }
            });
            let receipt = CollaborationCommandReceipt {
                command_id: request.command_id.clone(),
                owner_id: owner_id.to_owned(),
                request_digest: prepared.request_digest.clone(),
                operation: "install-package".to_owned(),
                resource_id: package_identity.id.clone(),
                catalog_revision: next.catalog_revision,
                binding_revision: None,
                committed_at: Utc::now(),
            };
            next.command_receipts
                .insert(receipt_key.clone(), receipt.clone());
            if self.cas(current.generation, &next).await? {
                return Ok(PackageInstallResponse { preflight, receipt });
            }
        }
        Err(CollaborationError::Conflict(
            "catalog changed repeatedly while installing the package".to_owned(),
        ))
    }

    pub async fn list_packages(
        &self,
    ) -> Result<Vec<CollaborationPackageRecord>, CollaborationError> {
        Ok(self.load_state().await?.packages.into_values().collect())
    }

    pub async fn get_package(
        &self,
        digest: &str,
    ) -> Result<CollaborationPackageRecord, CollaborationError> {
        validate_digest(digest)?;
        self.load_state()
            .await?
            .packages
            .into_values()
            .find(|record| record.identity.digest == digest)
            .ok_or_else(|| CollaborationError::NotFound(digest.to_owned()))
    }

    pub async fn get_package_version(
        &self,
        id: &str,
        version: &str,
    ) -> Result<CollaborationPackageRecord, CollaborationError> {
        validate_id(id)?;
        validate_semver(version)?;
        self.load_state()
            .await?
            .packages
            .into_values()
            .find(|record| record.identity.id == id && record.identity.version == version)
            .ok_or_else(|| CollaborationError::NotFound(format!("{id}@{version}")))
    }

    pub async fn list_definitions(
        &self,
    ) -> Result<Vec<CollaborationDefinitionRecord>, CollaborationError> {
        Ok(self.load_state().await?.definitions.into_values().collect())
    }

    pub async fn get_definition(
        &self,
        digest: &str,
    ) -> Result<CollaborationDefinitionRecord, CollaborationError> {
        validate_digest(digest)?;
        self.load_state()
            .await?
            .definitions
            .into_values()
            .find(|record| record.identity.digest == digest)
            .ok_or_else(|| CollaborationError::NotFound(digest.to_owned()))
    }

    pub(super) async fn load_state(&self) -> Result<CollaborationCatalogState, CollaborationError> {
        self.storage
            .load_state()
            .await
            .map_err(|error| CollaborationError::Storage(error.to_string()))
    }

    pub(super) async fn cas(
        &self,
        generation: u64,
        next: &CollaborationCatalogState,
    ) -> Result<bool, CollaborationError> {
        self.storage
            .compare_and_swap(generation, next)
            .await
            .map_err(|error| CollaborationError::Storage(error.to_string()))
    }
}

fn reject_immutable_conflicts(
    state: &CollaborationCatalogState,
    package: &ImmutableDefinitionRef,
    definitions: &[CollaborationDefinitionRecord],
) -> Result<(), CollaborationError> {
    for existing in state.packages.values() {
        if existing.identity.id == package.id
            && existing.identity.version == package.version
            && existing.identity.digest != package.digest
        {
            return Err(CollaborationError::Conflict(format!(
                "package {} {} already exists with digest {}",
                package.id, package.version, existing.identity.digest
            )));
        }
    }
    for incoming in definitions {
        for existing in state.definitions.values() {
            if existing.identity.id == incoming.identity.id
                && existing.identity.version == incoming.identity.version
                && existing.identity.digest != incoming.identity.digest
            {
                return Err(CollaborationError::Conflict(format!(
                    "definition {} {} already exists with digest {}",
                    incoming.identity.id, incoming.identity.version, existing.identity.digest
                )));
            }
        }
    }
    Ok(())
}

fn enforce_catalog_revision(expected: Option<u64>, actual: u64) -> Result<(), CollaborationError> {
    if let Some(expected) = expected {
        if expected != actual {
            return Err(CollaborationError::Conflict(format!(
                "package expectedCatalogRevision {expected} does not match {actual}"
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_owner(owner_id: &str) -> Result<(), CollaborationError> {
    if owner_id.trim().is_empty() || owner_id == "anonymous" {
        return Err(CollaborationError::Invalid(
            "authenticated owner context is required".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn receipt_key(owner_id: &str, command_id: &str) -> String {
    format!("{owner_id}\u{1f}{command_id}")
}
