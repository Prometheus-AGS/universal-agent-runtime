#![deny(clippy::unwrap_used, clippy::expect_used)]

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
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Write as _;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use tracing::{Instrument, info, warn};

use crate::AppState;
use crate::config_manager::ConfigManager;
use crate::llm::Orchestrator;
use crate::mcp::registry::McpRegistry;
use crate::normalized::NormalizedEvent as DriverEvent;
use crate::session::SessionStore;
use crate::uar::api::sse::{
    enrich_agui_spec_payload, to_agui_event, to_agui_spec_event, to_runtime_entity_event,
};
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
    persistence::PersistenceLayer,
    prompt_cache::{PromptCacheProvider, SurrealMemPromptCacheProvider},
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

type ShutdownCleanup = Arc<dyn Fn() + Send + Sync + 'static>;
type ShutdownAsyncCleanup =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync + 'static>;

#[cfg(windows)]
mod windows_service;

#[cfg(windows)]
#[doc(hidden)]
pub use windows_service::run as run_windows_service;

const SHUTDOWN_DEADLINE_MARKER: &[u8] = b"UAR_SHUTDOWN outcome=deadline_enforced\n";
const SHUTDOWN_GRACEFUL_MARKER: &[u8] = b"UAR_SHUTDOWN outcome=graceful_complete\n";
const SHUTDOWN_HARD_STOP_ALLOWANCE: Duration = Duration::from_millis(500);

#[derive(Clone)]
struct ShutdownCoordinator {
    started: tokio_util::sync::CancellationToken,
    cleanup_complete: tokio_util::sync::CancellationToken,
    process_complete: Arc<(Mutex<bool>, Condvar)>,
}

