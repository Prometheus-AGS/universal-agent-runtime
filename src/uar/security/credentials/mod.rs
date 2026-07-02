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

pub use encryption::CredentialEncryption;
pub use resolver::{CredentialResolver, ResolvedCredential};
pub use store::{
    CredentialMetadata, CredentialRecord, CredentialScope, CredentialStore,
    InMemoryCredentialStore, SharedCredentialStore, SurrealCredentialStore,
};
#[cfg(feature = "sqlx")]
pub use store::PostgresCredentialStore;

use std::sync::Arc;

/// Wiring object that owns the credential store + encryption and hands out a
/// [`CredentialResolver`]. Attached to `AppState`/`RunManager` as
/// `Option<Arc<ProviderService>>`; `None` ⇒ single-tenant (env/config only).
#[derive(Debug, Clone)]
pub struct ProviderService {
    store: Arc<dyn CredentialStore>,
    encryption: Arc<CredentialEncryption>,
}

impl ProviderService {
    /// Construct from a store and an explicit encryption handle.
    #[must_use]
    pub fn new(store: Arc<dyn CredentialStore>, encryption: Arc<CredentialEncryption>) -> Self {
        Self { store, encryption }
    }

    /// Construct from a store, reading the encryption key from
    /// `CREDENTIAL_ENCRYPTION_KEY`. Returns `Ok(None)` when the key is absent
    /// so callers can stay single-tenant without configuration.
    pub fn from_env(store: Arc<dyn CredentialStore>) -> anyhow::Result<Option<Self>> {
        match CredentialEncryption::from_env()? {
            None => Ok(None),
            Some(enc) => Ok(Some(Self::new(store, Arc::new(enc)))),
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
}
