use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::uar::compiler::session::CompilerSession;
use crate::uar::compiler::session::persistence::SessionStorage;
use crate::uar::compiler::storage::{ReportRecord, SpecRecord, SpecStorage};
use crate::uar::persistence::providers::surreal::{
    empty_when_table_missing, none_when_table_missing,
};

/// SurrealDB implementation of SpecStorage and SessionStorage.
#[derive(Debug)]
pub struct SurrealCompilerStorage {
    db: Surreal<Any>,
}

impl SurrealCompilerStorage {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Helper to convert a value to a SurrealDB-compatible JSON value.
    fn to_db_value<T: Serialize>(value: &T) -> Result<serde_json::Value> {
        serde_json::to_value(value).context("failed to serialize value for SurrealDB")
    }

    /// Helper to convert a SurrealDB value to a domain type.
    fn from_db_value<T: DeserializeOwned>(value: serde_json::Value) -> Result<T> {
        serde_json::from_value(value).context("failed to deserialize value from SurrealDB")
    }

    /// Helper to convert an optional SurrealDB value.
    fn from_db_opt<T: DeserializeOwned>(value: Option<serde_json::Value>) -> Result<Option<T>> {
        value.map(Self::from_db_value).transpose()
    }

    /// Helper to convert a list of SurrealDB values.
    fn from_db_vec<T: DeserializeOwned>(values: Vec<serde_json::Value>) -> Result<Vec<T>> {
        values.into_iter().map(Self::from_db_value).collect()
    }
}

#[async_trait]
impl SpecStorage for SurrealCompilerStorage {
    async fn create_spec(&self, record: SpecRecord) -> Result<SpecRecord> {
        let content = Self::to_db_value(&record)?;
        let _: Option<serde_json::Value> = self
            .db
            .create(("uar_specs", record.id.as_str()))
            .content(content)
            .await
            .context("failed to create spec in SurrealDB")?;
        Ok(record)
    }

    async fn get_spec(&self, id: &str) -> Result<Option<SpecRecord>> {
        let record: Option<serde_json::Value> =
            none_when_table_missing(self.db.select(("uar_specs", id)).await)?;
        Self::from_db_opt(record)
    }

    async fn list_specs(&self) -> Result<Vec<SpecRecord>> {
        let records: Vec<serde_json::Value> =
            empty_when_table_missing(self.db.select("uar_specs").await)?;
        Self::from_db_vec(records)
    }

    async fn update_spec(&self, id: &str, content: String) -> Result<Option<SpecRecord>> {
        // Optimization: MERGE might be better, but we need to update updated_at too.
        // For now, fetch-modify-save pattern unless we want to use specific SQL queries.
        // Let's use a merge query.
        let updated_at = chrono::Utc::now();
        let query = "UPDATE type::record('uar_specs', $id) MERGE { content: $content, updated_at: $updated_at } RETURN AFTER";
        let mut response = self
            .db
            .query(query)
            .bind(("id", id.to_string()))
            .bind(("content", content))
            .bind(("updated_at", updated_at))
            .await?;

        // Take the first result from the first query. A missing table means
        // no spec has ever been written, so there is nothing to update —
        // `None` (404 at the API layer), not an error.
        let record: Option<serde_json::Value> = none_when_table_missing(response.take(0))?;
        Self::from_db_opt(record)
    }

