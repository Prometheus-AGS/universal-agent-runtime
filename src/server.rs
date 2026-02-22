use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{any, get, post},
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

use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tower_http::cors::{Any, CorsLayer};

use tracing::{info, warn};

use crate::AppState;
use crate::config::AppConfig;
use crate::llm::{ChatCompletionsDriver, LlmDriver, LlmSettings, Orchestrator};
use crate::mcp::registry::McpRegistry;
use crate::normalized::NormalizedEvent as DriverEvent;
use crate::session::SessionStore;
use crate::uar::api::sse::to_agui_event;
use crate::uar::settings::resilience_policy::{
    PolicySource, ResiliencePolicy, resolve_effective_policy,
};
use crate::uar::{
    self,
    defaults::ensure_default_knowledge_base,
    domain::events::MemoryItem,
    governance::engine::GovernanceEngine,
    memory::{
        MemoryService,
        auto_capture::{self, ConversationMessage},
        context_builder,
    },
    persistence::{
        PersistenceLayer,
        providers::{postgres::PostgresProvider, surreal::SurrealDbProvider},
    },
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
            storage::{FilesystemStorageProvider, SkillStorageProvider},
        },
    },
    security::{
        api_keys::{ApiKeyService, InMemoryApiKeyStorage},
        claims::UserContext,
    },
};