impl ShutdownCoordinator {
    fn new() -> Self {
        Self {
            started: tokio_util::sync::CancellationToken::new(),
            cleanup_complete: tokio_util::sync::CancellationToken::new(),
            process_complete: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    fn begin(&self, timeout: Duration) {
        if self.started.is_cancelled() {
            return;
        }
        self.started.cancel();

        let fuse_state = Arc::clone(&self.process_complete);
        let fuse = std::thread::Builder::new()
            .name("uar-shutdown-hard-stop".to_string())
            .spawn(move || {
                if shutdown_wait_expired(&fuse_state, timeout + SHUTDOWN_HARD_STOP_ALLOWANCE) {
                    std::process::exit(0);
                }
            });

        let watchdog_state = Arc::clone(&self.process_complete);
        let watchdog = std::thread::Builder::new()
            .name("uar-shutdown-deadline".to_string())
            .spawn(move || {
                if shutdown_wait_expired(&watchdog_state, timeout) {
                    emit_shutdown_marker_nonblocking(SHUTDOWN_DEADLINE_MARKER);
                    std::process::exit(0);
                }
            });

        if fuse.is_err() && watchdog.is_err() {
            emit_shutdown_marker_nonblocking(SHUTDOWN_DEADLINE_MARKER);
            std::process::exit(0);
        }
    }

    fn mark_cleanup_complete(&self) {
        self.cleanup_complete.cancel();
    }

    async fn wait_for_cleanup(&self) {
        if self.started.is_cancelled() {
            self.cleanup_complete.cancelled().await;
        }
    }

    fn complete(&self) {
        if !self.started.is_cancelled() {
            return;
        }
        let (lock, wake) = &*self.process_complete;
        let mut completed = match lock.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if *completed {
            return;
        }
        *completed = true;
        wake.notify_all();
        drop(completed);
        emit_shutdown_marker_nonblocking(SHUTDOWN_GRACEFUL_MARKER);
        info!(
            name: "server.shutdown.graceful_complete",
            "Server shutdown completed within the graceful deadline"
        );
    }
}

fn shutdown_wait_expired(state: &Arc<(Mutex<bool>, Condvar)>, timeout: Duration) -> bool {
    let (lock, wake) = &**state;
    let completed = match lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let (completed, _) = match wake.wait_timeout_while(completed, timeout, |done| !*done) {
        Ok(result) => result,
        Err(poisoned) => poisoned.into_inner(),
    };
    !*completed
}

#[cfg(unix)]
fn emit_shutdown_marker_nonblocking(marker: &[u8]) {
    use std::os::unix::fs::OpenOptionsExt as _;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    const NONBLOCK: i32 = 0x800;
    #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos",
        target_os = "visionos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    const NONBLOCK: i32 = 0x4;
    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos",
        target_os = "visionos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    const NONBLOCK: i32 = 0;

    if let Ok(mut stderr) = std::fs::OpenOptions::new()
        .write(true)
        .custom_flags(NONBLOCK)
        .open("/dev/stderr")
    {
        let _ = stderr.write(marker);
    }
}

#[cfg(not(unix))]
fn emit_shutdown_marker_nonblocking(marker: &[u8]) {
    let _ = std::io::stderr().write(marker);
}

#[cfg(feature = "in-memory-backend")]
use crate::uar::persistence::providers::memory::InMemoryProvider;
#[cfg(feature = "surreal-backend")]
use crate::uar::persistence::providers::surreal::SurrealDbProvider;

#[cfg(feature = "postgres-backend")]
use crate::uar::persistence::providers::postgres::PostgresProvider;

/// Tower middleware that enters a `tracing` span with request-scoped identifiers
/// before the request reaches the route handler. The span carries `request_id`,
/// `agent_id`, and `run_id` fields so that `UarError::into_response` captures a
/// non-empty `SpanTrace` for observability. `agent_id` and `run_id` are set to
/// `"none"` here; handlers that have them available can record them into the
/// current span with `Span::current().record(...)`.
async fn request_span_layer(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let span = tracing::info_span!(
        "http_request",
        request_id = %request_id,
        agent_id = "none",
        run_id = "none",
    );

    next.run(request).instrument(span).await
}

/// Start the Axum server with the provided configuration manager.
pub async fn start_server(config_manager: Arc<ConfigManager>) -> anyhow::Result<()> {
    start_server_with_listener(config_manager, None, None, None, None, None, None).await
}

#[cfg(windows)]
pub(crate) async fn start_server_with_shutdown(
    config_manager: Arc<ConfigManager>,
    process_shutdown: tokio_util::sync::CancellationToken,
) -> anyhow::Result<()> {
    start_server_with_listener(
        config_manager,
        None,
        None,
        None,
        None,
        None,
        Some(process_shutdown),
    )
    .await
}

async fn start_server_with_listener(
    config_manager: Arc<ConfigManager>,
    listener: Option<tokio::net::TcpListener>,
    a2a_grpc_listener: Option<tokio::net::TcpListener>,
    mcp_config_path: Option<std::path::PathBuf>,
    ready: Option<tokio::sync::oneshot::Sender<std::net::SocketAddr>>,
    http_shutdown: Option<tokio_util::sync::CancellationToken>,
    process_shutdown: Option<tokio_util::sync::CancellationToken>,
) -> anyhow::Result<()> {
    let config = config_manager.current();
    let embedded_lock = surrealkv_lock_path(
        &config.persistence.provider,
        &config.persistence.database_url,
    );
    let shutdown_coordinator = ShutdownCoordinator::new();
    let result = run_server_with_listener(
        config_manager,
        listener,
        a2a_grpc_listener,
        mcp_config_path,
        ready,
        http_shutdown,
        process_shutdown,
        shutdown_coordinator.clone(),
    )
    .await;

    if result.is_ok() && shutdown_coordinator.started.is_cancelled() {
        if let Some(lock_path) = embedded_lock {
            wait_for_surrealkv_lock_release(&lock_path).await;
        }
        shutdown_coordinator.complete();
    }
    result
}

fn surrealkv_lock_path(provider: &str, database_url: &str) -> Option<std::path::PathBuf> {
    if !matches!(provider, "surreal" | "surrealdb") {
        return None;
    }
    let endpoint = database_url.trim();
    let path = endpoint
        .strip_prefix("surrealkv://")
        .or_else(|| endpoint.strip_prefix("rocksdb://"))
        .or_else(|| (!endpoint.contains("://")).then_some(endpoint))?;
    if path.is_empty() || matches!(path, "surrealkv" | "rocksdb" | "memory" | "mem") {
        return None;
    }
    Some(std::path::PathBuf::from(path).join("LOCK"))
}

async fn wait_for_surrealkv_lock_release(lock_path: &std::path::Path) {
    loop {
        let file = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(lock_path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) => {
                tracing::warn!(path = %lock_path.display(), %error, "Could not observe SurrealKV lock release yet");
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }
        };
        match file.try_lock() {
            Ok(()) => {
                if let Err(error) = std::fs::File::unlock(&file) {
                    tracing::warn!(path = %lock_path.display(), %error, "Could not release SurrealKV shutdown observer lock");
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    continue;
                }
                info!(path = %lock_path.display(), "SurrealKV lock released before normal completion");
                return;
            }
            Err(std::fs::TryLockError::WouldBlock) => {}
            Err(std::fs::TryLockError::Error(error)) => {
                tracing::warn!(path = %lock_path.display(), %error, "Could not observe SurrealKV lock release yet");
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[expect(clippy::expect_used, reason = "init-time fatal configuration failure")]
async fn run_server_with_listener(
    config_manager: Arc<ConfigManager>,
    listener: Option<tokio::net::TcpListener>,
    _a2a_grpc_listener: Option<tokio::net::TcpListener>,
    mcp_config_path: Option<std::path::PathBuf>,
    ready: Option<tokio::sync::oneshot::Sender<std::net::SocketAddr>>,
    http_shutdown: Option<tokio_util::sync::CancellationToken>,
    process_shutdown: Option<tokio_util::sync::CancellationToken>,
    shutdown_coordinator: ShutdownCoordinator,
) -> anyhow::Result<()> {
    // UAR owns the process-level jsonwebtoken provider. Install it at the
    // shared startup funnel so in-process clients cannot initialize another
    // provider before the first authenticated request reaches middleware.
    crate::uar::security::jwt::ensure_rustcrypto_provider()
        .map_err(|error| anyhow::anyhow!("initializing JWT crypto provider: {error}"))?;

    // Install the Prometheus recorder before anything can record a metric.
    // `metrics::with_recorder` resolves the global recorder on every macro
    // call and silently discards writes to a no-op when none is installed, so
    // any metric recorded before this point is lost — and the `Counter` handle
    // it returns stays bound to that no-op. Lazy initialisation on the first
    // `/metrics` scrape is therefore not sufficient: request metrics recorded
    // before the first scrape would never appear. Idempotent, so the binaries'
    // own startup call remains harmless.
    #[cfg(feature = "telemetry")]
    crate::uar::telemetry::metrics::init();

    let config = config_manager.current();
    let mut llm_config = config.llm.clone();
    normalize_legacy_openai_base_url(&mut llm_config);
    info!(
        name: "llm.config.loaded",
        model = %llm_config.model,
        "LLM configuration loaded"
    );

    // Establish and seal every tool-capable network ingress before constructing
    // RunManager. Bound sockets are inert until governance finalization activates
    // their admission tokens below.
    let (governance_mutation, governance_gate, governance_status) =
        uar::governance::runtime_control::governance_runtime_handles(&config.server.host);
    governance_mutation.record_installed_authentication(config.security.jwt_required);
    governance_mutation.declare_ingress("primary-http")?;
    let listener = match listener {
        Some(listener) => listener,
        None => {
            let addr = format!("{}:{}", config.server.host, config.server.port);
            tokio::net::TcpListener::bind(&addr).await?
        }
    };
    let primary_addr = listener.local_addr()?;
    let mut ingress_proofs =
        vec![governance_mutation.register_bound_ingress("primary-http", primary_addr)?];

    let companion = bind_companion_listener(&config.server.host, primary_addr.port()).await;
    if let Some(companion_listener) = &companion {
        governance_mutation.declare_ingress("companion-http")?;
        ingress_proofs.push(
            governance_mutation
                .register_bound_ingress("companion-http", companion_listener.local_addr()?)?,
        );
    }

    #[cfg(feature = "a2a-transport")]
    let a2a_grpc_listener = {
        governance_mutation.declare_ingress("a2a-grpc")?;
        let listener = match _a2a_grpc_listener {
            Some(listener) => listener,
            None => {
                let grpc_addr =
                    tokio::net::lookup_host((config.server.host.as_str(), config.server.grpc_port))
                        .await?
                        .next()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "server.host '{}' did not resolve for A2A gRPC",
                                config.server.host
                            )
                        })?;
                tokio::net::TcpListener::bind(grpc_addr).await?
            }
        };
        ingress_proofs
            .push(governance_mutation.register_bound_ingress("a2a-grpc", listener.local_addr()?)?);
        listener
    };
    let governance_admission_tokens =
        governance_mutation.seal_ingress_inventory(&ingress_proofs)?;

    // Initialize Persistence & RAG
    let mut ingest_service: Option<Arc<IngestService>> = None;
    let embedding_config =
        crate::uar::rag::embeddings::EmbeddingConfig::from(&config.llm.embedding);
    let embedding_backend = match crate::uar::rag::embeddings::build_backend(&embedding_config) {
        Ok(b) => b,
        Err(e) => {
            warn!(
                backend = %embedding_config.backend,
                error = %e,
                "Embedding backend is unavailable; chat and agent interfaces will start without vector retrieval"
            );
            Arc::new(
                crate::uar::rag::embeddings::UnavailableEmbeddingBackend::new(
                    embedding_config.vector_dimension,
                    e.to_string(),
                ),
            )
        }
    };
    let vector_matcher = Arc::new(VectorMatcher::new(
        Arc::clone(&embedding_backend),
        config.models.vector_threshold,
    ));

    // Embeddings are intentionally lazy. UAR must bind its public interfaces
    // before optional ONNX/model preparation begins; keyword/TF-IDF skill
    // selection and chat do not require an embedding backend at startup.

    // Initialize persistence based on config.
    // All three branches produce trait-object arcs so the types unify across arms.
    let (
        persistence_layer,
        compiler_storage,
        agent_registry,
        live_bus,
        credential_store,
        a2ui_design_system_store,
        surreal_live_bus,
    ): (
        Arc<dyn PersistenceLayer>,
        Option<(
            Arc<dyn crate::uar::compiler::storage::SpecStorage>,
            Arc<dyn crate::uar::compiler::session::persistence::SessionStorage>,
        )>,
        Option<Arc<dyn crate::uar::api::a2a::AgentRegistry>>,
        Option<Arc<dyn crate::uar::realtime::RealtimeBus>>,
        Option<Arc<dyn uar::security::credentials::CredentialStore>>,
        uar::a2ui::design_systems::store::SharedDesignSystemStore,
        Option<Arc<crate::uar::realtime::surreal_bus::LiveQueryBus>>,
    ) = if config.persistence.provider == "memory" {
        #[cfg(feature = "in-memory-backend")]
        {
            (
                Arc::new(InMemoryProvider::new()) as Arc<dyn PersistenceLayer>,
                None,
                None,
                None,
                None,
                Arc::new(uar::a2ui::design_systems::store::InMemoryDesignSystemStore::new()),
                None,
            )
        }
        #[cfg(not(feature = "in-memory-backend"))]
        {
            anyhow::bail!(
                "persistence.provider = 'memory' requires the `in-memory-backend` Cargo feature"
            );
        }
    } else if matches!(
        config.persistence.provider.as_str(),
        "surreal" | "surrealdb"
    ) {
        #[cfg(feature = "surreal-backend")]
        {
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
            let a2ui_store = Arc::new(
                uar::a2ui::design_systems::store::SurrealDesignSystemStore::new(db.clone()),
            )
                as uar::a2ui::design_systems::store::SharedDesignSystemStore;
            let compiler_store = Arc::new(
                crate::uar::compiler::storage::surreal::SurrealCompilerStorage::new(db.clone()),
            );
            let spec: Arc<dyn crate::uar::compiler::storage::SpecStorage> =
                Arc::clone(&compiler_store) as Arc<dyn crate::uar::compiler::storage::SpecStorage>;
            let sess: Arc<dyn crate::uar::compiler::session::persistence::SessionStorage> =
                compiler_store
                    as Arc<dyn crate::uar::compiler::session::persistence::SessionStorage>;
            let registry = Arc::new(crate::uar::api::a2a::SurrealAgentRegistry::new(db.clone()))
                as Arc<dyn crate::uar::api::a2a::AgentRegistry>;

            // Durable per-user credential store on the same DB connection.
            let credential_store = Some(Arc::new(
                uar::security::credentials::SurrealCredentialStore::new(db.clone()),
            )
                as Arc<dyn uar::security::credentials::CredentialStore>);

            // Start the live-query bus on the same DB connection.
            let surreal_live_bus =
                Arc::new(crate::uar::realtime::surreal_bus::LiveQueryBus::start(db));
            let live_bus =
                Some(Arc::clone(&surreal_live_bus) as Arc<dyn crate::uar::realtime::RealtimeBus>);

            (
                Arc::new(provider) as Arc<dyn PersistenceLayer>,
                Some((spec, sess)),
                Some(registry),
                live_bus,
                credential_store,
                a2ui_store,
                Some(surreal_live_bus),
            )
        }
        #[cfg(not(feature = "surreal-backend"))]
        {
            anyhow::bail!(
                "persistence.provider = '{}' requires the `surreal-backend` Cargo feature",
                config.persistence.provider
            );
        }
    } else {
        // Non-surreal persistence requested. Only available when the
        // `postgres-backend` Cargo feature is enabled at build time.
        #[cfg(feature = "postgres-backend")]
        {
            let provider = PostgresProvider::new(&config.persistence.database_url)
                .await
                .expect("Failed to initialize Postgres");
            let pool = provider.get_pool().clone();
            let a2ui_store = Arc::new(
                uar::a2ui::design_systems::store::PostgresDesignSystemStore::new(pool.clone()),
            )
                as uar::a2ui::design_systems::store::SharedDesignSystemStore;

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
            let registry = Arc::new(crate::uar::api::a2a::PostgresAgentRegistry::new(
                pool.clone(),
            )) as Arc<dyn crate::uar::api::a2a::AgentRegistry>;

            // Durable per-user credential store on the same pool (CH-02) — no
            // longer falls back to in-memory on Postgres.
            let credential_store = Some(Arc::new(
                uar::security::credentials::PostgresCredentialStore::new(pool),
            )
                as Arc<dyn uar::security::credentials::CredentialStore>);

            (
                Arc::new(provider) as Arc<dyn PersistenceLayer>,
                Some((spec, sess)),
                Some(registry),
                live_bus,
                credential_store,
                a2ui_store,
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
    let persistence = Some(Arc::clone(&persistence_layer));

    // Initialize Ingest Service if persistence is available
    let mut ingestion_watcher = None;
    if let Some(p) = &persistence {
        let ingest = Arc::new(IngestService::new(
            Arc::clone(p),
            Arc::clone(&embedding_backend),
            ChunkingStrategy::Semantic { threshold: 0.5 },
        ));
        ingest_service = Some(Arc::clone(&ingest));

        // Spawn File Watcher
        let ingest_svc_clone = Arc::clone(&ingest);
        ingestion_watcher = Some(tokio::spawn(async move {
            let ingest_dir = std::path::PathBuf::from("/data/ingest");
            if !ingest_dir.exists() {
                let _ = tokio::fs::create_dir_all(&ingest_dir).await;
            }
            if ingest_dir.exists() {
                ingest_svc_clone
                    .watch(ingest_dir, "default".to_string())
                    .await;
            }
        }));

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
    let mcp_config_path = mcp_config_path
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new("mcp.json"));
    let mut mcp_registry =
        match McpRegistry::load_from_file(mcp_config_path.to_string_lossy().as_ref()).await {
            Ok(registry) => registry,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    path = %mcp_config_path.display(),
                    "Could not load MCP config — starting with empty MCP registry. \
                     Tools from the configured file will not be available until it is created."
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
    match skill_service.initialize().await {
        Ok(()) => {
            if let Err(error) = skill_service.reconcile_config_skills().await {
                tracing::error!(?error, "Failed to reconcile configuration skills");
            }
        }
        Err(error) => {
            eprintln!("Warning: Failed to initialize skills: {error:?}");
        }
    }
    let skills = skill_service.registry().clone();
    let skill_service = Arc::new(skill_service);

    // Load built-in (Manifest-kind, Builtin-origin) skills from the active
    // skill-pack root (CH-16: env override -> sibling checkout -> installed
    // plugin -> embedded submodule, see pack_detection). Failure here is
    // non-fatal.
    {
        let (builtins, pack_provenance) =
            uar::runtime::skills::builtin_loader::discover_builtin_skills();
        info!(
            name: "skills.pack.resolved",
            source = ?pack_provenance.source,
            root = %pack_provenance.root.display(),
            version = pack_provenance.version.as_deref().unwrap_or("<unknown>"),
            skill_count = builtins.len(),
            "Resolved active skill-pack root"
        );
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

    // Resolve the durable governance preference only after the bound ingress
    // inventory is sealed, and before RunManager or any serve loop is exposed.
    let settings_manager: Option<Arc<crate::uar::settings::manager::SettingsManager>> =
        if let Some(p) = &persistence {
            let mgr = Arc::new(
                crate::uar::settings::manager::SettingsManager::new(Arc::clone(p))
                    .with_governance_runtime(
                        governance_mutation.clone(),
                        governance_status.clone(),
                    ),
            );
            let governance_bootstrap = async {
                let persisted = mgr
                    .load_optional_persisted_value("governance.enabled")
                    .await?
                    .map(|value| {
                        value.as_bool().ok_or_else(|| {
                            anyhow::anyhow!("persisted governance.enabled is not a boolean")
                        })
                    })
                    .transpose()?;
                let plan = governance_mutation.preference_plan(persisted)?;
                mgr.apply_governance_preference_plan(&plan).await?;
                let stats = mgr
                    .initialize_with_governance_default(&config, plan.target_enabled)
                    .await?;
                governance_mutation.finalize_preference(&plan)?;
                Ok::<_, anyhow::Error>(stats)
            }
            .await;

            match governance_bootstrap {
                Ok(stats) => {
                    info!(
                        seeded = stats.seeded,
                        updated = stats.updated,
                        drift = stats.drift_count,
                        types = stats.types_upserted,
                        "Settings bootstrapped from config into DB"
                    );
                    if let Err(e) = mgr
                        .seed_providers_from_registry(provider_registry.as_ref())
                        .await
                    {
                        tracing::error!(error = ?e, "Failed to seed configured providers into the settings DB");
                    }
                    if let Err(e) = crate::uar::settings::hydrate_provider_registry_from_settings(
                        provider_registry.as_ref(),
                        mgr.as_ref(),
                    )
                    .await
                    {
                        tracing::error!(error = ?e, "Failed to hydrate provider registry from settings database");
                    }
                    if let Err(e) = crate::uar::api::mcp_admin::hydrate_registry(&mcp, &mgr).await {
                        tracing::error!(error = ?e, "Failed to hydrate MCP registry from settings database");
                    }
                }
                Err(error) => {
                    tracing::error!(
                        %error,
                        "Governance/settings bootstrap failed — governance remains enabled and mutation unavailable"
                    );
                    governance_mutation.finalize_mutation_unavailable()?;
                }
            }
            Some(mgr)
        } else {
            governance_mutation.finalize_mutation_unavailable()?;
            info!("No persistence layer — settings manager disabled");
            None
        };
    governance_mutation.activate_admission_tokens()?;
    let governance_boot_status = governance_status.snapshot();
    info!(
        boot_instance_id = %governance_boot_status.boot_instance_id,
        revision = governance_boot_status.revision,
        effective_enabled = governance_boot_status.effective_enabled,
        may_disable = governance_boot_status.may_disable,
        "Governance boot posture finalized"
    );

    // Initialize Governance Policy Engine (before RunManager so it can gate the
    // orchestrator tool loop in addition to the HTTP governance layer).
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

    let a2ui_realtime_backbone = uar::a2ui::realtime::InMemoryReplayBackbone::new();

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
        .with_agent_graph(uar::defaults::orchestrator_graph())
        .with_skill_service(Arc::clone(&skill_service))
        .with_provider_registry(Arc::clone(&provider_registry))
        .with_native_skills(Arc::clone(&native_skill_registry))
        .with_a2ui_backbone(Arc::clone(&a2ui_realtime_backbone))
        .with_message_context_strategy(config.context_strategy.clone())
        .with_governance_engine(Arc::clone(&governance_engine))
        .with_governance_gate(governance_gate.clone())
        .with_failover_config(config.failover.clone())
        .with_resilience_policy(uar::settings::resilience_policy::ResiliencePolicy::from(
            &config.resilience,
        ))
        .with_global_cost_budget(config.llm.budget.as_ref())
        .await;
        if let Some(ref svc) = provider_service {
            rm = rm.with_provider_service(Arc::clone(svc));
        }
        rm
    });

    // Root run-cancellation token: cancelling it on shutdown aborts all
    // in-flight runs (they emit a terminal `Cancelled` event) within the drain
    // window, instead of being killed abruptly at process teardown.
    let run_cancellation_root = run_manager.root_cancellation_token();

    // CH-03: periodic provider-health sweep, consuming the previously-dead
    // `health_check_secs` config. Shares the same shutdown token as the other
    // background loops (gRPC listener, run cancellation).
    {
        let health_check_secs = config.llm.health_check_secs.unwrap_or(30);
        Arc::clone(provider_registry.health())
            .spawn_monitor_loop(health_check_secs, run_cancellation_root.clone());
    }

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

    // Initialize API Key Service
    let api_key_storage: Arc<dyn uar::security::api_keys::ApiKeyStorage> =
        Arc::new(InMemoryApiKeyStorage::new());
    let api_key_service = Arc::new(
        ApiKeyService::new(
            Arc::clone(&api_key_storage),
            config.security.jwt_secret.expose_secret(),
        )
        .with_registered_claims(
            config.security.jwt_issuer.clone(),
            config.security.jwt_audience.clone(),
        ),
    );
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

    // Initialize A2A state (shared task store + compiler service).
    #[cfg(feature = "a2a-transport")]
    let a2a_task_store = uar::api::a2a::TaskStore::new();
    #[cfg(feature = "a2a-transport")]
    let a2a_state = Arc::new(uar::api::a2a::A2AState {
        compiler_service: Arc::clone(&compiler_service),
        task_store: a2a_task_store,
        security: config.security.clone(),
        base_url: format!("http://{}:{}", config.server.host, config.server.port),
    });
    #[cfg(feature = "a2a-transport")]
    info!("A2A state initialized");
    let federated_agent_registry: Arc<dyn uar::api::a2a::AgentRegistry> = agent_registry
        .unwrap_or_else(|| Arc::new(crate::uar::api::a2a::registry::InMemoryAgentRegistry::new()));

    let prompt_cache_provider: Arc<dyn PromptCacheProvider> =
        match SurrealMemPromptCacheProvider::new().await {
            Ok(provider) => Arc::new(provider),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "failed to initialize prompt cache provider: {e}"
                ));
            }
        };

    let user_settings_store = Arc::new(
        crate::uar::runtime::user_settings_store::UserSettingsStore::new(Arc::clone(
            &persistence_layer,
        )),
    );

    let state = AppState {
        mcp: Arc::clone(&mcp),
        orchestrator,
        sessions,
        run_manager,
        ingest_service,
        vector_matcher: Arc::clone(&vector_matcher),
        embedding_backend,
        persistence: persistence.clone(),
        rate_limiter,
        config: config_manager.current(),
        config_manager: Arc::clone(&config_manager),
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
    let ingestion_pool_shared: Option<Arc<IngestionWorkerPool>> =
        if let (Some(p), Some(ingest)) = (&persistence, &state.ingest_service) {
            match IngestionWorkerPool::new(
                0,   // auto-detect CPU count
                100, // max queue depth
                Arc::clone(ingest),
                Arc::clone(p),
                config_manager.current(),
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

    // Shared durable-replay backbone for A2UI surface state patches (Change
    // 20, a2ui-realtime-backbone-from-flint-realtime-fabric). Both A2UI
    // routers below share one instance so replay is consistent regardless
    // of which router a request comes through.
    #[cfg(feature = "a2a-transport")]
    let a2a_routes: axum::Router<AppState> = Router::new()
        .nest(
            "/a2a/compiler",
            uar::api::a2a::build_rpc_router().with_state::<AppState>(Arc::clone(&a2a_state)),
        )
        .nest(
            "/.well-known",
            uar::api::a2a::build_well_known_router().with_state::<AppState>(Arc::clone(&a2a_state)),
        )
        .nest(
            "/a2a/registry",
            uar::api::a2a::build_discovery_router().with_state::<AppState>(Arc::new(
                uar::api::a2a::DiscoveryApiState {
                    registry: Arc::clone(&federated_agent_registry),
                },
            )),
        );
    #[cfg(not(feature = "a2a-transport"))]
    let a2a_routes: axum::Router<AppState> = Router::new();

    let app = Router::new();
    #[cfg(feature = "api-docs")]
    let app = app.merge(utoipa_swagger_ui::SwaggerUi::new("/api/docs").url(
        "/api/openapi.json",
        crate::uar::api::openapi::build_openapi_spec(),
    ));
    let app = app
        .route("/health", get(liveness_handler))
        .route("/healthz", get(liveness_handler))
        .route("/readyz", get(readiness_handler))
        .route("/.well-known/uar-config", get(uar_config_schema_handler))
        .route(
            "/.well-known/uar-config/reload",
            post(uar_config_reload_handler),
        )
        .route("/.well-known/security.txt", get(security_txt_handler))
        .route("/api/models", get(api_models))
        .route("/api/catalog", get(api_catalog))
        .route("/api/uar/route", post(api_route_model))
        .route("/api/generate-title", post(api_generate_title))
        .route("/api/chat/completion", post(api_chat_completion))
        .route("/api/upload", post(uar::api::upload::upload_handler))
        .route("/api/live", get(uar::api::live::live_stream_all))
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
                    governance_status: Some(governance_status.clone()),
                    settings_mutation_auth_required: config
                        .security
                        .settings_mutation_auth_required,
                    settings_admin_key: config.security.settings_admin_key.clone(),
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
                realtime_backbone: Arc::clone(&a2ui_realtime_backbone),
                design_system_store: Arc::clone(&a2ui_design_system_store),
            };
            uar::a2ui::routes::build_schema_router().with_state(a2ui_state)
        })
        // A2UI artifact-response injection (shares /api/uar/runs prefix)
        .nest("/api/uar/runs", {
            let a2ui_state = uar::a2ui::routes::A2uiApiState {
                registry: Arc::clone(&state.a2ui_registry),
                run_manager: Arc::clone(&state.run_manager),
                realtime_backbone: Arc::clone(&a2ui_realtime_backbone),
                design_system_store: Arc::clone(&a2ui_design_system_store),
            };
            uar::a2ui::routes::build_response_router().with_state(a2ui_state)
        })
        // Tool-call approval HITL gate: POST /api/uar/runs/{run_id}/approval
        .route(
            "/api/uar/runs/{run_id}/approval",
            post(handle_tool_call_approval),
        )
        .merge(a2a_routes)
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
        .nest(
            "/api/uar/mcp",
            uar::api::mcp_admin::build_router().with_state(state.clone()),
        )
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
        .route(
            "/api/uar/sessions/{id}/prompt-caching",
            get(uar::api::discovery::get_effective_prompt_caching),
        )
        .route(
            "/api/uar/conversations/{id}/policy",
            get(uar::api::discovery::get_conversation_policy)
                .put(uar::api::discovery::save_conversation_policy)
                .delete(uar::api::discovery::delete_conversation_policy),
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
        // Cedar governance: authorize requests carrying `X-Agent-Id` against the
        // loaded policy set (permit-all by default; anonymous requests pass
        // through). Previously defined but never mounted.
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state.governance_engine),
            uar::governance::middleware::governance_layer,
        ))
        // Apply Timeout Layer if not disabled
        // We use a large timeout if disabled instead of conditional layering to keep types consistent
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(request_span_layer));

    // ── ACP (Agent Communication Protocol) endpoint ──────────────────────────
    // Mounted conditionally so zero overhead is incurred when disabled.
    let app = if config.acp.enabled {
        info!(path = %config.acp.path, "ACP server enabled");
        let acp_router = uar::api::acp::routes::AcpRouter::new(config.acp.auth_required)
            .into_router(Arc::new(state.clone()))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                uar::security::middleware::auth_middleware,
            ));
        app.nest_service(&config.acp.path, acp_router)
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
    // Spawns a gRPC server on a separate port (default 50051) alongside HTTP,
    // sharing the root run-cancellation token so it drains at the same moment
    // in-flight runs are aborted (see the shutdown signal handler below).
    #[cfg(feature = "a2a-transport")]
    let grpc_handle = {
        let grpc_addr = a2a_grpc_listener.local_addr()?;
        let grpc_admission = governance_admission_tokens
            .iter()
            .find(|token| token.ingress_id() == "a2a-grpc")
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!("sealed governance inventory omitted A2A gRPC admission")
            })?;
        if !grpc_admission.is_active() {
            anyhow::bail!("A2A gRPC admission activated before governance finalization");
        }
        let grpc_service =
            crate::uar::api::a2a::grpc::GrpcAgentService::new(Arc::clone(&a2a_state));
        let grpc_shutdown = run_cancellation_root.clone();
        info!(name: "a2a.grpc.serving", address = %grpc_addr, "A2A gRPC transport serving");
        tokio::spawn(async move {
            let result = tonic::transport::Server::builder()
                .add_service(grpc_service.into_server())
                .serve_with_incoming_shutdown(
                    tokio_stream::wrappers::TcpListenerStream::new(a2a_grpc_listener),
                    async move {
                        grpc_shutdown.cancelled().await;
                        info!(name: "a2a.grpc.shutdown", "A2A gRPC transport shutting down");
                    },
                )
                .await;
            if let Err(e) = result {
                tracing::error!(error = %e, "A2A gRPC server error");
            }
        })
    };

    if governance_admission_tokens
        .iter()
        .any(|token| !token.is_active())
    {
        anyhow::bail!("HTTP admission activated before governance finalization");
    }

    if let Some(ready) = ready {
        ready
            .send(primary_addr)
            .map_err(|_| anyhow::anyhow!("sidecar supervisor stopped before readiness"))?;
    }

    let shutdown_cleanup = ingestion_pool_shared
        .map(|pool| Arc::new(move || pool.shutdown()) as Arc<dyn Fn() + Send + Sync + 'static>);
    let async_resource_cleanup = {
        let mcp = Arc::clone(&mcp);
        let surreal_live_bus = surreal_live_bus.clone();
        Arc::new(move || {
            let mcp = Arc::clone(&mcp);
            let surreal_live_bus = surreal_live_bus.clone();
            Box::pin(async move {
                let mcp_shutdown = mcp.shutdown();
                let live_query_shutdown = async move {
                    if let Some(bus) = surreal_live_bus {
                        bus.shutdown().await;
                    }
                };
                tokio::join!(mcp_shutdown, live_query_shutdown);
            }) as Pin<Box<dyn Future<Output = ()> + Send + 'static>>
        }) as ShutdownAsyncCleanup
    };
    let http_result = serve_on_listener(
        listener,
        companion,
        app,
        config.server.shutdown_timeout_secs,
        shutdown_cleanup,
        Some(async_resource_cleanup),
        run_cancellation_root,
        http_shutdown,
        process_shutdown,
        shutdown_coordinator.clone(),
    )
    .await;

    #[cfg(feature = "a2a-transport")]
    if let Err(e) = grpc_handle.await {
        tracing::error!(error = %e, "A2A gRPC task panicked");
    }

    shutdown_coordinator.wait_for_cleanup().await;
    if let Some(watcher) = ingestion_watcher {
        watcher.abort();
        if let Err(error) = watcher.await
            && !error.is_cancelled()
        {
            tracing::warn!(%error, "ingestion watcher task failed during shutdown");
        }
    }

    http_result
}

