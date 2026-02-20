//! PostgreSQL-backed agent registry.
//!
//! Implements the [`AgentRegistry`] trait using a shared `PgPool` from `sqlx`.
//! Data is persisted to the `uar_agents` table created by migration
//! `20260219000002_create_agent_registry.sql`.
//!
//! Uses `sqlx::query()` (non-macro) throughout to avoid the compile-time
//! database introspection requirement of `sqlx::query!()`.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;

use super::registry::{AgentInfo, AgentRegistry};

/// PostgreSQL-backed implementation of [`AgentRegistry`].
#[derive(Debug, Clone)]
pub struct PostgresAgentRegistry {
    pool: PgPool,
}

impl PostgresAgentRegistry {
    /// Create a new registry backed by the given connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AgentRegistry for PostgresAgentRegistry {
    async fn register_agent(&self, mut agent: AgentInfo) -> Result<String> {
        agent.updated_at = Utc::now();
        let id = agent.id.clone();

        sqlx::query(
            r"
            INSERT INTO uar_agents (id, name, description, base_url, capabilities, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                name         = EXCLUDED.name,
                description  = EXCLUDED.description,
                base_url     = EXCLUDED.base_url,
                capabilities = EXCLUDED.capabilities,
                updated_at   = EXCLUDED.updated_at
            ",
        )
        .bind(&agent.id)
        .bind(&agent.name)
        .bind(&agent.description)
        .bind(&agent.base_url)
        .bind(&agent.capabilities)
        .bind(agent.created_at)
        .bind(agent.updated_at)
        .execute(&self.pool)
        .await
        .context("failed to register agent")?;

        Ok(id)
    }

    async fn get_agent(&self, id: &str) -> Result<Option<AgentInfo>> {
        let row = sqlx::query(
            r"
            SELECT id, name, description, base_url, capabilities, created_at, updated_at
            FROM uar_agents
            WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("failed to fetch agent")?;

        Ok(row.map(|r| {
            use sqlx::Row;
            AgentInfo {
                id: r.get("id"),
                name: r.get("name"),
                description: r.get("description"),
                base_url: r.get("base_url"),
                capabilities: r.get("capabilities"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }
        }))
    }

    async fn list_agents(&self) -> Result<Vec<AgentInfo>> {
        let rows = sqlx::query(
            r"
            SELECT id, name, description, base_url, capabilities, created_at, updated_at
            FROM uar_agents
            ORDER BY created_at DESC
            ",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to list agents")?;

        Ok(rows
            .into_iter()
            .map(|r| {
                use sqlx::Row;
                AgentInfo {
                    id: r.get("id"),
                    name: r.get("name"),
                    description: r.get("description"),
                    base_url: r.get("base_url"),
                    capabilities: r.get("capabilities"),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                }
            })
            .collect())
    }
}
