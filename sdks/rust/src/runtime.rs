//! Direct, transport-independent UAR embedding API.
//!
//! Enable `embedded` to link the UAR application kernel into the host process.
//! This module never opens a socket. The optional `server` feature exposes an
//! explicit server entry point for hosts that intentionally want HTTP.

use std::sync::Arc;

#[cfg(feature = "embedded")]
use tokio::sync::broadcast;
#[cfg(feature = "embedded")]
use universal_agent_runtime::{
    config::LlmConfig,
    embedded::{EmbeddedRuntime, EmbeddedRuntimeBuilder},
    llm::{LlmDriver, ProviderConfig},
    mcp::registry::McpRegistry,
    uar::{
        a2ui::registry::A2uiRegistry,
        domain::{agent_store, artifact::AgentArtifact, events::MemoryItem},
        persistence::PersistenceLayer,
        rag::embeddings::EmbeddingBackend,
        runtime::{
            manager::{RunManager, StreamEvent},
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
