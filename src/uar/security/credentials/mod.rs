//! Multi-tenant provider credential subsystem.
//!
//! Provides encryption-at-rest ([`encryption`]) for per-user provider API
//! keys. Storage ([`store`]), scoped resolution ([`resolver`]), and the
//! wiring [`ProviderService`] are layered on top in subsequent steps.
//!
//! Host source policy is independent of encrypted storage availability.
//! Every ordinary provider capture must consume a policy decision.

pub mod policy;
pub mod owner;
pub mod migration;
pub mod maintenance;
pub mod encryption;
pub use policy::{CredentialTenancyPolicy, ProviderCredentialPolicyConfig, CredentialSourceDecision, CredentialSource, CredentialPolicyError};
pub use owner::CredentialOwnerBinding;
pub use migration::PrincipalCredentialOwnerMapping;
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
/// `Option<Arc<ProviderService>>`; source policy remains mandatory when absent.
#[derive(Debug, Clone)]
pub struct ProviderService {
    store: Arc<dyn CredentialStore>,
    encryption: Arc<CredentialEncryption>,
    pub(super) legacy_owner_mappings: Arc<Vec<LegacyCredentialOwnerMapping>>,
    pub(super) principal_owner_mappings: Arc<Vec<PrincipalCredentialOwnerMapping>>,
    policy: Arc<CredentialTenancyPolicy>,
    /// Per-read store bound handed to every resolver this service builds,
    /// sourced from `resilience.admission_timeout_ms`.
    store_timeout: std::time::Duration,
}

impl ProviderService {
    /// Construct from a store and an explicit encryption handle.
    #[must_use]
    pub fn new(store: Arc<dyn CredentialStore>, encryption: Arc<CredentialEncryption>) -> Self {
        Self { store, encryption, legacy_owner_mappings: Arc::new(Vec::new()), principal_owner_mappings: Arc::new(Vec::new()), policy: Arc::new(CredentialTenancyPolicy::default()), store_timeout: std::time::Duration::from_millis(10_000) }
    }

    /// Bound each credential-store read performed during run admission.
    #[must_use]
    pub fn with_store_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.store_timeout = timeout.max(std::time::Duration::from_millis(500));
        self
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
            .with_store_timeout(self.store_timeout)
    }

    #[must_use]
    pub fn store(&self) -> &Arc<dyn CredentialStore> {
        &self.store
    }

    #[must_use]
    pub fn encryption(&self) -> &Arc<CredentialEncryption> {
        &self.encryption
    }

    pub fn policy(&self) -> &Arc<CredentialTenancyPolicy> { &self.policy }

    /// Install the independently validated host policy and fixed-local mappings.
    pub fn with_policy(mut self, policy: Arc<CredentialTenancyPolicy>) -> anyhow::Result<Self> {
        self.principal_owner_mappings = Arc::new(policy.principal_mappings().to_vec());
        self.policy = policy;
        Ok(self)
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
                || !owner::is_bound_owner(&mapping.destination_owner_key)
                || mapping.legacy_scope_id.trim().is_empty() || mapping.provider_id.trim().is_empty()
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
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        legacy_scope_id: &str,
        provider_id: &str,
        reason: &str,
    ) -> anyhow::Result<CredentialMigrationReceipt> {
        self.migrate_legacy_conditioned(principal, scope, legacy_scope_id, provider_id, reason, None).await
    }

    pub async fn migrate_legacy_conditioned(&self, principal: &CredentialOwnerBinding, scope: CredentialScope, legacy_scope_id: &str, provider_id: &str, reason: &str, if_match: Option<&str>) -> anyhow::Result<CredentialMigrationReceipt> {
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
            source_owner_key: None, required_revision: None, if_match: migration::validate_if_match(if_match)?,
        };
        self.store.migrate_legacy(principal, &authorization).await
    }
    pub async fn migrate_principal_credential(&self, principal: &CredentialOwnerBinding, source_owner_key: &str, scope: CredentialScope, source_scope_id: &str, provider_id: &str, reason: &str, if_match: Option<&str>) -> anyhow::Result<CredentialMigrationReceipt> {
        if reason != "operator_approved_local_namespace_migration" { return Err(CredentialMigrationError::MappingDenied.into()); }
        let mapping = self.principal_owner_mappings.iter().find(|mapping| mapping.source_owner_key == source_owner_key && mapping.scope == scope && mapping.source_scope_id == source_scope_id && mapping.provider_id == provider_id && mapping.destination_owner_key == principal.storage_key()).ok_or(CredentialMigrationError::MappingDenied)?;
        self.store.migrate_legacy(principal, &principal_store::CredentialMigrationAuthorization {
            mapping: LegacyCredentialOwnerMapping { scope, legacy_scope_id: source_scope_id.to_owned(), provider_id: provider_id.to_owned(), destination_owner_key: mapping.destination_owner_key.clone(), approval_id: mapping.approval_id.clone() },
            reason: reason.to_owned(), source_owner_key: Some(source_owner_key.to_owned()), required_revision: Some(mapping.source_revision.clone()), if_match: migration::validate_if_match(if_match)?,
        }).await
    }

}
