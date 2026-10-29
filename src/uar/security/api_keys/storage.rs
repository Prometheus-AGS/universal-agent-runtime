//! Host-owned durable API-key records. Creation is insert-only and revocation
//! keeps its tombstone; no normal API can restore an old authority record.

#[cfg(any(feature = "surreal-backend", feature = "postgres-backend"))]
use super::{ApiKeyRecord, ApiKeyStorage};
#[cfg(any(feature = "surreal-backend", feature = "postgres-backend"))]
use async_trait::async_trait;
#[cfg(any(feature = "surreal-backend", feature = "postgres-backend"))]
async fn authority_query<T, E>(
    query: impl IntoFuture<Output = Result<T, E>>,
) -> Result<T, ApiKeyStorageError> {
    // Only an actual backend result establishes authority availability. Outer
    // request cancellation is separate and never establishes mutation rollback.
    query.await.map_err(|_| ApiKeyStorageError::Unavailable)
}

/// Sanitized persistence outcomes. Backend errors may contain credential data
/// and must never cross the authentication or operator receipt boundary.
#[derive(Debug, thiserror::Error)]
pub enum ApiKeyStorageError {
    #[error("API-key authority storage is unavailable")]
    Unavailable,
    #[error("API-key record already exists; reissue with a new key")]
    AlreadyExists,
}

#[cfg(feature = "surreal-backend")]
#[derive(Debug)]
pub struct SurrealApiKeyStorage {
    db: surrealdb::Surreal<surrealdb::engine::any::Any>,
}

#[cfg(feature = "surreal-backend")]
impl SurrealApiKeyStorage {
    /// Reuse the runtime's selected DB handle; no reconnect or memory fallback.
    pub fn new(db: surrealdb::Surreal<surrealdb::engine::any::Any>) -> Self {
        Self {
            db,
        }
    }

    /// Compatibility entrypoint. Request budgets may cancel the caller; they
    /// do not determine authority availability or certify persistence rollback.
    pub fn with_request_timeout_ms(self, _request_timeout_ms: u64) -> Self { self }

    async fn records(
        &self,
        query: &str,
        parameter: Option<(&'static str, String)>,
    ) -> anyhow::Result<Vec<ApiKeyRecord>> {
        let mut request = self.db.query(query);
        if let Some(parameter) = parameter {
            request = request.bind(parameter);
        }
        let mut response = authority_query(request)
            .await?
            .check()
            .map_err(|_| ApiKeyStorageError::Unavailable)?;
        let rows: Vec<surrealdb::types::Value> = response
            .take(0)
            .map_err(|_| ApiKeyStorageError::Unavailable)?;
        rows.into_iter()
            .map(|row| {
                let mut json = crate::uar::persistence::providers::surreal::surreal_to_json(row)
                    .map_err(|_| ApiKeyStorageError::Unavailable)?;
                serde_json::from_value(json["data"].take())
                    .map_err(|_| ApiKeyStorageError::Unavailable.into())
            })
            .collect()
    }
}

#[cfg(feature = "surreal-backend")]
#[async_trait]
impl ApiKeyStorage for SurrealApiKeyStorage {
    async fn insert(&self, record: ApiKeyRecord) -> anyhow::Result<()> {
        let data = crate::uar::persistence::providers::surreal::to_db_value(&record)
            .map_err(|_| ApiKeyStorageError::Unavailable)?;
        // CREATE, never UPSERT: a duplicate ID cannot erase a revocation.
        let query = self
            .db
            .query("CREATE type::record('auth_api_keys', $key_id) SET data = $data")
            .bind(("key_id", record.id))
            .bind(("data", data));
        authority_query(query)
            .await?
            .check()
            .map_err(|_| ApiKeyStorageError::Unavailable)?;
        Ok(())
    }

    async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<ApiKeyRecord>> {
        Ok(self
            .records(
                "SELECT data FROM type::record('auth_api_keys', $key_id)",
                Some(("key_id", id.to_owned())),
            )
            .await?
            .into_iter()
            .next())
    }

