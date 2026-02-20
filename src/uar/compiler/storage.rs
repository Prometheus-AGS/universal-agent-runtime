//! Spec storage abstraction for persisting agent specifications and compile reports.
//!
//! The [`SpecStorage`] trait provides a clean abstraction over the underlying
//! storage engine. The default [`InMemorySpecStorage`] is suitable for development
//! and testing. A SurrealDB implementation can be added by implementing the trait.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod postgres;
pub mod surreal;

use super::pipeline::CompileOutput;

// ─────────────────────────────────────────────────────────────────────────────
// Domain types
// ─────────────────────────────────────────────────────────────────────────────

/// A stored agent specification document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecRecord {
    /// Unique ID for this spec.
    pub id: String,
    /// Human-readable name (from the document's agent name, if parseable).
    pub name: String,
    /// The raw Markdown content of the UAR-AGENT-MD document.
    pub content: String,
    /// Optional description / summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// When this spec was first stored.
    pub created_at: DateTime<Utc>,
    /// When this spec was last updated.
    pub updated_at: DateTime<Utc>,
    /// ID of the most recent compile report for this spec (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_report_id: Option<String>,
}

impl SpecRecord {
    /// Create a new spec record from raw Markdown content.
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            content: content.into(),
            description: None,
            created_at: now,
            updated_at: now,
            latest_report_id: None,
        }
    }
}

/// A stored compile report (wraps [`CompileOutput`] with storage metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    /// Unique ID for this report (same as `CompileOutput.report.id`).
    pub id: String,
    /// The spec this report was produced from.
    pub spec_id: String,
    /// The full compile output (descriptor + signature + report).
    pub output: CompileOutput,
    /// When this report was stored.
    pub created_at: DateTime<Utc>,
}

