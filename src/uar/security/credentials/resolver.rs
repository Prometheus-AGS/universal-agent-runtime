//! Scoped credential resolution.
//!
//! Resolves a provider API key by trying scopes in strict priority order:
//! `Session → Agent → User → System`. Returning `None` means "no stored
//! credential" — the caller keeps its existing env/config key, which is the
//! single-tenant (env) terminal step of the chain.
//!
//! The resolved plaintext key is held in a [`secrecy::SecretString`] so it is
//! never accidentally logged or serialized.

use std::sync::Arc;

use secrecy::SecretString;

use super::encryption::CredentialEncryption;
use super::store::{CredentialScope, CredentialStore};

/// A successfully resolved credential. The plaintext key is wrapped in
/// [`SecretString`]; call [`secrecy::ExposeSecret::expose_secret`] only at the
/// point of use (constructing the upstream client).
#[derive(Debug, Clone)]
pub struct ResolvedCredential {
    pub provider_id: String,
    pub scope: CredentialScope,
    pub api_key: SecretString,
}

/// Resolves credentials across scopes, decrypting on demand.
#[derive(Debug, Clone)]
pub struct CredentialResolver {
    store: Arc<dyn CredentialStore>,
    encryption: Arc<CredentialEncryption>,
}

impl CredentialResolver {
    #[must_use]
    pub fn new(store: Arc<dyn CredentialStore>, encryption: Arc<CredentialEncryption>) -> Self {
        Self { store, encryption }
    }

    /// Resolve the credential for `(user_id, provider_id)` with no session/agent context.
    pub async fn resolve(
        &self,
        user_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<ResolvedCredential>> {
        self.resolve_with_context(user_id, provider_id, None, None)
            .await
    }

    /// Resolve with the full scoped chain: `session → agent → user → system`.
    ///
    /// Returns the first scope that has a credential for `provider_id`, or
    /// `None` if every scope misses (caller falls through to env/config).
    pub async fn resolve_with_context(
        &self,
        user_id: &str,
        provider_id: &str,
        session_id: Option<&str>,
        agent_id: Option<&str>,
    ) -> anyhow::Result<Option<ResolvedCredential>> {
        // Ordered (scope, scope_id) lookups. First hit wins.
        let mut candidates: Vec<(CredentialScope, &str)> = Vec::with_capacity(4);
        if let Some(sid) = session_id {
            candidates.push((CredentialScope::Session, sid));
        }
        if let Some(aid) = agent_id {
            candidates.push((CredentialScope::Agent, aid));
        }
        candidates.push((CredentialScope::User, user_id));
        candidates.push((CredentialScope::System, "system"));

        for (scope, scope_id) in candidates {
            if let Some(row) = self.store.get(scope, scope_id, provider_id).await? {
                // Decrypt only the matched row. The original crypto error is
                // intentionally discarded (`_decrypt_err`) so no ciphertext or
                // key material can leak through the error chain; we surface only
                // provider/scope.
                let plaintext =
                    self.encryption.decrypt(&row.api_key_encrypted).map_err(|_decrypt_err| {
                        anyhow::anyhow!(
                            "failed to decrypt {} credential for provider '{}'",
                            scope.as_str(),
                            provider_id
                        )
                    })?;
                return Ok(Some(ResolvedCredential {
                    provider_id: provider_id.to_string(),
                    scope,
                    api_key: SecretString::from(plaintext),
                }));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::security::credentials::store::{
        CredentialRecord, InMemoryCredentialStore,
    };
    use chrono::Utc;
    use secrecy::ExposeSecret;

    fn enc() -> Arc<CredentialEncryption> {
        Arc::new(CredentialEncryption::from_key(&[b'k'; 32]))
    }

    async fn store_with(
        encryption: &CredentialEncryption,
        entries: &[(CredentialScope, &str, &str, &str)],
    ) -> Arc<InMemoryCredentialStore> {
        let store = Arc::new(InMemoryCredentialStore::new());
        for (scope, scope_id, provider, plaintext) in entries {
            let now = Utc::now();
            store
                .put(CredentialRecord {
                    scope: *scope,
                    scope_id: (*scope_id).to_string(),
                    provider_id: (*provider).to_string(),
                    api_key_encrypted: encryption.encrypt(plaintext).unwrap(),
                    api_key_hint: CredentialEncryption::key_hint(plaintext, 4),
                    created_at: now,
                    updated_at: now,
                })
                .await
                .unwrap();
        }
        store
    }

    #[tokio::test]
    async fn highest_scope_wins() {
        let e = enc();
        let store = store_with(
            &e,
            &[
                (CredentialScope::Session, "sess1", "openai", "session-key"),
                (CredentialScope::User, "userA", "openai", "user-key"),
            ],
        )
        .await;
        let r = CredentialResolver::new(store, e);
        let got = r
            .resolve_with_context("userA", "openai", Some("sess1"), None)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.scope, CredentialScope::Session);
        assert_eq!(got.api_key.expose_secret(), "session-key");
    }

    #[tokio::test]
    async fn falls_through_to_user() {
        let e = enc();
        let store = store_with(&e, &[(CredentialScope::User, "userA", "openai", "user-key")]).await;
        let r = CredentialResolver::new(store, e);
        let got = r
            .resolve_with_context("userA", "openai", Some("sess1"), Some("agent1"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.scope, CredentialScope::User);
        assert_eq!(got.api_key.expose_secret(), "user-key");
    }

    #[tokio::test]
    async fn all_miss_returns_none() {
        let e = enc();
        let store = store_with(&e, &[]).await;
        let r = CredentialResolver::new(store, e);
        let got = r.resolve_with_context("userA", "openai", None, None).await.unwrap();
        assert!(got.is_none(), "no stored credential => None (env fallback)");
    }

    #[tokio::test]
    async fn provider_specific() {
        let e = enc();
        let store = store_with(&e, &[(CredentialScope::User, "userA", "openai", "user-key")]).await;
        let r = CredentialResolver::new(store, e);
        let got = r.resolve_with_context("userA", "anthropic", None, None).await.unwrap();
        assert!(got.is_none(), "must not return another provider's key");
    }
}
