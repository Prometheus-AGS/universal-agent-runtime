//! Transport-independent embedded UAR application kernel.
//!
//! This is the composition root for applications that link UAR as a library.
//! It deliberately does not construct [`crate::AppState`], bind a socket, load
//! a sidecar, or start transport/background services. Hosts inject the local
//! LLM and persistence implementations while UAR continues to own agents,
//! policy, skills, tools, run lifecycle, canonical events, and A2UI state.

use std::sync::Arc;

use crate::{
    Result, UarError,
    config::LlmConfig,
    llm::{
        LlmDriver, ProviderConfig, ProviderRegistry, orchestrator::Orchestrator,
        registry::ModelConfig,
    },
    mcp::registry::McpRegistry,
    session::SessionStore,
    uar::{
        a2ui::{realtime::InMemoryReplayBackbone, registry::A2uiRegistry},
        defaults::{ensure_default_knowledge_base, seed_builtin_agents},
        governance::engine::GovernanceEngine,
        memory::service::MemoryService,
        persistence::PersistenceLayer,
        rag::embeddings::{EmbeddingBackend, UnavailableEmbeddingBackend},
        runtime::{
            manager::RunManager,
            matching::VectorMatcher,
            native_skill::NativeSkillRegistry,
            skills::{service::SkillService, storage::DatabaseStorageProvider},
        },
        settings::manager::SettingsManager,
    },
};

const DEFAULT_VECTOR_DIMENSION: usize = 384;
const DEFAULT_VECTOR_THRESHOLD: f32 = 0.5;

/// Fully initialized, transport-free UAR runtime state.
///
/// Every field is a real live component. There are intentionally no `Option`
/// component accessors: a successful build is the readiness guarantee.
#[derive(Clone)]
pub struct EmbeddedRuntime {
    llm_config: Arc<LlmConfig>,
    orchestrator: Arc<Orchestrator>,
    run_manager: Arc<RunManager>,
    persistence: Arc<dyn PersistenceLayer>,
    settings_manager: Arc<SettingsManager>,
    provider_registry: Arc<ProviderRegistry>,
    mcp: Arc<McpRegistry>,
    native_skills: Arc<NativeSkillRegistry>,
    skill_service: Arc<SkillService>,
    vector_matcher: Arc<VectorMatcher>,
    embedding_backend: Arc<dyn EmbeddingBackend>,
    a2ui_registry: Arc<A2uiRegistry>,
    a2ui_backbone: Arc<InMemoryReplayBackbone>,
    /// Owns a SECOND embedded SurrealKV store, separate from `persistence`.
    /// `None` when memory is disabled.
    memory_service: Option<Arc<MemoryService>>,
}

impl std::fmt::Debug for EmbeddedRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddedRuntime")
            .field("model", &self.llm_config.model)
            .field("provider_registry", &self.provider_registry)
            .field("run_manager", &self.run_manager)
            .field("persistence", &self.persistence)
            .field("settings_manager", &self.settings_manager)
            .field("mcp", &self.mcp)
            .field("native_skills", &self.native_skills)
            .field("skill_service", &self.skill_service)
            .field("vector_matcher", &self.vector_matcher)
            .field("embedding_backend", &self.embedding_backend)
            .field("a2ui_registry", &self.a2ui_registry)
            .finish_non_exhaustive()
    }
}

impl EmbeddedRuntime {
    /// Begin constructing an embedded runtime.
    #[must_use]
    pub fn builder() -> EmbeddedRuntimeBuilder {
        EmbeddedRuntimeBuilder::new()
    }

    #[must_use]
    pub fn llm_config(&self) -> &LlmConfig {
        &self.llm_config
    }

    #[must_use]
    pub fn orchestrator(&self) -> Arc<Orchestrator> {
        Arc::clone(&self.orchestrator)
    }

    #[must_use]
    pub fn run_manager(&self) -> Arc<RunManager> {
        Arc::clone(&self.run_manager)
    }

    #[must_use]
    pub fn persistence(&self) -> Arc<dyn PersistenceLayer> {
        Arc::clone(&self.persistence)
    }

    /// The runtime settings manager, backing the embedded admin surface and the
    /// Global run-policy scope. It is the same instance the run manager uses to
    /// resolve `run_policy.global`, so settings written here take effect on the
    /// next resolved run.
    #[must_use]
    pub fn settings_manager(&self) -> Arc<SettingsManager> {
        Arc::clone(&self.settings_manager)
    }

    #[must_use]
    pub fn provider_registry(&self) -> Arc<ProviderRegistry> {
        Arc::clone(&self.provider_registry)
    }