/// Best-effort bind of the loopback companion address for dual-stack `localhost`.
///
/// When the primary host is an IPv4 address (`0.0.0.0` or `127.0.0.1`), clients
/// that resolve `localhost` to IPv6 `::1` would otherwise get connection-refused.
/// Binding `[::1]` on the same port closes that gap. Symmetrically, an IPv6
/// primary gets a `127.0.0.1` companion. Returns `None` (and logs) when no
/// companion applies (hostname primary) or the bind fails — startup never fails
/// because of this.
async fn bind_companion_listener(host: &str, port: u16) -> Option<tokio::net::TcpListener> {
    let companion_ip = match host.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(_)) => "::1", // IPv4 primary → IPv6 loopback companion
        Ok(std::net::IpAddr::V6(_)) => "127.0.0.1", // IPv6 primary → IPv4 loopback companion
        Err(_) => return None,                // hostname or unparseable → no companion
    };
    let addr = format!("{companion_ip}:{port}");
    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            info!(
                name: "server.companion.bound",
                address = %addr,
                "Dual-stack companion listener bound"
            );
            Some(listener)
        }
        Err(e) => {
            tracing::warn!(
                name: "server.companion.skip",
                address = %addr,
                error = %e,
                "Could not bind dual-stack companion listener — continuing with primary only"
            );
            None
        }
    }
}

