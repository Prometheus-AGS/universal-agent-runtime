use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{any, get, post, put},
};
use chrono::Utc;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use tracing::{info, warn};

use crate::AppState;
use crate::config::AppConfig;
use crate::llm::{LlmDriver, Orchestrator};
use crate::mcp::registry::McpRegistry;
use crate::normalized::NormalizedEvent as DriverEvent;
use crate::session::SessionStore;
use crate::uar::api::sse::{to_agui_event, to_runtime_entity_event};
use crate::uar::settings::resilience_policy::{
    PolicySource, ResiliencePolicy, resolve_effective_policy,
};
use crate::uar::telemetry::metrics as telemetry_metrics;
use crate::uar::{
    self,
    defaults::{ensure_default_knowledge_base, seed_builtin_agents},
    domain::events::MemoryItem,
    governance::engine::GovernanceEngine,
    memory::{
        MemoryService,
        auto_capture::{self, ConversationMessage},
        context_builder,
    },
    persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
    prompt_cache::{CachedEntry, PromptCacheProvider, SurrealMemPromptCacheProvider},
    rag::{
        chunking::ChunkingStrategy, ingest::IngestService, ingestion_worker::IngestionWorkerPool,
    },
    runtime::{
        actor::system::ActorCollaboration,
        manager::RunManager,
        matching::vector::VectorMatcher,
        native_skill::NativeSkillRegistry,
        skills::{
            SkillService,
            storage::{DatabaseStorageProvider, FilesystemStorageProvider, SkillStorageProvider},
        },
    },
    security::{
        api_keys::{ApiKeyService, InMemoryApiKeyStorage},
        claims::UserContext,
    },
};

#[cfg(feature = "postgres-backend")]
use crate::uar::persistence::providers::postgres::PostgresProvider;

