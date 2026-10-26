pub mod scoped;
pub mod knowledge_namespace;
use crate::session::Session;
use crate::uar::a2ui::presentations::{Presentation, PresentationDraft};
use crate::uar::domain::knowledge::{
    DocumentStatus, KnowledgeBase, KnowledgeChunk, KnowledgeDocument, KnowledgeMatch,
};
use crate::uar::domain::policy::ConversationPolicyRecord;
use crate::uar::domain::prompt_caching::UserPromptCachingSettings;
use crate::uar::domain::skills::{Skill, SkillMatch};
use crate::uar::settings::schema::{Settings, SettingsType};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod accounting;
pub mod agui_runs;
pub mod client_tools;
pub mod mcp_elicitations;
pub mod approval_decisions;
pub mod ingestion_records;
pub mod agent_instances;
pub mod agent_threads;
pub mod channel_observers;
pub mod checkpoint_resume;
pub mod observers;
pub mod presentations;
pub mod providers;
pub mod run_history;
pub mod tool_admission;

use crate::uar::runtime::thread::{AgentEdge, AgentThread};
use agent_instances::{AgentInstanceRecord, AgentInstanceStoreError};
use agent_threads::{CanonicalToolReceipt, PersistedAgentThread};
use channel_observers::{ChannelInboxEntry, ChannelSubscription, ChannelObserverStoreError};
use observers::{
    ObserverOccurrence, ObserverOccurrenceBounds, ObserverStoreError, ObserverSubscription,
};
use tool_admission::ToolAdmissionEvidence;

pub(crate) fn tenant_storage_key(owner_id: &str, resource_id: &str) -> String {
    format!("{}:{owner_id}:{resource_id}", owner_id.len())
}

/// Principal sessions occupy a disjoint namespace from legacy subject rows.
/// A subject that resembles a credential key must never acquire old history.
pub(crate) fn session_storage_key(owner_id: &str, session_id: &str) -> String {
    let key = tenant_storage_key(owner_id, session_id);
    if owner_id.starts_with("v1:k:") { format!("principal-session:{key}") } else { key }
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

/// Store eligibility captured by the trusted host when it constructs a provider.
///
/// This intentionally contains no endpoint, namespace, credential, or mutable
/// configuration. Callers project it onto their own admission diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableStoreEligibility {
    LocalDurable,
    RemoteAttested,
    RemoteNotAttested,
    UnsupportedBackend,
}

impl DurableStoreEligibility {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::LocalDurable => "local_durable",
            Self::RemoteAttested => "remote_attested",
            Self::RemoteNotAttested => "remote_not_attested",
            Self::UnsupportedBackend => "unsupported_backend",
        }
    }

    pub(crate) const fn permits_durable_work(self) -> bool {
        matches!(self, Self::LocalDurable | Self::RemoteAttested)
    }
}

#[async_trait]
pub trait PersistenceLayer: Send + Sync + std::fmt::Debug {
    fn supports_durable_client_tools(&self) -> bool { false }
    /// Persist before exposing a caller-tool handoff. Existing batches are never overwritten.
    async fn create_client_tool_continuation(&self, _row: &client_tools::ClientToolContinuation) -> Result<bool> { anyhow::bail!("durable_client_tools_unsupported") }
    async fn load_client_tool_continuation(&self, _owner: &str, _tenant: Option<&str>, _agent: &str, _batch: &str) -> Result<Option<client_tools::ClientToolContinuation>> { anyhow::bail!("durable_client_tools_unsupported") }
    async fn load_client_tool_continuation_for_run(&self, _owner: &str, _tenant: Option<&str>, _agent: &str, _run: &str) -> Result<Option<client_tools::ClientToolContinuation>> { anyhow::bail!("durable_client_tools_unsupported") }
    /// Recover a protocol session's trusted selected agent without a caller-supplied agent override.
    async fn load_client_tool_continuation_for_session(&self, _owner: &str, _tenant: Option<&str>, _session: &str, _batch: &str) -> Result<Option<client_tools::ClientToolContinuation>> { anyhow::bail!("durable_client_tools_unsupported") }
    /// Atomic full-batch claim/completion. A miss must not authorize host effects.
    async fn compare_and_swap_client_tool_continuation(&self, _before: &client_tools::ClientToolContinuation, _after: &client_tools::ClientToolContinuation) -> Result<bool> { anyhow::bail!("durable_client_tools_unsupported") }