    async fn delete_spec(&self, id: &str) -> Result<bool> {
        // Delete the spec
        let _: Option<serde_json::Value> = self.db.delete(("uar_specs", id)).await?;

        // Delete associated reports
        // Query: DELETE uar_reports WHERE spec_id = $id
        self.db
            .query("DELETE uar_reports WHERE spec_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        Ok(true)
    }

    async fn save_report(&self, record: ReportRecord) -> Result<ReportRecord> {
        let content = Self::to_db_value(&record)?;
        let _: Option<serde_json::Value> = self
            .db
            .create(("uar_reports", record.id.as_str()))
            .content(content)
            .await
            .context("failed to save report to SurrealDB")?;

        // Update parent spec
        let query = "UPDATE type::record('uar_specs', $spec_id) MERGE { latest_report_id: $report_id, updated_at: $updated_at }";
        self.db
            .query(query)
            .bind(("spec_id", record.spec_id.clone()))
            .bind(("report_id", record.id.clone()))
            .bind(("updated_at", chrono::Utc::now()))
            .await?;

        Ok(record)
    }

    async fn get_report(&self, id: &str) -> Result<Option<ReportRecord>> {
        let record: Option<serde_json::Value> =
            none_when_table_missing(self.db.select(("uar_reports", id)).await)?;
        Self::from_db_opt(record)
    }

    async fn list_reports_for_spec(&self, spec_id: &str) -> Result<Vec<ReportRecord>> {
        let query = "SELECT * FROM uar_reports WHERE spec_id = $spec_id ORDER BY created_at DESC";
        let mut response = self
            .db
            .query(query)
            .bind(("spec_id", spec_id.to_string()))
            .await?;

        let records: Vec<serde_json::Value> = empty_when_table_missing(response.take(0))?;
        Self::from_db_vec(records)
    }
}

#[async_trait]
impl SessionStorage for SurrealCompilerStorage {
    async fn save_session(&self, session: CompilerSession) -> Result<CompilerSession> {
        let id = session.id.clone();
        let _: Option<serde_json::Value> = self
            .db
            .upsert(("uar_compiler_sessions", id.as_str()))
            .content(Self::to_db_value(&session)?)
            .await
            .context("failed to save session to SurrealDB")?;
        Ok(session)
    }

    async fn get_session(&self, id: &str) -> Result<Option<CompilerSession>> {
        let record: Option<serde_json::Value> =
            none_when_table_missing(self.db.select(("uar_compiler_sessions", id)).await)?;
        Self::from_db_opt(record)
    }

    async fn delete_session(&self, id: &str) -> Result<bool> {
        let _: Option<serde_json::Value> = self.db.delete(("uar_compiler_sessions", id)).await?;
        Ok(true)
    }

    async fn list_sessions(&self) -> Result<Vec<CompilerSession>> {
        let records: Vec<serde_json::Value> =
            empty_when_table_missing(self.db.select("uar_compiler_sessions").await)?;
        Self::from_db_vec(records)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fresh-database reads — embedded SurrealKV, no external server.
//
// SurrealDB is schemaless: a table does not exist until its first record is
// written, so reads against a never-written table fail at the driver level.
// These cases pin the contract that an absent table reads as empty/absent
// rather than propagating an error to the API layer as a 500.
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod fresh_db_tests {
    use super::*;

    async fn fresh_storage() -> SurrealCompilerStorage {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "uar-compiler-fresh-db-test-{}",
            uuid::Uuid::new_v4()
        ));
        let db = surrealdb::engine::any::connect(format!("surrealkv://{}", dir.display()))
            .await
            .expect("connect embedded surrealkv");
        db.use_ns("uar_test")
            .use_db("uar_test")
            .await
            .expect("use ns/db");
        SurrealCompilerStorage::new(db)
    }

    #[tokio::test]
    async fn list_specs_on_fresh_database_returns_empty() {
        let storage = fresh_storage().await;
        let specs = storage
            .list_specs()
            .await
            .expect("listing specs on a fresh database must not error");
        assert!(specs.is_empty(), "expected no specs, got {}", specs.len());
    }

    #[tokio::test]
    async fn get_spec_on_fresh_database_returns_none() {
        let storage = fresh_storage().await;
        let spec = storage
            .get_spec("never-written")
            .await
            .expect("getting a spec on a fresh database must not error");
        assert!(spec.is_none());
    }

    #[tokio::test]
    async fn get_report_on_fresh_database_returns_none() {
        let storage = fresh_storage().await;
        let report = storage
            .get_report("never-written")
            .await
            .expect("getting a report on a fresh database must not error");
        assert!(report.is_none());
    }

    #[tokio::test]
    async fn list_reports_for_spec_on_fresh_database_returns_empty() {
        let storage = fresh_storage().await;
        let reports = storage
            .list_reports_for_spec("never-written")
            .await
            .expect("listing reports on a fresh database must not error");
        assert!(reports.is_empty());
    }

    #[tokio::test]
    async fn list_sessions_on_fresh_database_returns_empty() {
        let storage = fresh_storage().await;
        let sessions = storage
            .list_sessions()
            .await
            .expect("listing sessions on a fresh database must not error");
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn get_session_on_fresh_database_returns_none() {
        let storage = fresh_storage().await;
        let session = storage
            .get_session("never-written")
            .await
            .expect("getting a session on a fresh database must not error");
        assert!(session.is_none());
    }
}