/// Start the Axum server with the provided configuration.
pub async fn start_server(config: Arc<AppConfig>) -> anyhow::Result<()> {
    let mut llm_config = config.llm.clone();
    normalize_legacy_openai_base_url(&mut llm_config);
    info!(
        name: "llm.config.loaded",
        model = %llm_config.model,
        "LLM configuration loaded"
    );

    // Initialize Persistence & RAG
    let mut ingest_service: Option<Arc<IngestService>> = None;
    let vector_matcher = Arc::new(VectorMatcher::new(
        config.models.vector_threshold,
        config.models.models_dir.clone(),
    ));

    // Initialize VectorMatcher explicitly (shared)
    if let Err(e) = vector_matcher.initialize().await {
        tracing::error!("Failed to initialize VectorMatcher: {:?}", e);
    }

    // Initialize persistence based on config.
    // All three branches produce trait-object arcs so the types unify across arms.
    let (persistence_layer, compiler_storage, agent_registry, live_bus, credential_store): (
        Arc<dyn PersistenceLayer>,
        Option<(
            Arc<dyn crate::uar::compiler::storage::SpecStorage>,
            Arc<dyn crate::uar::compiler::session::persistence::SessionStorage>,
        )>,
        Option<Arc<dyn crate::uar::api::a2a::AgentRegistry>>,
        Option<Arc<dyn crate::uar::realtime::RealtimeBus>>,
        Option<Arc<dyn uar::security::credentials::CredentialStore>>,
    ) = if matches!(
        config.persistence.provider.as_str(),
        "surreal" | "surrealdb"
    ) {
        let provider = SurrealDbProvider::new(
            &config.persistence.database_url,
            config.persistence.surreal_user.as_deref(),
            config.persistence.surreal_pass.as_deref(),
            config.persistence.surreal_ns.as_deref(),
            config.persistence.surreal_db.as_deref(),
        )
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to initialize SurrealDB at '{}': {e}\n\
                Hint: another instance may already be running and holding the database lock.",
                config.persistence.database_url
            )
        })?;

        // Create compiler storage sharing the same DB connection
        let db = provider.client();
        let compiler_store = Arc::new(
            crate::uar::compiler::storage::surreal::SurrealCompilerStorage::new(db.clone()),
        );
        let spec: Arc<dyn crate::uar::compiler::storage::SpecStorage> =
            Arc::clone(&compiler_store) as Arc<dyn crate::uar::compiler::storage::SpecStorage>;
        let sess: Arc<dyn crate::uar::compiler::session::persistence::SessionStorage> =
            compiler_store as Arc<dyn crate::uar::compiler::session::persistence::SessionStorage>;
        let registry = Arc::new(crate::uar::api::a2a::SurrealAgentRegistry::new(db.clone()))
            as Arc<dyn crate::uar::api::a2a::AgentRegistry>;

        // Durable per-user credential store on the same DB connection.
        let credential_store = Some(Arc::new(
            uar::security::credentials::SurrealCredentialStore::new(db.clone()),
        )
            as Arc<dyn uar::security::credentials::CredentialStore>);

        // Start the live-query bus on the same DB connection.
        let live_bus = Some(
            Arc::new(crate::uar::realtime::surreal_bus::LiveQueryBus::start(db))
                as Arc<dyn crate::uar::realtime::RealtimeBus>,
        );

        (
            Arc::new(provider) as Arc<dyn PersistenceLayer>,
            Some((spec, sess)),
            Some(registry),
            live_bus,
            credential_store,
        )
    } else {
        // Non-surreal persistence requested. Only available when the
        // `postgres-backend` Cargo feature is enabled at build time.
        #[cfg(feature = "postgres-backend")]
        {
            let provider = PostgresProvider::new(&config.persistence.database_url)
                .await
                .expect("Failed to initialize Postgres");
            let pool = provider.get_pool().clone();

            // Start the Postgres LISTEN/NOTIFY realtime bus on the same pool.
            let live_bus = Some(Arc::new(
                crate::uar::realtime::postgres_bus::PostgresNotifyBus::start(pool.clone()),
            ) as Arc<dyn crate::uar::realtime::RealtimeBus>);

            let compiler_store = Arc::new(
                crate::uar::compiler::storage::postgres::PostgresCompilerStorage::new(pool.clone()),
            );
            let spec: Arc<dyn crate::uar::compiler::storage::SpecStorage> =
                Arc::clone(&compiler_store) as Arc<dyn crate::uar::compiler::storage::SpecStorage>;
            let sess: Arc<dyn crate::uar::compiler::session::persistence::SessionStorage> =
                compiler_store
                    as Arc<dyn crate::uar::compiler::session::persistence::SessionStorage>;
            let registry = Arc::new(crate::uar::api::a2a::PostgresAgentRegistry::new(pool))
                as Arc<dyn crate::uar::api::a2a::AgentRegistry>;

            (
                Arc::new(provider) as Arc<dyn PersistenceLayer>,
                Some((spec, sess)),
                Some(registry),
                live_bus,
                // Postgres-backed credential store not implemented; falls back
                // to in-memory below (matches the api_keys precedent).
                None,
            )
        }
        #[cfg(not(feature = "postgres-backend"))]
        {
            anyhow::bail!(
                "persistence.provider = '{}' requires the `postgres-backend` Cargo \
                 feature to be enabled at build time. This build of UAR was compiled \
                 with embedded SurrealDB only — set persistence.provider = \"surreal\" \
                 in your config or rebuild with --features postgres-backend.",
                config.persistence.provider
            );
        }
    };
    let persistence = Some(persistence_layer);

    // Initialize Ingest Service if persistence is available
    if let Some(p) = &persistence {
        let ingest = Arc::new(IngestService::new(
            Arc::clone(p),
            Arc::clone(&vector_matcher),
            ChunkingStrategy::Semantic { threshold: 0.5 },
        ));
        ingest_service = Some(Arc::clone(&ingest));

        // Spawn File Watcher
        let ingest_svc_clone = Arc::clone(&ingest);
        tokio::spawn(async move {
            let ingest_dir = std::path::PathBuf::from("/data/ingest");
            if !ingest_dir.exists() {
                let _ = tokio::fs::create_dir_all(&ingest_dir).await;
            }
            if ingest_dir.exists() {
                ingest_svc_clone
                    .watch(ingest_dir, "default".to_string())
                    .await;
            }
        });

        // Ensure default knowledge base exists
        if let Err(e) = ensure_default_knowledge_base(&**p, None).await {
            tracing::error!("Failed to ensure default KB: {:?}", e);
        } else {
            info!("Default knowledge base ensured.");
        }

        // Seed built-in agents so they are persisted, realtime-backed entities.
        // They are idempotently upserted on every boot so system prompt updates
        // take effect without a manual re-seed.
        if let Err(e) = seed_builtin_agents(&**p).await {
            tracing::warn!("Failed to seed built-in agents: {:?}", e);
        }

        info!("Persistence and RAG enabled.");
    }

    // Initialise the memory system if enabled in config.
    let memory_service: Option<Arc<MemoryService>> = if config.memory.enabled {
        match MemoryService::new(config.memory.clone()).await {
            Ok(svc) => {
                info!(
                    "MemoryService initialized (embedding_provider={}, auto_capture={}, inject_context={})",
                    config.memory.embedding_provider,
                    config.memory.auto_capture,
                    config.memory.inject_context
                );
                Some(Arc::new(svc))
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to initialize MemoryService — memory disabled");
                None
            }
        }
    } else {
        info!("Memory system disabled (UAR_MEMORY__ENABLED not set)");
        None
    };

    // MCP: connect once at startup
    // We update this to include native tools if persistence is present
    let mut mcp_registry = match McpRegistry::load_from_file("mcp.json").await {
        Ok(registry) => registry,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Could not load mcp.json — starting with empty MCP registry. \
                 Tools from mcp.json will not be available until the file is created."
            );
            McpRegistry::empty()
        }
    };

    // Register memory tools — live service if enabled, no-op shims otherwise.
    let save_tool = Arc::new(crate::uar::tools::memory::MemorySaveTool::new(
        memory_service.clone(),
    ));
    let recall_tool = Arc::new(crate::uar::tools::memory::MemoryRecallTool::new(
        memory_service.clone(),
    ));
    let list_tool = Arc::new(crate::uar::tools::memory::MemoryListTool::new(
        memory_service.clone(),
    ));
    let delete_tool = Arc::new(crate::uar::tools::memory::MemoryDeleteTool::new(
        memory_service.clone(),
    ));
    let update_tool = Arc::new(crate::uar::tools::memory::MemoryUpdateTool::new(
        memory_service.clone(),
    ));
    let history_tool = Arc::new(crate::uar::tools::memory::MemoryHistoryTool::new(
        memory_service.clone(),
    ));
    mcp_registry = mcp_registry
        .with_native_tool(save_tool)
        .with_native_tool(recall_tool)
        .with_native_tool(list_tool)
        .with_native_tool(delete_tool)
        .with_native_tool(update_tool)
        .with_native_tool(history_tool);
    info!(
        "Native tools (memory_save, memory_recall, memory_list, memory_delete_by_id, memory_update_by_id, memory_history) registered — active={}",
        memory_service.is_some()
    );

    let mcp = Arc::new(mcp_registry);

    for (name, _tool) in mcp.tools() {
        info!(name: "mcp.tool.discovered", tool = %name, "MCP tool discovered");
    }

    // Initialize Native Skill Registry and register built-in skills
    let native_skill_registry = Arc::new(NativeSkillRegistry::new());
    uar::runtime::native_skills::register_builtins(
        &native_skill_registry,
        &config.native_tools,
        persistence.clone(),
    )
    .await;
    info!(
        "Native skill registry initialized with {} skills",
        native_skill_registry.len().await
    );

    // Create orchestrator
    let orchestrator = Arc::new(Orchestrator::new(
        llm_config.clone(),
        Arc::clone(&mcp),
        Arc::clone(&native_skill_registry),
    )?);

    // Session store
    let sessions = SessionStore::new();

    // Skills initialization — use SkillService with filesystem provider
    let mut skill_service =
        SkillService::new(persistence.clone(), Some(Arc::clone(&vector_matcher)));
    let fs_provider: Arc<dyn SkillStorageProvider> = Arc::new(FilesystemStorageProvider::new(
        "fs-skills",
        "Local Skills",
        "skills",
    ));
    skill_service.add_provider(fs_provider);
    // Register the database provider so skills pushed via the API are reloaded
    // after a restart. The DB provider is added after the filesystem provider
    // so API-pushed skills (provider_id = "api") take precedence on name conflict.
    if let Some(ref persistence_layer) = persistence {
        let db_provider: Arc<dyn SkillStorageProvider> = Arc::new(DatabaseStorageProvider::new(
            "db-skills",
            "Database Skills",
            Arc::clone(persistence_layer),
        ));
        skill_service.add_provider(db_provider);
    }
    if let Err(e) = skill_service.initialize().await {
        eprintln!("Warning: Failed to initialize skills: {e:?}");
    }
    let skills = skill_service.registry().clone();
    let skill_service = Arc::new(skill_service);

    // Load built-in (Manifest-kind, Builtin-origin) skills from the embedded
    // prometheus-skill-system submodule. Failure here is non-fatal.
    {
        let builtins = uar::runtime::skills::builtin_loader::discover_builtin_skills();
        if !builtins.is_empty() {
            skill_service.register_builtins(builtins).await;
        }
    }

    // Initialize Provider Registry
    let provider_registry = Arc::new(crate::llm::ProviderRegistry::new());
    provider_registry.seed_from_llm_config(&llm_config).await;
    if !config.providers.is_empty() {
        provider_registry
            .seed_from_configs(config.providers.clone())
            .await;
    }

    // Multi-tenant provider credentials. Active only when CREDENTIAL_ENCRYPTION_KEY
    // is set; otherwise `None` ⇒ single-tenant (env/config key path, unchanged).
    // Uses the durable SurrealDB-backed store when persistence is Surreal;
    // otherwise falls back to an in-memory store (matches the api_keys precedent).
    let provider_service: Option<Arc<uar::security::credentials::ProviderService>> = {
        let store: Arc<dyn uar::security::credentials::CredentialStore> =
            credential_store.clone().unwrap_or_else(|| {
                Arc::new(uar::security::credentials::InMemoryCredentialStore::new())
            });
        match uar::security::credentials::ProviderService::from_env(store) {
            Ok(Some(svc)) => {
                tracing::info!("Multi-tenant provider credentials enabled");
                Some(Arc::new(svc))
            }
            Ok(None) => None,
            Err(e) => {
                tracing::error!(error = %e, "CREDENTIAL_ENCRYPTION_KEY invalid; multi-tenant credentials disabled");
                None
            }
        }
    };

    let run_manager = Arc::new({
        let mut rm = RunManager::new(
            llm_config.clone(),
            Arc::clone(&mcp),
            sessions.clone(),
            Arc::clone(&skills),
            Arc::clone(&vector_matcher),
            persistence.clone(),
        )
        .await
        .with_skill_service(Arc::clone(&skill_service))
        .with_provider_registry(Arc::clone(&provider_registry))
        .with_native_skills(Arc::clone(&native_skill_registry))
        .with_message_context_strategy(config.context_strategy.clone());
        if let Some(ref svc) = provider_service {
            rm = rm.with_provider_service(Arc::clone(svc));
        }
        rm
    });

    // Initialize Global Rate Limiter
    #[allow(clippy::cast_sign_loss)]
    let burst_size = config.resilience.burst_size.max(0.0) as u32;
    let rate_limiter = Arc::new(uar::security::rate_limit::AppRateLimiter::new(
        config.resilience.requests_per_second,
        burst_size,
    ));

    // Initialize Actor Collaboration System
    let actor_system = Arc::new(ActorCollaboration::new(
        llm_config.clone(),
        Arc::clone(&mcp),
        Arc::clone(&native_skill_registry),
    ));
    info!("Actor collaboration system initialized");

    // Initialize Governance Policy Engine
    let governance_engine = match GovernanceEngine::load_from_dir("policies").await {
        Ok(engine) => {
            info!(
                policy_count = engine.policy_count().await,
                "Governance policy engine loaded"
            );
            Arc::new(engine)
        }
        Err(e) => {
            warn!(
                error = %e,
                "Failed to load policies from directory — using permissive default"
            );
            Arc::new(
                GovernanceEngine::with_default_permit()
                    .expect("default permit policy should parse"),
            )
        }
    };

    // Initialize API Key Service
    let api_key_storage: Arc<dyn uar::security::api_keys::ApiKeyStorage> =
        Arc::new(InMemoryApiKeyStorage::new());
    let api_key_service = Arc::new(ApiKeyService::new(
        Arc::clone(&api_key_storage),
        &config.security.jwt_secret,
    ));
    info!("API key service initialized");

    // Initialize Compiler Service
    let compiler_service = if let Some((spec_store, session_store)) = compiler_storage {
        info!("Initializing Compiler Service with persistent storage");
        Arc::new(uar::compiler::CompilerService::new(
            spec_store,
            session_store,
        ))
    } else {
        info!("Initializing Compiler Service with in-memory storage");
        Arc::new(uar::compiler::CompilerService::in_memory())
    };
    info!("Compiler service initialized");

    // Initialize A2A state (shared task store + compiler service)
    let a2a_task_store = uar::api::a2a::TaskStore::new();
    let a2a_state = Arc::new(uar::api::a2a::A2AState {
        compiler_service: Arc::clone(&compiler_service),
        task_store: a2a_task_store,
        base_url: format!("http://{}:{}", config.server.host, config.server.port),
    });
    info!("A2A state initialized");
    let federated_agent_registry: Arc<dyn uar::api::a2a::AgentRegistry> = agent_registry
        .unwrap_or_else(|| Arc::new(crate::uar::api::a2a::registry::InMemoryAgentRegistry::new()));

    // Initialize SettingsManager — seed config into DB, detect drift on restart.
    let settings_manager: Option<Arc<crate::uar::settings::manager::SettingsManager>> =
        if let Some(p) = &persistence {
            let mgr = Arc::new(crate::uar::settings::manager::SettingsManager::new(
                Arc::clone(p),
            ));
            match mgr.initialize(&config).await {
                Ok(stats) => {
                    info!(
                        seeded = stats.seeded,
                        updated = stats.updated,
                        drift = stats.drift_count,
                        types = stats.types_upserted,
                        "Settings bootstrapped from config into DB"
                    );
                    // Seed the env/YAML/CLI-configured providers (in the in-memory
                    // registry) into the DB on first boot so they are visible +
                    // editable in the admin UI and survive restarts. DB-wins-after-
                    // first-boot: existing rows (e.g. API-edited) are left untouched.
                    if let Err(e) = mgr
                        .seed_providers_from_registry(provider_registry.as_ref())
                        .await
                    {
                        tracing::error!(error = ?e, "Failed to seed configured providers into the settings DB");
                    }
                    // Runtime LLM provider state comes from DB after bootstrap (YAML/env/CLI seeded rows).
                    if let Err(e) = crate::uar::settings::hydrate_provider_registry_from_settings(
                        provider_registry.as_ref(),
                        mgr.as_ref(),
                    )
                    .await
                    {
                        tracing::error!(
                            error = ?e,
                            "Failed to hydrate provider registry from settings database"
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Settings bootstrap failed — continuing without persistent settings")
                }
            }
            Some(mgr)
        } else {
            info!("No persistence layer — settings manager disabled");
            None
        };

    let prompt_cache_provider: Arc<dyn PromptCacheProvider> =
        match SurrealMemPromptCacheProvider::new().await {
            Ok(provider) => Arc::new(provider),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "failed to initialize prompt cache provider: {e}"
                ));
            }
        };

    let user_settings_store =
        Arc::new(crate::uar::runtime::user_settings_store::UserSettingsStore::new());

    let state = AppState {
        mcp,
        orchestrator,
        sessions,
        run_manager,
        ingest_service,
        vector_matcher: Arc::clone(&vector_matcher),
        persistence: persistence.clone(),
        rate_limiter,
        config: Arc::clone(&config),
        skill_service: Arc::clone(&skill_service),
        provider_registry: Arc::clone(&provider_registry),
        model_router: Arc::new(crate::llm::ModelRouter::new(Arc::clone(&provider_registry))),
        native_skill_registry: Arc::clone(&native_skill_registry),
        federated_agent_registry: Arc::clone(&federated_agent_registry),
        actor_system: Arc::clone(&actor_system),
        governance_engine: Arc::clone(&governance_engine),
        api_key_service: Some(Arc::clone(&api_key_service)),
        provider_service: provider_service.clone(),
        compiler_service: Some(Arc::clone(&compiler_service)),
        settings_manager: settings_manager.clone(),
        memory_service: memory_service.clone(),
        live_bus: live_bus.clone(),
        prompt_cache_provider,
        user_settings_store: Arc::clone(&user_settings_store),
        a2ui_registry: uar::a2ui::registry::A2uiRegistry::with_builtins(),
        agent_sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        #[cfg(feature = "wasm-runtime")]
        wasm_sandbox: {
            use crate::uar::runtime::wasm::{config::WasmConfig, sandbox::WasmSandbox};
            match WasmSandbox::new(WasmConfig::default()) {
                Ok(sb) => {
                    info!("Wasm sandbox runtime initialized");
                    Some(Arc::new(sb))
                }
                Err(e) => {
                    warn!(error = %e, "Failed to initialize Wasm sandbox — disabled");
                    None
                }
            }
        },
    };

    let provider_api_state = uar::api::providers::ProviderApiState {
        registry: Arc::clone(&state.provider_registry),
        settings_manager: state.settings_manager.clone(),
    };

    // ── Shared ingestion worker pool ─────────────────────────────────────────────
    // Built once before router assembly. A single `Arc` is cloned into both
    // knowledge-base router states so there is exactly one pool and one set of
    // OS threads. The `Arc` is also retained here so we can call
    // `ingestion_pool_shared.shutdown()` during graceful shutdown after the
    // signal fires — `shutdown(&self)` is callable through `Arc` in the new
    // crate revision (CR-02 fix).
    let ingestion_pool_shared: Option<Arc<IngestionWorkerPool>> = if let (
        Some(p),
        Some(ingest),
    ) = (&persistence, &state.ingest_service)
    {
        match IngestionWorkerPool::new(
            0,   // auto-detect CPU count
            100, // max queue depth
            Arc::clone(ingest),
            Arc::clone(p),
            Arc::clone(&config),
        ) {
            Ok(pool) => {
                info!("Shared ingestion worker pool initialized (single instance)");
                Some(Arc::new(pool))
            }
            Err(e) => {
                tracing::error!("Failed to create shared ingestion pool: {:?}", e);
                None
            }
        }
    } else {
        None
    };

    // Build router
    // Memory MCP router — built before the main router so type inference is unambiguous.
    let mem_mcp_router: axum::Router<()> = if let Some(ref svc) = state.memory_service {
        uar::memory::mcp_server::memory_mcp_router(Arc::clone(svc))
    } else {
        axum::Router::new()
    };

    // UAR Runtime MCP router — exposes agent listing, run creation, skill inventory,
    // and spec compilation as MCP tools at /mcp/uar.
    let uar_mcp_router: axum::Router<()> = uar::mcp_server::uar_mcp_router(
        Arc::clone(&state.run_manager),
        Arc::clone(&state.native_skill_registry),
        state.persistence.clone(),
    );

    let app = Router::new()
        .merge(utoipa_swagger_ui::SwaggerUi::new("/api/docs").url(
            "/api/openapi.json",
            crate::uar::api::openapi::build_openapi_spec(),
        ))
        .route("/health", get(liveness_handler))
        .route("/healthz", get(liveness_handler))
        .route("/readyz", get(readiness_handler))
        .route("/api/models", get(api_models))
        .route("/api/catalog", get(api_catalog))
        .route("/api/uar/route", post(api_route_model))
        .route("/api/generate-title", post(api_generate_title))
        .route("/api/chat/completion", post(api_chat_completion))
        .route("/api/upload", post(uar::api::upload::upload_handler))
        .route("/api/live/{topic}", get(uar::api::live::live_stream))
        .route(
            "/api/attachments/{id}",
            get(uar::api::upload::serve_attachment_handler),
        )
        .route("/api/chat", any(legacy_chat_route_disabled))
        .route("/api/chat/{*path}", any(legacy_chat_route_disabled))
        .route("/api/sessions", any(legacy_sessions_route_disabled))
        .route("/api/sessions/{*path}", any(legacy_sessions_route_disabled))
        .nest(
            "/api/uar",
            uar::api::router().with_state(Arc::clone(&state.run_manager)),
        )
        // Skills API
        .nest(
            "/api/uar/skills",
            uar::api::skills::build_router().with_state(Arc::clone(&state.skill_service)),
        )
        // Agent-Skills Bindings API
        .nest(
            "/api/uar/agents",
            uar::api::skills::build_agent_skills_router()
                .with_state(Arc::clone(&state.skill_service)),
        )
        // Providers API
        .nest(
            "/api/uar/providers",
            uar::api::providers::build_router().with_state(provider_api_state.clone()),
        )
        // Discovery API (agents, sessions, skills, tools catalogs)
        .nest(
            "/api/uar/discovery",
            uar::api::discovery::build_router().with_state(state.clone()),
        )
        // Auth API (API key management)
        .nest(
            "/api/uar/auth",
            uar::api::auth::build_router().with_state(Arc::new(uar::api::auth::AuthApiState {
                api_key_service: Arc::clone(&api_key_service),
            })),
        )
        // Compiler API (spec management + compilation)
        .nest(
            "/api/uar/compiler",
            uar::api::compiler::build_router().with_state(Arc::new(
                uar::api::compiler::CompilerApiState {
                    compiler_service: Arc::clone(&compiler_service),
                    persistence: persistence.clone(),
                },
            )),
        )
        // Actors API
        .nest(
            "/api/uar/actors",
            uar::api::actors::build_router().with_state(Arc::clone(&state.actor_system)),
        )
        // Settings API (runtime config administration)
        .nest(
            "/api/uar/settings",
            uar::api::settings::build_router().with_state(Arc::new(
                uar::api::settings::SettingsApiState {
                    settings_manager: settings_manager.clone(),
                    settings_mutation_auth_required: config
                        .security
                        .settings_mutation_auth_required,
                },
            )),
        )
        // Per-user settings API (prompt caching preferences etc.)
        .nest(
            "/api/uar/user",
            uar::api::user_settings::build_router().with_state(Arc::clone(&user_settings_store)),
        )
        // Per-user provider credential API (multi-tenant BYO keys)
        .nest(
            "/api/uar/credentials",
            uar::api::credentials::build_router().with_state(state.provider_service.clone()),
        )
        // Admin Memory API (CRUD)
        .nest(
            "/api/admin/memories",
            uar::api::memory_admin::build_router().with_state(state.clone()),
        )
        // Memory MCP HTTP endpoint — exposes the full in-process memory MCP server
        // over streamable-HTTP so Claude Desktop and other MCP clients can connect.
        // Uses nest_service (not nest) because mem_mcp_router is Router<()> and
        // the outer router is Router<AppState> — nest_service accepts any Service.
        .nest_service(&config.memory.mcp_http_path, mem_mcp_router)
        // UAR Runtime MCP endpoint — exposes agent registry, run creation, skill
        // inventory, and spec compilation as MCP tools at /mcp/uar.
        .nest_service("/mcp/uar", uar_mcp_router)
        // A2UI schema listing endpoint
        .nest("/api/uar/a2ui", {
            let a2ui_state = uar::a2ui::routes::A2uiApiState {
                registry: Arc::clone(&state.a2ui_registry),
                run_manager: Arc::clone(&state.run_manager),
            };
            uar::a2ui::routes::build_schema_router().with_state(a2ui_state)
        })
        // A2UI artifact-response injection (shares /api/uar/runs prefix)
        .nest("/api/uar/runs", {
            let a2ui_state = uar::a2ui::routes::A2uiApiState {
                registry: Arc::clone(&state.a2ui_registry),
                run_manager: Arc::clone(&state.run_manager),
            };
            uar::a2ui::routes::build_response_router().with_state(a2ui_state)
        })
        // Tool-call approval HITL gate: POST /api/uar/runs/{run_id}/approval
        .route(
            "/api/uar/runs/{run_id}/approval",
            post(handle_tool_call_approval),
        )
        // A2A Compiler Agent — JSON-RPC endpoint
        .nest(
            "/a2a/compiler",
            uar::api::a2a::build_rpc_router().with_state(Arc::clone(&a2a_state)),
        )
        // A2A well-known AgentCard
        .nest(
            "/.well-known",
            uar::api::a2a::build_well_known_router().with_state(Arc::clone(&a2a_state)),
        )
        // A2A Registry & Discovery
        .nest(
            "/a2a/registry",
            uar::api::a2a::build_discovery_router().with_state(Arc::new(
                uar::api::a2a::DiscoveryApiState {
                    registry: Arc::clone(&federated_agent_registry),
                },
            )),
        )
        // Knowledge Base API
        .nest("/api/uar/knowledge-bases", {
            // Use the shared ingestion pool (single instance, hoisted above).
            let ingestion_pool = ingestion_pool_shared.clone();

            uar::api::knowledge::build_router().with_state(Arc::new(
                uar::api::knowledge::KnowledgeApiState {
                    persistence: persistence
                        .clone()
                        .expect("Persistence required for KB API"),
                    vector_matcher: Arc::clone(&vector_matcher),
                    ingestion_pool,
                },
            ))
        })
        // ── Short-path aliases (frontend uses /api/X without the /uar/ prefix) ──────────
        // Providers: GET/POST /api/providers, GET/PUT/DELETE /api/providers/{id}, etc.
        .nest(
            "/api/providers",
            uar::api::providers::build_router().with_state(provider_api_state.clone()),
        )
        // Skills: GET /api/skills, GET/DELETE /api/skills/{id}, etc.
        .nest(
            "/api/skills",
            uar::api::skills::build_router().with_state(Arc::clone(&state.skill_service)),
        )
        // Agent–skill bindings: GET/PUT /api/agents/{id}/skills, etc.
        .nest(
            "/api/agents",
            uar::api::skills::build_agent_skills_router()
                .with_state(Arc::clone(&state.skill_service)),
        )
        // Auth (API key management): GET/POST /api/auth/keys, DELETE /api/auth/keys/{id}
        .nest(
            "/api/auth",
            uar::api::auth::build_router().with_state(Arc::new(uar::api::auth::AuthApiState {
                api_key_service: Arc::clone(&api_key_service),
            })),
        )
        // Compiler: GET/POST /api/compiler/sessions, etc.
        .nest(
            "/api/compiler",
            uar::api::compiler::build_router().with_state(Arc::new(
                uar::api::compiler::CompilerApiState {
                    compiler_service: Arc::clone(&compiler_service),
                    persistence: persistence.clone(),
                },
            )),
        )
        // Knowledge bases: GET/POST /api/knowledge, etc.
        .nest("/api/knowledge", {
            // Use the shared ingestion pool (alias path; same Arc as above).
            let ingestion_pool = ingestion_pool_shared.clone();
            uar::api::knowledge::build_router().with_state(Arc::new(
                uar::api::knowledge::KnowledgeApiState {
                    persistence: persistence
                        .clone()
                        .expect("Persistence required for KB API"),
                    vector_matcher: Arc::clone(&vector_matcher),
                    ingestion_pool,
                },
            ))
        })
        // Discovery catalog endpoints: GET /api/agents (list), GET /api/tools
        // The agents router above handles /api/agents/{id}/skills CRUD;
        // the discovery handlers provide the flat catalog list.
        .route(
            "/api/agents",
            get(uar::api::discovery::list_agents).post(uar::api::discovery::create_agent),
        )
        .route(
            "/api/agents/{id}",
            put(uar::api::discovery::update_agent_full)
                .patch(uar::api::discovery::patch_agent)
                .delete(uar::api::discovery::delete_agent),
        )
        .route("/api/tools", get(uar::api::discovery::list_tools))
        .route(
            "/api/tools/{name}/execute",
            post(uar::api::discovery::execute_tool),
        )
        .route("/api/uar/mcp/health", get(api_mcp_health))
        .route(
            "/api/uar/sessions/{id}/context-stats",
            get(api_context_stats),
        )
        // Agent session config: per-conversation overrides of agent defaults
        .route(
            "/api/uar/sessions/{id}/agent-config",
            get(uar::api::discovery::get_agent_session_config)
                .post(uar::api::discovery::save_agent_session_config),
        )
        .route(
            "/api/uar/sessions/{id}/effective-config",
            get(uar::api::discovery::get_effective_config),
        )
        .route("/api/uar/skills/reload", post(api_skills_reload))
        // ────────────────────────────────────────────────────────────────────────────
        .route("/api/ingest", post(uar::api::ingest::ingest_handler))
        .route(
            "/api/memory",
            post(uar::api::memory::save_memory_handler)
                .get(uar::api::memory::search_memory_handler),
        )
        // Persistence info + SSE sync stream
        .route("/api/config/persistence", get(persistence_info_handler))
        .route("/api/uar/sync/stream", get(sync_stream_handler))
        .route("/api/{*path}", any(api_route_not_found))
        .route("/v1/chat/completions", post(api_chat_completion))
        .route("/v1/messages", post(api_messages))
        .route("/v1/models", get(api_v1_models))
        .route("/v1/models/{model_id}", get(api_v1_model_detail))
        .route("/metrics", get(api_metrics))
        // ── SPA client-side routes ────────────────────────────────────────────
        // These paths are handled by React Router in the browser. When a user
        // hard-refreshes or navigates directly to one of them, the browser sends
        // a real GET request to the server. We explicitly serve index.html so
        // that React Router can take over and render the correct view.
        // Without this, ServeDir's not_found_service can race with the 404 path
        // in some middleware configurations and leak a 404 to the browser.
        .route("/", get(spa_index_handler))
        .route("/threads", get(spa_index_handler))
        .route("/admin", get(spa_index_handler))
        .route("/admin/{*path}", get(spa_index_handler))
        .route("/about", get(spa_index_handler))
        // Serve the React SPA from static/.
        // ServeDir serves /assets/*, /favicon.svg, /manifest.json etc. with correct MIME types.
        // The not_found_service fallback delivers index.html for any other unknown paths.
        .fallback_service({
            let dir = resolve_static_dir();
            let index = dir.join("index.html");
            ServeDir::new(dir).not_found_service(ServeFile::new(index))
        })
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            uar::security::middleware::auth_middleware,
        ))
        // Apply Timeout Layer if not disabled
        // We use a large timeout if disabled instead of conditional layering to keep types consistent
        .layer(TraceLayer::new_for_http());

    // ── ACP (Agent Communication Protocol) endpoint ──────────────────────────
    // Mounted conditionally so zero overhead is incurred when disabled.
    let app = if config.acp.enabled {
        info!(path = %config.acp.path, "ACP server enabled");
        app.nest_service(
            &config.acp.path,
            uar::api::acp::routes::AcpRouter::new().into_router(Arc::new(state.clone())),
        )
    } else {
        app
    };

    // We can't easily conditionally apply a layer in the chain if types differ.
    // Standard pattern:
    // let app = app.layer(...)
    // if condition { app = app.layer(...) } -> type changes!
    // Axum Router type changes with layers.
    // Actually, TimeoutLayer is a Service layer.

    // Instead of fighting types, we can use `ServiceBuilder` but even then.
    // A common workaround:
    // Configure all layers in a `ServiceBuilder`? No, same type issue.
    //
    // Option A: Use `BoxRoute` / `BoxService` (performance hit).
    // Option B: Always apply a layer, but make it a No-Op? `TimeoutLayer` doesn't have a no-op mode easily.
    // Option C: Middleware refactoring.

    // Actually, for timeouts, we likely ALWAYs want a timeout unless debugging.
    // But user requirement says "can be turned off".

    // Let's use `MapRequest` or similar? No.
    // Let's go with the `stack` approach if possible or accepted boilerplate.

    // Or we just accept the Box overhead? It's fine for this app.
    // `app.boxed()` ?

    // Let's try to just rebuild the router or use `tower::ServiceBuilder`.
    // Actually, `Router` has `layer`.

    // If I do:
    // let app = Router::new()...;
    // let app = if config.enabled { app.layer(Limit) } else { app };
    // This fails because `app` type changes.

    // Solution: `tower::util::OptionLayer`? No, maybe simpler:
    // Just wrap logic in a custom middleware that applies the timeout?
    // `Timeout` is a service, not just middleware fn.

    // Let's stick to adding them unconditionally for now BUT utilize a very large timeout if disabled?
    // "Timeout disabled" -> Duration::MAX?
    // That effectively disables it without changing types.

    let timeout_duration = if config.resilience.timeout_disabled {
        Duration::from_secs(365 * 24 * 60 * 60) // 1 year
    } else {
        Duration::from_millis(config.resilience.request_timeout_ms.max(1_000))
    };

    let app = app
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit
        .layer(axum::middleware::from_fn(
            move |req: Request, next: Next| {
                let duration = timeout_duration;
                async move {
                    let path = req.uri().path().to_string();
                    if !should_apply_request_timeout(&path) {
                        return next.run(req).await;
                    }
                    match tokio::time::timeout(duration, next.run(req)).await {
                        Ok(res) => res,
                        Err(_) => {
                            (StatusCode::REQUEST_TIMEOUT, "Request timed out").into_response()
                        }
                    }
                }
            },
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            uar::security::rate_limit::rate_limit_middleware,
        ))
        .layer(build_permissive_cors_layer())
        .layer(axum::middleware::from_fn(
            |req: Request, next: Next| async move {
                let method = req.method().to_string();
                let path = req.uri().path().to_string();
                let timer = telemetry_metrics::request_timer();
                let response = next.run(req).await;
                let status = response.status().as_u16();
                let duration = timer.elapsed();
                telemetry_metrics::record_request(&method, &path, status, duration);
                response
            },
        ))
        .with_state(state);

    // ── A2A v0.3 gRPC transport ──────────────────────────────────────────────
    // Disabled pending tonic-build proto compilation setup.
    // When enabled, this spawns a gRPC server on a separate port (default 50051)
    // alongside the main HTTP server.
    // {
    //     let grpc_port = config.server.grpc_port;
    //     let grpc_addr: std::net::SocketAddr = format!("0.0.0.0:{grpc_port}")
    //         .parse()
    //         .expect("invalid gRPC address");
    //     let grpc_service =
    //         crate::uar::api::a2a::grpc::GrpcAgentService::new(Arc::clone(&a2a_state));
    //     tokio::spawn(async move {
    //         if let Err(e) = tonic::transport::Server::builder()
    //             .add_service(grpc_service.into_server())
    //             .serve(grpc_addr)
    //             .await
    //         {
    //             tracing::error!(error = %e, "A2A gRPC server error");
    //         }
    //     });
    // }

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let shutdown_timeout = Duration::from_secs(config.server.shutdown_timeout_secs);

    info!(
        name: "server.started",
        address = %addr,
        shutdown_timeout_secs = config.server.shutdown_timeout_secs,
        "Server started"
    );

    // Build a one-shot shutdown channel so the signal handler can notify
    // the Axum serve loop AND the pool shutdown in a single coordinated
    // sequence, without spawning extra tasks.
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Spawn the signal handler: wait for SIGINT/SIGTERM, then:
    //   1. Shut down the ingestion worker pool (drains with timeout, detaches wedges).
    //   2. Fire the Axum graceful-shutdown trigger.
    {
        let pool_for_shutdown = ingestion_pool_shared.clone();
        tokio::spawn(async move {
            prometheus_parking_lot::core::shutdown::wait_for_signal().await;
            info!(
                name: "server.shutdown",
                timeout_secs = shutdown_timeout.as_secs(),
                "Shutdown signal received — draining ingestion pool"
            );
            // Drain and cancel any wedged ingestion workers before closing sockets.
            if let Some(pool) = pool_for_shutdown {
                pool.shutdown();
            }
            info!(name: "server.shutdown.pool_drained", "Ingestion pool shut down — closing HTTP connections");
            // Signal Axum to stop accepting and drain open connections.
            let _ = shutdown_tx.send(());
        });
    }

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
            info!(
                name: "server.draining",
                timeout_secs = shutdown_timeout.as_secs(),
                "Draining in-flight HTTP connections"
            );
            tokio::time::sleep(shutdown_timeout).await;
        })
        .await?;

    info!(name: "server.stopped", "Server shut down gracefully");
    Ok(())
}