    fn supports_durable_mcp_elicitations(&self) -> bool { false }
    async fn create_mcp_elicitation(&self, _row: &mcp_elicitations::McpElicitationRecord) -> Result<()> { anyhow::bail!("durable_elicitation_unsupported") }
    async fn list_mcp_elicitations(&self, _owner: &str, _run: &str) -> Result<Vec<mcp_elicitations::McpElicitationRecord>> { anyhow::bail!("durable_elicitation_unsupported") }
    async fn compare_and_swap_mcp_elicitation(&self, _before: &mcp_elicitations::McpElicitationRecord, _after: &mcp_elicitations::McpElicitationRecord) -> Result<bool> { anyhow::bail!("durable_elicitation_unsupported") }
    fn supports_durable_tool_receipts(&self) -> bool { false }
    fn supports_durable_agui_runs(&self) -> bool { false }
    async fn create_agui_run(&self, _row: &agui_runs::AguiRunBinding, _previous: Option<&agui_runs::AguiRunBinding>) -> Result<bool> { anyhow::bail!("durable_agui_unsupported") }
    async fn load_agui_run(&self, _owner: &str, _agent: &str, _thread: &str, _run: &str) -> Result<Option<agui_runs::AguiRunBinding>> { anyhow::bail!("durable_agui_unsupported") }
    async fn list_agui_runs(&self, _owner: &str, _agent: &str, _thread: &str) -> Result<Vec<agui_runs::AguiRunBinding>> { anyhow::bail!("durable_agui_unsupported") }
    async fn compare_and_swap_agui_run(&self, _before: &agui_runs::AguiRunBinding, _after: &agui_runs::AguiRunBinding) -> Result<bool> { anyhow::bail!("durable_agui_unsupported") }
    /// Host reconciliation only: excluded producer plus resolved effects, never
    /// elapsed time or an absent actor. Unsupported stores retain cancellation.
    async fn reconcile_a2a_cancellation(&self, _owner: &str, _user: &str, _agent: &str, _task: &str) -> Result<Option<crate::uar::api::a2a::durable::A2aBinding>> { Ok(None) }
    fn supports_durable_a2a_bindings(&self) -> bool { false }
    async fn create_a2a_binding(&self, _record: &crate::uar::api::a2a::durable::A2aBinding) -> Result<bool> { anyhow::bail!("durable_a2a_bindings_unsupported") }
    async fn load_a2a_binding(&self, _owner: &str, _agent: &str, _task: &str) -> Result<Option<crate::uar::api::a2a::durable::A2aBinding>> { anyhow::bail!("durable_a2a_bindings_unsupported") }
    async fn compare_and_swap_a2a_binding(&self, _before: &crate::uar::api::a2a::durable::A2aBinding, _after: &crate::uar::api::a2a::durable::A2aBinding) -> Result<bool> { anyhow::bail!("durable_a2a_bindings_unsupported") }
    async fn list_a2a_bindings(&self, _owner: &str, _agent: &str) -> Result<Vec<crate::uar::api::a2a::durable::A2aBinding>> { anyhow::bail!("durable_a2a_bindings_unsupported") }

    /// Restart-durable public run history. Every published event is written
    /// through; a terminal-only snapshot does not satisfy the contract.
    fn supports_durable_run_history(&self) -> bool { false }
    async fn upsert_run_history_record(&self, _record: &run_history::RunHistoryRecord) -> Result<()> { anyhow::bail!("durable_run_history_unsupported") }
    async fn load_run_history_record(&self, _run_id: &str) -> Result<Option<run_history::RunHistoryRecord>> { anyhow::bail!("durable_run_history_unsupported") }
    async fn append_run_history_event(&self, _event: &run_history::RunHistoryEvent) -> Result<()> { anyhow::bail!("durable_run_history_unsupported") }
    async fn load_run_history_events(&self, _run_id: &str) -> Result<Vec<run_history::RunHistoryEvent>> { anyhow::bail!("durable_run_history_unsupported") }

