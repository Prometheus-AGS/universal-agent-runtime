use crate::session::Session;
use crate::uar::domain::knowledge::{
    DocumentStatus, KnowledgeBase, KnowledgeChunk, KnowledgeDocument, KnowledgeMatch,
};
use crate::uar::domain::policy::ConversationPolicyRecord;
use crate::uar::domain::skills::{Skill, SkillMatch};
use crate::uar::settings::schema::{Settings, SettingsType};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod providers;

pub(crate) fn tenant_storage_key(owner_id: &str, resource_id: &str) -> String {
    format!("{}:{owner_id}:{resource_id}", owner_id.len())
}

/// Metadata for a file uploaded during a chat session.
/// The actual bytes live on disk at `file_path`; served via GET /api/attachments/{id}.
#[derive(Debug, Clone)]
pub struct AttachmentMeta {
    pub id: String,
    pub session_id: String,
    pub filename: String,
    pub content_type: String,
    /// Absolute path to the stored file on disk.
    pub file_path: String,
    pub file_size: i64,
    pub is_image: bool,
    /// Extracted text for non-image documents (PDF, Word, plain text).
    pub text_content: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct PostgresProvider;

#[async_trait]
pub trait PersistenceLayer: Send + Sync + std::fmt::Debug {
    // Session Management
    async fn save_session(&self, session: &Session) -> Result<()>;
    async fn load_session(&self, owner_id: &str, id: &str) -> Result<Option<Session>>;

    /// Save or replace a conversation-scoped chat policy.
    async fn save_conversation_policy(&self, record: &ConversationPolicyRecord) -> Result<()>;

    /// Load a conversation-scoped chat policy.
    async fn load_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<Option<ConversationPolicyRecord>>;

    /// Delete a conversation-scoped chat policy.
    async fn delete_conversation_policy(&self, owner_id: &str, conversation_id: &str)
    -> Result<()>;

    // Skill Management
    async fn save_skill(&self, skill: &Skill, embedding: &[f32]) -> Result<()>;
    async fn search_skills(&self, query_vec: &[f32], limit: usize) -> Result<Vec<SkillMatch>>;

    /// List all persisted skills.
    async fn list_skills(&self) -> Result<Vec<Skill>> {
        Ok(Vec::new())
    }

