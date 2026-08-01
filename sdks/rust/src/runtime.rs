//! Direct, transport-independent UAR embedding API.
//!
//! Enable `embedded` to link the UAR application kernel into the host process.
//! This module never opens a socket. The optional `server` feature exposes an
//! explicit server entry point for hosts that intentionally want HTTP.

use std::sync::Arc;

#[cfg(feature = "embedded")]
use tokio::sync::broadcast;
#[cfg(feature = "embedded")]
// Aliased so the admin methods below read as `uar_admin::skills::list(...)`,
// making it obvious at each call site that the SDK delegates to the shared
// transport-free services rather than reimplementing them.
use universal_agent_runtime::uar::admin as uar_admin;
use universal_agent_runtime::uar::domain::knowledge as uar_knowledge;
use universal_agent_runtime::uar::domain::memory as uar_memory;
// Re-exported: the memory admin API takes a `UserContext`, so a consumer would
// otherwise have to depend on the UAR crate directly just to call the SDK.
pub use universal_agent_runtime::uar::security::claims::{UserClaims, UserContext};
use universal_agent_runtime::uar::a2ui::schema as uar_a2ui;
use universal_agent_runtime::uar::domain::skills as uar_skills;
use universal_agent_runtime::{
    config::LlmConfig,
    embedded::{EmbeddedRuntime, EmbeddedRuntimeBuilder},
    llm::{LlmDriver, ProviderConfig},
    mcp::registry::McpRegistry,
    uar::{
        a2ui::registry::A2uiRegistry,
        domain::{
            agent_store,
            artifact::AgentArtifact,
            events::MemoryItem,
            policy::{ConversationPolicyRecord, RunPolicy},
        },
        persistence::PersistenceLayer,
        rag::embeddings::EmbeddingBackend,
        runtime::{
            manager::{EffectiveConfig, RunManager, SeedMessage, StreamEvent},
            native_skill::NativeSkillRegistry,
        },
        settings::schema::{SettingsType, SettingsWithMeta},
    },
};

/// A point-in-time view of the embedded runtime's settings.
///
/// Mirrors the shape the HTTP admin surface exposes: the current setting values
/// (with their transient source/drift metadata) and the registered setting types
/// (JSON Schemas) that describe and validate them.
#[cfg(feature = "embedded")]
#[derive(Debug, Clone)]
pub struct SettingsSnapshot {
    /// Current setting values with their transient metadata.
    pub values: Vec<SettingsWithMeta>,
    /// Registered setting types (JSON Schema per namespace).
    pub types: Vec<SettingsType>,
}

use crate::error::{Error, Result};

/// A fully initialized in-process UAR runtime.
///
/// Successful construction is the readiness guarantee: accessors return live
/// components rather than the previous `Option<AppState>` placeholder.
#[derive(Debug, Clone)]
pub struct Runtime {
    #[cfg(feature = "embedded")]
    inner: Arc<EmbeddedRuntime>,
}