    async fn bind_accounting_delegation(&self, _link: &accounting::AccountingDelegation) -> Result<()> { Err(accounting::AccountingError::Unsupported.into()) }
    async fn get_accounting_delegation(&self, _digest: &str) -> Result<Option<accounting::AccountingCall>> { Err(accounting::AccountingError::Unsupported.into()) }
    /// Durable accounting is a host capability, not an in-memory fallback.
    fn supports_durable_accounting(&self) -> bool { false }
    /// Captured backing-store eligibility for admission and durable-work gates.
    fn durable_store_eligibility(&self) -> DurableStoreEligibility {
        DurableStoreEligibility::UnsupportedBackend
    }
    async fn admit_remote_grant(&self, _owner: &str, _accounting_id: &str,
        _grant: &crate::uar::api::a2a::contract::UarUsageGrant,
        _activity: accounting::RemoteGrantActivity) -> Result<accounting::RemoteGrantState> {
        Err(accounting::AccountingError::Unsupported.into())
    }
    async fn ensure_accounting_scope(&self, _id: &accounting::ScopeIdentity, _limits: &accounting::ScopeLimits) -> Result<accounting::AccountingScope> { Err(accounting::AccountingError::Unsupported.into()) }
    async fn get_accounting_scope(&self, _id: &accounting::ScopeIdentity) -> Result<Option<accounting::AccountingScope>> { Err(accounting::AccountingError::Unsupported.into()) }
    async fn update_accounting_scope(&self, _change: &accounting::ScopeUpdate) -> Result<accounting::AccountingScope> { Err(accounting::AccountingError::Unsupported.into()) }
    async fn admit_accounting_call(&self, _admission: &accounting::CallAdmission) -> Result<accounting::AccountingCall> { Err(accounting::AccountingError::Unsupported.into()) }
    async fn record_accounting_usage(&self, _usage: &accounting::CallUsage) -> Result<accounting::AccountingCall> { Err(accounting::AccountingError::Unsupported.into()) }
    async fn list_accounting_calls(&self, _owner: &str, _run: &str) -> Result<Vec<accounting::AccountingCall>> { Err(accounting::AccountingError::Unsupported.into()) }

    /// Acceptance is unavailable unless bytes, job, and document commit atomically.
    fn supports_durable_ingestion(&self) -> bool { false }

    async fn accept_ingestion(&self, _document: &KnowledgeDocument, _source: &[u8]) -> Result<ingestion_records::IngestionJob> {
        Err(ingestion_records::IngestionStoreError::Unsupported.into())
    }

    async fn discover_ingestion(&self, _limit: usize) -> Result<Vec<ingestion_records::IngestionJob>> {
        Err(ingestion_records::IngestionStoreError::Unsupported.into())
    }

    /// The next durable ingestion deadline, if accepted work remains.
    ///
    /// This is an authoritative schedule read for a one-shot wakeup after a
    /// durable state change. It is not a polling signal and does not certify
    /// that a processing job completed.
    async fn next_ingestion_due_at(&self) -> Result<Option<i64>> {
        Err(ingestion_records::IngestionStoreError::Unsupported.into())
    }

    async fn get_ingestion_job(&self, _owner: &str, _document: &str) -> Result<Option<ingestion_records::IngestionJob>> {
        Err(ingestion_records::IngestionStoreError::Unsupported.into())
    }

    async fn get_ingestion_source(&self, _job: &ingestion_records::IngestionJob) -> Result<Option<Vec<u8>>> {
        Err(ingestion_records::IngestionStoreError::Unsupported.into())
    }

    async fn transition_ingestion(&self, _before: &ingestion_records::IngestionJob, _after: &ingestion_records::IngestionJob) -> Result<bool> {
        Err(ingestion_records::IngestionStoreError::Unsupported.into())
    }

    /// Commits the complete chunk generation, embedding identity and Indexed state together.
    async fn publish_ingestion(&self, _job: &ingestion_records::IngestionJob, _publication: &ingestion_records::IngestionPublication) -> Result<()> {
        Err(ingestion_records::IngestionStoreError::Unsupported.into())
    }