    /// Delete a skill by ID.
    async fn delete_skill(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    // =========================================================================
    // Knowledge Base Management
    // =========================================================================

    /// Save or update a knowledge base definition.
    async fn save_knowledge_base(&self, kb: &KnowledgeBase) -> Result<()>;

    /// Get a knowledge base by ID.
    async fn get_knowledge_base(&self, owner_id: &str, id: &str) -> Result<Option<KnowledgeBase>>;

    /// Get a knowledge base by name.
    async fn get_knowledge_base_by_name(
        &self,
        owner_id: &str,
        name: &str,
    ) -> Result<Option<KnowledgeBase>>;

    /// List all knowledge bases.
    async fn list_knowledge_bases(&self, owner_id: &str) -> Result<Vec<KnowledgeBase>>;

    /// Delete a knowledge base and all its chunks/documents.
    async fn delete_knowledge_base(&self, owner_id: &str, id: &str) -> Result<()>;

    // =========================================================================
    // Knowledge Chunk Management
    // =========================================================================

    /// Save a knowledge chunk.
    async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()>;

    /// Search knowledge across ALL knowledge bases (original behavior).
    async fn search_knowledge(
        &self,
        owner_id: &str,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>>;

    /// Search knowledge scoped to specific knowledge base IDs.
    async fn search_knowledge_scoped(
        &self,
        owner_id: &str,
        kb_ids: &[&str],
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>>;

    // =========================================================================
    // Document Tracking
    // =========================================================================

    /// Save a document record.
    async fn save_document(&self, doc: &KnowledgeDocument) -> Result<()>;

    /// Get a document by ID.
    async fn get_document(&self, owner_id: &str, id: &str) -> Result<Option<KnowledgeDocument>>;

    /// List documents in a knowledge base.
    async fn list_documents(&self, owner_id: &str, kb_id: &str) -> Result<Vec<KnowledgeDocument>>;

    /// Count documents in a knowledge base. Default impl uses `list_documents`;
    /// providers should override with a backend-native COUNT query for efficiency.
    async fn count_documents(&self, owner_id: &str, kb_id: &str) -> Result<usize> {
        Ok(self.list_documents(owner_id, kb_id).await?.len())
    }

    /// Update document processing status.
    async fn update_document_status(
        &self,
        owner_id: &str,
        doc_id: &str,
        status: &DocumentStatus,
    ) -> Result<()>;

    /// Delete a document and all its associated chunks.
    async fn delete_document(&self, owner_id: &str, doc_id: &str) -> Result<()>;

    // =========================================================================
    // Agent Persistence
    // =========================================================================

    async fn save_agent(&self, agent: &crate::uar::domain::artifact::AgentArtifact) -> Result<()>;
    async fn load_agent(
        &self,
        id: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>>;
    async fn load_agent_by_name(
        &self,
        name: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>>;
    async fn list_agents(&self) -> Result<Vec<crate::uar::domain::artifact::AgentArtifact>>;

    /// Delete an agent by ID.
    async fn delete_agent(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    // =========================================================================
    // Memory System
    // =========================================================================

    async fn save_memory(&self, memory: &crate::uar::domain::memory::Memory) -> Result<()>;
    async fn search_memory(
        &self,
        agent_id: Option<&str>,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<crate::uar::domain::memory::MemoryMatch>>;

    // =========================================================================
    // Settings Management
    // =========================================================================

    /// Upsert a settings type definition (core UAR namespace or plugin-provided).
    /// Callers pass in the full struct including JSON Schema — idempotent on key.
    /// Returns the **actual** `id` stored in the database: on a first insert this
    /// matches `st.id`, but on a conflict-update the original row's `id` is kept,
    /// so callers must use the returned value when constructing FK references.
    async fn upsert_settings_type(&self, _st: &SettingsType) -> Result<Uuid> {
        Ok(_st.id)
    }

    /// List all registered settings types.
    async fn list_settings_types(&self) -> Result<Vec<SettingsType>> {
        Ok(Vec::new())
    }

    /// Get a settings type by key slug (e.g. "server").
    async fn get_settings_type(&self, _key: &str) -> Result<Option<SettingsType>> {
        Ok(None)
    }

    /// Upsert a single setting value.
    ///
    /// Implementations MUST validate `setting.data` against the parent type's
    /// JSON Schema (fetched via `get_settings_type`) and return `Err` if invalid.
    async fn upsert_setting(&self, _setting: &Settings) -> Result<()> {
        Ok(())
    }

    /// Get one setting by its unique dotted key (e.g. "server.port").
    async fn get_setting(&self, _key: &str) -> Result<Option<Settings>> {
        Ok(None)
    }

    /// List settings, optionally scoped to a type key and/or parent_id.
    async fn list_settings(
        &self,
        _type_key: Option<&str>,
        _parent_id: Option<Uuid>,
    ) -> Result<Vec<Settings>> {
        Ok(Vec::new())
    }

    /// Delete a setting by key (reset-to-default workflow).
    async fn delete_setting(&self, _key: &str) -> Result<()> {
        Ok(())
    }

    // =========================================================================
    // Chat Attachment Storage
    // =========================================================================

    /// Persist attachment metadata after a file has been written to disk.
    async fn insert_attachment(&self, _meta: &AttachmentMeta) -> Result<()> {
        Ok(())
    }

    /// Retrieve attachment metadata by ID.
    async fn get_attachment(&self, _id: &str) -> Result<Option<AttachmentMeta>> {
        Ok(None)
    }

    /// List all attachments uploaded during a given chat session.
    async fn list_attachments_for_session(&self, _session_id: &str) -> Result<Vec<AttachmentMeta>> {
        Ok(Vec::new())
    }

    // =========================================================================
    // Checkpoint Persistence
    // =========================================================================

    /// Persist a graph execution checkpoint.
    async fn save_checkpoint(
        &self,
        _checkpoint: &crate::uar::runtime::checkpoint::Checkpoint,
    ) -> Result<()> {
        Ok(())
    }

    /// Load a single checkpoint by its unique ID.
    async fn load_checkpoint(
        &self,
        _id: &str,
    ) -> Result<Option<crate::uar::runtime::checkpoint::Checkpoint>> {
        Ok(None)
    }

    /// List all checkpoints recorded for a given run, ordered by creation time.
    async fn list_checkpoints(
        &self,
        _run_id: &str,
    ) -> Result<Vec<crate::uar::runtime::checkpoint::Checkpoint>> {
        Ok(Vec::new())
    }

    // =========================================================================
    // Cost Budget History (CH-07)
    // =========================================================================

    /// Durably record one cost-budget spend event. `CostBudgetTracker`
    /// (`src/uar/runtime/cost_budget.rs`) is intentionally in-memory-only for
    /// the hot path; this is the durable roll-up layer its own doc comment
    /// anticipated subscribing to. Default no-op so this remains additive for
    /// any future `PersistenceLayer` implementer that doesn't need history.
    async fn record_cost_entry(&self, _scope: &str, _scope_id: &str, _cost_usd: f64) -> Result<()> {
        Ok(())
    }

    /// List durable cost-history entries for a `(scope, scope_id)`, ordered by
    /// `recorded_at` ascending.
    async fn list_cost_history(&self, _scope: &str, _scope_id: &str) -> Result<Vec<CostEntry>> {
        Ok(Vec::new())
    }
}

/// One durable cost-budget spend event (CH-07). Mirrors
/// `cost_budget::BudgetScope`/`record()`'s in-memory accounting, but as a
/// persisted, append-only history entry rather than an aggregate.
#[derive(Debug, Clone)]
pub struct CostEntry {
    pub scope: String,
    pub scope_id: String,
    pub cost_usd: f64,
    pub recorded_at: DateTime<Utc>,
}
