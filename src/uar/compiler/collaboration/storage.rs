use std::sync::RwLock;

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::uar::domain::collaboration::CollaborationCatalogState;

#[async_trait]
pub trait CollaborationStorage: Send + Sync + std::fmt::Debug {
    async fn load_state(&self) -> Result<CollaborationCatalogState>;

    /// Replace the complete catalog only when its durable generation still
    /// matches `expected_generation`. One replacement is the atomic I1 commit.
    async fn compare_and_swap(
        &self,
        expected_generation: u64,
        next: &CollaborationCatalogState,
    ) -> Result<bool>;
}

#[derive(Debug, Default)]
pub struct InMemoryCollaborationStorage {
    state: RwLock<CollaborationCatalogState>,
}

impl InMemoryCollaborationStorage {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CollaborationStorage for InMemoryCollaborationStorage {
    async fn load_state(&self) -> Result<CollaborationCatalogState> {
        self.state
            .read()
            .map(|state| state.clone())
            .map_err(|_| anyhow::anyhow!("collaboration catalog lock is poisoned"))
    }

    async fn compare_and_swap(
        &self,
        expected_generation: u64,
        next: &CollaborationCatalogState,
    ) -> Result<bool> {
        let mut state = self
            .state
            .write()
            .map_err(|_| anyhow::anyhow!("collaboration catalog lock is poisoned"))?;
        if state.generation != expected_generation {
            return Ok(false);
        }
        *state = next.clone();
        Ok(true)
    }
}

#[cfg(feature = "surreal-backend")]
#[derive(Debug)]
pub struct SurrealCollaborationStorage {
    db: surrealdb::Surreal<surrealdb::engine::any::Any>,
}

#[cfg(feature = "surreal-backend")]
impl SurrealCollaborationStorage {
    #[must_use]
    pub fn new(db: surrealdb::Surreal<surrealdb::engine::any::Any>) -> Self {
        Self { db }
    }
}

#[cfg(feature = "surreal-backend")]
#[async_trait]
impl CollaborationStorage for SurrealCollaborationStorage {
    async fn load_state(&self) -> Result<CollaborationCatalogState> {
        use crate::uar::persistence::providers::surreal::none_when_table_missing;

        let value: Option<serde_json::Value> = none_when_table_missing(
            self.db
                .select(("uar_collaboration_state", "catalog"))
                .await,
        )?;
        value
            .map(serde_json::from_value)
            .transpose()
            .context("failed to decode collaboration catalog state")
            .map(Option::unwrap_or_default)
    }

    async fn compare_and_swap(
        &self,
        expected_generation: u64,
        next: &CollaborationCatalogState,
    ) -> Result<bool> {
        let content = serde_json::to_value(next)
            .context("failed to encode collaboration catalog state")?;
        if expected_generation == 0 {
            let current: Option<serde_json::Value> =
                crate::uar::persistence::providers::surreal::none_when_table_missing(
                    self.db
                        .select(("uar_collaboration_state", "catalog"))
                        .await,
                )?;
            if current.is_none() {
                let created: Result<Option<serde_json::Value>, _> = self
                    .db
                    .create(("uar_collaboration_state", "catalog"))
                    .content(content.clone())
                    .await;
                return match created {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                };
            }
        }

        let mut response = self
            .db
            .query(
                "UPDATE type::record('uar_collaboration_state', 'catalog') \
                 CONTENT $state WHERE generation = $expected RETURN AFTER",
            )
            .bind(("state", content))
            .bind(("expected", expected_generation))
            .await
            .context("failed to compare-and-swap collaboration catalog state")?;
        let updated: Vec<serde_json::Value> = response
            .take(0)
            .context("failed to read collaboration catalog CAS result")?;
        Ok(!updated.is_empty())
    }
}

#[cfg(feature = "postgres-backend")]
#[derive(Debug, Clone)]
pub struct PostgresCollaborationStorage {
    pool: sqlx::PgPool,
}

#[cfg(feature = "postgres-backend")]
impl PostgresCollaborationStorage {
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres-backend")]
#[async_trait]
impl CollaborationStorage for PostgresCollaborationStorage {
    async fn load_state(&self) -> Result<CollaborationCatalogState> {
        use sqlx::Row;

        let row = sqlx::query("SELECT data FROM uar_collaboration_state WHERE id = 'catalog'")
            .fetch_optional(&self.pool)
            .await
            .context("failed to load collaboration catalog state")?;
        row.map(|row| serde_json::from_value(row.get("data")))
            .transpose()
            .context("failed to decode collaboration catalog state")
            .map(Option::unwrap_or_default)
    }

    async fn compare_and_swap(
        &self,
        expected_generation: u64,
        next: &CollaborationCatalogState,
    ) -> Result<bool> {
        let data = serde_json::to_value(next)
            .context("failed to encode collaboration catalog state")?;
        let affected = if expected_generation == 0 {
            sqlx::query(
                r"
                INSERT INTO uar_collaboration_state (id, generation, data, updated_at)
                VALUES ('catalog', $1, $2, NOW())
                ON CONFLICT (id) DO UPDATE
                SET generation = EXCLUDED.generation,
                    data = EXCLUDED.data,
                    updated_at = NOW()
                WHERE uar_collaboration_state.generation = 0
                ",
            )
            .bind(next.generation as i64)
            .bind(data)
            .execute(&self.pool)
            .await
            .context("failed to initialize collaboration catalog state")?
            .rows_affected()
        } else {
            sqlx::query(
                r"
                UPDATE uar_collaboration_state
                SET generation = $1, data = $2, updated_at = NOW()
                WHERE id = 'catalog' AND generation = $3
                ",
            )
            .bind(next.generation as i64)
            .bind(data)
            .bind(expected_generation as i64)
            .execute(&self.pool)
            .await
            .context("failed to compare-and-swap collaboration catalog state")?
            .rows_affected()
        };
        Ok(affected == 1)
    }
}