    /// Legacy accepted metadata without recoverable bytes must require reupload.
    async fn reconcile_ingestion(&self) -> Result<()> {
        Err(ingestion_records::IngestionStoreError::Unsupported.into())
    }

    /// Atomically freeze a KB's retained sources and revoke/requeue every member.
    async fn begin_knowledge_regeneration(&self, _owner: &str, _kb: &str,
        _target: &crate::uar::domain::knowledge::EmbeddingFingerprint)
        -> Result<crate::uar::domain::knowledge::KnowledgeRegeneration> {
        Err(ingestion_records::IngestionStoreError::Unsupported.into())
    }

    /// Channel-source observations are a separate, explicitly negotiated profile.
    fn supports_channel_observers(&self) -> bool { false }

    fn channel_observer_unavailable_reason(&self) -> &'static str { "durable_channel_store_unavailable" }

    async fn create_channel_subscription(&self, _record: &ChannelSubscription) -> Result<ChannelSubscription> {
        Err(ChannelObserverStoreError::Unsupported.into())
    }

    async fn load_channel_subscription(&self, _owner: &str, _workspace: &str, _id: &str) -> Result<Option<ChannelSubscription>> {
        Err(ChannelObserverStoreError::Unsupported.into())
    }

    async fn list_channel_subscriptions(&self, _owner: &str, _workspace: &str) -> Result<Vec<ChannelSubscription>> {
        Err(ChannelObserverStoreError::Unsupported.into())
    }

    async fn compare_and_swap_channel_subscription(&self, _before: &ChannelSubscription, _after: &ChannelSubscription) -> Result<bool> {
        Err(ChannelObserverStoreError::Unsupported.into())
    }

    async fn create_channel_inbox_entry(&self, _record: &ChannelInboxEntry) -> Result<ChannelInboxEntry> {
        Err(ChannelObserverStoreError::Unsupported.into())
    }

    async fn load_channel_inbox_entry(&self, _owner: &str, _workspace: &str, _subscription: &str, _delivery: &str) -> Result<Option<ChannelInboxEntry>> {
        Err(ChannelObserverStoreError::Unsupported.into())
    }

    /// Read the exact owner's subscription inbox without changing its cursor or state.
    async fn list_channel_inbox_entries(&self, _owner: &str, _workspace: &str, _subscription: &str) -> Result<Vec<ChannelInboxEntry>> {
        Err(ChannelObserverStoreError::Unsupported.into())
    }

    async fn compare_and_swap_channel_inbox_entry(&self, _before: &ChannelInboxEntry, _after: &ChannelInboxEntry) -> Result<bool> {
        Err(ChannelObserverStoreError::Unsupported.into())
    }

    /// True only for a provider with a transactional instance outbox and durable observer inbox.
    fn supports_durable_observers(&self) -> bool {
        false
    }

    async fn create_observer_subscription(
        &self,
        _subscription: &ObserverSubscription,
    ) -> Result<ObserverSubscription> {
        Err(ObserverStoreError::Unsupported.into())
    }

    async fn load_observer_subscription(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _subscription_id: &str,
    ) -> Result<Option<ObserverSubscription>> {
        Err(ObserverStoreError::Unsupported.into())
    }

    async fn list_observer_subscriptions(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
    ) -> Result<Vec<ObserverSubscription>> {
        Err(ObserverStoreError::Unsupported.into())
    }

    /// Trusted runtime restart scan; API handlers must use scoped listing instead.
    async fn list_all_observer_subscriptions(&self) -> Result<Vec<ObserverSubscription>> {
        Err(ObserverStoreError::Unsupported.into())
    }

    async fn compare_and_swap_observer_subscription(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _expected_revision: u64,
        _next: &ObserverSubscription,
    ) -> Result<bool> {
        Err(ObserverStoreError::Unsupported.into())
    }

    async fn list_instance_occurrences(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _source_instance_id: &str,
        _after_sequence: Option<u64>,
        _limit: usize,
    ) -> Result<Vec<ObserverOccurrence>> {
        Err(ObserverStoreError::Unsupported.into())
    }