impl Runtime {
    /// Create a new direct-runtime builder.
    #[must_use]
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::default()
    }

    /// Access the complete UAR application kernel.
    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn inner(&self) -> &EmbeddedRuntime {
        &self.inner
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn orchestrator(&self) -> Arc<universal_agent_runtime::llm::orchestrator::Orchestrator> {
        self.inner.orchestrator()
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn run_manager(&self) -> Arc<RunManager> {
        self.inner.run_manager()
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn provider_registry(&self) -> Arc<universal_agent_runtime::llm::ProviderRegistry> {
        self.inner.provider_registry()
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn persistence(&self) -> Arc<dyn PersistenceLayer> {
        self.inner.persistence()
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn vector_matcher(
        &self,
    ) -> Arc<universal_agent_runtime::uar::runtime::matching::VectorMatcher> {
        self.inner.vector_matcher()
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn skill_service(
        &self,
    ) -> Arc<universal_agent_runtime::uar::runtime::skills::service::SkillService> {
        self.inner.skill_service()
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn native_skills(&self) -> Arc<NativeSkillRegistry> {
        self.inner.native_skills()
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn a2ui_registry(&self) -> Arc<A2uiRegistry> {
        self.inner.a2ui_registry()
    }

    /// Start a run from an already-resolved UAR agent artifact.
    #[cfg(feature = "embedded")]
    pub async fn start_run(
        &self,
        artifact: AgentArtifact,
        input: impl Into<String>,
        conversation_id: Option<String>,
        user_id: Option<String>,
        memory: Vec<MemoryItem>,
    ) -> String {
        self.inner
            .run_manager()
            .start_run(artifact, input.into(), conversation_id, user_id, memory)
            .await
    }

    /// Start a run, seeding an empty (cold-started) session with `seed_history`
    /// so the model receives prior turns. A session that already holds messages
    /// is not re-seeded. See [`SeedMessage`].
    #[cfg(feature = "embedded")]
    pub async fn start_run_with_history(
        &self,
        artifact: AgentArtifact,
        input: impl Into<String>,
        conversation_id: Option<String>,
        user_id: Option<String>,
        memory: Vec<MemoryItem>,
        seed_history: Vec<SeedMessage>,
    ) -> String {
        self.inner
            .run_manager()
            .start_run_with_history(
                artifact,
                input.into(),
                conversation_id,
                user_id,
                memory,
                seed_history,
            )
            .await
    }

    /// Resolve a persisted agent by id and start a run.
    #[cfg(feature = "embedded")]
    pub async fn start_agent_run(
        &self,
        agent_id: &str,
        input: impl Into<String>,
        conversation_id: Option<String>,
        user_id: Option<String>,
        memory: Vec<MemoryItem>,
    ) -> Result<String> {
        let artifact = self
            .inner
            .persistence()
            .load_agent(agent_id)
            .await
            .map_err(runtime_error)?
            .ok_or_else(|| Error::Runtime(format!("UAR agent '{agent_id}' was not found")))?;
        Ok(self
            .start_run(artifact, input, conversation_id, user_id, memory)
            .await)
    }

    /// Subscribe to canonical run events without SSE serialization.
    #[cfg(feature = "embedded")]
    pub async fn subscribe(&self, run_id: &str) -> Result<broadcast::Receiver<StreamEvent>> {
        self.inner
            .run_manager()
            .subscribe(run_id)
            .await
            .ok_or_else(|| Error::Runtime(format!("UAR run '{run_id}' was not found")))
    }

    /// Replay canonical run events after an optional event id.
    #[cfg(feature = "embedded")]
    pub async fn history_since(
        &self,
        run_id: &str,
        last_event_id: Option<u64>,
    ) -> Result<Vec<StreamEvent>> {
        self.inner
            .run_manager()
            .history_since(run_id, last_event_id)
            .await
            .ok_or_else(|| Error::Runtime(format!("UAR run '{run_id}' was not found")))
    }

    #[cfg(feature = "embedded")]
    pub async fn cancel_run(&self, run_id: &str) -> bool {
        self.inner.run_manager().cancel_run(run_id).await
    }

    #[cfg(feature = "embedded")]
    pub async fn resolve_tool_approval(&self, run_id: &str, approved: bool) -> bool {
        self.inner
            .run_manager()
            .resolve_approval(run_id, approved)
            .await
    }

    /// Continue a conversation from a user interaction with an A2UI surface.
    #[cfg(feature = "embedded")]
    pub async fn continue_with_a2ui_interaction(
        &self,
        run_id: &str,
        interaction: serde_json::Value,
    ) -> Result<String> {
        self.inner
            .run_manager()
            .continue_with_interaction(run_id, interaction)
            .await
            .map_err(Error::Runtime)
    }

    // =====================================================================
    // Embedded administration surface
    //
    // Typed settings and agent-definition management backed by the runtime's
    // own SettingsManager + persistence, so an embedded host can run its
    // control plane with no HTTP service.
    // =====================================================================

    /// Read a typed setting by key (e.g. `run_policy.global`).
    ///
    /// Returns the raw JSON value, or `None` when the setting is unset.
    ///
    /// # Errors
    ///
    /// Currently infallible for reads; returns `Result` for forward
    /// compatibility with validating backends.
    #[cfg(feature = "embedded")]
    pub async fn get_setting(&self, key: &str) -> Result<Option<serde_json::Value>> {
        Ok(self.inner.settings_manager().get_value(key).await)
    }

    /// Write a typed setting by key (e.g. `run_policy.global`).
    ///
    /// The value is validated against the setting type's JSON Schema before it
    /// is persisted. The next resolved run observes the new value.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is unknown or the value fails schema
    /// validation.
    #[cfg(feature = "embedded")]
    pub async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<()> {
        self.inner
            .settings_manager()
            .set_value(key, value)
            .await
            .map_err(runtime_error)
    }

    /// Snapshot the current settings values and their registered types.
    ///
    /// # Errors
    ///
    /// Returns an error if the registered types cannot be listed.
    #[cfg(feature = "embedded")]
    pub async fn settings_snapshot(&self) -> Result<SettingsSnapshot> {
        let settings_manager = self.inner.settings_manager();
        let values = settings_manager.list_all_with_meta().await;
        let types = settings_manager.list_types().await.map_err(runtime_error)?;
        Ok(SettingsSnapshot { values, types })
    }

    /// List persisted agent definitions.
    ///
    /// # Errors
    ///
    /// Returns an error if the persistence read fails.
    #[cfg(feature = "embedded")]
    pub async fn list_agents(&self) -> Result<Vec<AgentArtifact>> {
        agent_store::list_agents(self.inner.persistence().as_ref())
            .await
            .map_err(runtime_error)
    }

    /// Load a single agent definition by id.
    ///
    /// # Errors
    ///
    /// Returns an error if the persistence read fails.
    #[cfg(feature = "embedded")]
    pub async fn get_agent(&self, id: &str) -> Result<Option<AgentArtifact>> {
        agent_store::get_agent(self.inner.persistence().as_ref(), id)
            .await
            .map_err(runtime_error)
    }

    /// Insert or replace an agent definition as provided (id and kind preserved).
    ///
    /// # Errors
    ///
    /// Returns an error if the persistence write fails.
    #[cfg(feature = "embedded")]
    pub async fn upsert_agent(&self, agent: AgentArtifact) -> Result<()> {
        agent_store::upsert_agent(self.inner.persistence().as_ref(), &agent)
            .await
            .map_err(runtime_error)
    }

    /// Delete an agent definition by id.
    ///
    /// # Errors
    ///
    /// Returns an error for built-in agents (which cannot be deleted) or if the
    /// persistence delete fails.
    #[cfg(feature = "embedded")]
    pub async fn delete_agent(&self, id: &str) -> Result<()> {
        agent_store::delete_agent(self.inner.persistence().as_ref(), id)
            .await
            .map_err(runtime_error)
    }

    // =========================================================================
    // Resource administration
    //
    // These exist so an EMBEDDED container (mobile, macOS) can administer the
    // same registries a remote one can. Before this, the admin logic lived only
    // in the axum handlers, so an embedded host had no listener to call and its
    // control plane had to report skills/MCP/knowledge/memory as unavailable.
    //
    // Every method delegates to `uar::admin::*`, the transport-free services the
    // HTTP layer also calls — one implementation, two containers.
    // =========================================================================

    /// List every skill known to this runtime.
    ///
    /// # Errors
    /// Returns an error if the persistence read fails.
    #[cfg(feature = "embedded")]
    pub async fn list_skills(&self) -> Result<Vec<uar_skills::Skill>> {
        uar_admin::skills::list(&self.inner.persistence())
            .await
            .map_err(runtime_error)
    }

    /// Persist a skill together with the embedding used for semantic matching.
    ///
    /// The embedding is supplied by the caller because the embedding backend is
    /// a runtime concern: an embedded device may use a different one than a
    /// server, and the admin layer has no business choosing.
    ///
    /// # Errors
    /// Returns an error if the persistence write fails.
    #[cfg(feature = "embedded")]
    pub async fn save_skill(&self, skill: &uar_skills::Skill, embedding: &[f32]) -> Result<()> {
        uar_admin::skills::save(&self.inner.persistence(), skill, embedding)
            .await
            .map_err(runtime_error)
    }

    /// Delete a skill by id.
    ///
    /// # Errors
    /// Returns an error if the persistence write fails.
    #[cfg(feature = "embedded")]
    pub async fn delete_skill(&self, id: &str) -> Result<()> {
        uar_admin::skills::delete(&self.inner.persistence(), id)
            .await
            .map_err(runtime_error)
    }

    /// List every knowledge base.
    ///
    /// # Errors
    /// Returns an error if the persistence read fails.
    #[cfg(feature = "embedded")]
    pub async fn list_knowledge_bases(&self) -> Result<Vec<uar_knowledge::KnowledgeBase>> {
        uar_admin::knowledge::list(&self.inner.persistence())
            .await
            .map_err(runtime_error)
    }

    /// Load one knowledge base by id.
    ///
    /// # Errors
    /// Returns an error if the persistence read fails.
    #[cfg(feature = "embedded")]
    pub async fn get_knowledge_base(
        &self,
        id: &str,
    ) -> Result<Option<uar_knowledge::KnowledgeBase>> {
        uar_admin::knowledge::get(&self.inner.persistence(), id)
            .await
            .map_err(runtime_error)
    }

    /// Create or replace a knowledge base.
    ///
    /// # Errors
    /// Returns an error if the persistence write fails.
    #[cfg(feature = "embedded")]
    pub async fn save_knowledge_base(&self, kb: &uar_knowledge::KnowledgeBase) -> Result<()> {
        uar_admin::knowledge::save(&self.inner.persistence(), kb)
            .await
            .map_err(runtime_error)
    }

    /// Delete a knowledge base by id.
    ///
    /// # Errors
    /// Returns an error if the persistence write fails.
    #[cfg(feature = "embedded")]
    pub async fn delete_knowledge_base(&self, id: &str) -> Result<()> {
        uar_admin::knowledge::delete(&self.inner.persistence(), id)
            .await
            .map_err(runtime_error)
    }

    // -------------------------------------------------------------------------
    // Memory
    //
    // Full CRUD, not search-only. A user must be able to see what the runtime
    // remembered about them and correct or remove it, and that has to hold on
    // every deployment — a phone with no server is exactly where a wrong or
    // stale memory is least reviewable, not most.
    //
    // Every call goes through `MemoryService` rather than `PersistenceLayer`.
    // That is not a stylistic choice: `PersistenceLayer::save_memory` and
    // `search_memory` are documented NO-OP stubs on `SurrealDbProvider` that
    // return `Ok(())` / `vec![]`. Routing writes there would compile, run, and
    // silently discard the data. The service also owns EMBEDDING, so a row
    // written around it is invisible to every later semantic search.
    // -------------------------------------------------------------------------

    /// Memories visible to a user, optionally narrowed to an agent or session.
    ///
    /// # Errors
    /// Returns an error if the runtime has no memory service, or the read fails.
    #[cfg(feature = "embedded")]
    pub async fn list_memories(
        &self,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<uar_memory::Memory>> {
        uar_admin::memory::list(
            self.inner.memory_service().as_ref(),
            user_ctx,
            agent_id,
            session_id,
        )
        .await
        .map_err(runtime_error)
    }

    /// Load one memory by id.
    ///
    /// # Errors
    /// Returns an error if the runtime has no memory service, or the read fails.
    #[cfg(feature = "embedded")]
    pub async fn get_memory(&self, id: &str) -> Result<Option<uar_memory::Memory>> {
        uar_admin::memory::get(self.inner.memory_service().as_ref(), id)
            .await
            .map_err(runtime_error)
    }

    /// Add a memory.
    ///
    /// # Errors
    /// Returns an error if the runtime has no memory service, or the write fails.
    #[cfg(feature = "embedded")]
    pub async fn add_memory(
        &self,
        content: impl Into<String>,
        scope: uar_memory::MemoryScope,
        memory_type: uar_memory::MemoryType,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<uar_memory::Memory> {
        uar_admin::memory::add(
            self.inner.memory_service().as_ref(),
            content,
            scope,
            memory_type,
            user_ctx,
            agent_id,
            session_id,
        )
        .await
        .map_err(runtime_error)
    }

    /// Replace a memory's content. The service records a history entry, so an
    /// edit is auditable rather than destructive.
    ///
    /// # Errors
    /// Returns an error if the runtime has no memory service, or the write fails.
    #[cfg(feature = "embedded")]
    pub async fn update_memory(&self, id: &str, content: String) -> Result<uar_memory::Memory> {
        uar_admin::memory::update(self.inner.memory_service().as_ref(), id, content)
            .await
            .map_err(runtime_error)
    }

    /// Delete a memory by id.
    ///
    /// # Errors
    /// Returns an error if the runtime has no memory service, or the write fails.
    #[cfg(feature = "embedded")]
    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        uar_admin::memory::delete(self.inner.memory_service().as_ref(), id)
            .await
            .map_err(runtime_error)
    }

    /// Semantic search over a user's memories.
    ///
    /// Takes a TEXT query, not a precomputed vector: the service owns the
    /// embedding backend, and a caller that had to embed the query itself would
    /// have to match the store's model exactly or get silently wrong results.
    ///
    /// # Errors
    /// Returns an error if the runtime has no memory service, or the read fails.
    #[cfg(feature = "embedded")]
    pub async fn search_memory(
        &self,
        query: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
        categories: Option<&[String]>,
    ) -> Result<Vec<uar_memory::Memory>> {
        uar_admin::memory::search(
            self.inner.memory_service().as_ref(),
            query,
            user_ctx,
            agent_id,
            session_id,
            limit,
            categories,
        )
        .await
        .map_err(runtime_error)
    }

    /// List configured MCP servers.
    ///
    /// Reads the database. Config files seed it once at boot; the store is
    /// authoritative afterwards, which is what lets a runtime change take
    /// effect without a restart or a file-polling loop.
    #[cfg(feature = "embedded")]
    pub async fn list_mcp_servers(
        &self,
    ) -> std::collections::HashMap<String, uar_admin::mcp::StoredMcpServer> {
        uar_admin::mcp::list(&self.inner.mcp(), Some(&self.inner.settings_manager())).await
    }

    /// Save an MCP server.
    ///
    /// Returns a [`SaveResult`] rather than `()` because the outcome is not
    /// always "applied": editing an ALREADY-CONNECTED server is stored durably
    /// but deferred to the next session, so a live connection is not torn down
    /// mid-flight. Callers should surface `Deferred` rather than implying the
    /// change took effect.
    ///
    /// # Errors
    /// Returns an error if settings storage is unavailable or the write fails.
    #[cfg(feature = "embedded")]
    pub async fn save_mcp_server(
        &self,
        name: String,
        server: uar_admin::mcp::StoredMcpServer,
    ) -> Result<uar_admin::mcp::SaveResult> {
        uar_admin::mcp::save(
            &self.inner.mcp(),
            Some(&self.inner.settings_manager()),
            name,
            server,
        )
        .await
        .map_err(runtime_error)
    }

    /// Every tool this runtime can dispatch, from both backends.
    ///
    /// Each entry carries its `source`, because native tools work with no
    /// network while MCP tools need a reachable server — a client that cannot
    /// tell them apart cannot explain why some tools survive going offline.
    #[cfg(feature = "embedded")]
    pub async fn list_tools(&self) -> Vec<uar_admin::tools::ToolEntry> {
        uar_admin::tools::list(&self.inner.native_skills(), &self.inner.mcp()).await
    }

    /// A2UI artifact schemas this runtime can render.
    ///
    /// Read-only: registering a schema changes how every future artifact
    /// renders, so that belongs to composition rather than an admin call.
    #[cfg(feature = "embedded")]
    pub async fn list_a2ui_schemas(&self) -> Vec<uar_a2ui::ArtifactSchema> {
        uar_admin::a2ui::list(&self.inner.a2ui_registry()).await
    }

    /// Remove an MCP server from the store and from the live registry.
    ///
    /// Unlike an edit, a removal applies immediately even when connected: the
    /// caller's intent is that the server stop being used.
    ///
    /// # Errors
    /// Returns an error if settings storage is unavailable or the write fails.
    #[cfg(feature = "embedded")]
    pub async fn delete_mcp_server(
        &self,
        name: &str,
    ) -> Result<uar_admin::mcp::SaveResult> {
        uar_admin::mcp::delete(&self.inner.mcp(), Some(&self.inner.settings_manager()), name)
            .await
            .map_err(runtime_error)
    }

    /// Save or replace the per-conversation run policy. This is the third scope
    /// (Conversation) of the Global → Agent → Conversation → Turn precedence, so
    /// a model set here overrides the agent and global defaults for that
    /// conversation only.
    ///
    /// # Errors
    ///
    /// Returns an error if the persistence write fails.
    #[cfg(feature = "embedded")]
    pub async fn save_conversation_policy(
        &self,
        conversation_id: &str,
        policy: RunPolicy,
    ) -> Result<ConversationPolicyRecord> {
        let record = ConversationPolicyRecord::new(conversation_id.to_string(), policy);
        self.inner
            .persistence()
            .save_conversation_policy(&record)
            .await
            .map_err(runtime_error)?;
        Ok(record)
    }

    /// Load the per-conversation run policy, if one has been saved.
    ///
    /// # Errors
    ///
    /// Returns an error if the persistence read fails.
    #[cfg(feature = "embedded")]
    pub async fn get_conversation_policy(
        &self,
        conversation_id: &str,
    ) -> Result<Option<ConversationPolicyRecord>> {
        self.inner
            .persistence()
            .load_conversation_policy(conversation_id)
            .await
            .map_err(runtime_error)
    }

    /// Delete the per-conversation run policy, reverting the conversation to the
    /// agent and global scopes.
    ///
    /// # Errors
    ///
    /// Returns an error if the persistence delete fails.
    #[cfg(feature = "embedded")]
    pub async fn delete_conversation_policy(&self, conversation_id: &str) -> Result<()> {
        self.inner
            .persistence()
            .delete_conversation_policy(conversation_id)
            .await
            .map_err(runtime_error)
    }

    /// Resolve the effective configuration for a conversation: the agent it
    /// resolves to, the stored requested policy (if any), and the effective run
    /// policy after full-precedence resolution and model backfill. Mirrors the
    /// service path's `GET /conversations/{id}/effective-config`.
    #[cfg(feature = "embedded")]
    pub async fn effective_config(&self, conversation_id: &str) -> EffectiveConfig {
        self.inner
            .run_manager()
            .effective_config(conversation_id)
            .await
    }

    /// Explicitly start UAR's HTTP server. This is not part of embedded mode
    /// and is unavailable unless the separate `server` feature is enabled.
    #[cfg(feature = "server")]
    pub async fn start_server(
        config_manager: Arc<universal_agent_runtime::config_manager::ConfigManager>,
    ) -> Result<()> {
        universal_agent_runtime::server::start_server(config_manager)
            .await
            .map_err(runtime_error)
    }
}

/// Builder for the direct runtime.
#[derive(Default)]
pub struct RuntimeBuilder {
    #[cfg(feature = "embedded")]
    llm_config: Option<LlmConfig>,
    #[cfg(feature = "embedded")]
    driver: Option<Arc<dyn LlmDriver>>,
    #[cfg(feature = "embedded")]
    provider: Option<ProviderConfig>,
    #[cfg(feature = "embedded")]
    persistence: Option<Arc<dyn PersistenceLayer>>,
    #[cfg(feature = "embedded")]
    embedding_backend: Option<Arc<dyn EmbeddingBackend>>,
    #[cfg(feature = "embedded")]
    mcp: Option<Arc<McpRegistry>>,
    #[cfg(feature = "embedded")]
    native_skills: Option<Arc<NativeSkillRegistry>>,
    #[cfg(feature = "embedded")]
    a2ui_registry: Option<Arc<A2uiRegistry>>,
    memory_service: Option<Arc<universal_agent_runtime::uar::memory::service::MemoryService>>,
    #[cfg(feature = "embedded")]
    seed_defaults: Option<bool>,
    #[cfg(feature = "embedded")]
    vector_threshold: Option<f32>,
}

impl RuntimeBuilder {
    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn llm_config(mut self, config: LlmConfig) -> Self {
        self.llm_config = Some(config);
        self
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn local_provider(mut self, driver: Arc<dyn LlmDriver>, provider: ProviderConfig) -> Self {
        self.driver = Some(driver);
        self.provider = Some(provider);
        self
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn persistence(mut self, persistence: Arc<dyn PersistenceLayer>) -> Self {
        self.persistence = Some(persistence);
        self
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn embedding_backend(mut self, backend: Arc<dyn EmbeddingBackend>) -> Self {
        self.embedding_backend = Some(backend);
        self
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn mcp(mut self, registry: Arc<McpRegistry>) -> Self {
        self.mcp = Some(registry);
        self
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn native_skills(mut self, registry: Arc<NativeSkillRegistry>) -> Self {
        self.native_skills = Some(registry);
        self
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn a2ui_registry(mut self, registry: Arc<A2uiRegistry>) -> Self {
        self.a2ui_registry = Some(registry);
        self
    }

    /// Attach an agent memory service.
    ///
    /// The caller constructs it so the host controls the store path and the
    /// embedding provider. Its path MUST differ from the persistence layer's:
    /// both take an exclusive SurrealKV directory lock.
    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn memory_service(
        mut self,
        service: Arc<universal_agent_runtime::uar::memory::service::MemoryService>,
    ) -> Self {
        self.memory_service = Some(service);
        self
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn seed_defaults(mut self, enabled: bool) -> Self {
        self.seed_defaults = Some(enabled);
        self
    }

    #[cfg(feature = "embedded")]
    #[must_use]
    pub fn vector_threshold(mut self, threshold: f32) -> Self {
        self.vector_threshold = Some(threshold);
        self
    }

    #[cfg(feature = "embedded")]
    pub async fn build(self) -> Result<Runtime> {
        let driver = self
            .driver
            .ok_or_else(|| Error::Config("embedded UAR requires a host local LLM driver".into()))?;
        let provider = self.provider.ok_or_else(|| {
            Error::Config("embedded UAR requires local provider/model metadata".into())
        })?;
        let persistence = self.persistence.ok_or_else(|| {
            Error::Config("embedded UAR requires a host persistence layer".into())
        })?;

        let mut builder = EmbeddedRuntimeBuilder::new()
            .local_provider(driver, provider)
            .persistence(persistence);
        if let Some(config) = self.llm_config {
            builder = builder.llm_config(config);
        }
        if let Some(backend) = self.embedding_backend {
            builder = builder.embedding_backend(backend);
        }
        if let Some(mcp) = self.mcp {
            builder = builder.mcp(mcp);
        }
        if let Some(native_skills) = self.native_skills {
            builder = builder.native_skills(native_skills);
        }
        if let Some(memory_service) = self.memory_service {
            builder = builder.memory_service(memory_service);
        }
        if let Some(a2ui_registry) = self.a2ui_registry {
            builder = builder.a2ui_registry(a2ui_registry);
        }
        if let Some(seed_defaults) = self.seed_defaults {
            builder = builder.seed_defaults(seed_defaults);
        }
        if let Some(vector_threshold) = self.vector_threshold {
            builder = builder.vector_threshold(vector_threshold);
        }

        let inner = builder.build().await.map_err(runtime_error)?;
        Ok(Runtime {
            inner: Arc::new(inner),
        })
    }

    #[cfg(not(feature = "embedded"))]
    pub async fn build(self) -> Result<Runtime> {
        Err(Error::Config(
            "embedded runtime requires the 'embedded' feature".into(),
        ))
    }
}

fn runtime_error(error: impl std::fmt::Display) -> Error {
    Error::Runtime(error.to_string())
}