impl ReportRecord {
    pub fn new(spec_id: impl Into<String>, output: CompileOutput) -> Self {
        let id = output.report.id.clone();
        Self {
            id,
            spec_id: spec_id.into(),
            output,
            created_at: Utc::now(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SpecStorage trait
// ─────────────────────────────────────────────────────────────────────────────

/// Storage abstraction for agent specs and compile reports.
///
/// Implement this trait to plug in any storage backend (SurrealDB, SQLite, etc.).
/// The default implementation is [`InMemorySpecStorage`].
#[async_trait]
pub trait SpecStorage: Send + Sync + std::fmt::Debug {
    // ── Spec CRUD ────────────────────────────────────────────────────────────

    /// Store a new spec record. Returns the stored record (with generated ID).
    async fn create_spec(&self, record: SpecRecord) -> Result<SpecRecord>;

    /// Retrieve a spec by ID.
    async fn get_spec(&self, id: &str) -> Result<Option<SpecRecord>>;

    /// List all stored specs (metadata only — content included).
    async fn list_specs(&self) -> Result<Vec<SpecRecord>>;

    /// Update the content of an existing spec.
    async fn update_spec(&self, id: &str, content: String) -> Result<Option<SpecRecord>>;

    /// Delete a spec and its associated reports.
    async fn delete_spec(&self, id: &str) -> Result<bool>;

    // ── Report storage ───────────────────────────────────────────────────────

    /// Store a compile report and link it to the spec.
    async fn save_report(&self, record: ReportRecord) -> Result<ReportRecord>;

    /// Retrieve a compile report by ID.
    async fn get_report(&self, id: &str) -> Result<Option<ReportRecord>>;

    /// List all reports for a given spec.
    async fn list_reports_for_spec(&self, spec_id: &str) -> Result<Vec<ReportRecord>>;
}

// ─────────────────────────────────────────────────────────────────────────────
// In-memory implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Thread-safe in-memory spec storage. Suitable for development and testing.
///
/// Data is lost on process restart. Replace with a SurrealDB or SQLite
/// implementation for production use.
#[derive(Debug, Default)]
pub struct InMemorySpecStorage {
    specs: RwLock<HashMap<String, SpecRecord>>,
    reports: RwLock<HashMap<String, ReportRecord>>,
}

impl InMemorySpecStorage {
    /// Create a new empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SpecStorage for InMemorySpecStorage {
    async fn create_spec(&self, record: SpecRecord) -> Result<SpecRecord> {
        let mut specs = self.specs.write().await;
        specs.insert(record.id.clone(), record.clone());
        Ok(record)
    }

    async fn get_spec(&self, id: &str) -> Result<Option<SpecRecord>> {
        let specs = self.specs.read().await;
        Ok(specs.get(id).cloned())
    }

    async fn list_specs(&self) -> Result<Vec<SpecRecord>> {
        let specs = self.specs.read().await;
        let mut records: Vec<SpecRecord> = specs.values().cloned().collect();
        // Stable ordering: newest first
        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(records)
    }

    async fn update_spec(&self, id: &str, content: String) -> Result<Option<SpecRecord>> {
        let mut specs = self.specs.write().await;
        if let Some(record) = specs.get_mut(id) {
            record.content = content;
            record.updated_at = Utc::now();
            Ok(Some(record.clone()))
        } else {
            Ok(None)
        }
    }

    async fn delete_spec(&self, id: &str) -> Result<bool> {
        let mut specs = self.specs.write().await;
        let removed = specs.remove(id).is_some();
        if removed {
            // Also remove associated reports
            let mut reports = self.reports.write().await;
            reports.retain(|_, r| r.spec_id != id);
        }
        Ok(removed)
    }

    async fn save_report(&self, record: ReportRecord) -> Result<ReportRecord> {
        // Update the spec's latest_report_id
        {
            let mut specs = self.specs.write().await;
            if let Some(spec) = specs.get_mut(&record.spec_id) {
                spec.latest_report_id = Some(record.id.clone());
                spec.updated_at = Utc::now();
            }
        }
        let mut reports = self.reports.write().await;
        reports.insert(record.id.clone(), record.clone());
        Ok(record)
    }

    async fn get_report(&self, id: &str) -> Result<Option<ReportRecord>> {
        let reports = self.reports.read().await;
        Ok(reports.get(id).cloned())
    }

    async fn list_reports_for_spec(&self, spec_id: &str) -> Result<Vec<ReportRecord>> {
        let reports = self.reports.read().await;
        let mut records: Vec<ReportRecord> = reports
            .values()
            .filter(|r| r.spec_id == spec_id)
            .cloned()
            .collect();
        // Newest first
        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(records)
    }
}

/// Type alias for a shared spec storage reference.
pub type SharedSpecStorage = Arc<dyn SpecStorage>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::compiler::report::{CompileOutcome, CompileReport};

    fn make_storage() -> InMemorySpecStorage {
        InMemorySpecStorage::new()
    }

    fn make_report(spec_id: &str) -> ReportRecord {
        let report = CompileReport {
            id: Uuid::new_v4().to_string(),
            agent_id: "test-agent".to_string(),
            version: "1.0.0".to_string(),
            timestamp: Utc::now(),
            stages: vec![],
            overall: CompileOutcome::Pass,
            total_duration_ms: 42,
        };
        // Build a minimal but valid CompileOutput JSON.
        // payload must be a valid AgentDescriptorIR (required fields: agent_name,
        // metadata.version, identity.name/role/persona; rest default to empty).
        let output_json = serde_json::json!({
            "descriptor": {
                "schema": "uar-agent-descriptor/v1",
                "agent_id": "test-agent",
                "version": "1.0.0",
                "content_hash": "abc123",
                "signer_public_key": "deadbeef",
                "payload": {
                    "agent_name": "test-agent",
                    "metadata": { "version": "1.0.0" },
                    "identity": { "name": "Test", "role": "assistant", "persona": "helpful" },
                    "ui": {},
                    "capabilities": {},
                    "skills": {},
                    "tools": {},
                    "mcp_servers": {},
                    "knowledge": {},
                    "memory": {},
                    "a2a": {},
                    "governance": {},
                    "budgets": {},
                    "execution": {},
                    "observability": {},
                    "deployment": {}
                },
                "fingerprints": {},
                "endpoints": [],
                "pep_bindings": []
            },
            "signature": "sig",
            "report": serde_json::to_value(&report).unwrap()
        });
        let output: CompileOutput = serde_json::from_value(output_json).unwrap();
        ReportRecord::new(spec_id, output)
    }

    #[tokio::test]
    async fn test_spec_crud() {
        let store = make_storage();

        // Create
        let spec = SpecRecord::new("my-agent", "## Identity\nname: my-agent");
        let created = store.create_spec(spec).await.unwrap();
        assert!(!created.id.is_empty());

        // Get
        let fetched = store.get_spec(&created.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "my-agent");

        // List
        let list = store.list_specs().await.unwrap();
        assert_eq!(list.len(), 1);

        // Update
        let updated = store
            .update_spec(&created.id, "## Identity\nname: updated".to_string())
            .await
            .unwrap();
        assert!(updated.is_some());
        assert!(updated.unwrap().content.contains("updated"));

        // Delete
        let deleted = store.delete_spec(&created.id).await.unwrap();
        assert!(deleted);
        assert!(store.get_spec(&created.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_report_storage() {
        let store = make_storage();

        // Create a spec first
        let spec = SpecRecord::new("agent-x", "content");
        let created = store.create_spec(spec).await.unwrap();

        // Save a report
        let report = make_report(&created.id);
        let report_id = report.id.clone();
        let saved = store.save_report(report).await.unwrap();
        assert_eq!(saved.id, report_id);

        // Spec should now have latest_report_id set
        let spec = store.get_spec(&created.id).await.unwrap().unwrap();
        assert_eq!(spec.latest_report_id, Some(report_id.clone()));

        // Get report
        let fetched = store.get_report(&report_id).await.unwrap();
        assert!(fetched.is_some());

        // List reports for spec
        let reports = store.list_reports_for_spec(&created.id).await.unwrap();
        assert_eq!(reports.len(), 1);

        // Delete spec cascades to reports
        store.delete_spec(&created.id).await.unwrap();
        let reports = store.list_reports_for_spec(&created.id).await.unwrap();
        assert!(reports.is_empty());
    }
}