    async fn instance_occurrence_bounds(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _source_instance_id: &str,
    ) -> Result<ObserverOccurrenceBounds> {
        Err(ObserverStoreError::Unsupported.into())
    }

    /// True only when this provider persists logical instances across restarts.
    fn supports_durable_agent_instances(&self) -> bool {
        false
    }

    /// Create an exact owner/workspace-scoped logical instance, never upserting.
    async fn create_agent_instance(
        &self,
        _record: &AgentInstanceRecord,
    ) -> Result<AgentInstanceRecord> {
        Err(AgentInstanceStoreError::Unsupported.into())
    }

    /// Read one exact logical instance without owner/workspace fallback.
    async fn load_agent_instance(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _instance_id: &str,
    ) -> Result<Option<AgentInstanceRecord>> {
        Err(AgentInstanceStoreError::Unsupported.into())
    }

    /// List one authorized workspace's logical instances.
    async fn list_agent_instances(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
    ) -> Result<Vec<AgentInstanceRecord>> {
        Err(AgentInstanceStoreError::Unsupported.into())
    }

    /// Atomically replace bounded state only while revision and epoch match.
    /// False means a caller must reload and reconcile before retrying.
    async fn compare_and_swap_agent_instance(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _expected_revision: u64,
        _expected_epoch: u64,
        _next: &AgentInstanceRecord,
    ) -> Result<bool> {
        Err(AgentInstanceStoreError::Unsupported.into())
    }

    /// Create a validated template in the verified owner's partition.
    async fn create_presentation(
        &self,
        owner_id: &str,
        draft: &PresentationDraft,
    ) -> Result<Presentation>;

    /// Read one template without exposing records belonging to another owner.
    async fn get_presentation(&self, owner_id: &str, id: &str) -> Result<Option<Presentation>>;

    /// List the owner's templates, including disabled records for management.
    async fn list_presentations(&self, owner_id: &str) -> Result<Vec<Presentation>>;

    /// Atomically replace editable content only at the expected revision.
    async fn update_presentation(
        &self,
        owner_id: &str,
        id: &str,
        expected_revision: u64,
        draft: &PresentationDraft,
    ) -> Result<Presentation>;

    /// Atomically delete only the requested owner's expected record revision.
    async fn delete_presentation(
        &self,
        owner_id: &str,
        id: &str,
        expected_revision: u64,
    ) -> Result<()>;

    // Session Management
    async fn save_session(&self, session: &Session) -> Result<()>;
    async fn load_session(&self, owner_id: &str, id: &str) -> Result<Option<Session>>;

    /// List every session whose serialized owner matches `owner_id`.
    ///
    /// This is an authenticated owner snapshot. Implementations must not
    /// broaden it with a legacy or all-tenant fallback.
    async fn list_sessions(&self, _owner_id: &str) -> Result<Vec<Session>> {
        anyhow::bail!("owner_scoped_session_listing_unsupported")
    }

    /// Delete a session and all owned downstream data (conversation policy,
    /// checkpoints, cost ledger entries, tool admission evidence) for the
    /// verified owner. Returns true when a session row existed.
    async fn delete_session(&self, owner_id: &str, id: &str) -> Result<bool>;

    /// List sessions whose last activity predates the cutoff. Default returns
    /// an empty vec; providers that persist sessions override this for TTL
    /// sweeps.
    async fn list_expired_sessions(
        &self,
        _cutoff: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }

    /// Subscribe to successful local durable session commits. This is a wakeup,
    /// not a state transfer: receivers reread the authoritative expiry after a
    /// generation change. `None` means this provider has no durable signal.
    fn subscribe_persisted_session_changes(&self) -> Option<tokio::sync::watch::Receiver<u64>> {
        None
    }

    /// Return the earliest future persisted idle deadline under the supplied
    /// policy. The deadline schedules a one-shot wakeup; it does not prove a
    /// session is removable until the deletion transaction rechecks it.
    async fn next_persisted_session_expiry(
        &self,
        _idle_timeout: chrono::Duration,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        Ok(None)
    }

    /// Save or replace a conversation-scoped chat policy.
    async fn save_conversation_policy(&self, record: &ConversationPolicyRecord) -> Result<()>;