fn normalize_legacy_openai_base_url(llm_config: &mut crate::config::LlmConfig) {
    let Some(base_url) = llm_config.base_url.as_deref() else {
        return;
    };
    let normalized = base_url.trim().trim_end_matches('/');
    if normalized != "https://api.openai.com" && normalized != "https://api.openai.com/v1" {
        return;
    }

    let (provider_id, _) = crate::llm::registry::split_model_string_pub(&llm_config.model);
    if provider_id == "openai" && normalized == "https://api.openai.com" {
        llm_config.base_url = Some("https://api.openai.com/v1".to_string());
        return;
    }
    if provider_id == "openai" {
        return;
    }

    if let Some(provider_base_url) = crate::llm::registry::fallback_base_url(&provider_id) {
        tracing::warn!(
            provider_id = %provider_id,
            old_base_url = %base_url,
            new_base_url = %provider_base_url,
            "Replacing legacy OpenAI base URL for non-OpenAI provider model"
        );
        llm_config.base_url = Some(provider_base_url.to_string());
    }
}


fn build_permissive_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// POST /api/uar/runs/{run_id}/approval
///
/// Human-in-the-loop gate for pending tool calls.
///
/// Body: `{ "approved": true | false }`.
/// Returns 200 if the run was waiting for this approval and it was resolved.
/// Returns 404 if no run with that id has a pending approval.
async fn handle_tool_call_approval(
    State(state): State<AppState>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let approved = body.get("approved").and_then(|v| v.as_bool()).unwrap_or(false);
    let resolved = state.run_manager.resolve_approval(&run_id, approved).await;
    if resolved {
        info!(
            name: "approval.resolved",
            run_id = %run_id,
            approved,
            "Tool-call approval resolved"
        );
        Json(serde_json::json!({ "ok": true, "run_id": run_id, "approved": approved }))
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "No pending approval found for run",
                "run_id": run_id
            })),
        )
            .into_response()
    }
}

async fn load_global_resilience_policy(state: &AppState) -> ResiliencePolicy {
    let mut policy = ResiliencePolicy::from(&state.config.resilience);

    let Some(mgr) = &state.settings_manager else {
        return policy;
    };

    macro_rules! apply_typed {
        ($key:literal, $ty:ty, $field:ident) => {
            if let Ok(Some(v)) = mgr.get_typed::<$ty>($key).await {
                policy.$field = v;
            }
        };
    }

    apply_typed!("resilience.rate_limit_enabled", bool, rate_limit_enabled);
    apply_typed!("resilience.requests_per_second", f32, requests_per_second);
    apply_typed!("resilience.burst_size", f32, burst_size);
    apply_typed!("resilience.request_timeout_ms", u64, request_timeout_ms);
    apply_typed!(
        "resilience.stream_start_timeout_ms",
        u64,
        stream_start_timeout_ms
    );
    apply_typed!("resilience.retries_enabled", bool, retries_enabled);
    apply_typed!("resilience.retry_max_attempts", u32, retry_max_attempts);
    apply_typed!("resilience.retry_base_delay_ms", u64, retry_base_delay_ms);
    apply_typed!(
        "resilience.retry_backoff_multiplier",
        f32,
        retry_backoff_multiplier
    );
    apply_typed!("resilience.retry_max_delay_ms", u64, retry_max_delay_ms);
    apply_typed!("resilience.retry_jitter_mode", String, retry_jitter_mode);
    apply_typed!(
        "resilience.retry_respect_retry_after",
        bool,
        retry_respect_retry_after
    );
    apply_typed!(
        "resilience.retryable_http_statuses",
        Vec<u16>,
        retryable_http_statuses
    );
    apply_typed!(
        "resilience.retryable_transport_errors",
        bool,
        retryable_transport_errors
    );
    apply_typed!("resilience.retry_budget_ms", u64, retry_budget_ms);

    if let Err(err) = policy.validate() {
        tracing::warn!(
            error = %err,
            "Invalid global resilience settings detected; falling back to config defaults"
        );
        return ResiliencePolicy::from(&state.config.resilience);
    }

    policy
}

async fn resolve_effective_resilience_policy(
    state: &AppState,
    agent_id: &str,
) -> (ResiliencePolicy, PolicySource) {
    let global = load_global_resilience_policy(state).await;
    let Some(mgr) = &state.settings_manager else {
        return (global, PolicySource::Global);
    };

    let mut lookup_keys = vec![format!("agent_config.{agent_id}")];
    if agent_id == "default-agent" {
        lookup_keys.push("agent_config.orchestrated".to_string());
    }

    for key in lookup_keys {
        if let Some(agent_cfg) = mgr.get_value(&key).await {
            match resolve_effective_policy(&global, Some(&agent_cfg)) {
                Ok(resolved) => return resolved,
                Err(err) => {
                    tracing::warn!(
                        setting_key = %key,
                        error = %err,
                        "Invalid per-agent resilience override; using global policy"
                    );
                }
            }
        }
    }

    (global, PolicySource::Global)
}

// ─────────────────────────────────────────────────────────────────────────────
// API Handlers
// ─────────────────────────────────────────────────────────────────────────────

// Removed index_handler and about_handler - now serving static HTML files

/// Liveness probe — lightweight, no dependency checks.
/// Returns 200 if the process can serve HTTP.
pub(crate) async fn liveness_handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

/// Readiness probe — verifies core dependencies are operational.
/// Returns 200 if all configured dependencies are reachable, 503 otherwise.
pub(crate) async fn readiness_handler(State(state): State<AppState>) -> Response {
    let mut checks = serde_json::Map::new();
    let mut all_ok = true;

    // Check PostgreSQL via persistence layer — attempt a lightweight operation
    match &state.persistence {
        Some(p) => {
            // Use list_skills with limit 0 as a lightweight connectivity test
            match p.list_skills().await {
                Ok(_) => {
                    checks.insert("postgres".into(), json!("ok"));
                }
                Err(_) => {
                    checks.insert("postgres".into(), json!("failed"));
                    all_ok = false;
                }
            }
        }
        None => {
            checks.insert("postgres".into(), json!("not_configured"));
        }
    }

    // Check SurrealDB via memory service
    match &state.memory_service {
        Some(_svc) => {
            // Memory service is initialized and connected
            checks.insert("surrealdb".into(), json!("ok"));
        }
        None => {
            checks.insert("surrealdb".into(), json!("not_configured"));
        }
    }

    // Check MCP registry — verify at least the registry is initialized
    let tool_count = state.mcp.tools().len();
    checks.insert("mcp".into(), json!({"status": "ok", "tools": tool_count}));

    let status_text = if all_ok { "ready" } else { "not_ready" };
    let status_code = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "status": status_text,
            "checks": checks
        })),
    )
        .into_response()
}

fn should_apply_request_timeout(path: &str) -> bool {
    !matches!(
        path,
        "/api/chat/completion" | "/v1/chat/completions" | "/v1/messages"
    )
}

#[derive(Debug, Deserialize)]
struct GenerateTitleRequest {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    assistant_message: Option<String>,
}

fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn title_from_text(input: &str) -> String {
    let normalized = collapse_whitespace(input);
    if normalized.is_empty() {
        return "New Conversation".to_string();
    }

    let mut words = normalized.split(' ').take(8).collect::<Vec<_>>().join(" ");
    if words.len() > 64 {
        words.truncate(64);
        words = words.trim_end().to_string();
    }
    words
}

async fn api_generate_title(Json(req): Json<GenerateTitleRequest>) -> Response {
    let seed = req
        .message
        .as_deref()
        .filter(|m| !m.trim().is_empty())
        .or_else(|| {
            req.assistant_message
                .as_deref()
                .filter(|m| !m.trim().is_empty())
        })
        .unwrap_or("New Conversation");

    let title = title_from_text(seed);
    Json(json!({ "title": title })).into_response()
}

/// GET /api/models
///
/// Returns the full model catalog (all providers × models from the compile-time
/// `ModelCatalog`), enriched with a `configured` flag indicating whether the
/// provider has a live API key registered in the `ProviderRegistry`.
///
/// Response shape:
/// ```json
/// {
///   "<provider_id>": {
///     "display_name": "OpenAI",
///     "base_url": "https://api.openai.com/v1",
///     "configured": true,
///     "models": {
///       "<model_id>": {
///         "name": "GPT-4o",
///         "limit": { "context": 128000, "input": 128000, "output": 16384 },
///         "cost": { "input": 2.50, "output": 10.00 },
///         "modalities": { "input": ["text", "image"], "output": ["text"] },
///         "tool_call": true,
///         "reasoning": false
///       }
///     }
///   }
/// }
/// ```
async fn api_models(State(state): State<AppState>) -> Response {
    let catalog = crate::llm::ModelCatalog::global();
    let configured_ids: std::collections::HashSet<String> = state
        .provider_registry
        .list()
        .await
        .into_iter()
        .filter(|p| p.enabled)
        .map(|p| p.id)
        .collect();

    let mut root = serde_json::Map::new();

    for provider in catalog.all_providers() {
        let mut models = serde_json::Map::new();
        for model in &provider.models {
            let input_cost = model.cost.as_ref().map(|c| c.input).unwrap_or(0.0);
            let output_cost = model.cost.as_ref().map(|c| c.output).unwrap_or(0.0);
            let context = if model.limits.context_window > 0 {
                model.limits.context_window
            } else {
                128_000
            };
            let max_output = if model.limits.max_output > 0 {
                model.limits.max_output
            } else {
                4_096
            };

            models.insert(
                model.id.clone(),
                json!({
                    "name": model.name,
                    "family": model.family,
                    "limit": {
                        "context": context,
                        "input": context,
                        "output": max_output
                    },
                    "cost": {
                        "input": input_cost,
                        "output": output_cost
                    },
                    "modalities": {
                        "input": model.modalities.input,
                        "output": model.modalities.output
                    },
                    "tool_call": model.capabilities.tool_call,
                    "reasoning": model.capabilities.reasoning,
                    "structured_output": model.capabilities.structured_output,
                    "streaming": model.capabilities.streaming,
                    "open_weights": model.open_weights
                }),
            );
        }

        root.insert(
            provider.id.clone(),
            json!({
                "display_name": provider.display_name,
                "base_url": provider.base_url,
                "configured": configured_ids.contains(&provider.id),
                "models": models
            }),
        );
    }

    Json(Value::Object(root)).into_response()
}

