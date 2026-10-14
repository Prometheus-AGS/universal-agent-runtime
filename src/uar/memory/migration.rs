//! Explicit, evidence-backed import from the quarantined legacy database.
//!
//! The operator must quiesce legacy writers before importing. No heuristic
//! interprets a legacy subject string as a complete principal key.
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use surreal_memory::{Memory, MemoryHistory, SurrealStorage, storage::MemoryStorage};
use crate::uar::security::claims::{CredentialPrincipal, UserContext};
use super::{owned::record_id, service::MemoryService};

impl MemoryService {
    /// Import exactly one legacy record after a trusted operator has proved its
    /// owner mapping. The target owner is captured from a verified context, not
    /// deserialized from the evidence. Original data/history remain quarantined.
    ///
    /// `evidence` is an auditable mapping reference (for example an operator
    /// change record), not an authorization token. Replaying the same import is
    /// idempotent; competing owner mappings or changed source data are rejected.
    pub async fn import_legacy_memory(
        &self,
        administrator: &UserContext,
        id: &str,
        owner: &UserContext,
        evidence: &str,
    ) -> Result<Memory> {
        self.admin_storage(administrator)?;
        let principal = CredentialPrincipal::from_verified_context(owner)?;
        anyhow::ensure!(!evidence.trim().is_empty(), "Legacy owner mapping evidence required");
        let target = self.storage.as_any().downcast_ref::<SurrealStorage>()
            .context("Legacy migration requires the production Surreal memory store")?;
        // SurrealDB 3.3 Clone creates a distinct session over the same engine.
        // Selecting the quarantined DB on this handle cannot retarget live reads.
        let legacy = target.db()?;
        legacy.use_db(self.config.database.clone()).await?;
        let namespace = self.config.namespace.as_str();
        let database = self.config.database.as_str();
        let source_record = if id.contains(':') {
            surrealdb::types::RecordId::parse_simple(id)?
        } else {
            surrealdb::types::RecordId::new("memory", id)
        };
        anyhow::ensure!(source_record.table.as_str() == "memory", "Expected legacy memory ID");
        let rows: Vec<LegacyMemoryRow> = legacy.query("SELECT * FROM $record")
            .bind(("record", source_record.clone())).await?.take(0)?;
        let row = rows.into_iter().next().context("Legacy memory not found")?;
        // Legacy storage uses string enums, while the public Rust model has
        // SurrealValue enum representation. Decode through its serde wire form.
        let source: Memory = serde_json::from_value(serde_json::to_value(row)?)?;
        // Even `v1:k:...` here remains a literal legacy subject. Only this
        // explicit mapping can assign it to a verified issuer/tenant/subject.
        anyhow::ensure!(source.user_id.as_deref() == Some(principal.subject()),
            "Legacy subject does not match the proven owner mapping");
        let source_id = source.id.as_ref().map(record_id).transpose()?.context("Legacy memory has no ID")?;
        let history: Vec<MemoryHistory> = legacy.query("SELECT * FROM memory_history WHERE memory_id = $record ORDER BY version ASC")
            .bind(("record", source_record)).await?.take(0)?;
        let source_digest = Sha256::digest(serde_json::to_vec(&(&source, &history))?)
            .iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let key_material = serde_json::to_vec(&(namespace, database, &source_id))?;
        let destination_key = format!("legacy-{}", Sha256::digest(key_material)
            .iter().map(|byte| format!("{byte:02x}")).collect::<String>());
        if let Some(existing) = target.get_memory(&destination_key).await? {
            ensure_same_import(&existing, &principal, &source_digest)?;
            return self.visible_memory(existing).await?.context("Imported memory was deleted or expired");
        }
        let mut imported = source.clone();
        imported.id = None;
        imported.task_stream_id = None;
        imported.user_id = Some(principal.storage_key());
        imported.metadata = Some(serde_json::json!({
            "uar_migration": {
                "schema": 1,
                "source_namespace": namespace,
                "source_database": database,
                "source_id": source_id,
                "source_digest": source_digest,
                "evidence": evidence,
                "administrator": administrator.trusted_host_id(),
                "history": history,
                "original_metadata": source.metadata,
                "original_task_stream_id": source.task_stream_id,
                "original_created_at": source.created_at,
                "original_updated_at": source.updated_at
            }
        }));
        let stored = self.save_controlled_memory(imported, Some(&destination_key), None).await?
            .context("Imported memory was deleted or expired")?;
        // A stable-key save may return a concurrent winner: never accept
        // another owner's assignment as successful migration.
        ensure_same_import(&stored, &principal, &source_digest)?;
        Ok(stored)
    }
}

fn ensure_same_import(memory: &Memory, principal: &CredentialPrincipal, digest: &str) -> Result<()> {
    anyhow::ensure!(memory.user_id.as_deref() == Some(principal.storage_key().as_str())
        && memory.metadata.as_ref().and_then(|value| value.get("uar_migration"))
            .and_then(|value| value.get("source_digest")).and_then(serde_json::Value::as_str) == Some(digest),
        "Legacy memory import conflicts with a prior owner mapping or source revision");
    Ok(())
}

use surrealdb::types::{RecordId, Datetime, SurrealValue};

#[derive(serde::Serialize, serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct LegacyMemoryRow {
    id: Option<RecordId>,
    content: String,
    embedding: Option<Vec<f32>>,
    scope: String,
    memory_type: String,
    user_id: Option<String>,
    session_id: Option<String>,
    agent_id: Option<String>,
    task_stream_id: Option<RecordId>,
    categories: Vec<String>,
    metadata: Option<serde_json::Value>,
    token_count: Option<u32>,
    importance: f32,
    access_count: u32,
    last_accessed_at: Option<Datetime>,
    valid_until: Option<Datetime>,
    version: u32,
    created_at: Datetime,
    updated_at: Datetime,
}