    /// Load a conversation-scoped chat policy.
    async fn load_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<Option<ConversationPolicyRecord>>;

    /// Conditionally write in the principal namespace. None expects no record;
    /// an existing policy must still equal the complete supplied baseline.
    async fn save_principal_conversation_policy(
        &self,
        record: &ConversationPolicyRecord,
        expected: Option<&crate::uar::domain::policy::RunPolicy>,
    ) -> Result<bool>;

    /// Read only the verified principal namespace, without legacy fallback.
    async fn load_principal_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<Option<ConversationPolicyRecord>>;

    /// Delete a conversation-scoped chat policy.
    async fn delete_conversation_policy(&self, owner_id: &str, conversation_id: &str)
    -> Result<()>;

    /// Save or replace one verified user's prompt-caching preferences.
    async fn save_user_prompt_caching_settings(
        &self,
        settings: &UserPromptCachingSettings,
    ) -> Result<()>;

    /// Load prompt-caching preferences by the collision-safe verified principal key.
    async fn load_user_prompt_caching_settings(
        &self,
        principal_id: &str,
    ) -> Result<Option<UserPromptCachingSettings>>;

    // Skill Management
    async fn save_skill(&self, skill: &Skill, embedding: &[f32]) -> Result<()>;
    /// Insert one skill only when its ID is not already durable.
    ///
    /// Returns `false` on an existing ID without changing that existing row.
    /// Unlike [`Self::save_skill`], this operation is the collision-safe
    /// create path for remote skill-content import.
    async fn create_skill_if_absent(&self, _skill: &Skill, _embedding: &[f32]) -> Result<bool> {
        anyhow::bail!("skill_create_if_absent_unsupported")
    }
    /// Change only existing skill selection fields; preserve every other stored field.
    /// Returns false when no durable skill row exists.
    async fn update_skill_selection(
        &self,
        skill_id: &str,
        enabled: bool,
        scoped_config: &[crate::uar::domain::skills::ScopedSkillConfig],
    ) -> Result<bool>;
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

    /// Save or update a knowledge base definition. The host supplies a verified
    /// principal storage key, or `anonymous` at the explicit anonymous boundary.
    /// Authenticated rows live in storage disjoint from all legacy subjects.
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

    /// Identity-aware retrieval; concrete durable stores fence one read snapshot.
    async fn search_knowledge_scoped_in_space(&self, owner_id: &str, kb_ids: &[&str],
        query_vec: &[f32], limit: usize, min_score: f32,
        embedding: &crate::uar::domain::knowledge::EmbeddingFingerprint) -> Result<Vec<KnowledgeMatch>> {
        let mut before = Vec::new();
        for id in kb_ids {
            let kb = self.get_knowledge_base(owner_id, id).await?
                .ok_or_else(|| anyhow::anyhow!("knowledge base unavailable"))?;
            anyhow::ensure!(!kb.regeneration_unavailable(), "knowledge regeneration is incomplete");
            kb.ensure_embedding_space(embedding)?;
            before.push(((*id).to_owned(), kb.config.indexed_embedding, kb.config.regeneration));
        }
        let result = self.search_knowledge_scoped(owner_id, kb_ids, query_vec, limit, min_score).await?;
        for (id, indexed, regeneration) in before {
            let after = self.get_knowledge_base(owner_id, &id).await?
                .ok_or_else(|| anyhow::anyhow!("knowledge base was revoked during retrieval"))?;
            anyhow::ensure!(after.config.indexed_embedding == indexed && after.config.regeneration == regeneration,
                "knowledge generation changed during retrieval");
        }
        Ok(result)
    }

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

    /// Insert a fresh root. Existing IDs/paths are errors, never upserts.
    /// The owner argument must come from the verified host context.
    async fn create_agent_root(
        &self,
        owner_id: &str,
        thread: &AgentThread,
    ) -> Result<PersistedAgentThread>;

    /// Atomically insert a pending child and its immutable edge. Both persisted
    /// parent and root must belong to the owner/tree and still have a live turn.
    /// An interrupted/error response is not proof that nothing committed: the
    /// caller must reconcile the exact child ID before releasing its reservation.
    async fn create_agent_child(
        &self,
        owner_id: &str,
        thread: &AgentThread,
        edge: &AgentEdge,
    ) -> Result<PersistedAgentThread>;