/// Start the server with a caller-provided, already-bound listener.
///
/// The readiness sender fires only after configuration, persistence, routes,
/// and the HTTP application are initialized. The listener remains owned by the
/// server for the entire startup path, so no other process can claim its port.
///
/// # Errors
///
/// Returns an error when runtime initialization or serving fails, or when the
/// supervising process drops the readiness receiver before startup completes.
pub async fn start_server_sidecar(
    config_manager: Arc<ConfigManager>,
    listener: tokio::net::TcpListener,
    ready: tokio::sync::oneshot::Sender<std::net::SocketAddr>,
    http_shutdown: Option<tokio_util::sync::CancellationToken>,
) -> anyhow::Result<()> {
    start_server_with_listener(
        config_manager,
        Some(listener),
        None,
        None,
        Some(ready),
        http_shutdown,
        None,
    )
    .await
}

/// Start the server with caller-provided, already-bound HTTP and A2A gRPC listeners.
///
/// This preserves both socket reservations across configuration loading and server
/// startup. It is intended for supervisors and integration harnesses that need
/// collision-free ephemeral ports for every ingress.
///
/// # Errors
///
/// Returns an error when runtime initialization or serving fails, or when the
/// supervising process drops the readiness receiver before startup completes.
pub async fn start_server_sidecar_with_listeners(
    config_manager: Arc<ConfigManager>,
    listener: tokio::net::TcpListener,
    a2a_grpc_listener: tokio::net::TcpListener,
    mcp_config_path: Option<std::path::PathBuf>,
    ready: tokio::sync::oneshot::Sender<std::net::SocketAddr>,
    http_shutdown: Option<tokio_util::sync::CancellationToken>,
) -> anyhow::Result<()> {
    start_server_with_listener(
        config_manager,
        Some(listener),
        Some(a2a_grpc_listener),
        mcp_config_path,
        Some(ready),
        http_shutdown,
        None,
    )
    .await
}

async fn serve_on_listener(
    listener: tokio::net::TcpListener,
    companion: Option<tokio::net::TcpListener>,
    app: axum::Router,
    shutdown_timeout_secs: u64,
    shutdown_cleanup: Option<ShutdownCleanup>,
    shutdown_async_cleanup: Option<ShutdownAsyncCleanup>,
    run_cancellation_root: tokio_util::sync::CancellationToken,
    http_shutdown: Option<tokio_util::sync::CancellationToken>,
    process_shutdown: Option<tokio_util::sync::CancellationToken>,
    shutdown_coordinator: ShutdownCoordinator,
) -> anyhow::Result<()> {
    let addr = listener.local_addr()?;
    let shutdown_timeout = Duration::from_secs(shutdown_timeout_secs);

    info!(
        name: "server.started",
        address = %addr,
        shutdown_timeout_secs = shutdown_timeout_secs,
        "Server started"
    );

    // A `CancellationToken` (rather than a one-shot channel) fans the
    // graceful-shutdown trigger out to BOTH the primary and the optional
    // companion serve loops from the single signal-handler task.
    let http_shutdown = http_shutdown.unwrap_or_else(tokio_util::sync::CancellationToken::new);

    // Spawn the signal handler: wait for SIGINT/SIGTERM, then:
    //   1. Shut down the ingestion worker pool (drains with timeout, detaches wedges).
    //   2. Fire the Axum graceful-shutdown trigger.
    {
        let cleanup_for_shutdown = shutdown_cleanup.clone();
        let async_cleanup_for_shutdown = shutdown_async_cleanup.clone();
        let run_cancellation_root = run_cancellation_root.clone();
        let http_shutdown = http_shutdown.clone();
        let shutdown_coordinator = shutdown_coordinator.clone();
        tokio::spawn(async move {
            match process_shutdown {
                Some(process_shutdown) => {
                    tokio::select! {
                        () = prometheus_parking_lot::core::shutdown::wait_for_signal() => {}
                        () = process_shutdown.cancelled() => {}
                    }
                }
                None => prometheus_parking_lot::core::shutdown::wait_for_signal().await,
            }
            shutdown_coordinator.begin(shutdown_timeout);
            info!(
                name: "server.shutdown",
                timeout_secs = shutdown_timeout.as_secs(),
                "Shutdown signal received — cancelling in-flight runs and draining ingestion pool"
            );
            // Abort in-flight runs first so they stop calling the LLM / tools and
            // emit a terminal Cancelled event before connections are drained.
            run_cancellation_root.cancel();
            // Stop both accept loops before potentially blocking cleanup so the
            // configured timeout is a drain deadline, not a pre-drain delay.
            http_shutdown.cancel();
            // Start independent resource cleanup concurrently. A wedged ingestion
            // worker must not delay MCP transport cancellation and stdio closure.
            let blocking_cleanup = async move {
                if let Some(cleanup) = cleanup_for_shutdown {
                    if let Err(e) = tokio::task::spawn_blocking(move || cleanup()).await {
                        tracing::warn!(error = %e, "ingestion pool shutdown task panicked");
                    }
                }
                info!(name: "server.shutdown.pool_drained", "Ingestion pool shut down");
            };
            let async_cleanup = async move {
                if let Some(cleanup) = async_cleanup_for_shutdown {
                    cleanup().await;
                }
                info!(name: "server.shutdown.async_resources_closed", "Async resources shut down");
            };
            tokio::join!(blocking_cleanup, async_cleanup);
            shutdown_coordinator.mark_cleanup_complete();
        });
    }

    // Build a graceful-shutdown future bound to the shared cancellation token.
    // `log_drain` is only emitted by the primary listener to avoid duplicate logs.
    let shutdown_future = |log_drain: bool| {
        let http_shutdown = http_shutdown.clone();
        async move {
            http_shutdown.cancelled().await;
            if log_drain {
                info!(
                    name: "server.draining",
                    timeout_secs = shutdown_timeout.as_secs(),
                    "Draining in-flight HTTP connections"
                );
            }
        }
    };

    let primary = axum::serve(listener, app.clone().into_make_service())
        .with_graceful_shutdown(shutdown_future(true));

    match companion {
        Some(companion_listener) => {
            if let Ok(companion_addr) = companion_listener.local_addr() {
                info!(
                    name: "server.started.companion",
                    address = %companion_addr,
                    "Dual-stack companion listener serving"
                );
            }
            let secondary = axum::serve(companion_listener, app.into_make_service())
                .with_graceful_shutdown(shutdown_future(false));
            tokio::try_join!(primary, secondary)?;
        }
        None => {
            primary.await?;
        }
    }

    info!(name: "server.http_stopped", "HTTP listeners stopped");
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
    axum::Extension(user): axum::Extension<UserContext>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let approved = body
        .get("approved")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if state
        .run_manager
        .get_run_for_user(&user.user_id, &run_id)
        .await
        .is_none()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "No pending approval found for run",
                "run_id": run_id
            })),
        )
            .into_response();
    }
    let resolved = state.run_manager.resolve_approval(&run_id, approved).await;
    if resolved {
        info!(
            name: "approval.resolved",
            run_id = %run_id,
            approved,
            "Tool-call approval resolved"
        );
        Json(serde_json::json!({
            "ok": true,
            "run_id": run_id,
            "approved": approved,
            "decision": if approved { "allow" } else { "deny" }
        }))
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

/// Serves an RFC 9116 `security.txt` at `GET /.well-known/security.txt`,
/// mirroring the reporting channel and disclosure SLA already documented in
/// `SECURITY.md`. Points at GitHub's private vulnerability reporting (the
/// project's actual reporting mechanism) rather than an email/PGP key, since
/// that is what this project actually uses.
pub(crate) async fn security_txt_handler()
-> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    // `Expires` per RFC 9116 SHOULD be no more than a year out; rotate this
    // value at least annually (operator task) — it does not update itself.
    const BODY: &str = "\
Contact: https://github.com/Prometheus-AGS/universal-agent-runtime/security/advisories/new
Expires: 2027-07-14T00:00:00.000Z
Preferred-Languages: en
Canonical: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/SECURITY.md
Policy: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/SECURITY.md
";
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; charset=utf-8",
        )],
        BODY,
    )
}

/// Serves the canonical JSON Schema for `AppConfig` — see
/// [`crate::config::AppConfig::json_schema`].
pub(crate) async fn uar_config_schema_handler() -> Json<Value> {
    Json(crate::config::AppConfig::json_schema())
}