    #[must_use]
    pub fn mcp(&self) -> Arc<McpRegistry> {
        Arc::clone(&self.mcp)
    }

    #[must_use]
    pub fn native_skills(&self) -> Arc<NativeSkillRegistry> {
        Arc::clone(&self.native_skills)
    }

    #[must_use]
    pub fn skill_service(&self) -> Arc<SkillService> {
        Arc::clone(&self.skill_service)
    }

    #[must_use]
    pub fn vector_matcher(&self) -> Arc<VectorMatcher> {
        Arc::clone(&self.vector_matcher)
    }

    #[must_use]
    pub fn embedding_backend(&self) -> Arc<dyn EmbeddingBackend> {
        Arc::clone(&self.embedding_backend)
    }

    /// The agent memory service, when this runtime was built with one.
    ///
    /// `None` when memory is disabled in config. Optional rather than required
    /// so a host that does not want memory still builds, and so a caller can
    /// report "memory is off" instead of a runtime panic.
    ///
    /// NOTE: this owns a SECOND embedded SurrealKV store, separate from the
    /// persistence layer's `uar-db`. Both take an exclusive directory lock, so
    /// they must never be pointed at the same path.
    #[must_use]
    pub fn memory_service(&self) -> Option<Arc<MemoryService>> {
        self.memory_service.clone()
    }

    #[must_use]
    pub fn a2ui_registry(&self) -> Arc<A2uiRegistry> {
        Arc::clone(&self.a2ui_registry)
    }

    #[must_use]
    pub fn a2ui_backbone(&self) -> Arc<InMemoryReplayBackbone> {
        Arc::clone(&self.a2ui_backbone)
    }
}

/// Builder for a complete in-process UAR runtime.
///
/// A host LLM driver, its provider metadata, and persistence are mandatory.
/// Requiring them prevents embedded production builds from silently falling
/// back to a remote provider or ephemeral state.
#[derive(Default)]
pub struct EmbeddedRuntimeBuilder {
    llm_config: Option<LlmConfig>,
    driver: Option<Arc<dyn LlmDriver>>,
    provider: Option<ProviderConfig>,
    persistence: Option<Arc<dyn PersistenceLayer>>,
    embedding_backend: Option<Arc<dyn EmbeddingBackend>>,
    mcp: Option<Arc<McpRegistry>>,
    native_skills: Option<Arc<NativeSkillRegistry>>,
    a2ui_registry: Option<Arc<A2uiRegistry>>,
    memory_service: Option<Arc<MemoryService>>,
    seed_defaults: bool,
    vector_threshold: Option<f32>,
}

impl std::fmt::Debug for EmbeddedRuntimeBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddedRuntimeBuilder")
            .field("llm_config", &self.llm_config)
            .field("driver", &self.driver.as_ref().map(|_| "registered"))
            .field("provider", &self.provider)
            .field("persistence", &self.persistence)
            .field("embedding_backend", &self.embedding_backend)
            .field("mcp", &self.mcp)
            .field("native_skills", &self.native_skills)
            .field("a2ui_registry", &self.a2ui_registry)
            .field("seed_defaults", &self.seed_defaults)
            .field("vector_threshold", &self.vector_threshold)
            .finish()
    }
}