/// GET /api/catalog
///
/// Returns a lightweight summary of all providers known by the compile-time
/// `ModelCatalog` — no API keys exposed, suitable for the admin UI discovery page.
async fn api_catalog(State(state): State<AppState>) -> Response {
    let catalog = crate::llm::ModelCatalog::global();
    // "Configured" means: (1) enabled in the registry AND (2) has a usable API
    // key. A provider that is enabled but has no key will show as
    // `credential-blocked`, not `configured`, to prevent false positives.
    let configured_ids: std::collections::HashSet<String> = state
        .provider_registry
        .list()
        .await
        .into_iter()
        .filter(|p| {
            p.enabled
                && p.api_key
                    .as_deref()
                    .map_or(false, |k| !k.trim().is_empty())
        })
        .map(|p| p.id)
        .collect();

    let providers: Vec<Value> = catalog
        .all_providers()
        .iter()
        .map(|p| {
            let env_var = p.auth.as_ref().and_then(|a| a.env_var.clone());
            let configured = configured_ids.contains(&p.id);
            let (status, status_detail) =
                provider_catalog_status(&p.id, configured, env_var.as_deref());
            json!({
                "id": p.id,
                "display_name": p.display_name,
                "base_url": p.base_url,
                "model_count": p.models.len(),
                "configured": configured,
                "auth_env_var": env_var,
                "status": status,
                "status_detail": status_detail,
                "endpoints": p.endpoints,
            })
        })
        .collect();

    Json(json!({
        "provider_count": catalog.provider_count(),
        "model_count": catalog.model_count(),
        "providers": providers
    }))
    .into_response()
}

fn provider_catalog_status(
    provider_id: &str,
    configured: bool,
    auth_env_var: Option<&str>,
) -> (&'static str, String) {
    if configured {
        return (
            "configured",
            format!("{provider_id} is configured and enabled in the provider registry."),
        );
    }

    if let Some(env_var) = auth_env_var.filter(|v| !v.trim().is_empty()) {
        return (
            "credential-blocked",
            format!("{provider_id} requires a configured credential such as {env_var}."),
        );
    }

    (
        "available",
        format!("{provider_id} is available in the catalog but is not configured."),
    )
}

/// POST /api/uar/route
///
/// Selects the best available model based on capability requirements.
///
/// Request body (all optional):
/// ```json
/// {
///   "needs_tools": true,
///   "needs_reasoning": false,
///   "needs_vision": false,
///   "needs_structured_output": false,
///   "min_context": 32000,
///   "max_cost_per_1m_input": 5.0,
///   "preferred_provider": "openai"
/// }
/// ```
///
/// Response:
/// ```json
/// { "model": "openai/gpt-4o" }
/// ```
/// or `404` if no suitable model is available.
async fn api_route_model(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Response {
    use crate::llm::router::RouteRequirements;

    let requirements = RouteRequirements {
        needs_tools: req
            .get("needs_tools")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        needs_reasoning: req
            .get("needs_reasoning")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        needs_vision: req
            .get("needs_vision")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        needs_structured_output: req
            .get("needs_structured_output")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        min_context: req.get("min_context").and_then(|v| v.as_u64()),
        max_cost_per_1m_input: req.get("max_cost_per_1m_input").and_then(|v| v.as_f64()),
        preferred_provider: req
            .get("preferred_provider")
            .and_then(|v| v.as_str())
            .map(String::from),
    };

    match state.model_router.route(&requirements).await {
        Some(model) => Json(json!({ "model": model })).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "No suitable model found" })),
        )
            .into_response(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OpenAI-Compatible /v1/models Endpoints
// ─────────────────────────────────────────────────────────────────────────────

/// GET /v1/models — list available models in OpenAI API format.
pub(crate) async fn api_v1_models(State(state): State<AppState>) -> Json<Value> {
    let catalog = crate::llm::ModelCatalog::global();
    let configured_ids: std::collections::HashSet<String> = state
        .provider_registry
        .list()
        .await
        .into_iter()
        .filter(|p| p.enabled)
        .map(|p| p.id)
        .collect();

    let now = Utc::now().timestamp();
    let mut models = Vec::new();

    for provider in catalog.all_providers() {
        if !configured_ids.contains(&provider.id) {
            continue;
        }
        for model in &provider.models {
            models.push(json!({
                "id": format!("{}/{}", provider.id, model.id),
                "object": "model",
                "created": now,
                "owned_by": provider.id,
            }));
        }
    }

    Json(json!({
        "object": "list",
        "data": models
    }))
}

/// GET /v1/models/{model_id} — retrieve details for a specific model.
async fn api_v1_model_detail(
    State(_state): State<AppState>,
    axum::extract::Path(model_id): axum::extract::Path<String>,
) -> Response {
    let catalog = crate::llm::ModelCatalog::global();

    // Parse "provider/model" format
    let parts: Vec<&str> = model_id.splitn(2, '/').collect();
    if parts.len() != 2 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "message": format!("Model '{model_id}' not found. Use provider/model format."),
                    "type": "invalid_request_error",
                    "code": "model_not_found"
                }
            })),
        )
            .into_response();
    }

    let (provider_id, raw_model_id) = (parts[0], parts[1]);

    match catalog.model(provider_id, raw_model_id) {
        Some(model) => {
            let now = Utc::now().timestamp();
            Json(json!({
                "id": format!("{provider_id}/{}", model.id),
                "object": "model",
                "created": now,
                "owned_by": provider_id,
                "capabilities": {
                    "tool_call": model.capabilities.tool_call,
                    "reasoning": model.capabilities.reasoning,
                    "structured_output": model.capabilities.structured_output,
                    "streaming": model.capabilities.streaming,
                },
                "limits": {
                    "context_window": model.limits.context_window,
                    "max_output": model.limits.max_output,
                },
            }))
            .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "message": format!("Model '{model_id}' not found."),
                    "type": "invalid_request_error",
                    "code": "model_not_found"
                }
            })),
        )
            .into_response(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Prometheus Metrics Endpoint
// ─────────────────────────────────────────────────────────────────────────────

/// GET /metrics — Prometheus text exposition format.
pub(crate) async fn api_metrics() -> Response {
    let handle = crate::uar::telemetry::metrics::metrics_handle();
    let output = handle.render();
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        output,
    )
        .into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP Health Endpoint
// ─────────────────────────────────────────────────────────────────────────────

/// GET /api/uar/mcp/health — returns health status of all configured MCP servers.
pub(crate) async fn api_mcp_health(State(state): State<AppState>) -> Json<Value> {
    let tools = state.mcp.tools();
    let tool_count = tools.len();

    // Group tools by server namespace
    let mut servers: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (namespaced_name, _tool) in tools {
        // Namespace format: server__tool
        if let Some(server) = namespaced_name.split("__").next() {
            *servers.entry(server.to_string()).or_insert(0) += 1;
        }
    }

    let server_list: Vec<Value> = servers
        .iter()
        .map(|(name, count)| {
            json!({
                "name": name,
                "status": "connected",
                "tool_count": count,
            })
        })
        .collect();

    Json(json!({
        "total_tools": tool_count,
        "servers": server_list,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Context Stats Endpoint
// ─────────────────────────────────────────────────────────────────────────────

/// GET /api/uar/sessions/{id}/context-stats — returns context window usage for a session.
async fn api_context_stats(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Response {
    let session = state.sessions.get(&session_id);
    match session {
        Some(s) => {
            let messages = s.messages();
            let token_count =
                crate::uar::runtime::context::token_service::TokenService::estimate_messages(
                    &messages,
                );
            let model_limit: usize = 128_000; // Default; could be resolved from model catalog
            let threshold = 0.85_f32;

            Json(json!({
                "session_id": session_id,
                "tokens_used": token_count,
                "tokens_limit": model_limit,
                "utilization": token_count as f64 / model_limit as f64,
                "threshold": threshold,
                "strategy": "SlidingWindow",
                "message_count": messages.len(),
            }))
            .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Session not found"})),
        )
            .into_response(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Skills Reload Endpoint
// ─────────────────────────────────────────────────────────────────────────────

/// POST /api/uar/skills/reload — manually trigger skill registry reload.
async fn api_skills_reload(State(state): State<AppState>) -> Json<Value> {
    // Trigger a refresh from all storage providers
    let count = match state.skill_service.refresh().await {
        Ok(skills) => skills.len(),
        Err(_) => state.skill_service.get_skills().await.len(),
    };
    Json(json!({
        "status": "reloaded",
        "skill_count": count,
    }))
}

async fn legacy_chat_route_disabled() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": {
                "message": "Legacy route disabled. Use POST /api/chat/completion",
                "type": "invalid_request_error",
                "code": "legacy_route_disabled"
            }
        })),
    )
        .into_response()
}

async fn legacy_sessions_route_disabled() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": {
                "message": "Session history route disabled. Reuse X-UAR-Session-ID with POST /api/chat/completion",
                "type": "invalid_request_error",
                "code": "legacy_route_disabled"
            }
        })),
    )
        .into_response()
}

/// Serves `static/index.html` for SPA client-side routes that need to survive
/// a hard-refresh or direct-URL navigation. The file is read fresh each call so
/// that a rolling update swaps content without a server restart.
/// Resolves the directory containing the built React SPA. Checked in order:
/// 1. `UAR_STATIC_DIR` environment variable.
/// 2. `./static` relative to the current working directory (dev/repo runs).
/// 3. `~/.uar/static` (the canonical location for installed binaries).
fn resolve_static_dir() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("UAR_STATIC_DIR") {
        let pb = std::path::PathBuf::from(p);
        if pb.exists() {
            return pb;
        }
    }
    let cwd = std::path::PathBuf::from("static");
    if cwd.exists() {
        return cwd;
    }
    if let Some(home) = dirs::home_dir() {
        let pb = home.join(".uar").join("static");
        if pb.exists() {
            return pb;
        }
    }
    cwd
}

async fn spa_index_handler() -> Response {
    let index = resolve_static_dir().join("index.html");
    match tokio::fs::read(&index).await {
        Ok(bytes) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            bytes,
        )
            .into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Frontend assets not found. The server may still be starting up.",
        )
            .into_response(),
    }
}

async fn api_route_not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": {
                "message": "API route not found",
                "type": "invalid_request_error",
                "code": "api_route_not_found"
            }
        })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
struct AnthropicMessagesRequest {
    #[serde(default)]
    model: String,
    #[serde(default)]
    messages: Vec<AnthropicMessageInput>,
    #[serde(default)]
    system: Option<AnthropicSystemInput>,
    #[serde(default)]
    tools: Vec<AnthropicToolInput>,
    #[serde(default)]
    stream: bool,
    /// Per-request prompt-caching override.
    ///
    /// When `true`, the UAR automatically injects `cache_control: {type: ephemeral}`
    /// into the last system block and the last human-turn content block before
    /// forwarding to Anthropic.
    #[serde(default)]
    prompt_caching_enabled: Option<bool>,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicMessageInput {
    role: String,
    #[serde(default)]
    content: AnthropicContentInput,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(untagged)]
enum AnthropicContentInput {
    Text(String),
    #[default]
    Empty,
    Blocks(Vec<AnthropicContentBlockInput>),
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AnthropicContentBlockInput {
    #[serde(rename = "type", default)]
    block_type: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    input: Option<Value>,
    #[serde(default)]
    source: Option<Value>,
    #[serde(default)]
    content: Option<Value>,
    #[serde(default)]
    tool_use_id: Option<String>,
    #[serde(default)]
    cache_control: Option<AnthropicCacheControlInput>,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AnthropicCacheControlInput {
    #[serde(rename = "type", default)]
    _cache_type: String,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(untagged)]
enum AnthropicSystemInput {
    Text(String),
    Blocks(Vec<AnthropicContentBlockInput>),
    #[default]
    Empty,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AnthropicToolInput {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    input_schema: Option<Value>,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Default)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
    cache_creation_input_tokens: u32,
    cache_read_input_tokens: u32,
}

#[derive(Debug, Clone)]
enum ActiveAnthropicBlock {
    Text { index: usize },
    Tool { index: usize, call_index: usize },
}

#[derive(Debug, Clone)]
struct ToolTrack {
    index: usize,
    id: String,
    name: String,
    input_json: String,
    sent_json: String,
    started: bool,
}

fn anthropic_error_response(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(json!({
            "type": "error",
            "error": {
                "type": "invalid_request_error",
                "message": message
            }
        })),
    )
        .into_response()
}

// ── Persistence info + sync stream endpoints ─────────────────────────────

/// GET /api/config/persistence — returns the configured persistence provider info.
async fn persistence_info_handler(State(state): State<AppState>) -> impl IntoResponse {
    let config = &state.config.persistence;
    let mode = if config.database_url.starts_with("surrealkv://")
        || config.database_url.starts_with("rocksdb://")
        || config.database_url.starts_with("mem://")
        || config.database_url.starts_with("file://")
    {
        "embedded"
    } else {
        "remote"
    };
    Json(json!({
        "provider": config.provider,
        "mode": mode,
        "database_url": config.database_url,
    }))
}

/// GET /api/uar/sync/stream — SSE stream for entity change events (embedded SurrealDB only).
///
/// V1: sends periodic heartbeats; real LIVE SELECT integration will come later.
async fn sync_stream_handler(State(state): State<AppState>) -> Response {
    let config = &state.config.persistence;
    let is_embedded = config.database_url.starts_with("surrealkv://")
        || config.database_url.starts_with("rocksdb://")
        || config.database_url.starts_with("mem://")
        || config.database_url.starts_with("file://");

    if !is_embedded {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "SSE sync stream is only available for embedded SurrealDB mode"
            })),
        )
            .into_response();
    }

    let persistence = state.persistence.clone();

    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        let mut last_check = chrono::Utc::now().to_rfc3339();

        // Send initial connected event so the client knows the stream is live.
        yield Ok::<_, std::convert::Infallible>(
            Event::default()
                .event("connected")
                .data(serde_json::json!({
                    "status": "connected",
                    "ts": chrono::Utc::now().to_rfc3339(),
                    "tables": ["knowledge_bases", "knowledge_documents", "skills", "agents", "settings"]
                }).to_string())
        );

        loop {
            interval.tick().await;

            let now = chrono::Utc::now();
            let now_rfc = now.to_rfc3339();

            // Poll entity tables for changes since last check.
            // KnowledgeBase has an `updated_at` RFC3339 field we can compare.
            if let Some(ref persistence) = persistence {
                match persistence.list_knowledge_bases().await {
                    Ok(kbs) => {
                        // Compare RFC3339 strings lexicographically (valid for ISO timestamps).
                        let changed: Vec<_> = kbs.iter()
                            .filter(|kb| kb.updated_at.as_str() > last_check.as_str())
                            .collect();
                        for kb in &changed {
                            yield Ok(Event::default().event("entity.change").data(
                                serde_json::json!({
                                    "table": "knowledge_bases",
                                    "action": "update",
                                    "id": kb.id,
                                    "record": kb,
                                    "ts": now_rfc
                                }).to_string()
                            ));
                        }
                    }
                    Err(e) => {
                        tracing::debug!(error = %e, "sync_stream: failed to poll knowledge_bases");
                    }
                }
            }

            last_check = now_rfc.clone();

            // Heartbeat with timestamp so clients can detect stale connections.
            yield Ok(Event::default().event("heartbeat").data(
                serde_json::json!({
                    "status": "connected",
                    "ts": now_rfc
                }).to_string()
            ));
        }
    };

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

fn anthropic_sse_event(name: &str, payload: Value) -> Event {
    let data = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    Event::default().event(name).data(data)
}

fn ensure_toolu_id(id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return format!("toolu_{}", Uuid::new_v4().simple());
    }
    if trimmed.starts_with("toolu_") {
        trimmed.to_string()
    } else {
        format!("toolu_{trimmed}")
    }
}

fn estimate_tokens(text: &str) -> u32 {
    crate::uar::runtime::context::token_service::TokenService::estimate_string(text)
        .min(u32::MAX as usize) as u32
}

fn hash_prompt_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn content_blocks(content: &AnthropicContentInput) -> Vec<AnthropicContentBlockInput> {
    match content {
        AnthropicContentInput::Text(text) => vec![AnthropicContentBlockInput {
            block_type: "text".to_string(),
            text: Some(text.clone()),
            ..AnthropicContentBlockInput::default()
        }],
        AnthropicContentInput::Blocks(blocks) => blocks.clone(),
        AnthropicContentInput::Empty => Vec::new(),
    }
}

fn system_blocks(system: &AnthropicSystemInput) -> Vec<AnthropicContentBlockInput> {
    match system {
        AnthropicSystemInput::Text(text) => vec![AnthropicContentBlockInput {
            block_type: "text".to_string(),
            text: Some(text.clone()),
            ..AnthropicContentBlockInput::default()
        }],
        AnthropicSystemInput::Blocks(blocks) => blocks.clone(),
        AnthropicSystemInput::Empty => Vec::new(),
    }
}

