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

    /// Parse a scope from its `as_str` form. Unknown values map to `System`
    /// (the safest, least-privileged default for a scoped lookup).
    #[must_use]
    pub fn from_str_lenient(s: &str) -> Self {
        match s {
            "session" => Self::Session,
            "agent" => Self::Agent,
            "user" => Self::User,
            _ => Self::System,
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
#[cfg(feature = "surreal-backend")]
#[derive(Debug, Clone)]
pub struct SurrealCredentialStore {
    db: surrealdb::Surreal<surrealdb::engine::any::Any>,
}

#[cfg(feature = "surreal-backend")]
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
#[cfg(feature = "surreal-backend")]
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

#[cfg(feature = "surreal-backend")]
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

// ─────────────────────────────────────────────────────────────────────────────
// Postgres-backed credential store (feature-gated: `sqlx`/`postgres-backend`).
//
// Mirrors `SurrealCredentialStore` for Postgres deployments — closes the
// multi-tenant-credentials gap where Postgres previously fell back to the
// in-memory store (see fable §R2). AES-256-GCM encryption is applied by the
// caller (`ProviderService`); this store only persists ciphertext.
//
// Requires this table (add via sqlx migration):
//   CREATE TABLE IF NOT EXISTS provider_credentials (
//     scope             TEXT NOT NULL,
//     scope_id          TEXT NOT NULL,
//     provider_id       TEXT NOT NULL,
//     api_key_encrypted TEXT NOT NULL,
//     api_key_hint      TEXT NOT NULL,
//     created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
//     updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
//     PRIMARY KEY (scope, scope_id, provider_id)
//   );
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "sqlx")]
#[derive(Debug, Clone)]
pub struct PostgresCredentialStore {
    pool: sqlx::PgPool,
}

#[cfg(feature = "sqlx")]
impl PostgresCredentialStore {
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "sqlx")]
#[async_trait::async_trait]
impl CredentialStore for PostgresCredentialStore {
    async fn put(&self, record: CredentialRecord) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO provider_credentials \
             (scope, scope_id, provider_id, api_key_encrypted, api_key_hint, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (scope, scope_id, provider_id) DO UPDATE SET \
             api_key_encrypted = EXCLUDED.api_key_encrypted, \
             api_key_hint = EXCLUDED.api_key_hint, \
             updated_at = EXCLUDED.updated_at",
        )
        .bind(record.scope.as_str())
        .bind(&record.scope_id)
        .bind(&record.provider_id)
        .bind(&record.api_key_encrypted)
        .bind(&record.api_key_hint)
        .bind(record.created_at)
        .bind(record.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get(
        &self,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>> {
        let row = sqlx::query(
            "SELECT scope, scope_id, provider_id, api_key_encrypted, api_key_hint, \
             created_at, updated_at FROM provider_credentials \
             WHERE scope = $1 AND scope_id = $2 AND provider_id = $3",
        )
        .bind(scope.as_str())
        .bind(scope_id)
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| row_to_record(&r)))
    }

    async fn list(
        &self,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>> {
        let rows = sqlx::query(
            "SELECT scope, scope_id, provider_id, api_key_encrypted, api_key_hint, \
             created_at, updated_at FROM provider_credentials \
             WHERE scope = $1 AND scope_id = $2 ORDER BY provider_id",
        )
        .bind(scope.as_str())
        .bind(scope_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(row_to_record).collect())
    }

    async fn delete(
        &self,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM provider_credentials \
             WHERE scope = $1 AND scope_id = $2 AND provider_id = $3",
        )
        .bind(scope.as_str())
        .bind(scope_id)
        .bind(provider_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(feature = "sqlx")]
fn row_to_record(r: &sqlx::postgres::PgRow) -> CredentialRecord {
    use sqlx::Row;
    CredentialRecord {
        scope: CredentialScope::from_str_lenient(&r.get::<String, _>("scope")),
        scope_id: r.get("scope_id"),
        provider_id: r.get("provider_id"),
        api_key_encrypted: r.get("api_key_encrypted"),
        api_key_hint: r.get("api_key_hint"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parity tests: `PostgresCredentialStore` against a real Postgres pool.
//
// This repo has no existing testcontainers wiring for Postgres — the
// `docker-compose.prod.postgres.yaml` service is the intended local target
// (`docker compose -f docker-compose.prod.postgres.yaml up -d postgres`).
// These mirror the four `InMemoryCredentialStore` tests above and are
// `#[ignore]`d by default (run with `cargo test -- --ignored`), consistent
// with this repo's existing live-infra test convention
// (tests/integration/live/*). `DATABASE_URL` defaults to the compose file's
// `uar`/`changeme`/`uar` local credentials.
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(all(test, feature = "sqlx"))]
mod postgres_tests {
    use super::*;

    async fn pool() -> sqlx::PgPool {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://uar:changeme@localhost:5432/uar".to_string());
        let pool = sqlx::PgPool::connect(&url)
            .await
            .expect("connect to local Postgres (docker-compose.prod.postgres.yaml)");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("run migrations");
        pool
    }

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
    #[ignore = "requires a local Postgres (docker-compose.prod.postgres.yaml)"]
    async fn store_and_retrieve_user_scoped() {
        let s = PostgresCredentialStore::new(pool().await);
        s.put(rec(CredentialScope::User, "pg-userA", "openai"))
            .await
            .unwrap();
        let got = s
            .get(CredentialScope::User, "pg-userA", "openai")
            .await
            .unwrap();
        assert!(got.is_some());
        assert_eq!(got.unwrap().provider_id, "openai");
        s.delete(CredentialScope::User, "pg-userA", "openai")
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires a local Postgres (docker-compose.prod.postgres.yaml)"]
    async fn cross_user_isolation() {
        let s = PostgresCredentialStore::new(pool().await);
        s.put(rec(CredentialScope::User, "pg-userA2", "openai"))
            .await
            .unwrap();
        let b = s
            .get(CredentialScope::User, "pg-userB2", "openai")
            .await
            .unwrap();
        assert!(b.is_none(), "user B must not see user A's credential");
        s.delete(CredentialScope::User, "pg-userA2", "openai")
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires a local Postgres (docker-compose.prod.postgres.yaml)"]
    async fn delete_removes() {
        let s = PostgresCredentialStore::new(pool().await);
        s.put(rec(CredentialScope::User, "pg-userA3", "openai"))
            .await
            .unwrap();
        assert!(
            s.delete(CredentialScope::User, "pg-userA3", "openai")
                .await
                .unwrap()
        );
        assert!(
            s.get(CredentialScope::User, "pg-userA3", "openai")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            !s.delete(CredentialScope::User, "pg-userA3", "openai")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    #[ignore = "requires a local Postgres (docker-compose.prod.postgres.yaml)"]
    async fn provider_isolation() {
        let s = PostgresCredentialStore::new(pool().await);
        s.put(rec(CredentialScope::User, "pg-userA4", "openai"))
            .await
            .unwrap();
        assert!(
            s.get(CredentialScope::User, "pg-userA4", "anthropic")
                .await
                .unwrap()
                .is_none()
        );
        s.delete(CredentialScope::User, "pg-userA4", "openai")
            .await
            .unwrap();
    }
}