impl EmbeddedRuntimeBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            seed_defaults: true,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn llm_config(mut self, config: LlmConfig) -> Self {
        self.llm_config = Some(config);
        self
    }

    /// Register the host's local inference implementation and its UAR catalog
    /// identity as one atomic operation.
    #[must_use]
    pub fn local_provider(mut self, driver: Arc<dyn LlmDriver>, provider: ProviderConfig) -> Self {
        self.driver = Some(driver);
        self.provider = Some(provider);
        self
    }

    #[must_use]
    pub fn persistence(mut self, persistence: Arc<dyn PersistenceLayer>) -> Self {
        self.persistence = Some(persistence);
        self
    }

    #[must_use]
    pub fn embedding_backend(mut self, backend: Arc<dyn EmbeddingBackend>) -> Self {
        self.embedding_backend = Some(backend);
        self
    }

    #[must_use]
    pub fn mcp(mut self, registry: Arc<McpRegistry>) -> Self {
        self.mcp = Some(registry);
        self
    }

    #[must_use]
    pub fn native_skills(mut self, registry: Arc<NativeSkillRegistry>) -> Self {
        self.native_skills = Some(registry);
        self
    }

    #[must_use]
    pub fn a2ui_registry(mut self, registry: Arc<A2uiRegistry>) -> Self {
        self.a2ui_registry = Some(registry);
        self
    }

    /// Attach an agent memory service.
    ///
    /// The caller constructs it (`MemoryService::new(MemoryConfig)`) so the host
    /// controls the store path and the embedding provider. That matters on a
    /// device: the default provider is Local, which needs no API key and works
    /// offline, but a host that wants OpenAI embeddings must opt in explicitly
    /// rather than have a network dependency appear by default.
    ///
    /// The store path MUST differ from the persistence layer's — both take an
    /// exclusive SurrealKV directory lock.
    #[must_use]
    pub fn memory_service(mut self, service: Arc<MemoryService>) -> Self {
        self.memory_service = Some(service);
        self
    }

    #[must_use]
    pub fn seed_defaults(mut self, enabled: bool) -> Self {
        self.seed_defaults = enabled;
        self
    }

    #[must_use]
    pub fn vector_threshold(mut self, threshold: f32) -> Self {
        self.vector_threshold = Some(threshold);
        self
    }

    /// Build a ready-to-run embedded UAR kernel without starting transports or
    /// background workers.
    pub async fn build(self) -> Result<EmbeddedRuntime> {
        let driver = self.driver.ok_or_else(|| {
            UarError::config(
                "E_EMBEDDED_LOCAL_DRIVER_REQUIRED",
                "embedded UAR requires a host local LLM driver",
            )
        })?;
        let provider = validate_local_provider(self.provider.ok_or_else(|| {
            UarError::config(
                "E_EMBEDDED_LOCAL_PROVIDER_REQUIRED",
                "embedded UAR requires local provider and model metadata",
            )
        })?)?;
        let persistence = self.persistence.ok_or_else(|| {
            UarError::config(
                "E_EMBEDDED_DURABLE_STORAGE_REQUIRED",
                "embedded UAR requires a host persistence layer",
            )
        })?;

        let default_model = provider
            .default_model
            .clone()
            .expect("validated provider has a default model");
        let mut llm_config = self.llm_config.unwrap_or_default();
        llm_config.model = format!("{}/{}", provider.id, default_model);
        llm_config.api_key = provider.api_key.clone();
        llm_config.base_url = (!provider.base_url.is_empty()).then(|| provider.base_url.clone());

        let embedding_backend = self.embedding_backend.unwrap_or_else(|| {
            Arc::new(UnavailableEmbeddingBackend::new(
                DEFAULT_VECTOR_DIMENSION,
                "the embedded host did not register an embedding provider",
            ))
        });
        let vector_matcher = Arc::new(VectorMatcher::new(
            Arc::clone(&embedding_backend),
            self.vector_threshold.unwrap_or(DEFAULT_VECTOR_THRESHOLD),
        ));
        let mcp = self.mcp.unwrap_or_else(|| Arc::new(McpRegistry::empty()));
        let native_skills = self
            .native_skills
            .unwrap_or_else(|| Arc::new(NativeSkillRegistry::new()));
        let a2ui_registry = self
            .a2ui_registry
            .unwrap_or_else(A2uiRegistry::with_builtins);
        let a2ui_backbone = InMemoryReplayBackbone::new();

        if self.seed_defaults {
            seed_builtin_agents(persistence.as_ref()).await?;
            ensure_default_knowledge_base(persistence.as_ref(), None).await?;
        }

        let mut skill_service = SkillService::new(
            Some(Arc::clone(&persistence)),
            Some(Arc::clone(&vector_matcher)),
        );
        skill_service.add_provider(Arc::new(DatabaseStorageProvider::new(
            "embedded-host-persistence",
            "Embedded host persistence",
            Arc::clone(&persistence),
        )));
        let skill_service = Arc::new(skill_service);
        skill_service.initialize().await?;
        let skills = skill_service.registry().clone();

        let provider_registry = Arc::new(ProviderRegistry::new());
        provider_registry
            .register_custom_provider(provider.clone())
            .await?;
        provider_registry.set_default(&provider.id).await?;

        // One shared settings manager backs both the Global run-policy scope
        // (via the run manager) and the embedded admin surface (via the SDK), so
        // a setting written through the admin surface is observed by the next
        // resolved run without a second cache. The embedded runtime has no
        // `AppConfig`, so instead of the full bootstrap it seeds only the
        // `run_policy` namespace (idempotently) so `run_policy.global` exists for
        // reads and writes.
        let settings_manager = Arc::new(SettingsManager::new(Arc::clone(&persistence)));
        settings_manager.ensure_run_policy_seed().await?;

        let governance = Arc::new(GovernanceEngine::with_default_permit()?);
        let sessions = SessionStore::new();
        let orchestrator = Arc::new(Orchestrator::from_driver(
            llm_config.clone(),
            Arc::clone(&mcp),
            Arc::clone(&native_skills),
            Arc::clone(&driver),
        ));

        let run_manager = Arc::new(
            RunManager::new(
                llm_config.clone(),
                Arc::clone(&mcp),
                sessions,
                skills,
                Arc::clone(&vector_matcher),
                Some(Arc::clone(&persistence)),
            )
            .await
            .with_llm_driver(driver)
            .with_skill_service(Arc::clone(&skill_service))
            .with_provider_registry(Arc::clone(&provider_registry))
            .with_native_skills(Arc::clone(&native_skills))
            .with_a2ui_backbone(Arc::clone(&a2ui_backbone))
            .with_governance_engine(governance)
            .with_settings_manager(Arc::clone(&settings_manager)),
        );

        Ok(EmbeddedRuntime {
            llm_config: Arc::new(llm_config),
            orchestrator,
            run_manager,
            persistence,
            settings_manager,
            provider_registry,
            mcp,
            native_skills,
            skill_service,
            vector_matcher,
            embedding_backend,
            a2ui_registry,
            a2ui_backbone,
            memory_service: self.memory_service,
        })
    }
}