/// Trigger an explicit configuration reload. Requires the `X-UAR-Admin-Key`
/// header when `security.settings_mutation_auth_required` is enabled.
async fn uar_config_reload_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let supplied_admin_key = headers
        .get("x-uar-admin-key")
        .and_then(|value| value.to_str().ok());
    if state.config.security.settings_mutation_auth_required
        && !crate::config::secret_value_matches(
            &state.config.security.settings_admin_key,
            supplied_admin_key,
        )
    {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.config_manager.reload().await {
        Ok(()) => Ok(Json(crate::config::AppConfig::json_schema())),
        Err(e) => {
            tracing::error!(name: "config.reload.endpoint_failed", error = %e, "Config reload endpoint failed");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
///         "reasoning": false,
///         "benchmarks": [
///           { "benchmark": "swe-bench-verified", "dimension": "coding", "score": 74.9,
///             "source_url": "...", "retrieved_date": "2026-06-01" }
///         ]
///       }
///     }
///   }
/// }
/// ```
async fn api_models(State(state): State<AppState>) -> Response {
    let catalog = crate::llm::ModelCatalog::global();
    let configured_providers = state
        .provider_registry
        .list()
        .await
        .into_iter()
        .map(|provider| (provider.id.clone(), provider))
        .collect::<std::collections::HashMap<_, _>>();

    let mut root = serde_json::Map::new();

    for provider in catalog.all_providers() {
        let mut models = serde_json::Map::new();
        for model in &provider.models {
            let enabled = configured_providers
                .get(&provider.id)
                .filter(|configured| configured.enabled)
                .and_then(|configured| {
                    configured
                        .models
                        .iter()
                        .find(|configured_model| configured_model.id == model.id)
                })
                .is_some_and(|configured_model| configured_model.enabled);
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

            // CH-10: sourced benchmark scores (CH-09), for the model-comparison
            // dashboard's side-by-side benchmark columns. Empty for the (common)
            // case where a model has no curated benchmark data.
            let benchmarks: Vec<Value> =
                crate::llm::benchmarks::scores_for(&format!("{}/{}", provider.id, model.id))
                    .iter()
                    .map(|s| {
                        json!({
                            "benchmark": s.benchmark,
                            "dimension": s.dimension,
                            "score": s.score,
                            "source_url": s.source_url,
                            "retrieved_date": s.retrieved_date
                        })
                    })
                    .collect();

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
                    "open_weights": model.open_weights,
                    "enabled": enabled,
                    "benchmarks": benchmarks
                }),
            );
        }

        root.insert(
            provider.id.clone(),
            json!({
                "display_name": provider.display_name,
                "base_url": provider.base_url,
                "configured": configured_providers.get(&provider.id).is_some_and(|configured| configured.enabled),
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
        .filter(|p| p.enabled && p.api_key.as_deref().map_or(false, |k| !k.trim().is_empty()))
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
#[cfg(feature = "telemetry")]
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

#[cfg(not(feature = "telemetry"))]
pub(crate) async fn api_metrics() -> Response {
    (
        StatusCode::NOT_FOUND,
        "telemetry capability is not compiled",
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
    axum::Extension(user_ctx): axum::Extension<UserContext>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Response {
    let session = state.sessions.get_for_user(&session_id, &user_ctx.user_id);
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
    /// Optional UAR conversation whose persisted policy should apply.
    #[serde(default)]
    session_id: Option<String>,
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

fn provider_anthropic_usage(
    prompt_tokens: u32,
    completion_tokens: u32,
    cached_tokens: Option<u32>,
    cache_creation_tokens: Option<u32>,
) -> AnthropicUsage {
    AnthropicUsage {
        input_tokens: prompt_tokens,
        output_tokens: completion_tokens,
        cache_creation_input_tokens: cache_creation_tokens.unwrap_or(0),
        cache_read_input_tokens: cached_tokens.unwrap_or(0),
    }
}

fn finalized_anthropic_usage(
    provider_usage: Option<AnthropicUsage>,
    estimated_output_tokens: u32,
) -> AnthropicUsage {
    provider_usage.unwrap_or_else(|| AnthropicUsage {
        output_tokens: estimated_output_tokens,
        ..AnthropicUsage::default()
    })
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
async fn sync_stream_handler(
    State(state): State<AppState>,
    axum::Extension(user): axum::Extension<UserContext>,
) -> Response {
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

        if let Some(ref persistence) = persistence {
            match persistence.list_knowledge_bases(&user.user_id).await {
                Ok(kbs) => {
                    yield Ok::<_, std::convert::Infallible>(
                        Event::default().event("entity.snapshot").data(
                            serde_json::json!({
                                "table": "knowledge_bases",
                                "records": kbs,
                                "ts": last_check
                            }).to_string()
                        )
                    );
                }
                Err(e) => {
                    tracing::debug!(error = %e, "sync_stream: failed to snapshot knowledge_bases");
                }
            }
        }

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
                match persistence.list_knowledge_bases(&user.user_id).await {
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
    headers: HeaderMap,
    user_ctx: Option<axum::Extension<UserContext>>,
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

    let global_prompt_caching = match &state.settings_manager {
        Some(manager) => manager
            .get_typed::<bool>("prompt_caching.enabled")
            .await
            .ok()
            .flatten()
            .unwrap_or(false),
        None => false,
    };
    let session_override = match (&req.session_id, user_ctx.as_ref()) {
        (Some(session_id), Some(user)) => {
            uar::api::discovery::load_conversation_policy(&state, &user.user_id, session_id)
                .await
                .and_then(|policy| policy.prompt_caching_enabled)
        }
        _ => None,
    };
    let jwt_principal = user_ctx
        .as_ref()
        .filter(|_| {
            headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.starts_with("Bearer "))
                && !headers.contains_key("x-api-key")
        })
        .and_then(|user| uar::api::user_settings::principal_storage_key(user));
    let user_override = if let Some(principal_id) = jwt_principal {
        match state
            .user_settings_store
            .caching_enabled_for(&principal_id)
            .await
        {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(%error, "failed to resolve user prompt-caching preference");
                return anthropic_error_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "User settings persistence unavailable",
                );
            }
        }
    } else {
        None
    };
    let effective_caching = crate::uar::domain::prompt_caching::resolve_effective_caching(
        req.prompt_caching_enabled,
        session_override,
        user_override,
        global_prompt_caching,
    );
    tracing::debug!(
        prompt_caching_enabled = effective_caching.enabled,
        source = ?effective_caching.source,
        "Resolved Anthropic-compatible prompt-caching setting"
    );

    let openai_messages = convert_anthropic_messages_to_openai(&req);

    let llm_request = crate::llm::LlmRequest {
        messages: openai_messages,
        tools: convert_anthropic_tools_to_openai(&req.tools),
        cache_strategy: effective_caching
            .enabled
            .then(crate::llm::anthropic_cache::CacheStrategy::default),
        thinking_config: None,
        anthropic_system: None,
        extra_params: None,
    };

    let driver = match crate::llm::orchestrator::build_driver(&resolved_llm_config) {
        Ok(driver) => driver,
        Err(err) => {
            tracing::error!(error = %err, "Failed to create LLM driver");
            return anthropic_error_response(
                StatusCode::BAD_GATEWAY,
                "Failed to initialize upstream model client",
            );
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
    if !req.stream {
        let mut final_text = String::new();
        let mut tool_blocks: Vec<Value> = Vec::new();
        let mut provider_usage: Option<AnthropicUsage> = None;
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
                    prompt_tokens,
                    completion_tokens,
                    cached_tokens,
                    cache_creation_tokens,
                    ..
                }) => {
                    provider_usage = Some(provider_anthropic_usage(
                        prompt_tokens,
                        completion_tokens,
                        cached_tokens,
                        cache_creation_tokens,
                    ));
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

        let usage = finalized_anthropic_usage(provider_usage, estimated_output_tokens);

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
        let mut provider_usage: Option<AnthropicUsage> = None;
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
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 0,
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
                    prompt_tokens,
                    completion_tokens,
                    cached_tokens,
                    cache_creation_tokens,
                    ..
                }) => {
                    provider_usage = Some(provider_anthropic_usage(
                        prompt_tokens,
                        completion_tokens,
                        cached_tokens,
                        cache_creation_tokens,
                    ));
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

                    let usage = finalized_anthropic_usage(
                        provider_usage.clone(),
                        estimated_output_tokens,
                    );
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
                    let usage = finalized_anthropic_usage(
                        provider_usage.clone(),
                        estimated_output_tokens,
                    );
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
                    let usage = finalized_anthropic_usage(
                        provider_usage.clone(),
                        estimated_output_tokens,
                    );
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
            let usage = finalized_anthropic_usage(
                provider_usage,
                estimated_output_tokens,
            );
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
    /// Official AG-UI protocol event vocabulary (`RUN_STARTED`,
    /// `TEXT_MESSAGE_CONTENT`, `TOOL_CALL_*`, `STATE_DELTA`, `RUN_FINISHED`,
    /// `RUN_ERROR`) instead of UAR's legacy `agui.*` names — CH-21/CH-18.
    /// What CopilotKit / Microsoft Agent Framework / Oracle A2UI clients
    /// (e.g. LibreFang) expect on the wire.
    #[serde(rename = "agui_spec")]
    AguiSpec,
}

impl StreamMode {
    fn emits_openai_chunks(&self) -> bool {
        matches!(self, Self::Openai | Self::Dual)
    }

    /// Legacy `agui.*`-named events (`Agui`/`Dual` only — unchanged
    /// behavior; `AguiSpec` uses [`Self::emits_agui_spec_chunks`] instead so
    /// existing consumers of the legacy names are unaffected).
    fn emits_agui_chunks(&self) -> bool {
        matches!(self, Self::Agui | Self::Dual)
    }

    /// Official AG-UI spec-vocabulary events (`AguiSpec` only).
    fn emits_agui_spec_chunks(&self) -> bool {
        matches!(self, Self::AguiSpec)
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
    /// Streaming payload mode: `openai` (default), `agui`, `dual`, or
    /// `agui_spec` (official AG-UI event vocabulary — CH-21/CH-18).
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
    /// Typed per-turn UAR policy override.
    #[serde(default)]
    run_policy: Option<uar::domain::policy::RunPolicy>,
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
    axum::Extension(user_ctx): axum::Extension<UserContext>,
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

    // Input guardrails: screen for prompt-injection / PII before the LLM call.
    // Detect-only by default (the finding is emitted on the run stream after it
    // starts); injection is blocked here only when explicitly enabled. The
    // finding carries a category + short reason — never the raw input.
    let input_finding = uar::guardrails::screen_input(&input_message, &state.config.guardrails);
    if let Some(ref finding) = input_finding {
        uar::telemetry::metrics::record_guardrail_flagged(finding.category.as_str());
        tracing::warn!(
            category = %finding.category.as_str(),
            reason = %finding.reason,
            "Chat input flagged by guardrail"
        );
        let g = &state.config.guardrails;
        let should_block = match finding.category {
            uar::guardrails::GuardrailCategory::Injection => g.block_on_injection,
            uar::guardrails::GuardrailCategory::Pii => g.block_on_pii,
        };
        if should_block {
            let code = match finding.category {
                uar::guardrails::GuardrailCategory::Injection => "guardrail_injection_blocked",
                uar::guardrails::GuardrailCategory::Pii => "guardrail_pii_blocked",
            };
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": {
                        "message": "Input rejected by guardrail policy",
                        "type": "guardrail_blocked",
                        "code": code
                    }
                })),
            )
                .into_response();
        }
    }

    let session_id = match resolve_session_id(&req, &headers) {
        Ok(Some(value)) => value,
        Ok(None) => Uuid::new_v4().to_string(),
        Err(resp) => return resp,
    };

    let mut turn_policy = req.run_policy.clone().unwrap_or_default();
    if let Some(agent_id) = &req.agent_id {
        turn_policy.agent_id = Some(agent_id.clone());
        turn_policy.chat_mode = Some(
            if matches!(agent_id.as_str(), "default-agent" | "orchestrator-agent") {
                uar::domain::policy::ChatMode::Uar
            } else {
                uar::domain::policy::ChatMode::Agent
            },
        );
    }
    if let Some(requested_model) = req.model.as_deref() {
        let resolved_turn_model = match resolve_requested_model(&state, Some(requested_model)).await
        {
            Ok(model) => model,
            Err(response) => return response,
        };
        turn_policy.model = Some(uar::domain::policy::ModelRoute {
            provider_id: resolved_turn_model.provider_id,
            model_id: resolved_turn_model.model_id,
        });
    }
    if !req.memory_enabled {
        turn_policy.memory_enabled = Some(false);
    }

    let initial_agent_id = req.agent_id.as_deref().unwrap_or("default-agent");
    let mut agent = uar::api::discovery::resolve_agent_for_run(&state, initial_agent_id).await;
    let mut effective_run_policy = uar::api::discovery::resolve_effective_run_policy(
        &state,
        &user_ctx.user_id,
        &session_id,
        &agent,
        Some(turn_policy.clone()),
    )
    .await;
    if let Some(effective_agent_id) = effective_run_policy.agent_id.clone()
        && effective_agent_id != agent.id
    {
        agent = uar::api::discovery::resolve_agent_for_run(&state, &effective_agent_id).await;
        effective_run_policy = uar::api::discovery::resolve_effective_run_policy(
            &state,
            &user_ctx.user_id,
            &session_id,
            &agent,
            Some(turn_policy),
        )
        .await;
    }

    let effective_model_name = effective_run_policy
        .model
        .as_ref()
        .map(|model| format!("{}/{}", model.provider_id, model.model_id));
    let resolved_model =
        match resolve_requested_model(&state, effective_model_name.as_deref()).await {
            Ok(model) => model,
            Err(response) => return response,
        };
    agent.policy.provider.default.provider = resolved_model.provider_id.clone();
    agent.policy.provider.default.model = resolved_model.model_id.clone();
    effective_run_policy.model = Some(uar::domain::policy::ModelRoute {
        provider_id: resolved_model.provider_id.clone(),
        model_id: resolved_model.model_id.clone(),
    });

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

    // --- Prompt caching: resolve effective setting for this request ---
    let global_prompt_caching = match &state.settings_manager {
        Some(manager) => manager
            .get_typed::<bool>("prompt_caching.enabled")
            .await
            .ok()
            .flatten()
            .unwrap_or(false),
        None => false,
    };
    let request_override = req.prompt_caching_enabled.or_else(|| {
        (effective_run_policy
            .provenance
            .get("prompt_caching_enabled")
            == Some(&uar::domain::policy::PolicyScope::Turn))
        .then_some(effective_run_policy.prompt_caching_enabled)
    });
    let session_override = (effective_run_policy
        .provenance
        .get("prompt_caching_enabled")
        == Some(&uar::domain::policy::PolicyScope::Conversation))
    .then_some(effective_run_policy.prompt_caching_enabled);
    let jwt_principal = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.starts_with("Bearer "))
        .filter(|_| !headers.contains_key("x-api-key"))
        .and_then(|_| uar::api::user_settings::principal_storage_key(&user_ctx));
    let user_override = if let Some(principal_id) = jwt_principal {
        match state
            .user_settings_store
            .caching_enabled_for(&principal_id)
            .await
        {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(%error, "failed to resolve user prompt-caching preference");
                return openai_error_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "User settings persistence unavailable",
                    Some("user_settings_unavailable"),
                );
            }
        }
    } else {
        None
    };
    let effective_prompt_caching = crate::uar::domain::prompt_caching::resolve_effective_caching(
        request_override,
        session_override,
        user_override,
        global_prompt_caching,
    );
    effective_run_policy.prompt_caching_enabled = effective_prompt_caching.enabled;
    tracing::debug!(
        prompt_caching_enabled = effective_prompt_caching.enabled,
        source = ?effective_prompt_caching.source,
        user_id = %user_ctx.user_id,
        "Resolved effective prompt-caching setting"
    );

    // --- Memory: context injection (pre-LLM-call) ---
    // Build context block and collect the raw hits so we can stream them to the client.
    let (memory_context_block, memory_recall_items) = if effective_run_policy.memory_enabled {
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
        .start_run_with_policy(
            agent,
            effective_input_with_memory,
            Some(session_id.clone()),
            Some(user_ctx.user_id.clone()),
            memory_recall_items,
            Some(effective_run_policy),
        )
        .await;

    // Surface a non-blocking input-guardrail finding on the run's event stream
    // (recorded in history before the client subscribes, so it replays).
    if let Some(finding) = input_finding {
        state
            .run_manager
            .emit_to_run(
                &run_id,
                uar::domain::events::NormalizedEvent::GuardrailFlagged {
                    run_id: Some(run_id.clone()),
                    category: finding.category.as_str().to_string(),
                    reason: finding.reason,
                },
            )
            .await;
    }

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
        let emit_agui_spec_chunks = req.stream_mode.emits_agui_spec_chunks();
        let stream_session_id = session_id.clone();
        // Keep an extra copy for the response headers (stream_session_id is moved into the closure).
        let response_session_id = session_id.clone();
        // Expose the server-assigned run_id so clients can target the cancel
        // endpoint (POST /api/uar/runs/{run_id}/cancel) and resume the stream.
        let response_run_id = run_id.clone();
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
        // Last-subscriber-drop guard: owned by the stream generator so a client
        // disconnect (generator dropped) cancels the run iff no subscriber
        // remains after a short grace period.
        let disconnect_guard = uar::runtime::manager::RunDisconnectGuard::new(
            Arc::clone(&state.run_manager),
            run_id.clone(),
        );

        let stream = async_stream::stream! {
            let _disconnect_guard = disconnect_guard;
            // Emit agui.memory.context event so frontend can show indicator.
            // No formal AG-UI event covers this UAR-specific signal, so spec
            // mode maps it to CUSTOM (same convention as to_agui_spec_event's
            // other UAR-only events) rather than dropping it silently.
            if (emit_agui_chunks || emit_agui_spec_chunks) && _stream_memory_ctx_count > 0 {
                let event_name = if emit_agui_spec_chunks { "CUSTOM" } else { "agui.memory.context" };
                let payload = if emit_agui_spec_chunks {
                    enrich_agui_spec_payload(
                        "CUSTOM",
                        serde_json::json!({
                            "name": "uar.memory.context_injected",
                            "value": { "count": _stream_memory_ctx_count },
                        }),
                        "0",
                        0,
                    )
                } else {
                    serde_json::json!({
                        "kind": "memory",
                        "phase": "injected",
                        "count": _stream_memory_ctx_count
                    })
                };
                let mem_event = Event::default()
                    .event(event_name)
                    .data(serde_json::to_string(&payload).unwrap_or_default());
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
            let mut seen_agui_spec_tool_call_ids: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut agui_spec_text_started = false;
            let mut agui_spec_reasoning_started = false;
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

                    // Official AG-UI spec-vocabulary events (CH-21/CH-18) —
                    // independent mode from the legacy agui.* names above, so
                    // existing `agui`/`dual` consumers see no change.
                    if emit_agui_spec_chunks {
                        let spec_run_id = match &normalized_event {
                            uar::domain::events::NormalizedEvent::RunStart { run_id, .. }
                            | uar::domain::events::NormalizedEvent::ChatDelta { run_id, .. }
                            | uar::domain::events::NormalizedEvent::ThinkingDelta { run_id, .. }
                            | uar::domain::events::NormalizedEvent::ReasoningDelta { run_id, .. }
                            | uar::domain::events::NormalizedEvent::RunDone { run_id }
                            | uar::domain::events::NormalizedEvent::RunDoneWithUsage { run_id, .. }
                            | uar::domain::events::NormalizedEvent::Error { run_id, .. }
                            | uar::domain::events::NormalizedEvent::Cancelled { run_id } => Some(run_id.as_str()),
                            _ => None,
                        };
                        if matches!(&normalized_event, uar::domain::events::NormalizedEvent::ChatDelta { .. })
                            && !agui_spec_text_started
                        {
                            agui_spec_text_started = true;
                            let run_id = spec_run_id.unwrap_or_default();
                            let payload = enrich_agui_spec_payload(
                                "TEXT_MESSAGE_START",
                                serde_json::json!({
                                    "messageId": format!("{run_id}:assistant"),
                                    "role": "assistant",
                                    "threadId": run_id,
                                    "runId": run_id,
                                }),
                                &event_id,
                                0,
                            );
                            yield Ok(Event::default()
                                .event("TEXT_MESSAGE_START")
                                .id(event_id.clone())
                                .data(payload.to_string()));
                        }
                        if matches!(&normalized_event,
                            uar::domain::events::NormalizedEvent::ThinkingDelta { .. }
                            | uar::domain::events::NormalizedEvent::ReasoningDelta { .. })
                            && !agui_spec_reasoning_started
                        {
                            agui_spec_reasoning_started = true;
                            let run_id = spec_run_id.unwrap_or_default();
                            for (ordinal, (name, payload)) in [
                                ("REASONING_START", serde_json::json!({
                                    "type": "REASONING_START", "profile": "uar.agui/1",
                                    "threadId": run_id, "runId": run_id,
                                })),
                                ("REASONING_MESSAGE_START", serde_json::json!({
                                    "type": "REASONING_MESSAGE_START", "profile": "uar.agui/1",
                                    "messageId": format!("{run_id}:reasoning"), "role": "reasoning",
                                    "threadId": run_id, "runId": run_id,
                                })),
                            ].into_iter().enumerate() {
                                let payload = enrich_agui_spec_payload(name, payload, &event_id, ordinal as u64);
                                yield Ok(Event::default().event(name).id(event_id.clone()).data(payload.to_string()));
                            }
                        }
                        if matches!(&normalized_event,
                            uar::domain::events::NormalizedEvent::RunDone { .. }
                            | uar::domain::events::NormalizedEvent::RunDoneWithUsage { .. }
                            | uar::domain::events::NormalizedEvent::Error { .. }
                            | uar::domain::events::NormalizedEvent::Cancelled { .. })
                        {
                            let run_id = spec_run_id.unwrap_or_default();
                            if agui_spec_text_started {
                                let payload = enrich_agui_spec_payload(
                                    "TEXT_MESSAGE_END",
                                    serde_json::json!({
                                        "messageId": format!("{run_id}:assistant"),
                                        "threadId": run_id,
                                        "runId": run_id,
                                    }),
                                    &event_id,
                                    0,
                                );
                                yield Ok(Event::default()
                                    .event("TEXT_MESSAGE_END")
                                    .id(event_id.clone())
                                    .data(payload.to_string()));
                            }
                            if agui_spec_reasoning_started {
                                for (offset, (name, payload)) in [
                                    ("REASONING_MESSAGE_END", serde_json::json!({
                                        "type": "REASONING_MESSAGE_END", "profile": "uar.agui/1",
                                        "messageId": format!("{run_id}:reasoning"),
                                        "threadId": run_id, "runId": run_id,
                                    })),
                                    ("REASONING_END", serde_json::json!({
                                        "type": "REASONING_END", "profile": "uar.agui/1",
                                        "threadId": run_id, "runId": run_id,
                                    })),
                                ].into_iter().enumerate() {
                                    let payload = enrich_agui_spec_payload(
                                        name, payload, &event_id, offset as u64 + 1,
                                    );
                                    yield Ok(Event::default().event(name).id(event_id.clone()).data(payload.to_string()));
                                }
                            }
                        }
                        // TOOL_CALL_START has no dedicated NormalizedEvent of
                        // its own — UAR's ToolDelta/ToolStart map to
                        // TOOL_CALL_ARGS/TOOL_CALL_END (see to_agui_spec_event).
                        // Synthesize START the first time a tool_call_id is
                        // seen, so agui_spec consumers get the full
                        // START/ARGS/END lifecycle the AG-UI spec expects.
                        let tool_call_start_info: Option<(String, Option<i64>, Option<String>)> =
                            match &normalized_event {
                                uar::domain::events::NormalizedEvent::ToolDelta {
                                    call_index,
                                    tool_call_id,
                                    ..
                                } => Some((tool_call_id.clone(), Some(*call_index as i64), None)),
                                uar::domain::events::NormalizedEvent::ToolStart {
                                    call_index,
                                    tool_call_id,
                                    tool,
                                    ..
                                } => Some((
                                    tool_call_id.clone(),
                                    Some(*call_index as i64),
                                    Some(tool.clone()),
                                )),
                                _ => None,
                            };
                        if let Some((tool_call_id, call_index, tool_name)) = tool_call_start_info
                            && seen_agui_spec_tool_call_ids.insert(tool_call_id.clone())
                        {
                            let start_payload = serde_json::json!({
                                "type": "TOOL_CALL_START",
                                "profile": "uar.agui/1",
                                "toolCallId": tool_call_id,
                                "toolCallName": tool_name.unwrap_or_else(|| format!("tool-{call_index:?}")),
                            });
                            let start_payload = enrich_agui_spec_payload(
                                "TOOL_CALL_START", start_payload, &event_id, 0,
                            );
                            let start_json = serde_json::to_string(&start_payload)
                                .unwrap_or_else(|_| "{}".to_string());
                            yield Ok(Event::default()
                                .event("TOOL_CALL_START")
                                .id(event_id.clone())
                                .data(start_json));
                        }

                        if let Some((event_name, payload)) = to_agui_spec_event(&normalized_event) {
                            let payload = enrich_agui_spec_payload(
                                event_name, payload, &event_id, 8,
                            );
                            let payload_json = serde_json::to_string(&payload)
                                .unwrap_or_else(|_| "{}".to_string());
                            let agui_spec_event = Event::default()
                                .event(event_name)
                                .id(event_id.clone())
                                .data(payload_json);
                            yield Ok(agui_spec_event);
                        }
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

                            // --- Response quality: sycophancy detection (fire-and-forget,
                            // post-stream; sync rule-based, no LLM call, no added latency) ---
                            #[cfg(feature = "response-quality")]
                            if !assistant_text_for_capture.is_empty() {
                                let sycophancy_cfg = state.config.sycophancy.clone();
                                let mgr = Arc::clone(&state.run_manager);
                                let rid = run_id.clone();
                                let text = assistant_text_for_capture.clone();
                                let orchestrator = Arc::clone(&state.orchestrator);
                                tokio::spawn(async move {
                                    let Some(outcome) = uar::quality::detect(&sycophancy_cfg, &text)
                                    else {
                                        return;
                                    };
                                    uar::telemetry::metrics::record_sycophancy_score(f64::from(
                                        outcome.score,
                                    ));
                                    if outcome.flagged {
                                        uar::telemetry::metrics::record_sycophancy_flagged();
                                        tracing::warn!(
                                            run_id = %rid,
                                            score = outcome.score,
                                            has_critical = outcome.has_critical,
                                            "Response flagged by sycophancy detection"
                                        );
                                        mgr.emit_to_run(
                                            &rid,
                                            uar::domain::events::NormalizedEvent::SycophancyFlagged {
                                                run_id: rid.clone(),
                                                sycophancy_score: outcome.score,
                                                has_critical: outcome.has_critical,
                                                correction_mandatory: outcome.correction_mandatory,
                                                classifications: outcome.classifications,
                                            },
                                        )
                                        .await;

                                        // Opt-in corrective pass (post-stream, single shot):
                                        // rewrite the flagged response and emit it as a
                                        // follow-up. Never delays the original stream.
                                        if sycophancy_cfg.auto_correct && !sycophancy_cfg.log_only {
                                            match orchestrator
                                                .chat_non_streaming(
                                                    uar::quality::correction_messages(&text),
                                                )
                                                .await
                                            {
                                                Ok(corrected) if !corrected.trim().is_empty() => {
                                                    mgr.emit_to_run(
                                                        &rid,
                                                        uar::domain::events::NormalizedEvent::SycophancyCorrected {
                                                            run_id: rid.clone(),
                                                            corrected_text: corrected,
                                                        },
                                                    )
                                                    .await;
                                                }
                                                Ok(_) => tracing::warn!(
                                                    run_id = %rid,
                                                    "Sycophancy auto-correct produced empty output"
                                                ),
                                                Err(e) => tracing::warn!(
                                                    run_id = %rid,
                                                    error = %e,
                                                    "Sycophancy auto-correct failed"
                                                ),
                                            }
                                        }
                                    }
                                });
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
        response.headers_mut().insert(
            HeaderName::from_static("x-uar-run-id"),
            HeaderValue::from_str(&response_run_id)
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
        && let Some(session) = state.sessions.get_for_user(&session_id, &user_ctx.user_id)
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
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use std::convert::Infallible;
    use std::io::Read as _;
    use std::net::{SocketAddr, TcpStream};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, ExitStatus, Stdio};
    use std::time::Instant;

    const SHUTDOWN_CHILD_ENV: &str = "UAR_SHUTDOWN_TEST_CHILD";
    const SHUTDOWN_MCP_FIXTURE: &str = r#"
import json
import pathlib
import sys

marker = pathlib.Path(sys.argv[1])
for line in sys.stdin:
    try:
        request = json.loads(line)
    except json.JSONDecodeError:
        continue
    request_id = request.get("id")
    if request_id is None:
        continue
    method = request.get("method")
    if method == "initialize":
        result = {
            "protocolVersion": request.get("params", {}).get("protocolVersion", "2024-11-05"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "uar-shutdown-test", "version": "1.0.0"},
        }
    elif method == "tools/list":
        result = {"tools": []}
    else:
        print(json.dumps({
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {"code": -32601, "message": "method not found"},
        }), flush=True)
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}), flush=True)

marker.write_bytes(b"stdin-closed")
"#;

    #[derive(Clone)]
    struct ShutdownSseState {
        control_dir: PathBuf,
        hold: bool,
    }

    #[derive(Deserialize)]
    struct ShutdownChildReady {
        primary: SocketAddr,
        companion: SocketAddr,
    }

    struct ShutdownChild {
        child: Child,
        control_dir: tempfile::TempDir,
        ready: ShutdownChildReady,
    }

    struct ShutdownChildExit {
        status: ExitStatus,
        stderr: String,
    }

    impl ShutdownChild {
        fn spawn(mode: &str, timeout_secs: u64) -> Self {
            let control_dir = tempfile::tempdir().expect("create shutdown control directory");
            let child = Command::new(std::env::current_exe().expect("resolve test executable"))
                .arg("--exact")
                .arg("server::tests::shutdown_process_child")
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(SHUTDOWN_CHILD_ENV, mode)
                .env("UAR_SHUTDOWN_TEST_TIMEOUT_SECS", timeout_secs.to_string())
                .env("UAR_SHUTDOWN_TEST_CONTROL_DIR", control_dir.path())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn shutdown child process");
            let mut this = Self {
                child,
                control_dir,
                ready: ShutdownChildReady {
                    primary: "127.0.0.1:1".parse().expect("placeholder primary"),
                    companion: "127.0.0.1:1".parse().expect("placeholder companion"),
                },
            };
            let ready_path = this.control_dir.path().join("ready.json");
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                if let Ok(bytes) = std::fs::read(&ready_path) {
                    this.ready = serde_json::from_slice(&bytes).expect("parse child readiness");
                    return this;
                }
                assert!(
                    this.child
                        .try_wait()
                        .expect("poll shutdown child")
                        .is_none(),
                    "shutdown child exited before readiness"
                );
                assert!(
                    Instant::now() < deadline,
                    "shutdown child readiness timeout"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        fn signal(&self, signal: &str) {
            let status = Command::new("/bin/kill")
                .arg(format!("-{signal}"))
                .arg(self.child.id().to_string())
                .status()
                .expect("deliver shutdown signal");
            assert!(status.success(), "failed to deliver {signal}: {status}");
        }

        fn wait_for_file(&mut self, name: &str) {
            let path = self.control_dir.path().join(name);
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if path.exists() {
                    return;
                }
                assert!(
                    self.child
                        .try_wait()
                        .expect("poll shutdown child")
                        .is_none(),
                    "shutdown child exited before publishing {name}"
                );
                assert!(Instant::now() < deadline, "timed out waiting for {name}");
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        fn wait_for_exit(&mut self, limit: Duration) -> Option<ShutdownChildExit> {
            let deadline = Instant::now() + limit;
            loop {
                if let Some(status) = self.child.try_wait().expect("poll shutdown child") {
                    let mut stderr = String::new();
                    if let Some(mut pipe) = self.child.stderr.take() {
                        pipe.read_to_string(&mut stderr)
                            .expect("read shutdown child stderr");
                    }
                    return Some(ShutdownChildExit { status, stderr });
                }
                if Instant::now() >= deadline {
                    return None;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

    impl Drop for ShutdownChild {
        fn drop(&mut self) {
            if self.child.try_wait().ok().flatten().is_none() {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
        }
    }

    async fn shutdown_test_sse(
        State(state): State<ShutdownSseState>,
    ) -> Sse<
        std::pin::Pin<Box<dyn futures::Stream<Item = Result<Event, Infallible>> + Send + 'static>>,
    > {
        std::fs::write(state.control_dir.join("stream-started"), b"started")
            .expect("publish SSE start");
        let stream = futures::stream::unfold(0_u8, move |step| {
            let hold = state.hold;
            async move {
                match step {
                    0 => Some((Ok(Event::default().data("started")), 1)),
                    1 if hold => {
                        std::future::pending::<Option<(Result<Event, Infallible>, u8)>>().await
                    }
                    1 => {
                        tokio::time::sleep(Duration::from_millis(250)).await;
                        Some((Ok(Event::default().data("done")), 2))
                    }
                    _ => None,
                }
            }
        });
        Sse::new(Box::pin(stream))
    }

    async fn run_shutdown_process_child(mode: &str, control_dir: &Path, timeout_secs: u64) {
        let primary = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind primary shutdown fixture");
        let primary_addr = primary.local_addr().expect("read primary address");
        let companion = tokio::net::TcpListener::bind(format!("[::1]:{}", primary_addr.port()))
            .await
            .expect("bind companion shutdown fixture");
        let companion_addr = companion.local_addr().expect("read companion address");
        let sse_state = ShutdownSseState {
            control_dir: control_dir.to_path_buf(),
            hold: matches!(mode, "held-sse"),
        };
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .route("/sse", get(shutdown_test_sse))
            .with_state(sse_state);
        let cleanup: Option<ShutdownCleanup> = if mode == "normal-cleanup" {
            let cleanup_marker = control_dir.join("registered-cleanup-complete");
            Some(Arc::new(move || {
                std::fs::write(&cleanup_marker, b"complete")
                    .expect("publish registered cleanup completion");
            }))
        } else if matches!(
            mode,
            "held-cleanup" | "held-cleanup-mcp" | "stderr-lock" | "stderr-backpressure"
        ) {
            Some(Arc::new(|| {
                loop {
                    std::thread::park_timeout(Duration::from_secs(60));
                }
            }))
        } else {
            None
        };
        let async_cleanup: Option<ShutdownAsyncCleanup> = if mode == "held-cleanup" {
            let cleanup_marker = control_dir.join("async-cleanup-complete");
            Some(Arc::new(move || {
                let cleanup_marker = cleanup_marker.clone();
                Box::pin(async move {
                    std::fs::write(cleanup_marker, b"complete")
                        .expect("publish async cleanup completion");
                })
            }))
        } else if mode == "held-cleanup-mcp" {
            let fixture_path = control_dir.join("mcp-shutdown-fixture.py");
            let eof_marker = control_dir.join("mcp-stdin-closed");
            std::fs::write(&fixture_path, SHUTDOWN_MCP_FIXTURE)
                .expect("write shutdown MCP fixture");
            let config = crate::mcp::config::McpConfig {
                mcp_servers: HashMap::from([(
                    "shutdown-fixture".to_string(),
                    crate::mcp::config::McpServerEntry::Stdio {
                        command: "python3".to_string(),
                        args: vec![
                            fixture_path.display().to_string(),
                            eof_marker.display().to_string(),
                        ],
                        env: HashMap::new(),
                        sandboxed: false,
                    },
                )]),
            };
            let registry = Arc::new(
                McpRegistry::from_config(&config)
                    .await
                    .expect("connect shutdown MCP fixture"),
            );
            Some(Arc::new(move || {
                let registry = Arc::clone(&registry);
                Box::pin(async move { registry.shutdown().await })
                    as Pin<Box<dyn Future<Output = ()> + Send + 'static>>
            }))
        } else {
            None
        };

        if mode == "stderr-lock" {
            let (locked_tx, locked_rx) = std::sync::mpsc::sync_channel(0);
            std::thread::spawn(move || {
                let _stderr = std::io::stderr().lock();
                locked_tx.send(()).expect("publish stderr lock");
                loop {
                    std::thread::park_timeout(Duration::from_secs(60));
                }
            });
            locked_rx.recv().expect("wait for stderr lock");
        } else if mode == "stderr-backpressure" {
            std::thread::spawn(|| {
                let mut stderr = std::io::stderr().lock();
                let block = [b'x'; 4096];
                loop {
                    if stderr.write_all(&block).is_err() {
                        return;
                    }
                }
            });
        }

        let ready_path = control_dir.join("ready.json");
        let ready_pending_path = control_dir.join("ready.json.pending");
        std::fs::write(
            &ready_pending_path,
            serde_json::to_vec(&json!({
                "primary": primary_addr,
                "companion": companion_addr,
            }))
            .expect("serialize shutdown readiness"),
        )
        .expect("write pending shutdown readiness");
        std::fs::rename(ready_pending_path, ready_path).expect("publish shutdown readiness");

        let coordinator = ShutdownCoordinator::new();
        serve_on_listener(
            primary,
            Some(companion),
            app,
            timeout_secs,
            cleanup,
            async_cleanup,
            tokio_util::sync::CancellationToken::new(),
            None,
            None,
            coordinator.clone(),
        )
        .await
        .expect("serve shutdown fixture");
        coordinator.wait_for_cleanup().await;
        coordinator.complete();
    }

    #[test]
    fn shutdown_process_child() {
        let Some(mode) = std::env::var_os(SHUTDOWN_CHILD_ENV) else {
            return;
        };
        let control_dir = PathBuf::from(
            std::env::var_os("UAR_SHUTDOWN_TEST_CONTROL_DIR")
                .expect("shutdown child control directory"),
        );
        let timeout_secs = std::env::var("UAR_SHUTDOWN_TEST_TIMEOUT_SECS")
            .expect("shutdown child timeout")
            .parse()
            .expect("parse shutdown child timeout");
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("build shutdown child runtime");
        runtime.block_on(run_shutdown_process_child(
            &mode.to_string_lossy(),
            &control_dir,
            timeout_secs,
        ));
    }

    fn read_sse(addr: SocketAddr) -> std::thread::JoinHandle<String> {
        std::thread::spawn(move || {
            let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))
                .expect("connect SSE client");
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .expect("set SSE read timeout");
            write!(
                stream,
                "GET /sse HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
            )
            .expect("write SSE request");
            let mut response = String::new();
            let _ = stream.read_to_string(&mut response);
            response
        })
    }

    fn listener_refuses_within(addr: SocketAddr, limit: Duration) -> bool {
        let deadline = Instant::now() + limit;
        loop {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_err() {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn shutdown_process_idle_sigterm_and_sigint_exit_within_one_second() {
        for signal in ["TERM", "INT"] {
            let mut child = ShutdownChild::spawn("idle", 2);
            let started = Instant::now();
            child.signal(signal);
            let exit = child
                .wait_for_exit(Duration::from_secs(1))
                .unwrap_or_else(|| panic!("idle {signal} did not exit within one second"));
            assert!(
                exit.status.success(),
                "idle {signal} status: {}",
                exit.status
            );
            assert!(
                exit.stderr
                    .contains("UAR_SHUTDOWN outcome=graceful_complete"),
                "idle {signal} missing graceful marker: {}",
                exit.stderr
            );
            assert!(!exit.stderr.contains("deadline_enforced"));
            assert!(started.elapsed() < Duration::from_secs(1));
        }
    }

    #[test]
    fn shutdown_process_real_sse_completes_and_both_listeners_refuse() {
        let mut child = ShutdownChild::spawn("active-sse", 2);
        let reader = read_sse(child.ready.primary);
        child.wait_for_file("stream-started");
        child.signal("TERM");
        assert!(
            listener_refuses_within(child.ready.primary, Duration::from_millis(500)),
            "primary listener accepted connections after SIGTERM"
        );
        assert!(
            listener_refuses_within(child.ready.companion, Duration::from_millis(500)),
            "companion listener accepted connections after SIGTERM"
        );
        let exit = child
            .wait_for_exit(Duration::from_secs(3))
            .expect("active SSE child did not exit inside its graceful window");
        let response = reader.join().expect("join SSE reader");
        assert!(response.contains("content-type: text/event-stream"));
        assert!(response.contains("data: started"));
        assert!(response.contains("data: done"));
        assert!(exit.status.success());
        assert!(
            exit.stderr
                .contains("UAR_SHUTDOWN outcome=graceful_complete")
        );
        assert!(!exit.stderr.contains("deadline_enforced"));
    }

    #[test]
    fn shutdown_process_held_sse_exits_at_deadline() {
        let mut child = ShutdownChild::spawn("held-sse", 1);
        let reader = read_sse(child.ready.primary);
        child.wait_for_file("stream-started");
        let started = Instant::now();
        child.signal("TERM");
        let exit = child
            .wait_for_exit(Duration::from_millis(1900))
            .expect("held SSE child did not enforce its deadline");
        let response = reader.join().expect("join held SSE reader");
        assert!(response.contains("data: started"));
        assert!(exit.status.success());
        assert!(started.elapsed() >= Duration::from_millis(900));
        assert!(started.elapsed() < Duration::from_millis(1900));
        assert!(
            exit.stderr
                .contains("UAR_SHUTDOWN outcome=deadline_enforced")
        );
        assert!(!exit.stderr.contains("graceful_complete"));
        assert!(!exit.stderr.contains("cleanup_complete"));
    }

    #[test]
    fn shutdown_process_held_registered_cleanup_exits_at_deadline() {
        let mut child = ShutdownChild::spawn("held-cleanup", 1);
        let started = Instant::now();
        child.signal("TERM");
        child.wait_for_file("async-cleanup-complete");
        let exit = child
            .wait_for_exit(Duration::from_millis(1900))
            .expect("held cleanup child did not enforce its deadline");
        assert!(exit.status.success());
        assert!(started.elapsed() >= Duration::from_millis(900));
        assert!(started.elapsed() < Duration::from_millis(1900));
        assert!(
            exit.stderr
                .contains("UAR_SHUTDOWN outcome=deadline_enforced")
        );
        assert!(!exit.stderr.contains("graceful_complete"));
        assert!(!exit.stderr.contains("cleanup_complete"));
    }

    #[test]
    fn shutdown_process_mcp_eof_precedes_held_cleanup_deadline() {
        let mut child = ShutdownChild::spawn("held-cleanup-mcp", 1);
        let started = Instant::now();
        child.signal("TERM");
        child.wait_for_file("mcp-stdin-closed");
        let exit = child
            .wait_for_exit(Duration::from_millis(1900))
            .expect("held cleanup with MCP child did not enforce its deadline");
        assert!(exit.status.success());
        assert!(started.elapsed() >= Duration::from_millis(900));
        assert!(started.elapsed() < Duration::from_millis(1900));
        assert!(
            exit.stderr
                .contains("UAR_SHUTDOWN outcome=deadline_enforced")
        );
        assert!(!exit.stderr.contains("graceful_complete"));
        assert!(!exit.stderr.contains("cleanup_complete"));
    }

    #[test]
    fn shutdown_process_registered_cleanup_precedes_graceful_completion() {
        let mut child = ShutdownChild::spawn("normal-cleanup", 2);
        child.signal("TERM");
        let exit = child
            .wait_for_exit(Duration::from_secs(1))
            .expect("normal registered cleanup did not finish within one second");
        assert!(exit.status.success());
        assert!(
            child
                .control_dir
                .path()
                .join("registered-cleanup-complete")
                .exists(),
            "normal process completion preceded registered cleanup"
        );
        assert!(
            exit.stderr
                .contains("UAR_SHUTDOWN outcome=graceful_complete")
        );
        assert!(!exit.stderr.contains("deadline_enforced"));
    }

    #[test]
    fn shutdown_process_stderr_lock_does_not_block_deadline() {
        let mut child = ShutdownChild::spawn("stderr-lock", 1);
        let started = Instant::now();
        child.signal("TERM");
        let exit = child
            .wait_for_exit(Duration::from_millis(1900))
            .expect("ordinary stderr lock blocked forced exit");
        assert!(exit.status.success());
        assert!(started.elapsed() < Duration::from_millis(1900));
        assert!(
            exit.stderr
                .contains("UAR_SHUTDOWN outcome=deadline_enforced")
        );
        assert!(!exit.stderr.contains("graceful_complete"));
    }

    #[test]
    fn shutdown_process_stderr_backpressure_does_not_block_deadline() {
        let mut child = ShutdownChild::spawn("stderr-backpressure", 1);
        std::thread::sleep(Duration::from_millis(200));
        let started = Instant::now();
        child.signal("TERM");
        let exit = child
            .wait_for_exit(Duration::from_millis(1900))
            .expect("backpressured stderr blocked forced exit");
        assert!(exit.status.success());
        assert!(started.elapsed() < Duration::from_millis(1900));
        assert!(!exit.stderr.contains("graceful_complete"));
    }

    #[test]
    fn ensure_toolu_id_prefixes_non_prefixed_ids() {
        assert_eq!(ensure_toolu_id("abc"), "toolu_abc");
        assert_eq!(ensure_toolu_id("toolu_xyz"), "toolu_xyz");
    }

    #[tokio::test]
    async fn security_txt_has_the_required_rfc9116_fields() {
        let (headers, body) = security_txt_handler().await;
        assert_eq!(headers[0].1, "text/plain; charset=utf-8");
        assert!(body.contains("Contact: https://"));
        assert!(body.contains("Expires: "));
        // A real reporting channel, not a fabricated email/PGP key.
        assert!(body.contains("security/advisories/new"));
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
            session_id: None,
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

    #[test]
    fn provider_anthropic_usage_preserves_reported_cache_fields_exactly() {
        let usage = provider_anthropic_usage(41, 7, Some(29), Some(13));
        assert_eq!(usage.input_tokens, 41);
        assert_eq!(usage.output_tokens, 7);
        assert_eq!(usage.cache_creation_input_tokens, 13);
        assert_eq!(usage.cache_read_input_tokens, 29);
    }

    #[test]
    fn absent_provider_usage_never_fabricates_cache_activity() {
        let usage = finalized_anthropic_usage(None, 9);
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 9);
        assert_eq!(usage.cache_creation_input_tokens, 0);
        assert_eq!(usage.cache_read_input_tokens, 0);
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