    /// Load only from this owner's namespace. Missing records return None;
    /// corrupt records or backend failures return errors, never empty success.
    async fn load_agent_thread(
        &self,
        owner_id: &str,
        thread_id: &str,
    ) -> Result<Option<PersistedAgentThread>>;

    /// Update mutable turn state with compare-and-swap on the storage revision.
    /// Lineage, artifact, creation time, and terminal outcomes cannot be rewritten.
    /// A new authorized turn may replace a terminal outcome with a new run ID.
    async fn update_agent_thread(
        &self,
        owner_id: &str,
        expected_revision: u64,
        thread: &AgentThread,
    ) -> Result<PersistedAgentThread>;

    /// List one owner's root-run tree, ordered by canonical path then thread ID
    /// using Rust string ordering (independent of database locale/collation).
    async fn list_agent_threads(
        &self,
        owner_id: &str,
        root_run_id: &str,
    ) -> Result<Vec<PersistedAgentThread>>;

    /// List immutable edges in canonical path/child-ID order. No deletion API is
    /// exposed: lineage remains available for recovery and lifetime accounting.
    async fn list_agent_edges(&self, owner_id: &str, root_run_id: &str) -> Result<Vec<AgentEdge>>;

    /// Append one immutable pre-format tool result. A repeated call ID is
    /// idempotent only when its canonical payload identity is unchanged.
    async fn save_canonical_tool_receipt(
        &self,
        receipt: &CanonicalToolReceipt,
    ) -> Result<CanonicalToolReceipt>;

    /// List one owner's canonical receipts for a run in stable execution order.
    async fn list_canonical_tool_receipts(
        &self,
        owner_id: &str,
        run_id: &str,
    ) -> Result<Vec<CanonicalToolReceipt>>;

    /// Whether approval history survives a process restart on this provider.
    fn supports_durable_approvals(&self) -> bool { false }

    async fn create_approval_record(&self, _record: &approval_decisions::ApprovalRecord) -> Result<()> {
        anyhow::bail!("Approval persistence is unavailable")
    }

    /// CAS one pending challenge. Human resolution checks the original deadline
    /// at the storage serialization point; a late intent commits Expired and
    /// returns false rather than authorizing Approved or replacing it with Denied.
    async fn transition_approval_record(&self, _before: &approval_decisions::ApprovalRecord, _after: &approval_decisions::ApprovalRecord) -> Result<bool> {
        anyhow::bail!("Approval persistence is unavailable")
    }

    async fn list_approval_records(&self, _owner: &str, _run: &str) -> Result<Vec<approval_decisions::ApprovalRecord>> {
        anyhow::bail!("Approval persistence is unavailable")
    }

    /// Append one sanitized lifecycle fact for an exact tool invocation.
    /// Implementations must be idempotent for an identical state and reject a
    /// conflicting duplicate. This history never reconstructs authority.
    async fn save_tool_admission_evidence(
        &self,
        _evidence: &ToolAdmissionEvidence,
    ) -> Result<ToolAdmissionEvidence> {
        anyhow::bail!("Tool admission evidence persistence is unavailable")
    }

    /// List one owner's admission evidence in persisted time order.
    async fn list_tool_admission_evidence(
        &self,
        _owner_id: &str,
    ) -> Result<Vec<ToolAdmissionEvidence>> {
        Ok(Vec::new())
    }

    async fn save_agent(&self, agent: &crate::uar::domain::artifact::AgentArtifact) -> Result<()>;
    /// Persist a merged agent only while the stored artifact matches the read
    /// baseline. A conflict returns false without overwriting another editor.
    async fn save_agent_if_unchanged(
        &self,
        expected: &crate::uar::domain::artifact::AgentArtifact,
        updated: &crate::uar::domain::artifact::AgentArtifact,
    ) -> Result<bool>;
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

    /// Change only global Presentation intent if the full saved policy still
    /// matches the supplied baseline. Missing or changed records return false.
    async fn compare_and_swap_global_presentation_policy(
        &self,
        expected: &serde_json::Value,
        selection: &crate::uar::domain::policy::ResourceSelection,
    ) -> Result<bool>;

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