/// Inject `cache_control: {type: ephemeral}` into an Anthropic request.
///
/// Per the Anthropic prompt-caching spec, markers are placed on:
/// 1. The last text block in the system prompt (stable prefix — highest reuse).
/// 2. The last text block of the last human-turn message (conversation context).
///
/// Blocks that already carry a `cache_control` are left untouched.
fn inject_anthropic_cache_control(req: &mut AnthropicMessagesRequest) {
    let ephemeral = AnthropicCacheControlInput {
        _cache_type: "ephemeral".to_string(),
        _extra: Default::default(),
    };

    // 1. Mark the last text block in the system prompt.
    if let Some(system) = &mut req.system {
        match system {
            AnthropicSystemInput::Text(text) => {
                let owned = std::mem::take(text);
                *system = AnthropicSystemInput::Blocks(vec![AnthropicContentBlockInput {
                    block_type: "text".to_string(),
                    text: Some(owned),
                    cache_control: Some(ephemeral.clone()),
                    ..AnthropicContentBlockInput::default()
                }]);
            }
            AnthropicSystemInput::Blocks(blocks) => {
                if let Some(last_text) = blocks
                    .iter_mut()
                    .rev()
                    .find(|b| b.block_type == "text" && b.cache_control.is_none())
                {
                    last_text.cache_control = Some(ephemeral.clone());
                }
            }
            AnthropicSystemInput::Empty => {}
        }
    }

    // 2. Mark the last text block of the last human-turn message.
    if let Some(last_human) = req.messages.iter_mut().rev().find(|m| m.role == "user") {
        match &mut last_human.content {
            AnthropicContentInput::Text(text) => {
                let owned = std::mem::take(text);
                last_human.content =
                    AnthropicContentInput::Blocks(vec![AnthropicContentBlockInput {
                        block_type: "text".to_string(),
                        text: Some(owned),
                        cache_control: Some(ephemeral),
                        ..AnthropicContentBlockInput::default()
                    }]);
            }
            AnthropicContentInput::Blocks(blocks) => {
                if let Some(last_text) = blocks
                    .iter_mut()
                    .rev()
                    .find(|b| b.block_type == "text" && b.cache_control.is_none())
                {
                    last_text.cache_control = Some(ephemeral);
                }
            }
            AnthropicContentInput::Empty => {}
        }
    }
}

fn tool_result_text(block: &AnthropicContentBlockInput) -> String {
    let Some(content) = &block.content else {
        return String::new();
    };

    match content {
        Value::String(s) => s.clone(),
        Value::Array(parts) => {
            let mut text = String::new();
            for part in parts {
                if let Some(part_text) = part.get("text").and_then(Value::as_str) {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(part_text);
                } else if let Some(part_str) = part.as_str() {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(part_str);
                }
            }
            if text.is_empty() {
                content.to_string()
            } else {
                text
            }
        }
        _ => content.to_string(),
    }
}

fn anthropic_image_to_openai_part(block: &AnthropicContentBlockInput) -> Option<Value> {
    let source = block.source.as_ref()?;
    let source_type = source
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let url = if source_type == "base64" {
        let media = source
            .get("media_type")
            .and_then(Value::as_str)
            .unwrap_or("image/png");
        let data = source.get("data").and_then(Value::as_str)?;
        format!("data:{media};base64,{data}")
    } else {
        source
            .get("url")
            .and_then(Value::as_str)
            .map(ToString::to_string)?
    };

    Some(json!({
        "type": "image_url",
        "image_url": {
            "url": url,
            "detail": "auto"
        }
    }))
}

fn flush_user_parts(messages: &mut Vec<Value>, parts: &mut Vec<Value>) {
    if parts.is_empty() {
        return;
    }

    let content = if parts.len() == 1
        && parts[0]
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|kind| kind == "text")
    {
        parts[0]
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
            .into()
    } else {
        Value::Array(parts.clone())
    };

    messages.push(json!({
        "role": "user",
        "content": content
    }));
    parts.clear();
}

fn convert_anthropic_messages_to_openai(req: &AnthropicMessagesRequest) -> Vec<Value> {
    let mut out = Vec::new();

    if let Some(system) = &req.system {
        for block in system_blocks(system) {
            if block.block_type == "text"
                && let Some(text) = block.text
            {
                out.push(json!({
                    "role": "system",
                    "content": text
                }));
            }
        }
    }

    for message in &req.messages {
        let role = message.role.to_ascii_lowercase();
        let blocks = content_blocks(&message.content);

        if role == "assistant" {
            let mut text = String::new();
            let mut tool_calls = Vec::<Value>::new();

            for block in blocks {
                match block.block_type.as_str() {
                    "text" => {
                        if let Some(part) = block.text
                            && !part.is_empty()
                        {
                            if !text.is_empty() {
                                text.push('\n');
                            }
                            text.push_str(&part);
                        }
                    }
                    "tool_use" => {
                        let generated_id = format!("toolu_{}", Uuid::new_v4().simple());
                        let call_id =
                            ensure_toolu_id(block.id.as_deref().unwrap_or(generated_id.as_str()));
                        let name = block.name.unwrap_or_else(|| "tool".to_string());
                        let input = block.input.unwrap_or_else(|| json!({}));
                        tool_calls.push(json!({
                            "id": call_id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": input.to_string()
                            }
                        }));
                    }
                    _ => {}
                }
            }

            if !text.is_empty() || !tool_calls.is_empty() {
                out.push(json!({
                    "role": "assistant",
                    "content": text,
                    "tool_calls": if tool_calls.is_empty() { Value::Null } else { Value::Array(tool_calls) }
                }));
            }
            continue;
        }

        if role == "user" {
            let mut user_parts = Vec::<Value>::new();

            for block in blocks {
                match block.block_type.as_str() {
                    "text" => {
                        if let Some(text) = block.text {
                            user_parts.push(json!({
                                "type": "text",
                                "text": text
                            }));
                        }
                    }
                    "image" => {
                        if let Some(part) = anthropic_image_to_openai_part(&block) {
                            user_parts.push(part);
                        }
                    }
                    "tool_result" => {
                        flush_user_parts(&mut out, &mut user_parts);
                        out.push(json!({
                            "role": "tool",
                            "tool_call_id": ensure_toolu_id(block.tool_use_id.as_deref().unwrap_or_default()),
                            "content": tool_result_text(&block),
                        }));
                    }
                    _ => {
                        if let Some(text) = block.text {
                            user_parts.push(json!({
                                "type": "text",
                                "text": text
                            }));
                        }
                    }
                }
            }
            flush_user_parts(&mut out, &mut user_parts);
            continue;
        }

        if role == "system" {
            let mut text = String::new();
            for block in blocks {
                if block.block_type == "text"
                    && let Some(part) = block.text
                {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(&part);
                }
            }

            if !text.is_empty() {
                out.push(json!({
                    "role": "system",
                    "content": text
                }));
            }
        }
    }

    out
}

fn convert_anthropic_tools_to_openai(tools: &[AnthropicToolInput]) -> Vec<Value> {
    tools
        .iter()
        .filter(|tool| !tool.name.trim().is_empty())
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description.clone().unwrap_or_default(),
                    "parameters": tool.input_schema.clone().unwrap_or_else(|| json!({ "type": "object", "properties": {} }))
                }
            })
        })
        .collect()
}

async fn compute_anthropic_input_usage(
    req: &AnthropicMessagesRequest,
    provider: &Arc<dyn PromptCacheProvider>,
) -> AnthropicUsage {
    let mut usage = AnthropicUsage::default();

    let mut blocks = Vec::<AnthropicContentBlockInput>::new();
    if let Some(system) = &req.system {
        blocks.extend(system_blocks(system));
    }
    for message in &req.messages {
        blocks.extend(content_blocks(&message.content));
    }

    for block in blocks {
        match block.block_type.as_str() {
            "text" => {
                let text = block.text.unwrap_or_default();
                if text.is_empty() {
                    continue;
                }
                let token_count = estimate_tokens(&text);
                if block.cache_control.is_some() {
                    let hash = hash_prompt_text(&text);
                    if let Some(entry) = provider.get(&hash).await {
                        usage.cache_read_input_tokens = usage
                            .cache_read_input_tokens
                            .saturating_add(entry.token_count);
                    } else {
                        usage.cache_creation_input_tokens = usage
                            .cache_creation_input_tokens
                            .saturating_add(token_count);
                        let _ = provider
                            .set(
                                &hash,
                                CachedEntry {
                                    token_count,
                                    compiled_representation: text.as_bytes().to_vec(),
                                    created_at: Utc::now(),
                                },
                            )
                            .await;
                    }
                } else {
                    usage.input_tokens = usage.input_tokens.saturating_add(token_count);
                }
            }
            "tool_result" => {
                let text = tool_result_text(&block);
                usage.input_tokens = usage.input_tokens.saturating_add(estimate_tokens(&text));
            }
            "tool_use" => {
                let rendered = block
                    .input
                    .as_ref()
                    .map_or_else(String::new, Value::to_string);
                usage.input_tokens = usage
                    .input_tokens
                    .saturating_add(estimate_tokens(&rendered));
            }
            _ => {
                if let Some(text) = block.text {
                    usage.input_tokens = usage.input_tokens.saturating_add(estimate_tokens(&text));
                }
            }
        }
    }

    usage
}

async fn resolve_anthropic_model(
    state: &AppState,
    requested_model: &str,
) -> Result<ResolvedModel, Response> {
    let default_provider_id = match state.provider_registry.default_id().await {
        Some(id) => id,
        None => {
            let providers = state.provider_registry.list().await;
            if let Some(first) = providers.first() {
                first.id.clone()
            } else {
                return Err(anthropic_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "No configured providers",
                ));
            }
        }
    };

    let mut provider_id = default_provider_id;
    let mut model_id = requested_model.trim().to_string();

    if let Some((provider_hint, model_hint)) = requested_model.split_once('/')
        && !provider_hint.trim().is_empty()
        && !model_hint.trim().is_empty()
        && state
            .provider_registry
            .get(provider_hint.trim())
            .await
            .is_some()
    {
        provider_id = provider_hint.trim().to_string();
        model_id = model_hint.trim().to_string();
    }

    if model_id.is_empty()
        && let Some(provider) = state.provider_registry.get(&provider_id).await
    {
        if let Some(default_model) = provider.default_model {
            model_id = default_model;
        } else if let Some(first_model) = provider.models.first() {
            model_id = first_model.id.clone();
        }
    }

    if model_id.is_empty() {
        model_id = "default".to_string();
    }

    Ok(ResolvedModel {
        provider_id,
        model_id,
    })
}

