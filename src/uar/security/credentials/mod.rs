//! Multi-tenant provider credential subsystem.
//!
//! Provides encryption-at-rest ([`encryption`]) for per-user provider API
//! keys. Storage ([`store`]), scoped resolution ([`resolver`]), and the
//! wiring [`ProviderService`] are layered on top in subsequent steps.
//!
//! Single-tenant deployments do not need any of this: when no
//! [`ProviderService`] is configured, credential resolution is skipped and the
//! existing env/config key is used unchanged.

pub mod encryption;
pub mod resolver;
pub mod store;
pub mod principal_store;

pub use encryption::CredentialEncryption;
pub use resolver::{CredentialResolver, ResolvedCredential};
pub use principal_store::{CredentialMigrationError, CredentialMigrationReceipt, LegacyCredentialOwnerMapping, PrincipalCredentialStore};
#[cfg(feature = "sqlx")]
pub use store::PostgresCredentialStore;
#[cfg(feature = "surreal-backend")]
pub use store::SurrealCredentialStore;
pub use store::{
    CredentialMetadata, CredentialRecord, CredentialScope, CredentialStore,
    InMemoryCredentialStore, SharedCredentialStore,
};

use std::sync::Arc;

/// Wiring object that owns the credential store + encryption and hands out a
/// [`CredentialResolver`]. Attached to `AppState`/`RunManager` as
/// `Option<Arc<ProviderService>>`; `None` ⇒ single-tenant (env/config only).
#[derive(Debug, Clone)]
pub struct ProviderService {
    store: Arc<dyn CredentialStore>,
    encryption: Arc<CredentialEncryption>,
    legacy_owner_mappings: Arc<Vec<LegacyCredentialOwnerMapping>>,
}

impl ProviderService {
    /// Construct from a store and an explicit encryption handle.
    #[must_use]
    pub fn new(store: Arc<dyn CredentialStore>, encryption: Arc<CredentialEncryption>) -> Self {
        Self { store, encryption, legacy_owner_mappings: Arc::new(Vec::new()) }
    }

    /// Construct from a store, reading the encryption key from
    /// `CREDENTIAL_ENCRYPTION_KEY`. Returns `Ok(None)` when the key is absent
    /// so callers can stay single-tenant without configuration.
    pub fn from_env(store: Arc<dyn CredentialStore>) -> anyhow::Result<Option<Self>> {
        match CredentialEncryption::from_env()? {
            None => Ok(None),
            Some(enc) => {
                let mut service = Self::new(store, Arc::new(enc));
                if let Some(path) = std::env::var_os("CREDENTIAL_OWNER_MAPPING_FILE") {
                    let contents = std::fs::read(path)
                        .map_err(|_| anyhow::anyhow!("cannot read operator credential owner mappings"))?;
                    let mappings = serde_json::from_slice(&contents)
                        .map_err(|_| anyhow::anyhow!("invalid operator credential owner mappings"))?;
                    service = service.with_legacy_owner_mappings(mappings)?;
                }
                Ok(Some(service))
            }
        }
    }

    #[must_use]
    pub fn resolver(&self) -> CredentialResolver {
        CredentialResolver::new(Arc::clone(&self.store), Arc::clone(&self.encryption))
    }

    #[must_use]
    pub fn store(&self) -> &Arc<dyn CredentialStore> {
        &self.store
    }

    #[must_use]
    pub fn encryption(&self) -> &Arc<CredentialEncryption> {
        &self.encryption
    }

    /// Install explicit owner mappings reviewed by the application operator.
    /// This is a trusted host composition method, never an HTTP operation.
    ///
    /// # Errors
    /// Rejects empty approvals, system mappings and conflicting source mappings.
    pub fn with_legacy_owner_mappings(mut self, mappings: Vec<LegacyCredentialOwnerMapping>) -> anyhow::Result<Self> {
        let mut sources = std::collections::HashSet::new();
        for mapping in &mappings {
            if mapping.scope == CredentialScope::System || mapping.approval_id.trim().is_empty()
                || mapping.destination_owner_key.trim().is_empty()
                || !sources.insert((mapping.scope, &mapping.legacy_scope_id, &mapping.provider_id)) {
                anyhow::bail!("invalid or conflicting operator credential owner mappings");
            }
        }
        self.legacy_owner_mappings = Arc::new(mappings);
        Ok(self)
    }

    /// Apply one exact operator-approved mapping to the authenticated caller.
    /// Request fields cannot choose a destination identity or approval.
    ///
    /// # Errors
    /// Denies absent mappings, conflicts, missing source rows and storage failures.
    pub async fn migrate_legacy_credential(
        &self,
        principal: &crate::uar::security::claims::CredentialPrincipal,
        scope: CredentialScope,
        legacy_scope_id: &str,
        provider_id: &str,
        reason: &str,
    ) -> anyhow::Result<CredentialMigrationReceipt> {
        let owner_key = principal.storage_key();
        let mapping = self.legacy_owner_mappings.iter().find(|mapping| {
            mapping.scope == scope && mapping.legacy_scope_id == legacy_scope_id
                && mapping.provider_id == provider_id && mapping.destination_owner_key == owner_key
        }).ok_or(CredentialMigrationError::MappingDenied)?;
        if reason.trim().is_empty() {
            return Err(CredentialMigrationError::MappingDenied.into());
        }
        let authorization = principal_store::CredentialMigrationAuthorization {
            mapping: mapping.clone(),
            reason: reason.to_owned(),
        };
        self.store.migrate_legacy(principal, &authorization).await
    }
}