    /// Trusted containment registrations never imply exclusion by themselves.
    async fn register_supervisor_domain(&self, _registration: &crate::uar::runtime::supervisor::SupervisorRegistration) -> Result<()> { Err(crate::uar::runtime::supervisor::SupervisorError::Unsupported.into()) }
    async fn load_supervisor_domain(&self, _epoch: &str) -> Result<Option<crate::uar::runtime::supervisor::SupervisorRegistration>> { Err(crate::uar::runtime::supervisor::SupervisorError::Unsupported.into()) }
    async fn create_supervisor_challenge(&self, _challenge: &crate::uar::runtime::supervisor::SupervisorChallenge) -> Result<()> { Err(crate::uar::runtime::supervisor::SupervisorError::Unsupported.into()) }
    async fn load_supervisor_challenge(&self, _id: &str) -> Result<Option<crate::uar::runtime::supervisor::SupervisorChallenge>> { Err(crate::uar::runtime::supervisor::SupervisorError::Unsupported.into()) }
    async fn commit_supervisor_checkpoint_exclusion(&self, _proof: &crate::uar::runtime::supervisor::VerifiedSupervisorExclusion) -> Result<()> { Err(crate::uar::runtime::supervisor::SupervisorError::Unsupported.into()) }

    /// True only for an atomic protected envelope, durable claim and exclusion store.
    fn supports_durable_checkpoint_resume(&self) -> bool { false }

    /// Atomically create immutable checkpoint bytes and complete owner metadata.
    async fn save_resumable_checkpoint(&self, _envelope: &checkpoint_resume::CheckpointResumeEnvelope) -> Result<()> {
        Err(checkpoint_resume::CheckpointResumeError::Unsupported.into())
    }

    /// Load exact principal-scoped resume evidence before any live-run lookup.
    async fn load_checkpoint_resume(&self, _owner_key: &str, _run_id: &str, _checkpoint_id: &str) -> Result<Option<checkpoint_resume::CheckpointResumeEnvelope>> {
        Err(checkpoint_resume::CheckpointResumeError::Unsupported.into())
    }

    /// List protected checkpoint envelopes for one complete verified principal.
    async fn list_checkpoint_resumes(&self, _owner_key: &str, _run_id: &str) -> Result<Vec<checkpoint_resume::CheckpointResumeEnvelope>> {
        Err(checkpoint_resume::CheckpointResumeError::Unsupported.into())
    }

    /// Claim one source run and exact revision only after trusted predecessor exclusion.
    async fn claim_checkpoint_resume(&self, _envelope: &checkpoint_resume::CheckpointResumeEnvelope, _claim: &checkpoint_resume::CheckpointContinuationClaim) -> Result<checkpoint_resume::CheckpointClaimOutcome> {
        Err(checkpoint_resume::CheckpointResumeError::Unsupported.into())
    }

    /// Trusted shutdown commits this only after actual producer/descendant joins
    /// and effect reconciliation. No timeout or lost connection proves exclusion.
    async fn exclude_checkpoint_runtime_epoch(&self, _runtime_epoch: &str) -> Result<()> {
        Err(checkpoint_resume::CheckpointResumeError::Unsupported.into())
    }

    /// Persist a graph execution checkpoint.
    async fn save_checkpoint(
        &self,
        _checkpoint: &crate::uar::runtime::checkpoint::Checkpoint,
    ) -> Result<()> {
        Err(checkpoint_resume::CheckpointResumeError::Unsupported.into())
    }

    /// Load a single checkpoint by its unique ID.
    async fn load_checkpoint(
        &self,
        _id: &str,
    ) -> Result<Option<crate::uar::runtime::checkpoint::Checkpoint>> {
        Err(checkpoint_resume::CheckpointResumeError::Unsupported.into())
    }

    /// List all checkpoints recorded for a given run, ordered by creation time.
    async fn list_checkpoints(
        &self,
        _run_id: &str,
    ) -> Result<Vec<crate::uar::runtime::checkpoint::Checkpoint>> {
        Err(checkpoint_resume::CheckpointResumeError::Unsupported.into())
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