    async fn list_by_subject(&self, subject: &str) -> anyhow::Result<Vec<ApiKeyRecord>> {
        self.records(
            "SELECT data FROM auth_api_keys WHERE data.subject = $subject",
            Some(("subject", subject.to_owned())),
        )
        .await
    }

    async fn revoke(&self, id: &str) -> anyhow::Result<bool> {
        Ok(!self.records("UPDATE type::record('auth_api_keys', $key_id) SET data.revoked = true RETURN AFTER", Some(("key_id", id.to_owned())))
            .await?.is_empty())
    }

    async fn all_active(&self) -> anyhow::Result<Vec<ApiKeyRecord>> {
        self.records(
            "SELECT data FROM auth_api_keys WHERE data.revoked = false",
            None,
        )
        .await
    }

    async fn all(&self) -> anyhow::Result<Vec<ApiKeyRecord>> {
        self.records("SELECT data FROM auth_api_keys", None).await
    }
}

#[cfg(feature = "postgres-backend")]
#[derive(Debug)]
pub struct PostgresApiKeyStorage {
    pool: sqlx::PgPool,
}

#[cfg(feature = "postgres-backend")]
impl PostgresApiKeyStorage {
    /// Reuse the runtime pool and its startup migrations.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
        }
    }

    /// Compatibility entrypoint. Request budgets may cancel the caller; they
    /// do not determine authority availability or certify persistence rollback.
    pub fn with_request_timeout_ms(self, _request_timeout_ms: u64) -> Self { self }

    async fn records(
        &self,
        query: &'static str,
        parameter: Option<&str>,
    ) -> anyhow::Result<Vec<ApiKeyRecord>> {
        use sqlx::Row;
        let mut request = sqlx::query(query);
        if let Some(parameter) = parameter {
            request = request.bind(parameter);
        }
        let rows = authority_query(request.fetch_all(&self.pool)).await?;
        rows.into_iter()
            .map(|row| {
                let json: serde_json::Value = row
                    .try_get("data")
                    .map_err(|_| ApiKeyStorageError::Unavailable)?;
                serde_json::from_value(json).map_err(|_| ApiKeyStorageError::Unavailable.into())
            })
            .collect()
    }
}

#[cfg(feature = "postgres-backend")]
#[async_trait]
impl ApiKeyStorage for PostgresApiKeyStorage {
    async fn insert(&self, record: ApiKeyRecord) -> anyhow::Result<()> {
        let data = serde_json::to_value(&record).map_err(|_| ApiKeyStorageError::Unavailable)?;
        let query = sqlx::query("INSERT INTO auth_api_keys (key_id, data) VALUES ($1, $2) ON CONFLICT (key_id) DO NOTHING")
            .bind(&record.id).bind(data);
        let inserted = authority_query(query.execute(&self.pool)).await?;
        if inserted.rows_affected() != 1 {
            return Err(ApiKeyStorageError::AlreadyExists.into());
        }
        Ok(())
    }

    async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<ApiKeyRecord>> {
        Ok(self
            .records("SELECT data FROM auth_api_keys WHERE key_id = $1", Some(id))
            .await?
            .into_iter()
            .next())
    }

    async fn list_by_subject(&self, subject: &str) -> anyhow::Result<Vec<ApiKeyRecord>> {
        self.records(
            "SELECT data FROM auth_api_keys WHERE data->>'subject' = $1",
            Some(subject),
        )
        .await
    }

    async fn revoke(&self, id: &str) -> anyhow::Result<bool> {
        let query = sqlx::query("UPDATE auth_api_keys SET data = jsonb_set(data, '{revoked}', 'true'::jsonb) WHERE key_id = $1")
            .bind(id);
        let updated = authority_query(query.execute(&self.pool)).await?;
        Ok(updated.rows_affected() == 1)
    }

    async fn all_active(&self) -> anyhow::Result<Vec<ApiKeyRecord>> {
        self.records(
            "SELECT data FROM auth_api_keys WHERE data->>'revoked' = 'false'",
            None,
        )
        .await
    }

    async fn all(&self) -> anyhow::Result<Vec<ApiKeyRecord>> {
        self.records("SELECT data FROM auth_api_keys", None).await
    }
}
