//! PostgreSQL-backed compiler storage.
//!
//! Implements both [`SpecStorage`] and [`SessionStorage`] using a shared
//! `PgPool` from `sqlx`. Data is persisted to `uar_specs`, `uar_reports`,
//! and `uar_compiler_sessions` tables created by the accompanying migration.
//!
//! Uses `sqlx::query()` (non-macro) throughout to avoid the compile-time
//! database introspection requirement of `sqlx::query!()`.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::uar::compiler::pipeline::CompileOutput;
use crate::uar::compiler::session::{CompilerSession, persistence::SessionStorage};

use super::{ReportRecord, SpecRecord, SpecStorage};

/// PostgreSQL-backed implementation of [`SpecStorage`] and [`SessionStorage`].
#[derive(Debug, Clone)]
pub struct PostgresCompilerStorage {
    pool: PgPool,
}

impl PostgresCompilerStorage {
    /// Create a new storage instance backed by the given connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SpecStorage
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl SpecStorage for PostgresCompilerStorage {
    async fn create_spec(&self, record: SpecRecord) -> Result<SpecRecord> {
        sqlx::query(
            r"
            INSERT INTO uar_specs (id, name, content, description, latest_report_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
        )
        .bind(&record.id)
        .bind(&record.name)
        .bind(&record.content)
        .bind(&record.description)
        .bind(&record.latest_report_id)
        .bind(record.created_at)
        .bind(record.updated_at)
        .execute(&self.pool)
        .await
        .context("failed to insert spec")?;

        Ok(record)
    }

    async fn get_spec(&self, id: &str) -> Result<Option<SpecRecord>> {
        let row = sqlx::query(
            r"
            SELECT id, name, content, description, latest_report_id,
                   created_at, updated_at
            FROM uar_specs
            WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("failed to fetch spec")?;

        Ok(row.map(|r| {
            use sqlx::Row;
            SpecRecord {
                id: r.get("id"),
                name: r.get("name"),
                content: r.get("content"),
                description: r.get("description"),
                latest_report_id: r.get("latest_report_id"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }
        }))
    }

    async fn list_specs(&self) -> Result<Vec<SpecRecord>> {
        let rows = sqlx::query(
            r"
            SELECT id, name, content, description, latest_report_id,
                   created_at, updated_at
            FROM uar_specs
            ORDER BY created_at DESC
            ",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to list specs")?;

        Ok(rows
            .into_iter()
            .map(|r| {
                use sqlx::Row;
                SpecRecord {
                    id: r.get("id"),
                    name: r.get("name"),
                    content: r.get("content"),
                    description: r.get("description"),
                    latest_report_id: r.get("latest_report_id"),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                }
            })
            .collect())
    }

    async fn update_spec(&self, id: &str, content: String) -> Result<Option<SpecRecord>> {
        let now = Utc::now();
        let row = sqlx::query(
            r"
            UPDATE uar_specs
            SET content = $2, updated_at = $3
            WHERE id = $1
            RETURNING id, name, content, description, latest_report_id,
                      created_at, updated_at
            ",
        )
        .bind(id)
        .bind(&content)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .context("failed to update spec")?;

        Ok(row.map(|r| {
            use sqlx::Row;
            SpecRecord {
                id: r.get("id"),
                name: r.get("name"),
                content: r.get("content"),
                description: r.get("description"),
                latest_report_id: r.get("latest_report_id"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }
        }))
    }

    async fn delete_spec(&self, id: &str) -> Result<bool> {
        // Associated reports are deleted via ON DELETE CASCADE in the migration.
        let result = sqlx::query("DELETE FROM uar_specs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("failed to delete spec")?;

        Ok(result.rows_affected() > 0)
    }

    async fn save_report(&self, record: ReportRecord) -> Result<ReportRecord> {
        let data =
            serde_json::to_value(&record.output).context("failed to serialize compile output")?;

        sqlx::query(
            r"
            INSERT INTO uar_reports (id, spec_id, data, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data
            ",
        )
        .bind(&record.id)
        .bind(&record.spec_id)
        .bind(&data)
        .bind(record.created_at)
        .execute(&self.pool)
        .await
        .context("failed to insert report")?;

        // Update the spec's latest_report_id
        let now = Utc::now();
        sqlx::query(
            r"
            UPDATE uar_specs
            SET latest_report_id = $2, updated_at = $3
            WHERE id = $1
            ",
        )
        .bind(&record.spec_id)
        .bind(&record.id)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("failed to update spec latest_report_id")?;

        Ok(record)
    }

    async fn get_report(&self, id: &str) -> Result<Option<ReportRecord>> {
        let row =
            sqlx::query("SELECT id, spec_id, data, created_at FROM uar_reports WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .context("failed to fetch report")?;

        row.map(|r| {
            use sqlx::Row;
            let data: serde_json::Value = r.get("data");
            let output: CompileOutput =
                serde_json::from_value(data).context("failed to deserialize compile output")?;
            let created_at: DateTime<Utc> = r.get("created_at");
            Ok(ReportRecord {
                id: r.get("id"),
                spec_id: r.get("spec_id"),
                output,
                created_at,
            })
        })
        .transpose()
    }

    async fn list_reports_for_spec(&self, spec_id: &str) -> Result<Vec<ReportRecord>> {
        let rows = sqlx::query(
            r"
            SELECT id, spec_id, data, created_at
            FROM uar_reports
            WHERE spec_id = $1
            ORDER BY created_at DESC
            ",
        )
        .bind(spec_id)
        .fetch_all(&self.pool)
        .await
        .context("failed to list reports")?;

        rows.into_iter()
            .map(|r| {
                use sqlx::Row;
                let data: serde_json::Value = r.get("data");
                let output: CompileOutput =
                    serde_json::from_value(data).context("failed to deserialize compile output")?;
                let created_at: DateTime<Utc> = r.get("created_at");
                Ok(ReportRecord {
                    id: r.get("id"),
                    spec_id: r.get("spec_id"),
                    output,
                    created_at,
                })
            })
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SessionStorage
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl SessionStorage for PostgresCompilerStorage {
    async fn save_session(&self, session: CompilerSession) -> Result<CompilerSession> {
        let data = serde_json::to_value(&session).context("failed to serialize session")?;
        let now = Utc::now();

        sqlx::query(
            r"
            INSERT INTO uar_compiler_sessions (id, data, created_at, updated_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data, updated_at = $4
            ",
        )
        .bind(&session.id)
        .bind(&data)
        .bind(session.created_at)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("failed to upsert session")?;

        Ok(session)
    }

    async fn get_session(&self, id: &str) -> Result<Option<CompilerSession>> {
        let row = sqlx::query("SELECT data FROM uar_compiler_sessions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("failed to fetch session")?;

        row.map(|r| {
            use sqlx::Row;
            let data: serde_json::Value = r.get("data");
            serde_json::from_value::<CompilerSession>(data).context("failed to deserialize session")
        })
        .transpose()
    }

    async fn delete_session(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM uar_compiler_sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("failed to delete session")?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_sessions(&self) -> Result<Vec<CompilerSession>> {
        let rows = sqlx::query("SELECT data FROM uar_compiler_sessions ORDER BY updated_at DESC")
            .fetch_all(&self.pool)
            .await
            .context("failed to list sessions")?;

        rows.into_iter()
            .map(|r| {
                use sqlx::Row;
                let data: serde_json::Value = r.get("data");
                serde_json::from_value::<CompilerSession>(data)
                    .context("failed to deserialize session")
            })
            .collect()
    }
}