async fn api_messages(
    State(state): State<AppState>,
    Json(mut req): Json<AnthropicMessagesRequest>,
) -> Response {
    if req.messages.is_empty() {
        return anthropic_error_response(
            StatusCode::BAD_REQUEST,
            "messages must contain at least one message",
        );
    }

    let resolved_model = match resolve_anthropic_model(&state, &req.model).await {
        Ok(value) => value,
        Err(resp) => return resp,
    };

    let resolved_llm_config = match state
        .provider_registry
        .resolve_to_llm_config(&resolved_model.provider_id, &resolved_model.model_id)
        .await
    {
        Some(cfg) => cfg,
        None => {
            return anthropic_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to resolve provider settings",
            );
        }
    };

    let effective_caching = crate::uar::domain::prompt_caching::resolve_effective_caching(
        req.prompt_caching_enabled,
        None,
        None,
        false,
    );

    if effective_caching {
        inject_anthropic_cache_control(&mut req);
        tracing::debug!("Injected cache_control blocks into Anthropic request");
    }

    let openai_messages = convert_anthropic_messages_to_openai(&req);

    let llm_request = crate::llm::LlmRequest {
        messages: openai_messages,
        tools: convert_anthropic_tools_to_openai(&req.tools),
        cache_strategy: None,
        thinking_config: None,
        anthropic_system: None,
    };

    let client_config = crate::config::build_client_config(&resolved_llm_config);
    let model_str = format!("{}/{}", resolved_model.provider_id, resolved_model.model_id);
    let driver_model = resolved_llm_config.model.clone();
    let driver: std::sync::Arc<dyn LlmDriver> = if crate::llm::detect_provider(&model_str)
        == "anthropic"
        && crate::llm::anthropic_native_driver_enabled()
    {
        std::sync::Arc::new(crate::llm::anthropic_driver::AnthropicDriver::new(
            resolved_llm_config.api_key.clone().unwrap_or_default(),
            driver_model.clone(),
            resolved_llm_config.base_url.clone(),
            None,
            Some(crate::llm::anthropic_cache::CacheStrategy::default()),
            None,
        ))
    } else {
        match crate::llm::LiterLlmDriver::new(client_config, driver_model, None) {
            Ok(d) => std::sync::Arc::new(d),
            Err(err) => {
                tracing::error!(error = %err, "Failed to create LiterLlmDriver");
                return anthropic_error_response(
                    StatusCode::BAD_GATEWAY,
                    "Failed to initialize upstream model client",
                );
            }
        }
    };
    let driver_stream = match driver.stream(llm_request).await {
        Ok(stream) => stream,
        Err(err) => {
            tracing::error!(error = %err, "Anthropic adapter failed to start stream");
            return anthropic_error_response(
                StatusCode::BAD_GATEWAY,
                "Upstream model request failed",
            );
        }
    };

    let response_model = if req.model.trim().is_empty() {
        resolved_model.model_id
    } else {
        req.model.clone()
    };
    let input_usage = compute_anthropic_input_usage(&req, &state.prompt_cache_provider).await;

    if !req.stream {
        let mut final_text = String::new();
        let mut tool_blocks: Vec<Value> = Vec::new();
        let mut provider_completion_tokens: Option<u32> = None;
        let mut estimated_output_tokens: u32 = 0;

        futures::pin_mut!(driver_stream);
        while let Some(next) = driver_stream.next().await {
            match next {
                Ok(DriverEvent::MessageDelta { text }) => {
                    estimated_output_tokens =
                        estimated_output_tokens.saturating_add(estimate_tokens(&text));
                    final_text.push_str(&text);
                }
                Ok(DriverEvent::ToolCallComplete {
                    id,
                    name,
                    arguments_json,
                    ..
                }) => {
                    estimated_output_tokens =
                        estimated_output_tokens.saturating_add(estimate_tokens(&arguments_json));
                    let parsed_input = serde_json::from_str::<Value>(&arguments_json)
                        .unwrap_or_else(|_| json!({ "raw": arguments_json }));
                    tool_blocks.push(json!({
                        "type": "tool_use",
                        "id": ensure_toolu_id(&id),
                        "name": name,
                        "input": parsed_input
                    }));
                }
                Ok(DriverEvent::Usage {
                    completion_tokens, ..
                }) => {
                    provider_completion_tokens = Some(completion_tokens);
                }
                Ok(DriverEvent::Done) => break,
                Ok(DriverEvent::Error { message, .. }) => {
                    return anthropic_error_response(StatusCode::BAD_GATEWAY, &message);
                }
                Err(err) => {
                    tracing::error!(error = %err, "Anthropic adapter stream error");
                    return anthropic_error_response(
                        StatusCode::BAD_GATEWAY,
                        "Upstream model stream failed",
                    );
                }
                _ => {}
            }
        }

        let mut usage = input_usage.clone();
        usage.output_tokens = provider_completion_tokens.unwrap_or(estimated_output_tokens);

        let mut content = Vec::<Value>::new();
        if !final_text.is_empty() {
            content.push(json!({
                "type": "text",
                "text": final_text
            }));
        }
        content.extend(tool_blocks);

        return Json(json!({
            "id": format!("msg_{}", Uuid::new_v4().simple()),
            "type": "message",
            "role": "assistant",
            "model": response_model,
            "content": content,
            "stop_reason": if content.iter().any(|b| b.get("type").and_then(Value::as_str).is_some_and(|t| t == "tool_use")) {
                "tool_use"
            } else {
                "end_turn"
            },
            "stop_sequence": Value::Null,
            "usage": usage
        }))
        .into_response();
    }

    let stream = async_stream::stream! {
        let message_id = format!("msg_{}", Uuid::new_v4().simple());
        let mut active_block: Option<ActiveAnthropicBlock> = None;
        let mut tool_tracks: HashMap<usize, ToolTrack> = HashMap::new();
        let mut next_block_index: usize = 0;
        let mut saw_tool_use = false;
        let mut estimated_output_tokens: u32 = 0;
        let mut provider_completion_tokens: Option<u32> = None;
        let mut finalized = false;

        yield Ok::<Event, std::convert::Infallible>(anthropic_sse_event("message_start", json!({
            "type": "message_start",
            "message": {
                "id": message_id,
                "type": "message",
                "role": "assistant",
                "model": response_model,
                "content": [],
                "stop_reason": Value::Null,
                "stop_sequence": Value::Null,
                "usage": AnthropicUsage {
                    input_tokens: input_usage.input_tokens,
                    output_tokens: 0,
                    cache_creation_input_tokens: input_usage.cache_creation_input_tokens,
                    cache_read_input_tokens: input_usage.cache_read_input_tokens,
                }
            }
        })));

        futures::pin_mut!(driver_stream);
        while let Some(next) = driver_stream.next().await {
            match next {
                Ok(DriverEvent::MessageDelta { text }) => {
                    if text.is_empty() {
                        continue;
                    }

                    if let Some(ActiveAnthropicBlock::Tool { index, .. }) = active_block.take() {
                        yield Ok(anthropic_sse_event("content_block_stop", json!({
                            "type": "content_block_stop",
                            "index": index
                        })));
                    }

                    let text_index = match active_block {
                        Some(ActiveAnthropicBlock::Text { index }) => index,
                        _ => {
                            let index = next_block_index;
                            next_block_index = next_block_index.saturating_add(1);
                            active_block = Some(ActiveAnthropicBlock::Text { index });
                            yield Ok(anthropic_sse_event("content_block_start", json!({
                                "type": "content_block_start",
                                "index": index,
                                "content_block": {
                                    "type": "text",
                                    "text": ""
                                }
                            })));
                            index
                        }
                    };

                    estimated_output_tokens = estimated_output_tokens.saturating_add(estimate_tokens(&text));
                    yield Ok(anthropic_sse_event("content_block_delta", json!({
                        "type": "content_block_delta",
                        "index": text_index,
                        "delta": {
                            "type": "text_delta",
                            "text": text
                        }
                    })));
                }
                Ok(DriverEvent::ToolCallDelta {
                    call_index,
                    id,
                    name,
                    arguments_delta,
                }) => {
                    if let Some(ActiveAnthropicBlock::Text { index }) = active_block.take() {
                        yield Ok(anthropic_sse_event("content_block_stop", json!({
                            "type": "content_block_stop",
                            "index": index
                        })));
                    }

                    let track = tool_tracks.entry(call_index).or_insert_with(|| ToolTrack {
                        index: {
                            let index = next_block_index;
                            next_block_index = next_block_index.saturating_add(1);
                            index
                        },
                        id: ensure_toolu_id(
                            id.as_deref()
                                .unwrap_or(format!("toolu_{}", Uuid::new_v4().simple()).as_str()),
                        ),
                        name: name.clone().unwrap_or_else(|| "tool".to_string()),
                        input_json: String::new(),
                        sent_json: String::new(),
                        started: false,
                    });

                    if let Some(tool_id) = id {
                        track.id = ensure_toolu_id(&tool_id);
                    }
                    if let Some(tool_name) = name {
                        track.name = tool_name;
                    }

                    if !track.started {
                        track.started = true;
                        yield Ok(anthropic_sse_event("content_block_start", json!({
                            "type": "content_block_start",
                            "index": track.index,
                            "content_block": {
                                "type": "tool_use",
                                "id": track.id,
                                "name": track.name,
                                "input": {}
                            }
                        })));
                    }
                    active_block = Some(ActiveAnthropicBlock::Tool {
                        index: track.index,
                        call_index,
                    });

                    if let Some(delta) = arguments_delta
                        && !delta.is_empty()
                    {
                        estimated_output_tokens =
                            estimated_output_tokens.saturating_add(estimate_tokens(&delta));
                        track.sent_json.push_str(&delta);
                        track.input_json.push_str(&delta);
                        yield Ok(anthropic_sse_event("content_block_delta", json!({
                            "type": "content_block_delta",
                            "index": track.index,
                            "delta": {
                                "type": "input_json_delta",
                                "partial_json": delta
                            }
                        })));
                    }
                }
                Ok(DriverEvent::ToolCallComplete {
                    call_index,
                    id,
                    name,
                    arguments_json,
                }) => {
                    if let Some(ActiveAnthropicBlock::Text { index }) = active_block.take() {
                        yield Ok(anthropic_sse_event("content_block_stop", json!({
                            "type": "content_block_stop",
                            "index": index
                        })));
                    }

                    let track = tool_tracks.entry(call_index).or_insert_with(|| ToolTrack {
                        index: {
                            let index = next_block_index;
                            next_block_index = next_block_index.saturating_add(1);
                            index
                        },
                        id: ensure_toolu_id(&id),
                        name: name.clone(),
                        input_json: String::new(),
                        sent_json: String::new(),
                        started: false,
                    });

                    track.id = ensure_toolu_id(&id);
                    track.name = name.clone();

                    if !track.started {
                        track.started = true;
                        yield Ok(anthropic_sse_event("content_block_start", json!({
                            "type": "content_block_start",
                            "index": track.index,
                            "content_block": {
                                "type": "tool_use",
                                "id": track.id,
                                "name": track.name,
                                "input": {}
                            }
                        })));
                    }

                    let remaining_json = if track.sent_json.is_empty() {
                        arguments_json.clone()
                    } else if arguments_json.starts_with(&track.sent_json) {
                        arguments_json[track.sent_json.len()..].to_string()
                    } else {
                        String::new()
                    };

                    if !remaining_json.is_empty() {
                        estimated_output_tokens = estimated_output_tokens
                            .saturating_add(estimate_tokens(&remaining_json));
                        track.sent_json.push_str(&remaining_json);
                        yield Ok(anthropic_sse_event("content_block_delta", json!({
                            "type": "content_block_delta",
                            "index": track.index,
                            "delta": {
                                "type": "input_json_delta",
                                "partial_json": remaining_json
                            }
                        })));
                    }

                    track.input_json = arguments_json;

                    if let Some(ActiveAnthropicBlock::Tool { index, call_index: active_call }) = active_block {
                        if active_call == call_index {
                            yield Ok(anthropic_sse_event("content_block_stop", json!({
                                "type": "content_block_stop",
                                "index": index
                            })));
                            active_block = None;
                        }
                    }
                    saw_tool_use = true;
                }
                Ok(DriverEvent::Usage {
                    completion_tokens, ..
                }) => {
                    provider_completion_tokens = Some(completion_tokens);
                }
                Ok(DriverEvent::Done) => {
                    if let Some(block) = active_block.take() {
                        let index = match block {
                            ActiveAnthropicBlock::Text { index } => index,
                            ActiveAnthropicBlock::Tool { index, .. } => index,
                        };
                        yield Ok(anthropic_sse_event("content_block_stop", json!({
                            "type": "content_block_stop",
                            "index": index
                        })));
                    }

                    let mut usage = input_usage.clone();
                    usage.output_tokens = provider_completion_tokens.unwrap_or(estimated_output_tokens);
                    yield Ok(anthropic_sse_event("message_delta", json!({
                        "type": "message_delta",
                        "delta": {
                            "stop_reason": if saw_tool_use { "tool_use" } else { "end_turn" },
                            "stop_sequence": Value::Null
                        },
                        "usage": usage
                    })));
                    yield Ok(anthropic_sse_event("message_stop", json!({
                        "type": "message_stop"
                    })));
                    finalized = true;
                    break;
                }
                Ok(DriverEvent::Error { message, .. }) => {
                    tracing::error!("Anthropic adapter upstream error: {}", message);
                    if let Some(block) = active_block.take() {
                        let index = match block {
                            ActiveAnthropicBlock::Text { index } => index,
                            ActiveAnthropicBlock::Tool { index, .. } => index,
                        };
                        yield Ok(anthropic_sse_event("content_block_stop", json!({
                            "type": "content_block_stop",
                            "index": index
                        })));
                    }
                    let mut usage = input_usage.clone();
                    usage.output_tokens = provider_completion_tokens.unwrap_or(estimated_output_tokens);
                    yield Ok(anthropic_sse_event("message_delta", json!({
                        "type": "message_delta",
                        "delta": {
                            "stop_reason": "error",
                            "stop_sequence": Value::Null
                        },
                        "usage": usage
                    })));
                    yield Ok(anthropic_sse_event("message_stop", json!({
                        "type": "message_stop"
                    })));
                    finalized = true;
                    break;
                }
                Err(err) => {
                    tracing::error!(error = %err, "Anthropic adapter upstream stream error");
                    if let Some(block) = active_block.take() {
                        let index = match block {
                            ActiveAnthropicBlock::Text { index } => index,
                            ActiveAnthropicBlock::Tool { index, .. } => index,
                        };
                        yield Ok(anthropic_sse_event("content_block_stop", json!({
                            "type": "content_block_stop",
                            "index": index
                        })));
                    }
                    let mut usage = input_usage.clone();
                    usage.output_tokens = provider_completion_tokens.unwrap_or(estimated_output_tokens);
                    yield Ok(anthropic_sse_event("message_delta", json!({
                        "type": "message_delta",
                        "delta": {
                            "stop_reason": "error",
                            "stop_sequence": Value::Null
                        },
                        "usage": usage
                    })));
                    yield Ok(anthropic_sse_event("message_stop", json!({
                        "type": "message_stop"
                    })));
                    finalized = true;
                    break;
                }
                _ => {}
            }
        }

        if !finalized {
            if let Some(block) = active_block.take() {
                let index = match block {
                    ActiveAnthropicBlock::Text { index } => index,
                    ActiveAnthropicBlock::Tool { index, .. } => index,
                };
                yield Ok(anthropic_sse_event("content_block_stop", json!({
                    "type": "content_block_stop",
                    "index": index
                })));
            }
            let mut usage = input_usage.clone();
            usage.output_tokens = provider_completion_tokens.unwrap_or(estimated_output_tokens);
            yield Ok(anthropic_sse_event("message_delta", json!({
                "type": "message_delta",
                "delta": {
                    "stop_reason": if saw_tool_use { "tool_use" } else { "end_turn" },
                    "stop_sequence": Value::Null
                },
                "usage": usage
            })));
            yield Ok(anthropic_sse_event("message_stop", json!({
                "type": "message_stop"
            })));
        }
    };

    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}

/// OpenAI-style message payload.
#[derive(Debug, Deserialize)]
struct OpenAiMessageInput {
    role: String,
    #[serde(default)]
    content: serde_json::Value,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum StreamMode {
    #[default]
    Openai,
    Agui,
    Dual,
}

impl StreamMode {
    fn emits_openai_chunks(&self) -> bool {
        matches!(self, Self::Openai | Self::Dual)
    }

    fn emits_agui_chunks(&self) -> bool {
        matches!(self, Self::Agui | Self::Dual)
    }
}

/// OpenAI-compatible completion request with UAR extensions.
#[derive(Debug, Deserialize)]
pub(crate) struct ChatCompletionRequest {
    /// Model name (`gpt-5.2`) or provider-scoped (`openai/gpt-5.2`).
    #[serde(default)]
    model: Option<String>,
    /// OpenAI standard messages array.
    #[serde(default)]
    messages: Vec<OpenAiMessageInput>,
    /// Convenience single-message field.
    #[serde(default)]
    message: Option<String>,
    /// Optional temperature (accepted for compatibility).
    #[serde(default)]
    temperature: Option<f32>,
    /// Optional tools (accepted for compatibility).
    #[serde(default)]
    tools: Option<Vec<serde_json::Value>>,
    /// Stream response chunks if true.
    #[serde(default)]
    stream: bool,
    /// Streaming payload mode: `openai` (default), `agui`, or `dual`.
    #[serde(default, alias = "stream_format")]
    stream_mode: StreamMode,
    /// Optional UAR extension; header is preferred.
    #[serde(default)]
    session_id: Option<String>,
    /// Files attached to this message (uploaded via POST /api/upload).
    #[serde(default)]
    attachments: Vec<crate::uar::api::upload::AttachmentInput>,
    /// Set to false to disable memory injection and auto-capture for this turn.
    /// Defaults to true (respects global/agent memory config).
    #[serde(default = "default_true")]
    memory_enabled: bool,
    /// Per-request prompt-caching override.
    ///
    /// `true` — enable caching for this turn (highest priority).
    /// `false` — disable caching for this turn.
    /// `null` / absent — inherit from user → agent → global settings.
    #[serde(default)]
    prompt_caching_enabled: Option<bool>,
    /// Agent id to use for this run.
    ///
    /// When present, overrides the session agent-config side-channel so the
    /// selected agent is authoritative on the request body (not a prior POST).
    /// Falls through to session → default-agent in the absence of this field.
    #[serde(default)]
    agent_id: Option<String>,
}

#[inline]
fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
struct OpenAiErrorBody {
    error: OpenAiErrorPayload,
}

#[derive(Debug, Serialize)]
struct OpenAiErrorPayload {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    param: Option<String>,
    code: Option<String>,
}

#[derive(Debug, Serialize)]
struct OpenAiChatCompletionResponse {
    id: String,
    object: &'static str,
    created: i64,
    model: String,
    choices: Vec<OpenAiChatChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<serde_json::Value>,
    session_id: String,
}

#[derive(Debug, Serialize)]
struct OpenAiChatChoice {
    index: usize,
    message: OpenAiAssistantMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
struct OpenAiAssistantMessage {
    role: &'static str,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiChunk {
    id: String,
    object: &'static str,
    created: i64,
    model: String,
    choices: Vec<OpenAiChunkChoice>,
}

#[derive(Debug, Serialize)]
struct OpenAiChunkChoice {
    index: usize,
    delta: OpenAiDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<&'static str>,
}

#[derive(Debug, Serialize, Default)]
struct OpenAiDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiDeltaToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_results: Option<Vec<OpenAiDeltaToolResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skills: Option<Vec<OpenAiDeltaSkill>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_updates: Option<Vec<OpenAiDeltaContextUpdate>>,
}

#[derive(Debug, Serialize)]
struct OpenAiDeltaToolCall {
    index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    call_type: Option<&'static str>,
    function: OpenAiDeltaToolCallFunction,
}

#[derive(Debug, Serialize)]
struct OpenAiDeltaToolCallFunction {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<String>,
}

#[derive(Debug, Serialize)]
struct OpenAiDeltaToolResult {
    index: usize,
    id: String,
    name: String,
    content: String,
    success: bool,
}

#[derive(Debug, Serialize)]
struct OpenAiDeltaSkill {
    id: String,
    title: String,
    selection_method: String,
}

#[derive(Debug, Serialize)]
struct OpenAiDeltaContextUpdate {
    strategy: crate::uar::domain::context::ContextStrategy,
    messages_removed: usize,
    tokens_saved: usize,
    was_applied: bool,
    summary_generated: bool,
}

#[derive(Debug)]
struct ResolvedModel {
    provider_id: String,
    model_id: String,
}

fn openai_error_response(status: StatusCode, message: &str, code: Option<&str>) -> Response {
    (
        status,
        Json(OpenAiErrorBody {
            error: OpenAiErrorPayload {
                message: message.to_string(),
                error_type: "invalid_request_error".to_string(),
                param: Some("model".to_string()),
                code: code.map(ToString::to_string),
            },
        }),
    )
        .into_response()
}

fn extract_text_content(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(parts) => {
            let mut text = String::new();
            for part in parts {
                if let Some(part_type) = part.get("type").and_then(serde_json::Value::as_str)
                    && part_type == "text"
                    && let Some(t) = part.get("text").and_then(serde_json::Value::as_str)
                {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(t);
                }
            }
            if text.is_empty() { None } else { Some(text) }
        }
        _ => None,
    }
}

fn extract_input_message(req: &ChatCompletionRequest) -> Option<String> {
    if let Some(message) = &req.message
        && !message.trim().is_empty()
    {
        return Some(message.clone());
    }

    req.messages
        .iter()
        .rev()
        .find(|m| m.role.eq_ignore_ascii_case("user"))
        .and_then(|m| extract_text_content(&m.content))
        .filter(|s| !s.trim().is_empty())
}

/// Build a JSON string representing an OpenAI-style multipart content array.
///
/// Layout:
///   1. For each non-image attachment: `{ type:"text", text:"[filename]\n<content>" }`
///   2. User message text: `{ type:"text", text:"..." }`
///   3. For each image attachment: `{ type:"image_url", image_url:{ url:"/api/attachments/{id}", detail:"auto" } }`
///
/// Returns `None` if there are no attachments (caller uses plain text instead).
fn build_multipart_content(
    user_text: &str,
    attachments: &[crate::uar::api::upload::AttachmentInput],
) -> Option<String> {
    if attachments.is_empty() {
        return None;
    }
    let mut parts: Vec<serde_json::Value> = Vec::new();

    // Document text blocks first (context before the question).
    for att in attachments {
        if !att.content_type.starts_with("image/") {
            if let Some(text) = &att.text_content {
                parts.push(serde_json::json!({
                    "type": "text",
                    "text": format!("[{}]\n{}", att.filename, text)
                }));
            }
        }
    }

    // User message text.
    if !user_text.is_empty() {
        parts.push(serde_json::json!({ "type": "text", "text": user_text }));
    }

    // Image parts last.
    for att in attachments {
        if att.content_type.starts_with("image/") {
            parts.push(serde_json::json!({
                "type": "image_url",
                "image_url": { "url": att.url, "detail": "auto" }
            }));
        }
    }

    if parts.is_empty() {
        return None;
    }

    serde_json::to_string(&parts).ok()
}

fn extract_cookie_session_id(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie_header.split(';').map(str::trim).find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        if name == "uar_session_id" {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn invalid_session_id_response() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": {
                "message": "session_id must be a valid UUID",
                "type": "invalid_request_error",
                "param": "session_id",
                "code": "invalid_session_id"
            }
        })),
    )
        .into_response()
}

fn validate_uuid_session_id(value: &str) -> Result<String, Response> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    Uuid::parse_str(trimmed)
        .map(|id| id.to_string())
        .map_err(|_| invalid_session_id_response())
}

fn resolve_session_id(
    req: &ChatCompletionRequest,
    headers: &HeaderMap,
) -> Result<Option<String>, Response> {
    if let Some(value) = headers
        .get("x-uar-session-id")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.trim().is_empty())
    {
        return validate_uuid_session_id(value).map(Some);
    }

    if let Some(value) = &req.session_id
        && !value.trim().is_empty()
    {
        return validate_uuid_session_id(value).map(Some);
    }

    if let Some(value) = extract_cookie_session_id(headers)
        && !value.trim().is_empty()
    {
        return validate_uuid_session_id(&value).map(Some);
    }

    Ok(None)
}