fn validate_local_provider(provider: ProviderConfig) -> Result<ProviderConfig> {
    if provider.id.trim().is_empty() {
        return Err(UarError::config(
            "E_EMBEDDED_LOCAL_PROVIDER_INVALID",
            "embedded local provider id must not be empty",
        ));
    }
    if !provider.enabled {
        return Err(UarError::config(
            "E_EMBEDDED_LOCAL_PROVIDER_DISABLED",
            "embedded local provider must be enabled",
        ));
    }
    let default_model = provider.default_model.as_deref().ok_or_else(|| {
        UarError::config(
            "E_EMBEDDED_LOCAL_MODEL_REQUIRED",
            "embedded local provider requires a default model",
        )
    })?;
    let model = provider
        .models
        .iter()
        .find(|model| model.id == default_model);
    match model {
        Some(ModelConfig { enabled: true, .. }) => Ok(provider),
        Some(_) => Err(UarError::config(
            "E_EMBEDDED_LOCAL_MODEL_DISABLED",
            "embedded local provider default model must be enabled",
        )),
        None => Err(UarError::config(
            "E_EMBEDDED_LOCAL_MODEL_NOT_FOUND",
            "embedded local provider default model is absent from its model catalog",
        )),
    }
}

#[cfg(all(test, feature = "in-memory-backend"))]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::{
        pin::Pin,
        sync::atomic::{AtomicBool, Ordering},
        time::Duration,
    };

    use anyhow::Result as AnyResult;
    use async_trait::async_trait;
    use futures::Stream;
    use serde_json::json;

    use super::*;
    use crate::{
        llm::{LlmRequest, mock_driver::MockLlmDriver, registry::ProtocolSetting},
        normalized::NormalizedEvent as ProviderEvent,
        uar::{
            defaults::default_agent,
            domain::{
                events::{ArtifactPayload, NormalizedEvent},
                skills::{Skill, SkillTriggers},
            },
            persistence::providers::memory::InMemoryProvider,
            runtime::native_skill::NativeSkill,
        },
    };

    fn local_provider() -> ProviderConfig {
        ProviderConfig {
            id: "embedded-local".to_string(),
            display_name: "Embedded local test model".to_string(),
            base_url: String::new(),
            api_key: None,
            protocol: ProtocolSetting::Auto,
            default_model: Some("offline-agent-model".to_string()),
            models: vec![ModelConfig {
                id: "offline-agent-model".to_string(),
                display_name: Some("Offline agent model".to_string()),
                context_window: Some(8_192),
                supports_vision: false,
                supports_tools: true,
                supports_reasoning: true,
                supports_structured_output: true,
                supports_streaming: true,
                max_output_tokens: Some(2_048),
                enabled: true,
            }],
            enabled: true,
        }
    }

    fn test_agent() -> crate::uar::domain::artifact::AgentArtifact {
        let mut agent = default_agent();
        agent.id = "embedded-test-agent".to_string();
        agent.metadata.title = "Embedded test agent".to_string();
        agent.memory.kb.enabled = false;
        agent
    }

    async fn build_runtime(
        driver: Arc<dyn LlmDriver>,
        persistence: Arc<InMemoryProvider>,
        native_skills: Option<Arc<NativeSkillRegistry>>,
    ) -> EmbeddedRuntime {
        let persistence: Arc<dyn PersistenceLayer> = persistence;
        let mut builder = EmbeddedRuntime::builder()
            .local_provider(driver, local_provider())
            .persistence(persistence);
        if let Some(native_skills) = native_skills {
            builder = builder.native_skills(native_skills);
        }
        builder.build().await.expect("embedded runtime is ready")
    }

    async fn wait_for_event(
        runtime: &EmbeddedRuntime,
        run_id: &str,
        predicate: impl Fn(&NormalizedEvent) -> bool,
    ) -> Vec<crate::uar::runtime::manager::StreamEvent> {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let history = runtime
                    .run_manager()
                    .history_since(run_id, None)
                    .await
                    .expect("run history exists");
                if history.iter().any(|event| predicate(&event.event)) {
                    return history;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("expected run event before timeout")
    }

    #[tokio::test]
    async fn build_requires_real_host_components_and_reports_ready_state() {
        let error = EmbeddedRuntime::builder().build().await.unwrap_err();
        assert_eq!(error.code(), "E_EMBEDDED_LOCAL_DRIVER_REQUIRED");

        let driver: Arc<dyn LlmDriver> = Arc::new(MockLlmDriver::echo());
        let error = EmbeddedRuntime::builder()
            .local_provider(driver, local_provider())
            .build()
            .await
            .unwrap_err();
        assert_eq!(error.code(), "E_EMBEDDED_DURABLE_STORAGE_REQUIRED");

        let runtime = build_runtime(
            Arc::new(MockLlmDriver::echo()),
            Arc::new(InMemoryProvider::new()),
            None,
        )
        .await;
        assert_eq!(
            runtime.run_manager().resolve_default_model().await,
            Some((
                "embedded-local".to_string(),
                "offline-agent-model".to_string()
            ))
        );
        assert_eq!(runtime.a2ui_registry().list().await.len(), 5);
        assert!(
            runtime
                .persistence()
                .load_agent("default-agent")
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn local_provider_routes_agent_runs_and_replays_ordered_events() {
        let driver = Arc::new(MockLlmDriver::new(vec![vec![
            ProviderEvent::ThinkingDelta {
                text: "checking locally".to_string(),
            },
            ProviderEvent::MessageDelta {
                text: "Offline answer".to_string(),
            },
            ProviderEvent::Done,
        ]]));
        let runtime = build_runtime(driver.clone(), Arc::new(InMemoryProvider::new()), None).await;

        let run_id = runtime
            .run_manager()
            .start_run(
                test_agent(),
                "answer without a network".to_string(),
                Some("offline-conversation".to_string()),
                None,
                Vec::new(),
            )
            .await;
        let history = wait_for_event(&runtime, &run_id, |event| {
            matches!(event, NormalizedEvent::RunDone { .. })
        })
        .await;

        assert!(history.windows(2).all(|pair| pair[0].id < pair[1].id));
        assert!(history.iter().any(|event| matches!(
            &event.event,
            NormalizedEvent::ThinkingDelta { text_delta, .. }
                if text_delta == "checking locally"
        )));
        assert!(history.iter().any(|event| matches!(
            &event.event,
            NormalizedEvent::ChatDelta { text_delta, .. }
                if text_delta == "Offline answer"
        )));

        let first_id = history.first().unwrap().id;
        let replay = runtime
            .run_manager()
            .history_since(&run_id, Some(first_id))
            .await
            .unwrap();
        assert!(replay.iter().all(|event| event.id > first_id));
        assert!(runtime.run_manager().subscribe(&run_id).await.is_some());

        let requests = driver.requests();
        assert_eq!(requests.len(), 1);
        assert!(
            requests[0]
                .messages
                .iter()
                .any(|message| message.to_string().contains("answer without a network"))
        );
    }

    #[tokio::test]
    async fn effective_run_policy_artifact_backfills_the_registry_default_model() {
        // The seeded/default agent leaves `provider.default` empty (it defers to
        // the registry default). Without the backfill the emitted
        // `effective_run_policy` artifact would carry an empty ModelRoute and the
        // downstream provenance chip (agent/provider/model) would show a blank
        // model. Assert the artifact reports the true executing route instead.
        let driver = Arc::new(MockLlmDriver::new(vec![vec![
            ProviderEvent::MessageDelta {
                text: "answer".to_string(),
            },
            ProviderEvent::Done,
        ]]));
        let runtime = build_runtime(driver, Arc::new(InMemoryProvider::new()), None).await;

        let run_id = runtime
            .run_manager()
            .start_run(
                test_agent(),
                "who produced this?".to_string(),
                Some("provenance-conversation".to_string()),
                None,
                Vec::new(),
            )
            .await;
        let history = wait_for_event(&runtime, &run_id, |event| {
            matches!(
                event,
                NormalizedEvent::Artifact { artifact, .. }
                    if artifact.artifact_type == "effective_run_policy"
            )
        })
        .await;

        let artifact = history
            .iter()
            .find_map(|event| match &event.event {
                NormalizedEvent::Artifact { artifact, .. }
                    if artifact.artifact_type == "effective_run_policy" =>
                {
                    Some(artifact.clone())
                }
                _ => None,
            })
            .expect("effective_run_policy artifact is emitted");

        let policy: serde_json::Value =
            serde_json::from_str(&artifact.content).expect("policy content is JSON");
        assert_eq!(policy["model"]["provider_id"], "embedded-local");
        assert_eq!(policy["model"]["model_id"], "offline-agent-model");
        assert_eq!(policy["agent_id"], "embedded-test-agent");
    }

    #[tokio::test]
    async fn conversation_policy_round_trips_and_effective_config_reflects_the_override() {
        use crate::uar::domain::policy::{ModelRoute, RunPolicy};

        let runtime = build_runtime(
            Arc::new(MockLlmDriver::echo()),
            Arc::new(InMemoryProvider::new()),
            None,
        )
        .await;
        let conversation_id = "conv-model-override";
        let persistence = runtime.persistence();

        // No conversation policy yet: effective_config resolves the default agent
        // and backfills the registry-default model (the M6c behavior).
        let baseline = runtime
            .run_manager()
            .effective_config(conversation_id)
            .await;
        assert!(baseline.requested_policy.is_none());
        let baseline_model = baseline.effective_policy.model.expect("model backfilled");
        assert_eq!(baseline_model.provider_id, "embedded-local");
        assert_eq!(baseline_model.model_id, "offline-agent-model");

        // Save a conversation-scoped model override and confirm it round-trips and
        // wins over the (empty) agent/global scopes.
        let mut policy = RunPolicy::default();
        policy.model = Some(ModelRoute {
            provider_id: "openai".to_string(),
            model_id: "gpt-4o".to_string(),
        });
        let record =
            crate::uar::domain::policy::ConversationPolicyRecord::new(conversation_id, policy);
        persistence
            .save_conversation_policy(&record)
            .await
            .expect("conversation policy persists");

        let loaded = persistence
            .load_conversation_policy(conversation_id)
            .await
            .expect("load ok")
            .expect("record present");
        assert_eq!(
            loaded.policy.model.as_ref().map(|m| m.model_id.as_str()),
            Some("gpt-4o")
        );

        let overridden = runtime
            .run_manager()
            .effective_config(conversation_id)
            .await;
        assert!(overridden.requested_policy.is_some());
        let effective_model = overridden
            .effective_policy
            .model
            .expect("conversation model resolves");
        assert_eq!(effective_model.provider_id, "openai");
        assert_eq!(effective_model.model_id, "gpt-4o");

        // Deleting the override reverts to the backfilled registry default.
        persistence
            .delete_conversation_policy(conversation_id)
            .await
            .expect("delete ok");
        let reverted = runtime
            .run_manager()
            .effective_config(conversation_id)
            .await;
        assert!(reverted.requested_policy.is_none());
        assert_eq!(
            reverted.effective_policy.model.map(|m| m.model_id),
            Some("offline-agent-model".to_string())
        );
    }

    #[tokio::test]
    async fn cold_started_session_is_seeded_from_supplied_history() {
        use crate::uar::runtime::manager::SeedMessage;

        // Echo driver returns the last user message, so the request the model
        // receives is observable through the driver's recorded requests.
        let driver = Arc::new(MockLlmDriver::echo());
        let runtime = build_runtime(driver.clone(), Arc::new(InMemoryProvider::new()), None).await;
        let conversation_id = "cold-start-conversation".to_string();

        // First turn: the session is empty (cold start), so the supplied history
        // must be replayed before the current input.
        let seed = vec![
            SeedMessage {
                role: "user".to_string(),
                content: "my name is Ada".to_string(),
                tool_call_id: None,
            },
            SeedMessage {
                role: "assistant".to_string(),
                content: "Nice to meet you, Ada.".to_string(),
                tool_call_id: None,
            },
        ];
        let run_id = runtime
            .run_manager()
            .start_run_with_history(
                test_agent(),
                "what is my name?".to_string(),
                Some(conversation_id.clone()),
                None,
                Vec::new(),
                seed.clone(),
            )
            .await;
        wait_for_event(&runtime, &run_id, |event| {
            matches!(event, NormalizedEvent::RunDone { .. })
        })
        .await;

        // The model's request contains the seeded prior turns AND the new input.
        let first = driver.requests();
        let rendered = first
            .last()
            .expect("a request reached the driver")
            .messages
            .iter()
            .map(|message| message.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            rendered.contains("my name is Ada"),
            "seeded user turn present"
        );
        assert!(
            rendered.contains("Nice to meet you, Ada."),
            "seeded assistant turn present"
        );
        assert!(
            rendered.contains("what is my name?"),
            "current input present"
        );

        // Second turn on the SAME (now warm) session: seeding must NOT duplicate
        // the history — the session already holds the prior turns plus the first
        // exchange, so passing the same seed again is a no-op.
        let run_id_2 = runtime
            .run_manager()
            .start_run_with_history(
                test_agent(),
                "and again?".to_string(),
                Some(conversation_id.clone()),
                None,
                Vec::new(),
                seed,
            )
            .await;
        wait_for_event(&runtime, &run_id_2, |event| {
            matches!(event, NormalizedEvent::RunDone { .. })
        })
        .await;
        let second = driver.requests();
        let rendered_2 = second
            .last()
            .expect("a second request reached the driver")
            .messages
            .iter()
            .map(|message| message.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            rendered_2.matches("my name is Ada").count(),
            1,
            "warm session is not re-seeded (no duplicate history)"
        );
    }

    #[derive(Debug)]
    struct PendingDriver;

    #[async_trait]
    impl LlmDriver for PendingDriver {
        async fn stream(
            &self,
            _request: LlmRequest,
        ) -> AnyResult<Pin<Box<dyn Stream<Item = AnyResult<ProviderEvent>> + Send>>> {
            Ok(Box::pin(futures::stream::pending()))
        }
    }

    #[tokio::test]
    async fn cancellation_is_terminal_visible_and_idempotent() {
        let runtime = build_runtime(
            Arc::new(PendingDriver),
            Arc::new(InMemoryProvider::new()),
            None,
        )
        .await;
        let run_id = runtime
            .run_manager()
            .start_run(test_agent(), "wait".to_string(), None, None, Vec::new())
            .await;

        assert!(runtime.run_manager().cancel_run(&run_id).await);
        wait_for_event(&runtime, &run_id, |event| {
            matches!(event, NormalizedEvent::Cancelled { .. })
        })
        .await;
        assert!(!runtime.run_manager().cancel_run(&run_id).await);
    }

    #[derive(Debug)]
    struct DeleteNoteSkill {
        executed: Arc<AtomicBool>,
    }

    #[async_trait]
    impl NativeSkill for DeleteNoteSkill {
        fn name(&self) -> &str {
            "delete_note"
        }

        fn description(&self) -> &str {
            "Delete a note after explicit approval"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            json!({"type":"object","properties":{"id":{"type":"string"}}})
        }

        async fn execute(&self, _arguments: serde_json::Value) -> AnyResult<serde_json::Value> {
            self.executed.store(true, Ordering::SeqCst);
            Ok(json!({"deleted": true}))
        }
    }

    #[tokio::test]
    async fn risky_tool_calls_pause_for_approval_then_resume_in_process() {
        let driver = Arc::new(MockLlmDriver::new(vec![
            vec![
                ProviderEvent::ToolCallDelta {
                    call_index: 0,
                    id: Some("delete-1".to_string()),
                    name: Some("delete_note".to_string()),
                    arguments_delta: Some(r#"{"id":"note-1"}"#.to_string()),
                },
                ProviderEvent::ToolCallComplete {
                    call_index: 0,
                    id: "delete-1".to_string(),
                    name: "delete_note".to_string(),
                    arguments_json: r#"{"id":"note-1"}"#.to_string(),
                },
                ProviderEvent::Done,
            ],
            vec![
                ProviderEvent::MessageDelta {
                    text: "The note was deleted.".to_string(),
                },
                ProviderEvent::Done,
            ],
        ]));
        let executed = Arc::new(AtomicBool::new(false));
        let native_skills = Arc::new(NativeSkillRegistry::new());
        native_skills
            .register(DeleteNoteSkill {
                executed: Arc::clone(&executed),
            })
            .await;
        let runtime = build_runtime(
            driver,
            Arc::new(InMemoryProvider::new()),
            Some(native_skills),
        )
        .await;
        let mut agent = test_agent();
        agent.policy.tools.allow = vec!["delete_note".to_string()];
        let run_id = runtime
            .run_manager()
            .start_run(agent, "delete note one".to_string(), None, None, Vec::new())
            .await;

        wait_for_event(&runtime, &run_id, |event| {
            matches!(event, NormalizedEvent::ToolCallApprovalRequired { .. })
        })
        .await;
        assert!(!executed.load(Ordering::SeqCst));
        assert!(runtime.run_manager().resolve_approval(&run_id, true).await);
        let history = wait_for_event(&runtime, &run_id, |event| {
            matches!(event, NormalizedEvent::RunDone { .. })
        })
        .await;
        assert!(executed.load(Ordering::SeqCst));
        assert!(history.iter().any(|event| matches!(
            &event.event,
            NormalizedEvent::ToolEnd { tool, ok: true, .. } if tool == "delete_note"
        )));
    }

    #[tokio::test]
    async fn persisted_skills_emit_events_and_a2ui_interactions_continue_the_agent() {
        let persistence = Arc::new(InMemoryProvider::new());
        persistence
            .save_skill(
                &Skill {
                    skill_id: "diagram-skill".to_string(),
                    version: "1.0.0".to_string(),
                    title: "Diagram skill".to_string(),
                    description: "Creates diagrams".to_string(),
                    triggers: SkillTriggers {
                        keywords: vec!["diagram".to_string()],
                        semantic: None,
                    },
                    prompt_overlay: "Create a clear diagram.".to_string(),
                    enabled: true,
                    ..Skill::default()
                },
                &[],
            )
            .await
            .unwrap();
        let driver = Arc::new(MockLlmDriver::new(vec![vec![
            ProviderEvent::MessageDelta {
                text: "Rendered locally".to_string(),
            },
            ProviderEvent::Done,
        ]]));
        let runtime = build_runtime(driver.clone(), Arc::clone(&persistence), None).await;
        let mut agent = test_agent();
        agent.policy.skills.prefer = vec!["diagram-skill".to_string()];
        persistence.save_agent(&agent).await.unwrap();

        let run_id = runtime
            .run_manager()
            .start_run(
                agent,
                "create a diagram".to_string(),
                Some("a2ui-conversation".to_string()),
                None,
                Vec::new(),
            )
            .await;
        let history = wait_for_event(&runtime, &run_id, |event| {
            matches!(event, NormalizedEvent::RunDone { .. })
        })
        .await;
        assert!(history.iter().any(|event| matches!(
            &event.event,
            NormalizedEvent::SkillActivated { skill_id, .. } if skill_id == "diagram-skill"
        )));

        runtime
            .run_manager()
            .emit_to_run(
                &run_id,
                NormalizedEvent::ArtifactInputRequest {
                    run_id: run_id.clone(),
                    artifact: ArtifactPayload {
                        artifact_id: "confirm-1".to_string(),
                        artifact_type: "confirm".to_string(),
                        title: "Continue?".to_string(),
                        content: "{}".to_string(),
                        language: Some("json".to_string()),
                        metadata: json!({"schema_id":"builtin.confirm"}),
                    },
                },
            )
            .await;
        let continuation_id = runtime
            .run_manager()
            .continue_with_interaction(&run_id, json!({"confirmed": true}))
            .await
            .unwrap();
        assert_ne!(continuation_id, run_id);
        wait_for_event(&runtime, &continuation_id, |event| {
            matches!(event, NormalizedEvent::RunDone { .. })
        })
        .await;
        assert!(driver.requests().iter().any(|request| {
            request.messages.iter().any(|message| {
                let serialized = message.to_string();
                serialized.contains("a2ui.user_action") && serialized.contains("confirmed")
            })
        }));
    }

    #[tokio::test]
    async fn rebuilding_the_kernel_restores_agents_and_skills_from_host_persistence() {
        let persistence = Arc::new(InMemoryProvider::new());
        let first = build_runtime(
            Arc::new(MockLlmDriver::echo()),
            Arc::clone(&persistence),
            None,
        )
        .await;
        let skill = Skill {
            skill_id: "restart-skill".to_string(),
            version: "1.0.0".to_string(),
            title: "Restart skill".to_string(),
            enabled: true,
            ..Skill::default()
        };
        persistence.save_skill(&skill, &[]).await.unwrap();
        drop(first);

        let second = build_runtime(
            Arc::new(MockLlmDriver::echo()),
            Arc::clone(&persistence),
            None,
        )
        .await;
        assert!(
            second
                .persistence()
                .load_agent("orchestrator-agent")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            second
                .skill_service()
                .get_skills()
                .await
                .iter()
                .any(|loaded| loaded.skill_id == "restart-skill")
        );
    }
}