/// Start the Axum server with the provided configuration.
pub async fn start_server(config: Arc<AppConfig>, settings: LlmSettings) -> anyhow::Result<()> {
    info!(
        name: "llm.config.loaded",
        base_url = %settings.base_url,
        model = %settings.model,
        "LLM configuration loaded"
    );

    // Initialize Persistence & RAG
    let mut ingest_service: Option<Arc<IngestService>> = None;
    let vector_matcher = Arc::new(VectorMatcher::new(0.75));

    // Initialize VectorMatcher explicitly (shared)
    if let Err(e) = vector_matcher.initialize().await {
        tracing::error!("Failed to initialize VectorMatcher: {:?}", e);
    }

    // Initialize persistence based on config.
    // All three branches produce trait-object arcs so the types unify across arms.
    let (persistence_layer, compiler_storage, agent_registry): (
        Arc<dyn PersistenceLayer>,
        Option<(
            Arc<dyn crate::uar::compiler::storage::SpecStorage>,
            Arc<dyn crate::uar::compiler::session::persistence::SessionStorage>,
        )>,
        Option<Arc<dyn crate::uar::api::a2a::AgentRegistry>>,
    ) = if matches!(
        config.persistence.provider.as_str(),
        "surreal" | "surrealdb"
    ) {
        let provider = SurrealDbProvider::new(
            &config.persistence.database_url,
            config.persistence.surreal_user.as_deref(),
            config.persistence.surreal_pass.as_deref(),
        )
        .await
        .expect("Failed to initialize SurrealDB");

        // Create compiler storage sharing the same DB connection
        let db = provider.client();
        let compiler_store = Arc::new(
            crate::uar::compiler::storage::surreal::SurrealCompilerStorage::new(db.clone()),
        );
        let spec: Arc<dyn crate::uar::compiler::storage::SpecStorage> =
            Arc::clone(&compiler_store) as Arc<dyn crate::uar::compiler::storage::SpecStorage>;
        let sess: Arc<dyn crate::uar::compiler::session::persistence::SessionStorage> =
            compiler_store as Arc<dyn crate::uar::compiler::session::persistence::SessionStorage>;
        let registry = Arc::new(crate::uar::api::a2a::SurrealAgentRegistry::new(db))
            as Arc<dyn crate::uar::api::a2a::AgentRegistry>;

        (
            Arc::new(provider) as Arc<dyn PersistenceLayer>,
            Some((spec, sess)),
            Some(registry),
        )
    } else {
        let provider = PostgresProvider::new(&config.persistence.database_url)
            .await
            .expect("Failed to initialize Postgres");
        let pool = provider.get_pool().clone();

        let compiler_store = Arc::new(
            crate::uar::compiler::storage::postgres::PostgresCompilerStorage::new(pool.clone()),
        );
        let spec: Arc<dyn crate::uar::compiler::storage::SpecStorage> =
            Arc::clone(&compiler_store) as Arc<dyn crate::uar::compiler::storage::SpecStorage>;
        let sess: Arc<dyn crate::uar::compiler::session::persistence::SessionStorage> =
            compiler_store as Arc<dyn crate::uar::compiler::session::persistence::SessionStorage>;
        let registry = Arc::new(crate::uar::api::a2a::PostgresAgentRegistry::new(pool))
            as Arc<dyn crate::uar::api::a2a::AgentRegistry>;

        (
            Arc::new(provider) as Arc<dyn PersistenceLayer>,
            Some((spec, sess)),
            Some(registry),
        )
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
    let mut mcp_registry = McpRegistry::load_from_file("mcp.json")
        .await
        .unwrap_or_else(|e| panic!("Failed to load MCP servers: {e:?}"));

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
    uar::runtime::native_skills::register_builtins(&native_skill_registry).await;
    info!(
        "Native skill registry initialized with {} skills",
        native_skill_registry.len().await
    );

    // Create orchestrator
    let orchestrator = Arc::new(Orchestrator::new(
        settings.clone(),
        Arc::clone(&mcp),
        Arc::clone(&native_skill_registry),
    ));

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
    if let Err(e) = skill_service.initialize().await {
        eprintln!("Warning: Failed to initialize skills: {e:?}");
    }
    let skills = skill_service.registry().clone();
    let skill_service = Arc::new(skill_service);

    // Initialize Provider Registry
    let provider_registry = Arc::new(crate::llm::ProviderRegistry::new());
    provider_registry.seed_from_settings(&settings).await;
    if !config.providers.is_empty() {
        provider_registry
            .seed_from_configs(config.providers.clone())
            .await;
    }

    let run_manager = Arc::new(
        RunManager::new(
            settings.clone(),
            Arc::clone(&mcp),
            sessions.clone(),
            Arc::clone(&skills),
            Arc::clone(&vector_matcher), // Passed explicitly
            persistence.clone(),         // Passed explicitly
        )
        .await
        .with_skill_service(Arc::clone(&skill_service))
        .with_provider_registry(Arc::clone(&provider_registry))
        .with_native_skills(Arc::clone(&native_skill_registry)),
    );

    // Initialize Global Rate Limiter
    #[allow(clippy::cast_sign_loss)]
    let burst_size = config.resilience.burst_size.max(0.0) as u32;
    let rate_limiter = Arc::new(uar::security::rate_limit::AppRateLimiter::new(
        config.resilience.requests_per_second,
        burst_size,
    ));

    // Initialize Actor Collaboration System
    let actor_system = Arc::new(ActorCollaboration::new(
        settings.clone(),
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
                Ok(stats) => info!(
                    seeded = stats.seeded,
                    updated = stats.updated,
                    drift = stats.drift_count,
                    types = stats.types_upserted,
                    "Settings bootstrapped from config into DB"
                ),
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
        native_skill_registry: Arc::clone(&native_skill_registry),
        federated_agent_registry: Arc::clone(&federated_agent_registry),
        actor_system: Arc::clone(&actor_system),
        governance_engine: Arc::clone(&governance_engine),
        api_key_service: Some(Arc::clone(&api_key_service)),
        compiler_service: Some(Arc::clone(&compiler_service)),
        settings_manager: settings_manager.clone(),
        memory_service: memory_service.clone(),
        prompt_cache_provider,
        a2ui_registry: uar::a2ui::registry::A2uiRegistry::with_builtins(),
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
        .route("/health", get(health_handler))
        .route("/healthz", get(health_handler))
        .route("/readyz", get(health_handler))
        .route("/api/models", get(api_models))
        .route("/api/generate-title", post(api_generate_title))
        .route("/api/chat/completion", post(api_chat_completion))
        .route("/api/upload", post(uar::api::upload::upload_handler))
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
            uar::api::providers::build_router().with_state(Arc::clone(&state.provider_registry)),
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
                },
            )),
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
            // Initialize ingestion worker pool if persistence available
            let ingestion_pool = if let Some(p) = &persistence {
                if let Some(ingest) = &state.ingest_service {
                    match IngestionWorkerPool::new(
                        0,   // auto-detect CPU count
                        100, // max queue depth
                        Arc::clone(ingest),
                        Arc::clone(p),
                    ) {
                        Ok(pool) => {
                            info!("Ingestion worker pool initialized");
                            Some(Arc::new(pool))
                        }
                        Err(e) => {
                            tracing::error!("Failed to create ingestion pool: {:?}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

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
            uar::api::providers::build_router().with_state(Arc::clone(&state.provider_registry)),
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
                },
            )),
        )
        // Knowledge bases: GET/POST /api/knowledge, etc.
        .nest("/api/knowledge", {
            let ingestion_pool = if let Some(p) = &persistence {
                if let Some(ingest) = &state.ingest_service {
                    match IngestionWorkerPool::new(0, 100, Arc::clone(ingest), Arc::clone(p)) {
                        Ok(pool) => Some(Arc::new(pool)),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            };
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
        .route("/api/agents", get(uar::api::discovery::list_agents))
        .route("/api/tools", get(uar::api::discovery::list_tools))
        // ────────────────────────────────────────────────────────────────────────────
        .route("/api/ingest", post(uar::api::ingest::ingest_handler))
        .route(
            "/api/memory",
            post(uar::api::memory::save_memory_handler)
                .get(uar::api::memory::search_memory_handler),
        )
        .route("/api/{*path}", any(api_route_not_found))
        .route("/v1/chat/completions", post(api_chat_completion))
        .route("/v1/messages", post(api_messages))
        // ── SPA client-side routes ────────────────────────────────────────────
        // These paths are handled by React Router in the browser. When a user
        // hard-refreshes or navigates directly to one of them, the browser sends
        // a real GET request to the server. We explicitly serve index.html so
        // that React Router can take over and render the correct view.
        // Without this, ServeDir's not_found_service can race with the 404 path
        // in some middleware configurations and leak a 404 to the browser.
        .route("/threads", get(spa_index_handler))
        .route("/admin", get(spa_index_handler))
        .route("/admin/{*path}", get(spa_index_handler))
        .route("/about", get(spa_index_handler))
        // Serve the React SPA from static/.
        // ServeDir serves /assets/*, /favicon.svg, /manifest.json etc. with correct MIME types.
        // The not_found_service fallback delivers index.html for any other unknown paths.
        .fallback_service(
            ServeDir::new("static").not_found_service(ServeFile::new("static/index.html")),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            uar::security::middleware::auth_middleware,
        ))
        // Apply Timeout Layer if not disabled
        // We use a large timeout if disabled instead of conditional layering to keep types consistent
        .layer(TraceLayer::new_for_http());

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
        .with_state(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(
        name: "server.started",
        address = %addr,
        "Server started"
    );

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn build_permissive_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
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

async fn health_handler() -> StatusCode {
    StatusCode::OK
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

async fn api_models(State(state): State<AppState>) -> Response {
    let providers = state.provider_registry.list().await;
    let mut root = serde_json::Map::new();

    for provider in providers.into_iter().filter(|p| p.enabled) {
        let mut models = serde_json::Map::new();

        let model_entries = if provider.models.is_empty() {
            provider
                .default_model
                .iter()
                .map(|id| {
                    (
                        id.clone(),
                        None,
                        Some(128_000_u32),
                        Some(4_096_u32),
                        false,
                        true,
                    )
                })
                .collect::<Vec<_>>()
        } else {
            provider
                .models
                .iter()
                .map(|model| {
                    (
                        model.id.clone(),
                        model.display_name.clone(),
                        model.context_window,
                        model.max_output_tokens,
                        model.supports_vision,
                        model.supports_tools,
                    )
                })
                .collect::<Vec<_>>()
        };

        for (
            model_id,
            display_name,
            context_window,
            max_output_tokens,
            supports_vision,
            supports_tools,
        ) in model_entries
        {
            let context = context_window.unwrap_or(128_000);
            let output = max_output_tokens.unwrap_or(4_096);
            let input_modalities: Value = if supports_vision {
                json!(["text", "image"])
            } else {
                json!(["text"])
            };

            models.insert(
                model_id.clone(),
                json!({
                    "name": display_name.unwrap_or(model_id),
                    "limit": {
                        "context": context,
                        "input": context,
                        "output": output
                    },
                    "cost": {
                        "input": 0.0,
                        "output": 0.0
                    },
                    "modalities": {
                        "input": input_modalities,
                        "output": ["text"]
                    },
                    "tool_call": supports_tools,
                    "reasoning": true
                }),
            );
        }

        root.insert(provider.id, json!({ "models": models }));
    }

    Json(Value::Object(root)).into_response()
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
async fn spa_index_handler() -> Response {
    match tokio::fs::read("static/index.html").await {
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
    format!("{:x}", hasher.finalize())
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
    Json(req): Json<AnthropicMessagesRequest>,
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

    let llm_settings = match state
        .provider_registry
        .resolve(&resolved_model.provider_id, &resolved_model.model_id)
        .await
    {
        Some(settings) => settings,
        None => {
            return anthropic_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to resolve provider settings",
            );
        }
    };

    let openai_messages = convert_anthropic_messages_to_openai(&req);

    let llm_request = crate::llm::LlmRequest {
        messages: openai_messages,
        tools: convert_anthropic_tools_to_openai(&req.tools),
    };

    let driver = ChatCompletionsDriver::new(llm_settings);
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
struct ChatCompletionRequest {
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
        if let Some(model_id) = default_provider.default_model.clone() {
            return Ok(ResolvedModel {
                provider_id: default_provider_id,
                model_id,
            });
        }
        if let Some(model) = default_provider.models.first() {
            return Ok(ResolvedModel {
                provider_id: default_provider_id,
                model_id: model.id.clone(),
            });
        }
        return Err(openai_error_response(
            StatusCode::NOT_FOUND,
            "Unknown model",
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
async fn api_chat_completion(
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

    // Prepare agent with provider/model policy resolved for this request.
    let mut agent = uar::defaults::default_agent();
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
                        uar::domain::events::NormalizedEvent::RunDone { .. } => {
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
    let wait_result = timeout(
        Duration::from_millis(effective_resilience_policy.request_timeout_ms),
        async {
            loop {
                match rx.recv().await {
                    Ok(event) => match event.event {
                        uar::domain::events::NormalizedEvent::ChatDelta { text_delta, .. } => {
                            assistant_text.push_str(&text_delta);
                        }
                        uar::domain::events::NormalizedEvent::RunDone { .. } => {
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
    .await;

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
            stream: true,
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
        use axum::{body::Body, http::header, http::Method, routing::get, Router};
        use tower::ServiceExt;

        let app = Router::new()
            .route("/ping", get(|| async { "ok" }))
            .layer(build_permissive_cors_layer());

        let req = Request::builder()
            .method(Method::OPTIONS)
            .uri("/ping")
            .header(header::ORIGIN, "https://example.com")
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "x-custom,content-type")
            .body(Body::empty())
            .expect("preflight request should build");

        let res = app.oneshot(req).await.expect("preflight request should succeed");
        assert!(res.status().is_success());
        assert!(res.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_some());
        assert!(res.headers().get(header::ACCESS_CONTROL_ALLOW_METHODS).is_some());
        assert!(res.headers().get(header::ACCESS_CONTROL_ALLOW_HEADERS).is_some());
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
}
