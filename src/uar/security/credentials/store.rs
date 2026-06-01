//! Storage for per-scope encrypted provider credentials.
//!
//! Mirrors the `api_keys` storage pattern: an `#[async_trait]` [`CredentialStore`]
//! trait, an always-available [`InMemoryCredentialStore`], and a SurrealDB-backed
//! [`SurrealCredentialStore`] that compiles in the default (surreal) build.
//!
//! Stored API keys are AES-256-GCM ciphertext (`api_key_encrypted`). Plaintext is
//! never persisted and never returned by the store.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// The scope a credential is bound to. Resolution tries scopes in priority
/// order: `Session → Agent → User → System` (see [`super::resolver`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialScope {
    /// Bound to a single chat/run session.
    Session,
    /// Bound to a named agent.
    Agent,
    /// Bound to an authenticated end user.
    User,
    /// Operator/platform-wide ("house account").
    System,
}

impl CredentialScope {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Agent => "agent",
            Self::User => "user",
            Self::System => "system",
        }
    }
}

/// A stored credential row. `api_key_encrypted` is AES-256-GCM ciphertext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRecord {
    pub scope: CredentialScope,
    /// Scope identifier: user id, agent id, session id, or `"system"`.
    pub scope_id: String,
    pub provider_id: String,
    /// AES-256-GCM encrypted key — never exposed via API.
    pub api_key_encrypted: String,
    /// Last-4 of the plaintext key, safe to display.
    pub api_key_hint: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Safe, plaintext-free view of a credential for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    pub scope: CredentialScope,
    pub scope_id: String,
    pub provider_id: String,
    pub api_key_hint: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&CredentialRecord> for CredentialMetadata {
    fn from(r: &CredentialRecord) -> Self {
        Self {
            scope: r.scope,
            scope_id: r.scope_id.clone(),
            provider_id: r.provider_id.clone(),
            api_key_hint: r.api_key_hint.clone(),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// Backend-agnostic credential storage.
#[async_trait]
pub trait CredentialStore: Send + Sync + std::fmt::Debug {
    /// Insert or replace the credential for `(scope, scope_id, provider_id)`.
    async fn put(&self, record: CredentialRecord) -> anyhow::Result<()>;

    /// Fetch the credential for an exact `(scope, scope_id, provider_id)` key.
    async fn get(
        &self,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>>;

    /// List all credentials at a `(scope, scope_id)` (e.g. all of a user's keys).
    async fn list(
        &self,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>>;

    /// Delete the credential for `(scope, scope_id, provider_id)`.
    /// Returns `true` if a row was removed.
    async fn delete(
        &self,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool>;
}

fn key_of(scope: CredentialScope, scope_id: &str, provider_id: &str) -> (String, String, String) {
    (
        scope.as_str().to_string(),
        scope_id.to_string(),
        provider_id.to_string(),
    )
}

/// In-memory credential store (used as the wired default and in tests),
/// matching the `InMemoryApiKeyStorage` precedent.
#[derive(Debug, Default)]
pub struct InMemoryCredentialStore {
    rows: RwLock<HashMap<(String, String, String), CredentialRecord>>,
}

impl InMemoryCredentialStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CredentialStore for InMemoryCredentialStore {
    async fn put(&self, record: CredentialRecord) -> anyhow::Result<()> {
        let k = key_of(record.scope, &record.scope_id, &record.provider_id);
        self.rows.write().await.insert(k, record);
        Ok(())
    }

    async fn get(
        &self,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>> {
        let k = key_of(scope, scope_id, provider_id);
        Ok(self.rows.read().await.get(&k).cloned())
    }

    async fn list(
        &self,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>> {
        Ok(self
            .rows
            .read()
            .await
            .values()
            .filter(|r| r.scope == scope && r.scope_id == scope_id)
            .cloned()
            .collect())
    }

    async fn delete(
        &self,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool> {
        let k = key_of(scope, scope_id, provider_id);
        Ok(self.rows.write().await.remove(&k).is_some())
    }
}

/// SurrealDB-backed credential store. Compiles in the default (surreal) build.
///
/// Rows live in the `provider_credentials` table, keyed deterministically by
/// `scope:scope_id:provider_id` so `put` is an idempotent UPSERT and the
/// `(scope, scope_id, provider_id)` triple is unique.
#[derive(Debug, Clone)]
pub struct SurrealCredentialStore {
    db: surrealdb::Surreal<surrealdb::engine::any::Any>,
}

impl SurrealCredentialStore {
    #[must_use]
    pub fn new(db: surrealdb::Surreal<surrealdb::engine::any::Any>) -> Self {
        Self { db }
    }

    fn record_id(scope: CredentialScope, scope_id: &str, provider_id: &str) -> String {
        // Deterministic id => uniqueness on the triple + idempotent upsert.
        format!("{}:{}:{}", scope.as_str(), scope_id, provider_id)
    }
}

/// Convert raw `SurrealDB` values into `CredentialRecord`s using the codebase's
/// `Value → serde_json::Value → serde` conversion (surrealdb 3.x does not let us
/// `take::<Vec<T>>` a serde type directly).
fn records_from_values(
    rows: Vec<surrealdb::types::Value>,
) -> anyhow::Result<Vec<CredentialRecord>> {
    use crate::uar::persistence::providers::surreal::surreal_to_json;
    rows.into_iter()
        .map(|v| {
            let json = surreal_to_json(v)?;
            serde_json::from_value(json)
                .map_err(|e| anyhow::anyhow!("deserialize CredentialRecord: {e}"))
        })
        .collect()
}

#[async_trait]
impl CredentialStore for SurrealCredentialStore {
    async fn put(&self, record: CredentialRecord) -> anyhow::Result<()> {
        use crate::uar::persistence::providers::surreal::to_db_value;
        let rid = Self::record_id(record.scope, &record.scope_id, &record.provider_id);
        let payload = to_db_value(&record)?;
        self.db
            .query("UPSERT type::record('provider_credentials', $rid) CONTENT $data")
            .bind(("rid", rid))
            .bind(("data", payload))
            .await?;
        Ok(())
    }

    async fn get(
        &self,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>> {
        let rid = Self::record_id(scope, scope_id, provider_id);
        let mut resp = self
            .db
            .query("SELECT * FROM type::record('provider_credentials', $rid)")
            .bind(("rid", rid))
            .await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        Ok(records_from_values(rows)?.into_iter().next())
    }

    async fn list(
        &self,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>> {
        let mut resp = self
            .db
            .query(
                "SELECT * FROM provider_credentials \
                 WHERE scope = $scope AND scope_id = $scope_id",
            )
            .bind(("scope", scope.as_str().to_string()))
            .bind(("scope_id", scope_id.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        records_from_values(rows)
    }

    async fn delete(
        &self,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool> {
        let rid = Self::record_id(scope, scope_id, provider_id);
        let existed = self.get(scope, scope_id, provider_id).await?.is_some();
        self.db
            .query("DELETE type::record('provider_credentials', $rid)")
            .bind(("rid", rid))
            .await?;
        Ok(existed)
    }
}

/// Convenience alias for a shared store handle.
pub type SharedCredentialStore = Arc<dyn CredentialStore>;

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(scope: CredentialScope, scope_id: &str, provider: &str) -> CredentialRecord {
        let now = Utc::now();
        CredentialRecord {
            scope,
            scope_id: scope_id.to_string(),
            provider_id: provider.to_string(),
            api_key_encrypted: "ct".to_string(),
            api_key_hint: "1234".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn store_and_retrieve_user_scoped() {
        let s = InMemoryCredentialStore::new();
        s.put(rec(CredentialScope::User, "userA", "openai"))
            .await
            .unwrap();
        let got = s
            .get(CredentialScope::User, "userA", "openai")
            .await
            .unwrap();
        assert!(got.is_some());
        assert_eq!(got.unwrap().provider_id, "openai");
    }

    #[tokio::test]
    async fn cross_user_isolation() {
        let s = InMemoryCredentialStore::new();
        s.put(rec(CredentialScope::User, "userA", "openai"))
            .await
            .unwrap();
        let b = s
            .get(CredentialScope::User, "userB", "openai")
            .await
            .unwrap();
        assert!(b.is_none(), "user B must not see user A's credential");
    }

    #[tokio::test]
    async fn delete_removes() {
        let s = InMemoryCredentialStore::new();
        s.put(rec(CredentialScope::User, "userA", "openai"))
            .await
            .unwrap();
        assert!(
            s.delete(CredentialScope::User, "userA", "openai")
                .await
                .unwrap()
        );
        assert!(
            s.get(CredentialScope::User, "userA", "openai")
                .await
                .unwrap()
                .is_none()
        );
        // second delete reports nothing removed
        assert!(
            !s.delete(CredentialScope::User, "userA", "openai")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn provider_isolation() {
        let s = InMemoryCredentialStore::new();
        s.put(rec(CredentialScope::User, "userA", "openai"))
            .await
            .unwrap();
        assert!(
            s.get(CredentialScope::User, "userA", "anthropic")
                .await
                .unwrap()
                .is_none()
        );
    }
}
