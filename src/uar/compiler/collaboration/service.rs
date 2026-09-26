use std::sync::Arc;

use chrono::Utc;
use thiserror::Error;

use crate::uar::domain::collaboration::{
    CollaborationCatalogState, CollaborationCommandReceipt, CollaborationDefinitionRecord,
    CollaborationPackageRecord, ImmutableDefinitionRef, PackageInstallResponse,
    PackagePreflightResponse, PackageSourceRequest,
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

impl From<anyhow::Error> for CollaborationError {
    fn from(error: anyhow::Error) -> Self {
        Self::Invalid(error.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct CollaborationCatalogService {
    pub(super) storage: Arc<dyn CollaborationStorage>,
}

impl CollaborationCatalogService {
    #[must_use]
    pub fn new(storage: Arc<dyn CollaborationStorage>) -> Self {
        Self { storage }
    }

    #[must_use]
    pub fn in_memory() -> Self {
        Self::new(Arc::new(InMemoryCollaborationStorage::new()))
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

    pub(super) async fn load_state(
        &self,
    ) -> Result<CollaborationCatalogState, CollaborationError> {
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

fn enforce_catalog_revision(
    expected: Option<u64>,
    actual: u64,
) -> Result<(), CollaborationError> {
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