fn model_known(provider: &crate::llm::ProviderConfig, model_id: &str) -> bool {
    provider.default_model.as_deref() == Some(model_id)
        || provider.models.iter().any(|m| m.id == model_id)
}

async fn resolve_requested_model(
    state: &AppState,
    requested_model: Option<&str>,
) -> Result<ResolvedModel, Response> {
    let default_provider_id = match state.provider_registry.default_id().await {
        Some(id) => id,
        None => {
            let providers = state.provider_registry.list().await;
            if let Some(first) = providers.first() {
                first.id.clone()
            } else {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error":{"message":"No configured providers"}})),
                )
                    .into_response());
            }
        }
    };

    let default_provider = match state.provider_registry.get(&default_provider_id).await {
        Some(p) => p,
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":{"message":"Default provider unavailable"}})),
            )
                .into_response());
        }
    };

    let requested_model = requested_model.unwrap_or("").trim();

    if requested_model.is_empty() {
        // 1. Use the provider's configured default model
        if let Some(model_id) = default_provider.default_model.clone() {
            return Ok(ResolvedModel {
                provider_id: default_provider_id,
                model_id,
            });
        }
        // 2. Use the global LLM config model (from .env / config.yaml)
        let (_, global_model) =
            crate::llm::registry::split_model_string_pub(&state.config.llm.model);
        if !global_model.is_empty() {
            tracing::debug!(
                fallback_model = %global_model,
                "No provider default_model set, falling back to global LLM config model"
            );
            return Ok(ResolvedModel {
                provider_id: default_provider_id,
                model_id: global_model,
            });
        }
        // 3. Pick a chat-capable model from the provider's model list (skip image/embedding models)
        let chat_model = default_provider.models.iter().find(|m| {
            !m.id.contains("image")
                && !m.id.contains("embedding")
                && !m.id.contains("tts")
                && !m.id.contains("whisper")
                && !m.id.contains("dall-e")
                && !m.id.contains("moderation")
        });
        if let Some(model) = chat_model {
            return Ok(ResolvedModel {
                provider_id: default_provider_id,
                model_id: model.id.clone(),
            });
        }
        // 4. Last resort: first model
        if let Some(model) = default_provider.models.first() {
            return Ok(ResolvedModel {
                provider_id: default_provider_id,
                model_id: model.id.clone(),
            });
        }
        return Err(openai_error_response(
            StatusCode::NOT_FOUND,
            "No chat-capable model found for the default provider. Configure a default model in Settings > LLM Configuration.",
            Some("model_not_found"),
        ));
    }

    if let Some((provider_id, model_id)) = requested_model.split_once('/') {
        let provider_id = provider_id.trim();
        let model_id = model_id.trim();
        if provider_id.is_empty() || model_id.is_empty() {
            return Err(openai_error_response(
                StatusCode::NOT_FOUND,
                "Unknown model",
                Some("model_not_found"),
            ));
        }

        let Some(provider) = state.provider_registry.get(provider_id).await else {
            return Err(openai_error_response(
                StatusCode::NOT_FOUND,
                "Unknown model",
                Some("model_not_found"),
            ));
        };

        if !model_known(&provider, model_id) {
            return Err(openai_error_response(
                StatusCode::NOT_FOUND,
                "Unknown model",
                Some("model_not_found"),
            ));
        }

        return Ok(ResolvedModel {
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
        });
    }

    if !model_known(&default_provider, requested_model) {
        return Err(openai_error_response(
            StatusCode::NOT_FOUND,
            "Unknown model",
            Some("model_not_found"),
        ));
    }

    Ok(ResolvedModel {
        provider_id: default_provider_id,
        model_id: requested_model.to_string(),
    })
}

/// OpenAI-compatible completion endpoint with optional UAR session continuity.
pub(crate) async fn api_chat_completion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChatCompletionRequest>,
) -> Response {
    if let Some(temp) = req.temperature {
        tracing::debug!(temperature = temp, "temperature received");
    }
    if let Some(tools) = &req.tools {
        tracing::debug!(tool_count = tools.len(), "tools payload received");
    }

    let Some(input_message) = extract_input_message(&req) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "message": "Request must include `message` or a user message in `messages`",
                    "type": "invalid_request_error",
                    "param": "messages",
                    "code": "invalid_request"
                }
            })),
        )
            .into_response();
    };

    let resolved_model = match resolve_requested_model(&state, req.model.as_deref()).await {
        Ok(m) => m,
        Err(resp) => return resp,
    };

    let session_id = match resolve_session_id(&req, &headers) {
        Ok(Some(value)) => value,
        Ok(None) => Uuid::new_v4().to_string(),
        Err(resp) => return resp,
    };

    // Resolve agent: request-body agent_id → session agent-config → default.
    //
    // Priority (highest first):
    //   1. `req.agent_id` — explicit selection from the chat request body.
    //   2. `agent_sessions[session_id].agent_id` — session side-channel POST.
    //   3. `default-agent` — fallback.
    let resolved_agent_id: Option<String> = req.agent_id.clone().or_else(|| {
        state
            .agent_sessions
            .try_read()
            .ok()
            .and_then(|sessions| sessions.get(&session_id).map(|cfg| cfg.agent_id.clone()))
    });

    let mut agent = if let Some(ref aid) = resolved_agent_id {
        uar::api::discovery::resolve_agent_for_run(&state, aid).await
    } else {
        uar::defaults::default_agent()
    };
    agent.policy.provider.default.provider = resolved_model.provider_id.clone();
    agent.policy.provider.default.model = resolved_model.model_id.clone();
    let agent_id_for_policy = agent.id.clone();
    let (effective_resilience_policy, policy_source) =
        resolve_effective_resilience_policy(&state, &agent_id_for_policy).await;

    tracing::info!(
        name: "resilience.policy.effective",
        agent_id = %agent_id_for_policy,
        source = ?policy_source,
        request_timeout_ms = effective_resilience_policy.request_timeout_ms,
        retries_enabled = effective_resilience_policy.retries_enabled,
        retry_max_attempts = effective_resilience_policy.retry_max_attempts,
        retry_base_delay_ms = effective_resilience_policy.retry_base_delay_ms,
        retry_max_delay_ms = effective_resilience_policy.retry_max_delay_ms,
        "Resolved effective resilience policy"
    );

    // If attachments were uploaded, assemble an OpenAI-style multipart content string
    // (document context blocks + user text + image_url parts).  Otherwise pass plain text.
    let effective_input =
        build_multipart_content(&input_message, &req.attachments).unwrap_or(input_message);

    // Extract UserContext from request extensions (set by auth middleware, may be anonymous).
    let user_ctx = {
        use crate::uar::security::claims::UserClaims;
        let uid = headers
            .get("x-uar-user-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("anonymous")
            .to_string();
        UserContext {
            user_id: uid.clone(),
            claims: UserClaims {
                sub: uid,
                name: None,
                roles: None,
                exp: 0,
            },
        }
    };

    // --- Prompt caching: resolve effective setting for this request ---
    let effective_prompt_caching = {
        use crate::uar::domain::prompt_caching::resolve_effective_caching;
        // Session-level override from request body (highest priority)
        let session_override = req.prompt_caching_enabled;
        // User-level preference (requires non-anonymous user)
        let user_pref = if user_ctx.user_id != "anonymous" {
            state
                .user_settings_store
                .caching_enabled_for(&user_ctx.user_id)
                .await
        } else {
            None
        };
        // Agent-level default (not yet stored per-agent — placeholder for future)
        let agent_pref: Option<bool> = None;
        // Global default: off unless explicitly configured
        let global = false;
        resolve_effective_caching(session_override, user_pref, agent_pref, global)
    };
    tracing::debug!(
        prompt_caching_enabled = effective_prompt_caching,
        user_id = %user_ctx.user_id,
        "Resolved effective prompt-caching setting"
    );

    // --- Memory: context injection (pre-LLM-call) ---
    // Build context block and collect the raw hits so we can stream them to the client.
    let (memory_context_block, memory_recall_items) = if req.memory_enabled {
        if let Some(svc) = &state.memory_service {
            let result = context_builder::build_context_with_hits(
                svc,
                &effective_input,
                &user_ctx,
                Some(&agent.id),
                Some(&session_id),
                &resolved_model.model_id,
            )
            .await;
            if !result.block.is_empty() {
                tracing::debug!(
                    chars = result.block.len(),
                    hits = result.hits.len(),
                    "Memory context block assembled"
                );
            }
            // Convert surreal_memory::Memory → MemoryItem for the stream event.
            let items: Vec<MemoryItem> = result
                .hits
                .iter()
                .map(|mem| {
                    let scope_label = format!("{:?}", mem.scope).to_lowercase();
                    let type_label = format!("{:?}", mem.memory_type).to_lowercase();
                    MemoryItem {
                        key: mem
                            .id
                            .as_ref()
                            .and_then(|r| {
                                serde_json::to_value(r)
                                    .ok()
                                    .and_then(|v| v.as_str().map(str::to_string))
                            })
                            .unwrap_or_else(|| format!("{scope_label}/{type_label}")),
                        value: mem.content.clone(),
                        source: "memory_context".to_string(),
                        scope: Some(scope_label),
                        memory_type: Some(type_label),
                        importance: Some(mem.importance),
                    }
                })
                .collect();
            (result.block, items)
        } else {
            (String::new(), vec![])
        }
    } else {
        (String::new(), vec![])
    };

    // Prepend memory context to the effective input sent to the LLM.
    let effective_input_with_memory = if memory_context_block.is_empty() {
        effective_input.clone()
    } else {
        format!(
            "[MEMORY CONTEXT]\n{}\n[/MEMORY CONTEXT]\n\n{}",
            memory_context_block, effective_input
        )
    };

    let run_id = state
        .run_manager
        .start_run(
            agent,
            effective_input_with_memory,
            Some(session_id.clone()),
            None,
            memory_recall_items,
        )
        .await;

    let Some(mut rx) = state.run_manager.subscribe(&run_id).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "message": "Failed to subscribe to run stream",
                    "type": "server_error"
                }
            })),
        )
            .into_response();
    };

    let completion_id = format!("chatcmpl-{}", Uuid::new_v4().simple());
    let created = Utc::now().timestamp();
    let model_name = format!("{}/{}", resolved_model.provider_id, resolved_model.model_id);

    // Capture per-request values needed inside the stream closure.
    let req_memory_enabled = req.memory_enabled;
    let stream_user_ctx = user_ctx.clone();
    let stream_memory_ctx_count = if memory_context_block.is_empty() {
        0usize
    } else {
        1usize
    };

    if req.stream {
        let emit_openai_chunks = req.stream_mode.emits_openai_chunks();
        let emit_agui_chunks = req.stream_mode.emits_agui_chunks();
        let stream_session_id = session_id.clone();
        // Keep an extra copy for the response headers (stream_session_id is moved into the closure).
        let response_session_id = session_id.clone();
        let stream_model_name = model_name.clone();
        let stream_completion_id = completion_id.clone();
        let replay = state
            .run_manager
            .history_since(&run_id, None)
            .await
            .unwrap_or_default();
        let replay_max_id = replay.last().map_or(0, |event| event.id);

        // Clone into stream-owned variables.
        let stream_memory_service = state.memory_service.clone();
        let stream_agent_id = agent_id_for_policy.clone();
        let _stream_memory_ctx_count = stream_memory_ctx_count;

        let stream = async_stream::stream! {
            // Emit agui.memory.context event so frontend can show indicator.
            if emit_agui_chunks && _stream_memory_ctx_count > 0 {
                let mem_event = Event::default()
                    .event("agui.memory.context")
                    .data(serde_json::to_string(&serde_json::json!({
                        "kind": "memory",
                        "phase": "injected",
                        "count": _stream_memory_ctx_count
                    })).unwrap_or_default());
                yield Ok::<Event, std::convert::Infallible>(mem_event);
            }

            if emit_openai_chunks {
                let first = OpenAiChunk {
                    id: stream_completion_id.clone(),
                    object: "chat.completion.chunk",
                    created,
                    model: stream_model_name.clone(),
                    choices: vec![OpenAiChunkChoice {
                        index: 0,
                        delta: OpenAiDelta {
                            role: Some("assistant"),
                            content: None,
                            tool_calls: None,
                            tool_results: None,
                            skills: None,
                            context_updates: None,
                        },
                        finish_reason: None,
                    }],
                };
                let first_json = serde_json::to_string(&first).unwrap_or_else(|_| "{}".to_string());
                yield Ok::<Event, std::convert::Infallible>(Event::default().data(first_json));
            }

            let mut tool_name_by_id: HashMap<String, String> = HashMap::new();
            let mut assistant_text_for_capture = String::new();
            let user_text_for_capture = effective_input.clone();

            macro_rules! process_stream_event {
                ($stream_event:expr) => {{
                    let stream_event = $stream_event;
                    let event_id = stream_event.id.to_string();
                    let normalized_event = stream_event.event;

                    if emit_agui_chunks
                        && let Some((event_name, payload)) = to_agui_event(&normalized_event)
                    {
                        let payload_json =
                            serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
                        let agui_event = Event::default()
                            .event(event_name)
                            .id(event_id.clone())
                            .data(payload_json);
                        yield Ok(agui_event);
                    }

                    // Emit runtime entity events alongside agui events.
                    // These feed the Runtime Console (Cockpit/Runs/Approvals) via
                    // the frontend's ingestRuntimeEvent() path. Always emitted
                    // when the event has a runtime entity mapping — independent
                    // of stream_mode — so the Console is populated even in
                    // openai-only mode clients.
                    if let Some((rt_event_name, rt_payload)) = to_runtime_entity_event(&normalized_event) {
                        let rt_json =
                            serde_json::to_string(&rt_payload).unwrap_or_else(|_| "{}".to_string());
                        let rt_event = Event::default()
                            .event(rt_event_name)
                            .id(event_id.clone())
                            .data(rt_json);
                        yield Ok(rt_event);
                    }

                    let mut should_stop = false;
                    match normalized_event {
                        uar::domain::events::NormalizedEvent::ChatDelta { ref text_delta, .. } => {
                            assistant_text_for_capture.push_str(text_delta);
                            let text_delta = text_delta.clone();
                            if emit_openai_chunks {
                                let chunk = OpenAiChunk {
                                    id: stream_completion_id.clone(),
                                    object: "chat.completion.chunk",
                                    created,
                                    model: stream_model_name.clone(),
                                    choices: vec![OpenAiChunkChoice {
                                        index: 0,
                                        delta: OpenAiDelta {
                                            role: None,
                                            content: Some(text_delta),
                                            tool_calls: None,
                                            tool_results: None,
                                            skills: None,
                                            context_updates: None,
                                        },
                                        finish_reason: None,
                                    }],
                                };
                                let json =
                                    serde_json::to_string(&chunk).unwrap_or_else(|_| "{}".to_string());
                                yield Ok(Event::default().data(json));
                            }
                        }
                        uar::domain::events::NormalizedEvent::SkillActivated {
                            skill_id,
                            title,
                            selection_method,
                            ..
                        } => {
                            if emit_openai_chunks {
                                let chunk = OpenAiChunk {
                                    id: stream_completion_id.clone(),
                                    object: "chat.completion.chunk",
                                    created,
                                    model: stream_model_name.clone(),
                                    choices: vec![OpenAiChunkChoice {
                                        index: 0,
                                        delta: OpenAiDelta {
                                            role: None,
                                            content: None,
                                            tool_calls: None,
                                            tool_results: None,
                                            skills: Some(vec![OpenAiDeltaSkill {
                                                id: skill_id,
                                                title,
                                                selection_method,
                                            }]),
                                            context_updates: None,
                                        },
                                        finish_reason: None,
                                    }],
                                };
                                let json =
                                    serde_json::to_string(&chunk).unwrap_or_else(|_| "{}".to_string());
                                yield Ok(Event::default().data(json));
                            }
                        }
                        uar::domain::events::NormalizedEvent::ContextAction(action) => {
                            if emit_openai_chunks {
                                let chunk = OpenAiChunk {
                                    id: stream_completion_id.clone(),
                                    object: "chat.completion.chunk",
                                    created,
                                    model: stream_model_name.clone(),
                                    choices: vec![OpenAiChunkChoice {
                                        index: 0,
                                        delta: OpenAiDelta {
                                            role: None,
                                            content: None,
                                            tool_calls: None,
                                            tool_results: None,
                                            skills: None,
                                            context_updates: Some(vec![OpenAiDeltaContextUpdate {
                                                strategy: action.strategy,
                                                messages_removed: action.messages_removed,
                                                tokens_saved: action.tokens_saved,
                                                was_applied: action.was_applied,
                                                summary_generated: action.summary_generated,
                                            }]),
                                        },
                                        finish_reason: None,
                                    }],
                                };
                                let json =
                                    serde_json::to_string(&chunk).unwrap_or_else(|_| "{}".to_string());
                                yield Ok(Event::default().data(json));
                            }
                        }
                        uar::domain::events::NormalizedEvent::ToolDelta {
                            call_index,
                            tool_call_id,
                            delta,
                            ..
                        } => {
                            if emit_openai_chunks {
                                let arguments_delta = match delta {
                                    serde_json::Value::String(s) => s,
                                    other => other.to_string(),
                                };
                                let tool_name = tool_name_by_id.get(&tool_call_id).cloned();
                                let chunk = OpenAiChunk {
                                    id: stream_completion_id.clone(),
                                    object: "chat.completion.chunk",
                                    created,
                                    model: stream_model_name.clone(),
                                    choices: vec![OpenAiChunkChoice {
                                        index: 0,
                                        delta: OpenAiDelta {
                                            role: None,
                                            content: None,
                                            tool_calls: Some(vec![OpenAiDeltaToolCall {
                                                index: call_index,
                                                id: Some(tool_call_id),
                                                call_type: Some("function"),
                                                function: OpenAiDeltaToolCallFunction {
                                                    name: tool_name,
                                                    arguments: Some(arguments_delta),
                                                },
                                            }]),
                                            tool_results: None,
                                            skills: None,
                                            context_updates: None,
                                        },
                                        finish_reason: None,
                                    }],
                                };
                                let json =
                                    serde_json::to_string(&chunk).unwrap_or_else(|_| "{}".to_string());
                                yield Ok(Event::default().data(json));
                            }
                        }
                        uar::domain::events::NormalizedEvent::ToolStart {
                            call_index,
                            tool_call_id,
                            tool,
                            input,
                            ..
                        } => {
                            tool_name_by_id.insert(tool_call_id.clone(), tool.clone());
                            if emit_openai_chunks {
                                let chunk = OpenAiChunk {
                                    id: stream_completion_id.clone(),
                                    object: "chat.completion.chunk",
                                    created,
                                    model: stream_model_name.clone(),
                                    choices: vec![OpenAiChunkChoice {
                                        index: 0,
                                        delta: OpenAiDelta {
                                            role: None,
                                            content: None,
                                            tool_calls: Some(vec![OpenAiDeltaToolCall {
                                                index: call_index,
                                                id: Some(tool_call_id),
                                                call_type: Some("function"),
                                                function: OpenAiDeltaToolCallFunction {
                                                    name: Some(tool),
                                                    arguments: Some(input.to_string()),
                                                },
                                            }]),
                                            tool_results: None,
                                            skills: None,
                                            context_updates: None,
                                        },
                                        finish_reason: None,
                                    }],
                                };
                                let json =
                                    serde_json::to_string(&chunk).unwrap_or_else(|_| "{}".to_string());
                                yield Ok(Event::default().data(json));
                            }
                        }
                        uar::domain::events::NormalizedEvent::ToolEnd {
                            call_index,
                            tool_call_id,
                            tool,
                            output,
                            ok,
                            ..
                        } => {
                            if emit_openai_chunks {
                                let chunk = OpenAiChunk {
                                    id: stream_completion_id.clone(),
                                    object: "chat.completion.chunk",
                                    created,
                                    model: stream_model_name.clone(),
                                    choices: vec![OpenAiChunkChoice {
                                        index: 0,
                                        delta: OpenAiDelta {
                                            role: None,
                                            content: None,
                                            tool_calls: None,
                                            tool_results: Some(vec![OpenAiDeltaToolResult {
                                                index: call_index,
                                                id: tool_call_id,
                                                name: tool,
                                                content: output.to_string(),
                                                success: ok,
                                            }]),
                                            skills: None,
                                            context_updates: None,
                                        },
                                        finish_reason: None,
                                    }],
                                };
                                let json =
                                    serde_json::to_string(&chunk).unwrap_or_else(|_| "{}".to_string());
                                yield Ok(Event::default().data(json));
                            }
                        }
                        uar::domain::events::NormalizedEvent::RunDone { .. }
                        | uar::domain::events::NormalizedEvent::RunDoneWithUsage { .. } => {
                            // --- Memory: auto-capture (fire-and-forget, post-stream) ---
                            if req_memory_enabled {
                                if let Some(ref svc) = stream_memory_service {
                                    if !assistant_text_for_capture.is_empty() {
                                        let msgs = vec![
                                            ConversationMessage { role: "user".into(), content: user_text_for_capture.clone() },
                                            ConversationMessage { role: "assistant".into(), content: assistant_text_for_capture.clone() },
                                        ];
                                        let svc2 = Arc::clone(svc);
                                        let ctx2 = stream_user_ctx.clone();
                                        let aid = stream_agent_id.clone();
                                        let sid = stream_session_id.clone();
                                        let mgr = Arc::clone(&state.run_manager);
                                        let rid = run_id.clone();
                                        tokio::spawn(async move {
                                            let captured = auto_capture::capture_from_stream_end(
                                                &svc2, &msgs, &ctx2, &aid, &sid,
                                            ).await;
                                            for mem in captured {
                                                let memory_id = mem.id
                                                    .as_ref()
                                                    .map(|id| {
                                                        serde_json::to_value(id)
                                                            .ok()
                                                            .and_then(|v| v.as_str().map(str::to_string))
                                                            .unwrap_or_default()
                                                    })
                                                    .unwrap_or_default();
                                                mgr.emit_to_run(
                                                    &rid,
                                                    uar::domain::events::NormalizedEvent::MemoryMutation {
                                                        run_id: rid.clone(),
                                                        operation: "created".to_string(),
                                                        memory_id,
                                                        content: mem.content.clone(),
                                                        scope: format!("{:?}", mem.scope).to_lowercase(),
                                                        memory_type: format!("{:?}", mem.memory_type).to_lowercase(),
                                                    },
                                                ).await;
                                            }
                                        });
                                    }
                                }
                            }

                            if emit_openai_chunks {
                                let final_chunk = OpenAiChunk {
                                    id: stream_completion_id.clone(),
                                    object: "chat.completion.chunk",
                                    created,
                                    model: stream_model_name.clone(),
                                    choices: vec![OpenAiChunkChoice {
                                        index: 0,
                                        delta: OpenAiDelta::default(),
                                        finish_reason: Some("stop"),
                                    }],
                                };
                                let json = serde_json::to_string(&final_chunk)
                                    .unwrap_or_else(|_| "{}".to_string());
                                yield Ok(Event::default().data(json));
                                yield Ok(Event::default().data("[DONE]"));
                            }
                            should_stop = true;
                        }
                        uar::domain::events::NormalizedEvent::Error { message, .. } => {
                            if emit_openai_chunks {
                                let payload = json!({
                                    "error": {
                                        "message": message,
                                        "type": "server_error"
                                    }
                                });
                                let json = serde_json::to_string(&payload)
                                    .unwrap_or_else(|_| "{}".to_string());
                                yield Ok(Event::default().data(json));
                                yield Ok(Event::default().data("[DONE]"));
                            }
                            should_stop = true;
                        }
                        _ => {}
                    }
                    should_stop
                }};
            }

            let mut stop_stream = false;
            for replay_event in replay {
                if process_stream_event!(replay_event) {
                    stop_stream = true;
                    break;
                }
            }

            if !stop_stream {
                while let Ok(event) = rx.recv().await {
                    if event.id <= replay_max_id {
                        continue;
                    }
                    if process_stream_event!(event) {
                        break;
                    }
                }
            }
        };

        let mut response = Sse::new(stream)
            .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
            .into_response();
        response.headers_mut().insert(
            HeaderName::from_static("x-uar-session-id"),
            HeaderValue::from_str(&response_session_id)
                .unwrap_or_else(|_| HeaderValue::from_static("invalid")),
        );
        if let Ok(cookie) = HeaderValue::from_str(&format!(
            "uar_session_id={response_session_id}; Path=/; HttpOnly; SameSite=Lax"
        )) {
            response.headers_mut().append(header::SET_COOKIE, cookie);
        }
        return response;
    }

    let mut assistant_text = String::new();
    let mut replay_result: Option<Result<(), String>> = None;
    if let Some(replay) = state.run_manager.history_since(&run_id, None).await {
        for event in replay {
            match event.event {
                uar::domain::events::NormalizedEvent::ChatDelta { text_delta, .. } => {
                    assistant_text.push_str(&text_delta);
                }
                uar::domain::events::NormalizedEvent::RunDone { .. }
                | uar::domain::events::NormalizedEvent::RunDoneWithUsage { .. } => {
                    replay_result = Some(Ok(()));
                    break;
                }
                uar::domain::events::NormalizedEvent::Error { message, .. } => {
                    replay_result = Some(Err(message));
                    break;
                }
                _ => {}
            }
        }
    }

    let wait_result = if let Some(result) = replay_result {
        Ok(result)
    } else {
        timeout(
            Duration::from_millis(effective_resilience_policy.request_timeout_ms),
            async {
                loop {
                    match rx.recv().await {
                        Ok(event) => match event.event {
                            uar::domain::events::NormalizedEvent::ChatDelta {
                                text_delta, ..
                            } => {
                                assistant_text.push_str(&text_delta);
                            }
                            uar::domain::events::NormalizedEvent::RunDone { .. }
                            | uar::domain::events::NormalizedEvent::RunDoneWithUsage { .. } => {
                                break Ok::<(), String>(());
                            }
                            uar::domain::events::NormalizedEvent::Error { message, .. } => {
                                break Err(message);
                            }
                            _ => {}
                        },
                        Err(e) => break Err(format!("Stream closed: {e}")),
                    }
                }
            },
        )
        .await
    };

    match wait_result {
        Ok(Ok(())) => {}
        Ok(Err(msg)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": {
                        "message": msg,
                        "type": "server_error"
                    }
                })),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::GATEWAY_TIMEOUT,
                Json(json!({
                    "error": {
                        "message": "Timed out waiting for model response",
                        "type": "timeout_error"
                    }
                })),
            )
                .into_response();
        }
    }

    if assistant_text.trim().is_empty()
        && let Some(session) = state.sessions.get(&session_id)
    {
        let messages = session.messages();
        if let Some(last_assistant) = messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, crate::llm::MessageRole::Assistant))
        {
            assistant_text = last_assistant.content.to_string();
        }
    }

    let body = OpenAiChatCompletionResponse {
        id: completion_id,
        object: "chat.completion",
        created,
        model: model_name,
        choices: vec![OpenAiChatChoice {
            index: 0,
            message: OpenAiAssistantMessage {
                role: "assistant",
                content: assistant_text,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: None,
        session_id: session_id.clone(),
    };

    let mut response = Json(body).into_response();
    response.headers_mut().insert(
        HeaderName::from_static("x-uar-session-id"),
        HeaderValue::from_str(&session_id).unwrap_or_else(|_| HeaderValue::from_static("invalid")),
    );
    if let Ok(cookie) = HeaderValue::from_str(&format!(
        "uar_session_id={session_id}; Path=/; HttpOnly; SameSite=Lax"
    )) {
        response.headers_mut().append(header::SET_COOKIE, cookie);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_toolu_id_prefixes_non_prefixed_ids() {
        assert_eq!(ensure_toolu_id("abc"), "toolu_abc");
        assert_eq!(ensure_toolu_id("toolu_xyz"), "toolu_xyz");
    }

    #[test]
    fn convert_anthropic_messages_round_trips_tool_blocks() {
        let req = AnthropicMessagesRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            messages: vec![
                AnthropicMessageInput {
                    role: "assistant".to_string(),
                    content: AnthropicContentInput::Blocks(vec![AnthropicContentBlockInput {
                        block_type: "tool_use".to_string(),
                        id: Some("call_1".to_string()),
                        name: Some("edit_file".to_string()),
                        input: Some(json!({"path":"foo.rs"})),
                        ..AnthropicContentBlockInput::default()
                    }]),
                    _extra: HashMap::new(),
                },
                AnthropicMessageInput {
                    role: "user".to_string(),
                    content: AnthropicContentInput::Blocks(vec![AnthropicContentBlockInput {
                        block_type: "tool_result".to_string(),
                        tool_use_id: Some("call_1".to_string()),
                        content: Some(json!("ok")),
                        ..AnthropicContentBlockInput::default()
                    }]),
                    _extra: HashMap::new(),
                },
            ],
            system: None,
            tools: Vec::new(),
            stream: false,
            prompt_caching_enabled: None,
            _extra: HashMap::new(),
        };

        let out = convert_anthropic_messages_to_openai(&req);
        assert_eq!(out.len(), 2);
        assert_eq!(
            out[0].get("role").and_then(Value::as_str),
            Some("assistant")
        );
        assert_eq!(
            out[0]
                .get("tool_calls")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(out[1].get("role").and_then(Value::as_str), Some("tool"));
        assert_eq!(
            out[1].get("tool_call_id").and_then(Value::as_str),
            Some("toolu_call_1")
        );
    }

    #[test]
    fn request_timeout_policy_skips_chat_stream_routes() {
        assert!(!should_apply_request_timeout("/api/chat/completion"));
        assert!(!should_apply_request_timeout("/v1/chat/completions"));
        assert!(!should_apply_request_timeout("/v1/messages"));

        assert!(should_apply_request_timeout("/api/generate-title"));
        assert!(should_apply_request_timeout("/api/threads"));
        assert!(should_apply_request_timeout("/assets/index.js"));
    }

    #[tokio::test]
    async fn cors_layer_allows_all_origins_methods_and_headers() {
        use axum::{Router, body::Body, http::Method, http::header, routing::get};
        use tower::ServiceExt;

        let app = Router::new()
            .route("/ping", get(|| async { "ok" }))
            .layer(build_permissive_cors_layer());

        let req = Request::builder()
            .method(Method::OPTIONS)
            .uri("/ping")
            .header(header::ORIGIN, "https://example.com")
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(
                header::ACCESS_CONTROL_REQUEST_HEADERS,
                "x-custom,content-type",
            )
            .body(Body::empty())
            .expect("preflight request should build");

        let res = app
            .oneshot(req)
            .await
            .expect("preflight request should succeed");
        assert!(res.status().is_success());
        assert!(
            res.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_some()
        );
        assert!(
            res.headers()
                .get(header::ACCESS_CONTROL_ALLOW_METHODS)
                .is_some()
        );
        assert!(
            res.headers()
                .get(header::ACCESS_CONTROL_ALLOW_HEADERS)
                .is_some()
        );
    }

    #[tokio::test]
    async fn compute_anthropic_input_usage_tracks_cache_create_and_hit() {
        let cache = SurrealMemPromptCacheProvider::new()
            .await
            .expect("cache should initialize");
        let provider: Arc<dyn PromptCacheProvider> = Arc::new(cache);
        let request = AnthropicMessagesRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            messages: vec![AnthropicMessageInput {
                role: "user".to_string(),
                content: AnthropicContentInput::Blocks(vec![AnthropicContentBlockInput {
                    block_type: "text".to_string(),
                    text: Some("cached text block".to_string()),
                    cache_control: Some(AnthropicCacheControlInput::default()),
                    ..AnthropicContentBlockInput::default()
                }]),
                _extra: HashMap::new(),
            }],
            system: None,
            tools: Vec::new(),
            stream: true,
            prompt_caching_enabled: None,
            _extra: HashMap::new(),
        };

        let first = compute_anthropic_input_usage(&request, &provider).await;
        assert_eq!(first.input_tokens, 0);
        assert!(first.cache_creation_input_tokens > 0);
        assert_eq!(first.cache_read_input_tokens, 0);

        let second = compute_anthropic_input_usage(&request, &provider).await;
        assert_eq!(second.input_tokens, 0);
        assert_eq!(second.cache_creation_input_tokens, 0);
        assert!(second.cache_read_input_tokens > 0);
    }

    #[test]
    fn normalize_legacy_openai_base_url_rewrites_known_non_openai_provider() {
        let mut config = crate::config::LlmConfig {
            model: "alibaba/qwen3.6-plus".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
            ..crate::config::LlmConfig::default()
        };

        normalize_legacy_openai_base_url(&mut config);

        assert_eq!(
            config.base_url.as_deref(),
            Some("https://dashscope-intl.aliyuncs.com/compatible-mode/v1")
        );
    }

    #[test]
    fn normalize_legacy_openai_base_url_adds_openai_v1_path() {
        let mut config = crate::config::LlmConfig {
            model: "openai/gpt-5.4".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
            ..crate::config::LlmConfig::default()
        };

        normalize_legacy_openai_base_url(&mut config);

        assert_eq!(
            config.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
    }

    #[test]
    fn provider_catalog_status_marks_moonshot_credential_blocked() {
        let (status, detail) =
            provider_catalog_status("moonshotai", false, Some("MOONSHOT_API_KEY"));

        assert_eq!(status, "credential-blocked");
        assert!(detail.contains("MOONSHOT_API_KEY"));
        assert!(!detail.contains("sk-"));
    }

    #[test]
    fn provider_catalog_status_prefers_configured_over_missing_credential() {
        let (status, detail) =
            provider_catalog_status("moonshotai", true, Some("MOONSHOT_API_KEY"));

        assert_eq!(status, "configured");
        assert!(detail.contains("configured"));
    }
}
