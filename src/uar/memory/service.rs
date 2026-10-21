//! `MemoryService` — the primary facade over `SurrealStorage`.
//!
//! Wraps the surreal-memory `MemoryStorage` trait implementation with UAR-level
//! scope enforcement, error handling, and logging. All callers should use this
//! struct rather than accessing `SurrealStorage` directly.

use anyhow::{Context, Result};
use futures::{FutureExt, Stream};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::panic::AssertUnwindSafe;
use std::task::{Context as TaskContext, Poll};
use surrealdb::{Surreal, engine::any::Any};
use surrealdb::engine::any;
use surrealdb::opt::auth::Root;
use surreal_memory::{
    EmbeddingProvider, EmbeddingService, Memory, MemoryHistory, MemoryScope, MemoryType,
    SurrealStorage,
    entity::{Entity, KnowledgeGraph, Relation, SemanticSearchResult},
    mindmap::{MindMap, MindMapEdge, MindMapNode},
    storage::{
        MemoryStorage,
        surreal::{SurrealConfig, SurrealLifecycleMode, SurrealMode},
    },
    task_stream::{ContextWindow, TaskStream},
};

use crate::config::MemoryConfig;
use crate::uar::persistence::providers::surreal::SurrealEndpoint;
use crate::uar::runtime::lifecycle::{OperationLease, ResourceOwnership, ResourceScope};
use crate::uar::security::claims::{CredentialPrincipal, UserContext};
use super::owned::OwnedMemory;
use super::scoped_storage::ScopedMemoryStorage;

use super::workflow_mirror::{
    WorkflowMirrorCandidate, WorkflowMirrorRecord, workflow_candidate_from_memory,
};

/// Facade over `SurrealStorage` with scope-aware helpers.
#[derive(Clone)]
pub struct MemoryService {
    pub(crate) storage: Arc<dyn MemoryStorage>,
    surreal_storage: Arc<SurrealStorage>,
    pub(crate) config: MemoryConfig,
    pub(crate) embeddings: Arc<dyn EmbeddingService>,
    ownership: MemoryStorageOwnership,
    owned_client: Option<Surreal<Any>>,
    scope_open: Arc<AtomicBool>,
    lifecycle_scope: ResourceScope,
    close_started: Arc<AtomicBool>,
    close_result: tokio::sync::watch::Sender<Option<MemoryCloseReport>>,
    memory_changes: tokio::sync::watch::Sender<u64>,
}

/// Who owns the database engine behind this memory facade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryStorageOwnership {
    UarOwned,
    HostBorrowed,
}

/// The acknowledged result of closing memory's local scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryCloseReport {
    pub ownership: MemoryStorageOwnership,
    pub scope_released: bool,
    pub engine_close_acknowledged: bool,
    pub failure: Option<String>,
}

/// An admitted, table-scoped client for the memory database's lifecycle-owned
/// realtime source. It deliberately does not expose the raw database handle.
/// The caller owns every stream it obtains and must close each stream before
/// the memory scope is released.
#[derive(Clone)]
pub struct MemorySubscriptionClient {
    db: Surreal<Any>,
    scope_open: Arc<AtomicBool>,
    lifecycle_scope: ResourceScope,
}

/// A scope-accounted memory live query.
///
/// The accepted operation remains registered until `close` has received the
/// SDK's command acknowledgement. Dropping the stream transfers that close to
/// a retained runtime task; it never releases the parent lease merely because
/// the caller stopped observing notifications.
pub struct MemoryLiveStream {
    stream: Option<surrealdb::method::Stream<Vec<surrealdb::types::Value>>>,
    lease: Option<OperationLease>,
}

struct PendingMemoryLiveClose {
    operation_id: String,
    stream: Option<surrealdb::method::Stream<Vec<surrealdb::types::Value>>>,
    lease: Option<OperationLease>,
    /// A canceled caller transfers closing work once. A retained worker that
    /// is itself aborted records the obligation instead of retrying teardown.
    rescue_on_drop: bool,
}

impl PendingMemoryLiveClose {
    fn new(
        stream: surrealdb::method::Stream<Vec<surrealdb::types::Value>>,
        lease: OperationLease,
    ) -> Self {
        Self {
            operation_id: lease.id().to_owned(),
            stream: Some(stream),
            lease: Some(lease),
            rescue_on_drop: true,
        }
    }

    async fn close(mut self) -> Result<()> {
        let result = match self.stream.as_mut() {
            Some(stream) => stream.close().await.map_err(anyhow::Error::from),
            None => Ok(()),
        };
        // Keep the stream inside this guard until the awaited command resolves.
        // If this future is canceled during that await, Drop still owns both
        // the stream and lease and transfers them to the retained worker.
        drop(self.stream.take());
        self.finish(result)
    }

    async fn close_in_retained_worker(mut self) {
        self.rescue_on_drop = false;
        let _ = self.close().await;
    }

    fn finish(mut self, result: Result<()>) -> Result<()> {
        let Some(lease) = self.lease.take() else {
            return result;
        };
        match result {
            Ok(()) => {
                drop(lease);
                Ok(())
            }
            Err(error) => {
                lease.unresolved(format!(
                    "{}: memory live subscription closure outcome unknown: {error}",
                    self.operation_id
                ));
                Err(error)
            }
        }
    }

    fn retain_or_record(self) {
        match tokio::runtime::Handle::try_current() {
            Ok(runtime) => {
                runtime.spawn(async move {
                    self.close_in_retained_worker().await;
                });
            }
            Err(_) => self.unresolved_without_runtime(),
        }
    }

    fn unresolved_without_runtime(mut self) {
        if let Some(lease) = self.lease.take() {
            lease.unresolved(format!(
                "{}: memory live subscription close had no runtime for acknowledgement",
                self.operation_id
            ));
        }
    }
}

impl Drop for PendingMemoryLiveClose {
    fn drop(&mut self) {
        match (self.stream.take(), self.lease.take()) {
            (Some(stream), Some(lease)) => {
                let operation_id = self.operation_id.clone();
                if self.rescue_on_drop {
                    match tokio::runtime::Handle::try_current() {
                        Ok(runtime) => {
                            runtime.spawn(async move {
                                PendingMemoryLiveClose {
                                    operation_id,
                                    stream: Some(stream),
                                    lease: Some(lease),
                                    rescue_on_drop: false,
                                }
                                .close_in_retained_worker()
                                .await;
                            });
                        }
                        Err(_) => {
                            drop(stream);
                            lease.unresolved(format!(
                                "{operation_id}: memory live subscription close was canceled without a runtime"
                            ));
                        }
                    }
                } else {
                    drop(stream);
                    lease.unresolved(format!(
                        "{operation_id}: retained memory live subscription close task ended before acknowledgement"
                    ));
                }
            }
            (Some(stream), None) => drop(stream),
            (None, Some(lease)) => lease.unresolved(format!(
                "{}: memory live subscription close ended without a stream acknowledgement",
                self.operation_id
            )),
            (None, None) => {}
        }
    }
}

impl MemoryLiveStream {
    fn new(
        stream: surrealdb::method::Stream<Vec<surrealdb::types::Value>>,
        lease: OperationLease,
    ) -> Self {
        Self {
            stream: Some(stream),
            lease: Some(lease),
        }
    }

    /// Close the original live-query registration and retain an explicit
    /// unresolved obligation if the SDK cannot acknowledge that command.
    /// Cancellation transfers the same stream and lease to a retained worker.
    pub async fn close(&mut self) -> Result<()> {
        let Some(stream) = self.stream.take() else {
            return Ok(());
        };
        let Some(lease) = self.lease.take() else {
            drop(stream);
            return Ok(());
        };
        PendingMemoryLiveClose::new(stream, lease).close().await
    }
}

impl Stream for MemoryLiveStream {
    type Item = <surrealdb::method::Stream<Vec<surrealdb::types::Value>> as Stream>::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        context: &mut TaskContext<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.stream.as_mut() {
            Some(stream) => Pin::new(stream).poll_next(context),
            None => Poll::Ready(None),
        }
    }
}

impl Drop for MemoryLiveStream {
    fn drop(&mut self) {
        match (self.stream.take(), self.lease.take()) {
            (Some(stream), Some(lease)) => PendingMemoryLiveClose::new(stream, lease).retain_or_record(),
            (Some(stream), None) => drop(stream),
            (None, Some(lease)) => lease.unresolved(
                "memory live subscription stream was absent before closure acknowledgement",
            ),
            (None, None) => {}
        }
    }
}

impl MemorySubscriptionClient {
    /// Register a live stream for the memory table in the memory store.
    pub async fn subscribe_memory(&self) -> Result<MemoryLiveStream> {
        anyhow::ensure!(
            self.scope_open.load(Ordering::Acquire),
            "Memory lifecycle scope is closed"
        );
        let operation = self
            .lifecycle_scope
            .admit_named("memory_live_subscription")
            .map_err(anyhow::Error::from)?;
        let db = self.db.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let runtime = match tokio::runtime::Handle::try_current() {
            Ok(runtime) => runtime,
            Err(_) => {
                let operation_id = operation.id().to_owned();
                operation.unresolved(format!(
                    "{operation_id}: memory live subscription had no runtime for retained registration"
                ));
                anyhow::bail!("memory live subscription requires an active Tokio runtime");
            }
        };
        runtime.spawn(async move {
            let registered = AssertUnwindSafe(async {
                db.select::<Vec<surrealdb::types::Value>>("memory")
                    .live()
                    .await
                    .map_err(anyhow::Error::from)
            })
            .catch_unwind()
            .await;
            let result = match registered {
                Ok(Ok(stream)) => Ok(MemoryLiveStream::new(stream, operation)),
                Ok(Err(error)) => {
                    let operation_id = operation.id().to_owned();
                    operation.unresolved(format!(
                        "{operation_id}: memory live subscription registration outcome unknown: {error}"
                    ));
                    Err(error.context("memory live subscription registration failed"))
                }
                Err(_) => {
                    let operation_id = operation.id().to_owned();
                    operation.unresolved(format!(
                        "{operation_id}: memory live subscription registration panicked without a closure receipt"
                    ));
                    Err(anyhow::anyhow!("memory live subscription registration panicked"))
                }
            };
            if let Err(result) = sender.send(result) {
                if let Ok(stream) = result {
                    drop(stream);
                }
            }
        });
        receiver.await.map_err(|_| anyhow::anyhow!(
            "memory live subscription registration task ended before reporting its closure responsibility"
        ))?
    }

    /// Observe only this client's local transport transitions. A caller uses a
    /// loss event as an explicit lifecycle signal; it does not retry itself.
    pub fn subscribe_transport_state(
        &self,
    ) -> Result<tokio::sync::watch::Receiver<surrealdb::TransportState>> {
        anyhow::ensure!(
            self.scope_open.load(Ordering::Acquire),
            "Memory lifecycle scope is closed"
        );
        self.db
            .subscribe_transport_state()
            .map_err(anyhow::Error::from)
    }

    /// Observe the retained SDK generation after normal memory admission has
    /// closed. This is an observation-only control capability; it neither
    /// reconnects nor grants another database operation.
    pub fn transport_generation(&self) -> Result<u64> {
        self.db
            .transport_generation()
            .map_err(anyhow::Error::from)
    }
}

impl std::fmt::Debug for MemoryService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryService").finish_non_exhaustive()
    }
}

impl MemoryService {
    /// Initialise the service by connecting to (or creating) the embedded SurrealDB store.
    pub async fn new(config: MemoryConfig) -> Result<Self> {
        tracing::info!(
            db_path = %config.db_path,
            embedding_provider = %config.embedding_provider,
            "Initialising agent memory service"
        );

        // Build the embedding provider.
        //
        // Precedence:
        //   memory.embedding_api (set via YAML or UAR_MEMORY__EMBEDDING_API) — preferred
        //   memory.embedding_provider — legacy discriminator (used as fallback)
        //
        // The resolved adapter drives the URL scheme, the auth-header convention, and
        // the request shape. Unrecognized values produce a hard error rather than
        // silently selecting a default (the previous `_ =>` arm silently fell back
        // to `local`, which masked operator intent — see issue #420).
        let resolved_api = config
            .embedding_api
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| config.embedding_provider.clone());

        let embedding_svc: Arc<dyn EmbeddingService> = match resolved_api.as_str() {
            "openai" => {
                let api_key = config
                    .embedding_api_key
                    .clone()
                    .or_else(|| config.openai_api_key.clone())
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .or_else(|| std::env::var("LLM_API_KEY").ok())
                    .context("Memory: embedding_api=openai requires OPENAI_API_KEY, LLM_API_KEY, memory.embedding_api_key, or memory.openai_api_key config")?;
                let provider = EmbeddingProvider::OpenAI {
                    api_key,
                    model: config.embedding_model.clone(),
                    base_url: config.embedding_base_url.clone().unwrap_or_default(),
                    dimensions: config.embedding_dim.map(|n| n as usize),
                };
                Arc::from(surreal_memory::embeddings::create_embedding_service(provider).await?)
            }
            "anthropic" => {
                let api_key = config
                    .embedding_api_key
                    .clone()
                    .or_else(|| config.anthropic_api_key.clone())
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .context("Memory: embedding_api=anthropic requires ANTHROPIC_API_KEY, memory.embedding_api_key, or memory.anthropic_api_key config")?;
                let provider = EmbeddingProvider::Anthropic {
                    api_key,
                    model: config.embedding_model.clone(),
                    base_url: config.embedding_base_url.clone().unwrap_or_default(),
                    dimensions: config.embedding_dim.map(|n| n as usize),
                };
                Arc::from(surreal_memory::embeddings::create_embedding_service(provider).await?)
            }
            "mistral" => {
                let api_key = config
                    .embedding_api_key
                    .clone()
                    .or_else(|| config.mistral_api_key.clone())
                    .or_else(|| std::env::var("MISTRAL_API_KEY").ok())
                    .context("Memory: embedding_api=mistral requires MISTRAL_API_KEY, memory.embedding_api_key, or memory.mistral_api_key config")?;
                let provider = EmbeddingProvider::Mistral {
                    api_key,
                    model: config.embedding_model.clone(),
                    base_url: config.embedding_base_url.clone().unwrap_or_default(),
                    dimensions: config.embedding_dim.map(|n| n as usize),
                };
                Arc::from(surreal_memory::embeddings::create_embedding_service(provider).await?)
            }
            "cohere" => {
                let api_key = config
                    .embedding_api_key
                    .clone()
                    .or_else(|| config.cohere_api_key.clone())
                    .or_else(|| std::env::var("COHERE_API_KEY").ok())
                    .context("Memory: embedding_api=cohere requires COHERE_API_KEY, memory.embedding_api_key, or memory.cohere_api_key config")?;
                let provider = EmbeddingProvider::Cohere {
                    api_key,
                    model: config.embedding_model.clone(),
                };
                Arc::from(surreal_memory::embeddings::create_embedding_service(provider).await?)
            }
            "local" => {
                let provider = EmbeddingProvider::Local {
                    model_id: config.embedding_model.clone(),
                    model_path: None,
                };
                Arc::from(surreal_memory::embeddings::create_embedding_service(provider).await?)
            }
            other => {
                anyhow::bail!(
                    "Memory: embedding_api={other:?} is not supported. \
                     Supported values are: openai | anthropic | mistral | cohere | local. \
                     Set memory.embedding_api (or env UAR_MEMORY__EMBEDDING_API) explicitly."
                );
            }
        };

        tracing::info!(
            embedding_api = %resolved_api,
            embedding_model = %config.embedding_model,
            dimensions = embedding_svc.dimensions(),
            "Memory embedding service ready"
        );

        // UAR owns acquisition so the memory adapter stays in host-managed
        // mode: it neither retries nor declares outcomes from local deadlines.
        let configured_endpoint = config
            .surreal_endpoint
            .clone()
            .unwrap_or_else(|| format!("surrealkv://{}", config.db_path));
        let endpoint = SurrealEndpoint::parse(&configured_endpoint);
        let db = any::connect(endpoint.connection_string())
            .await
            .context("Failed to acquire SurrealDB for memory")?;
        let database = format!("{}__owner_v1", config.database);
        Self::initialize_owned_retained(config, embedding_svc, endpoint, db, database).await
    }

    /// Keep an acquired memory engine owned by a detached initializer until it
    /// is delivered to host lifecycle registration or its SDK shutdown
    /// acknowledgement completes. A canceled constructor waiter cannot strand
    /// an initialized embedded store between those states.
    async fn initialize_owned_retained(
        config: MemoryConfig,
        embeddings: Arc<dyn EmbeddingService>,
        endpoint: SurrealEndpoint,
        db: Surreal<Any>,
        database: String,
    ) -> Result<Self> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let initialized = AssertUnwindSafe(Self::build_owned_service(
                config,
                embeddings,
                endpoint,
                db.clone(),
                database,
            ))
            .catch_unwind()
            .await;
            let result = match initialized {
                Ok(Ok(service)) => Ok(service),
                Ok(Err(error)) => Err(shutdown_failed_owned_memory_client(&db, error).await),
                Err(_) => Err(
                    shutdown_failed_owned_memory_client(
                        &db,
                        anyhow::anyhow!(
                            "memory service initialization panicked after acquiring its database client"
                        ),
                    )
                    .await,
                ),
            };

            if let Err(result) = sender.send(result) {
                match result {
                    Ok(service) => close_undelivered_memory_service(service).await,
                    Err(error) => tracing::error!(
                        error = %error,
                        "memory service initialization failed after its waiter was canceled"
                    ),
                }
            }
        });

        receiver.await.map_err(|_| {
            anyhow::anyhow!(
                "memory service initializer ended before reporting whether its owned client was released"
            )
        })?
    }

    async fn build_owned_service(
        config: MemoryConfig,
        embeddings: Arc<dyn EmbeddingService>,
        endpoint: SurrealEndpoint,
        db: Surreal<Any>,
        database: String,
    ) -> Result<Self> {
        if endpoint.is_remote() {
            db.signin(Root {
                username: config.surreal_user.clone().unwrap_or_else(|| "root".to_string()),
                password: config.surreal_pass.clone().unwrap_or_else(|| "root".to_string()),
            })
            .await?;
        }
        db.use_ns(&config.namespace).use_db(&database).await?;
        let surreal_config = SurrealConfig {
            mode: SurrealMode::Server,
            embedded_path: None,
            endpoint: Some(endpoint.connection_string()),
            username: None,
            password: None,
            namespace: config.namespace.clone(),
            database,
            lifecycle_mode: SurrealLifecycleMode::HostManaged,
            ..Default::default()
        };
        let storage = Arc::new(
            SurrealStorage::from_host_connection(
            &surreal_config,
            db.clone(),
            Arc::clone(&embeddings),
        )
        .await
        .context("Failed to initialise SurrealDB for memory")?,
        );

        let lifecycle_scope = ResourceScope::standalone("memory-storage", "memory-service");
        let unscoped_storage: Arc<dyn MemoryStorage> = storage.clone();
        let (close_result, _) = tokio::sync::watch::channel(None);
        let service = Self {
            storage: Arc::new(ScopedMemoryStorage::new(unscoped_storage, lifecycle_scope.clone())),
            surreal_storage: storage,
            config,
            embeddings,
            ownership: MemoryStorageOwnership::UarOwned,
            owned_client: Some(db),
            scope_open: Arc::new(AtomicBool::new(true)),
            lifecycle_scope,
            close_started: Arc::new(AtomicBool::new(false)),
            close_result,
            memory_changes: tokio::sync::watch::channel(0_u64).0,
        };
        service.initialize_controls().await?;
        Ok(service)
    }

    /// Build a UAR memory facade around a database client retained by its host.
    ///
    /// The caller supplies an already authenticated, namespace-selected client
    /// and owns its reconnection and final engine close. Closing this facade
    /// prevents further memory access without invalidating the host client.
    pub async fn from_host_connection(
        config: MemoryConfig,
        db: Surreal<Any>,
        embeddings: Arc<dyn EmbeddingService>,
    ) -> Result<Self> {
        Self::initialize_borrowed_retained(config, db, embeddings).await
    }

    /// Initialize a borrowed facade independently of the caller's wait. Its
    /// cleanup closes only the facade's host-managed storage scope, never the
    /// database engine retained by the embedding host.
    async fn initialize_borrowed_retained(
        config: MemoryConfig,
        db: Surreal<Any>,
        embeddings: Arc<dyn EmbeddingService>,
    ) -> Result<Self> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let initialized = AssertUnwindSafe(Self::build_borrowed_service(config, db, embeddings))
                .catch_unwind()
                .await;
            let result = match initialized {
                Ok(result) => result,
                Err(_) => Err(anyhow::anyhow!(
                    "host-managed memory facade initialization panicked after receiving its client"
                )),
            };

            if let Err(result) = sender.send(result) {
                match result {
                    Ok(service) => close_undelivered_memory_service(service).await,
                    Err(error) => tracing::error!(
                        error = %error,
                        "host-managed memory facade initialization failed after its waiter was canceled"
                    ),
                }
            }
        });

        receiver.await.map_err(|_| {
            anyhow::anyhow!(
                "host-managed memory facade initializer ended before reporting its local scope result"
            )
        })?
    }

    async fn build_borrowed_service(
        config: MemoryConfig,
        db: Surreal<Any>,
        embeddings: Arc<dyn EmbeddingService>,
    ) -> Result<Self> {
        let surreal_config = SurrealConfig {
            mode: SurrealMode::Server,
            embedded_path: None,
            endpoint: config.surreal_endpoint.clone(),
            username: None,
            password: None,
            namespace: config.namespace.clone(),
            database: format!("{}__owner_v1", config.database),
            lifecycle_mode: SurrealLifecycleMode::HostManaged,
            ..Default::default()
        };
        let storage = Arc::new(
            SurrealStorage::from_host_connection(&surreal_config, db, Arc::clone(&embeddings))
                .await
                .context("Failed to initialise host-managed SurrealDB memory")?,
        );
        let lifecycle_scope = ResourceScope::standalone("borrowed-memory-storage", "memory-service");
        lifecycle_scope.set_ownership(ResourceOwnership::Borrowed)?;
        let unscoped_storage: Arc<dyn MemoryStorage> = storage.clone();
        let (close_result, _) = tokio::sync::watch::channel(None);
        let service = Self {
            storage: Arc::new(ScopedMemoryStorage::new(unscoped_storage, lifecycle_scope.clone())),
            surreal_storage: storage,
            config,
            embeddings,
            ownership: MemoryStorageOwnership::HostBorrowed,
            owned_client: None,
            scope_open: Arc::new(AtomicBool::new(true)),
            lifecycle_scope,
            close_started: Arc::new(AtomicBool::new(false)),
            close_result,
            memory_changes: tokio::sync::watch::channel(0_u64).0,
        };
        service.initialize_controls().await?;
        Ok(service)
    }

    /// Create an independently admitted facade that borrows this service's
    /// database client. Releasing it never closes the source service or engine.
    pub async fn borrowed_scope(&self) -> Result<Self> {
        self.ensure_scope_open()?;
        let _operation = self.admit_operation("borrowed_scope")?;
        let db = self.surreal_storage.db()
            .context("Memory borrowed scope requires the Surreal storage adapter")?;
        Self::from_host_connection(self.config.clone(), db, Arc::clone(&self.embeddings)).await
    }

    pub fn ownership(&self) -> MemoryStorageOwnership { self.ownership }

    /// The local admission scope for work performed through this facade.
    pub(crate) fn lifecycle_scope(&self) -> ResourceScope {
        self.lifecycle_scope.clone()
    }

    /// Request exactly one authenticated reconnect of this facade's retained
    /// SDK router. The caller supplies the observed generation so a stale
    /// controller cannot replace a newer transport.
    pub(crate) async fn reconnect(&self, expected_generation: u64) -> Result<u64> {
        self.ensure_scope_open()?;
        let _operation = self.admit_operation("memory_transport_reconnect")?;
        self.memory_db()?
            .reconnect(expected_generation)
            .await
            .context("memory transport reconnect failed")
    }

    /// Read the SDK-retained transport generation without opening admission.
    /// The trusted host uses this for compare-and-swap reconnect control.
    pub(crate) fn transport_generation(&self) -> Result<u64> {
        self.memory_db()?
            .transport_generation()
            .map_err(anyhow::Error::from)
            .context("memory transport generation unavailable")
    }

    pub(super) fn memory_db(&self) -> Result<Surreal<Any>> {
        self.ensure_scope_open()?;
        self.surreal_storage
            .db()
            .context("Memory lifecycle scope has no live Surreal storage client")
    }

    pub(super) fn raw_surreal_storage(&self) -> Result<Arc<SurrealStorage>> {
        self.ensure_scope_open()?;
        Ok(Arc::clone(&self.surreal_storage))
    }

    /// Committed-memory wakeup; consumers re-read the durable due time.
    pub fn subscribe_committed_memory_changes(&self) -> tokio::sync::watch::Receiver<u64> {
        self.memory_changes.subscribe()
    }

    pub(crate) fn notify_committed_memory_change(&self) {
        self.memory_changes.send_modify(|generation| {
            *generation = generation.saturating_add(1);
        });
    }

    /// Create a scoped subscription client for the separately owned memory
    /// database. This client is admission-checked and table-scoped so callers
    /// cannot retain a general-purpose raw handle past memory shutdown.
    pub fn subscription_client(&self) -> Result<MemorySubscriptionClient> {
        self.ensure_scope_open()?;
        let db = self
            .surreal_storage
            .db()
            .context("Memory realtime subscriptions require the Surreal storage adapter")?;
        Ok(MemorySubscriptionClient {
            db,
            scope_open: Arc::clone(&self.scope_open),
            lifecycle_scope: self.lifecycle_scope.clone(),
        })
    }

    /// Close this facade's admission scope after its dependent work drains.
    ///
    /// The first-party storage adapter calls the SDK close contract only for a
    /// UAR-owned engine. A borrowed client remains usable to its host.
    pub async fn close_scope(&self) -> MemoryCloseReport {
        if self.close_started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.lifecycle_scope.close_admission();
            self.scope_open.store(false, Ordering::Release);
            let lifecycle_scope = self.lifecycle_scope.clone();
            let storage = Arc::clone(&self.surreal_storage);
            let owned_client = self.owned_client.clone();
            let ownership = self.ownership;
            let result = self.close_result.clone();
            tokio::spawn(async move {
                let report = match AssertUnwindSafe(close_memory_resources(
                    lifecycle_scope,
                    storage,
                    owned_client,
                    ownership,
                ))
                .catch_unwind()
                .await
                {
                    Ok(report) => report,
                    Err(_) => MemoryCloseReport {
                        ownership,
                        scope_released: false,
                        engine_close_acknowledged: false,
                        failure: Some("memory lifecycle close task panicked".to_string()),
                    },
                };
                result.send_replace(Some(report));
            });
        }

        let mut result = self.close_result.subscribe();
        loop {
            if let Some(report) = result.borrow().clone() {
                return report;
            }
            if result.changed().await.is_err() {
                return MemoryCloseReport {
                    ownership: self.ownership,
                    scope_released: false,
                    engine_close_acknowledged: false,
                    failure: Some("memory lifecycle close task ended without a completion report".to_string()),
                };
            }
        }
    }

    fn ensure_scope_open(&self) -> Result<()> {
        anyhow::ensure!(
            self.scope_open.load(Ordering::Acquire),
            "Memory lifecycle scope is closed"
        );
        Ok(())
    }

    pub(super) fn admit_operation(&self, name: &str) -> Result<OperationLease> {
        self.lifecycle_scope.admit_named(name).map_err(anyhow::Error::from)
    }

    // ── Scoped memory ─────────────────────────────────────────────────────────

    /// Add a memory inside the complete verified principal namespace.
    pub async fn add(
        &self,
        content: impl Into<String>,
        scope: MemoryScope,
        memory_type: MemoryType,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        categories: Vec<String>,
        metadata: Option<serde_json::Value>,
        importance: f32,
    ) -> Result<Memory> {
        let mut memory = Memory::new(
            content,
            None,
            agent_id.map(String::from),
            session_id.map(String::from),
            categories,
        );
        memory.scope = scope;
        memory.memory_type = memory_type;
        memory.metadata = metadata;
        memory.importance = importance;

        let memory = self.for_context(user_ctx)?
            .add_memory(memory)
            .await
            .context("add_memory failed")?;
        self.notify_committed_memory_change();
        Ok(memory)
    }

    /// Semantic search over scoped memories. Automatically filters by user ownership.
    pub async fn search(
        &self,
        query: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
        categories: Option<&[String]>,
    ) -> Result<Vec<Memory>> {

        self.for_context(user_ctx)?
            .search_memories(query, None, agent_id, session_id, categories, limit)
            .await
            .context("search_memories failed")
    }

    /// Hybrid BM25 + HNSW vector search. Weights are read from config by default.
    pub async fn hybrid_search(
        &self,
        query: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
        vector_weight: Option<f32>,
        bm25_weight: Option<f32>,
    ) -> Result<Vec<Memory>> {
        let vw = vector_weight.unwrap_or(self.config.vector_weight);
        let bw = bm25_weight.unwrap_or(self.config.bm25_weight);

        self.for_context(user_ctx)?
            .hybrid_search_memories(query, None, agent_id, session_id, limit, vw, bw)
            .await
            .context("hybrid_search_memories failed")
    }

    /// Capture conversation records within the verified owner namespace.
    pub async fn auto_capture(
        &self,
        messages: Vec<serde_json::Value>,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<Memory>> {

        let memories = self.for_context(user_ctx)?
            .add_memories_from_conversation(messages, None, agent_id, session_id)
            .await
            .context("add_memories_from_conversation failed")?;
        if !memories.is_empty() { self.notify_committed_memory_change(); }
        Ok(memories)
    }

    /// List all memories for a scope, filtered by caller identity.
    pub async fn list(
        &self,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<Memory>> {

        self.for_context(user_ctx)?
            .get_all_memories(None, agent_id, session_id)
            .await
            .context("get_all_memories failed")
    }

    /// Add a KBD/OpenSpec workflow mirror record through the memory service boundary.
    pub async fn add_workflow_mirror_record(
        &self,
        record: WorkflowMirrorRecord,
        user_ctx: &UserContext,
    ) -> Result<Memory> {
        let write = record.to_memory_write();
        self.add(
            write.content,
            write.scope,
            write.memory_type,
            user_ctx,
            None,
            None,
            write.categories,
            Some(write.metadata),
            write.importance,
        )
        .await
    }

    /// List workflow mirror recovery candidates from scoped memory records.
    pub async fn list_workflow_mirror_candidates(
        &self,
        user_ctx: &UserContext,
    ) -> Result<Vec<WorkflowMirrorCandidate>> {
        let memories = self.list(user_ctx, None, None).await?;
        Ok(memories
            .iter()
            .filter_map(|memory| workflow_candidate_from_memory(memory).ok())
            .collect())
    }

    /// Retrieve a single memory by its record ID string (e.g. `"memory:abc123"`).
    pub async fn get(&self, id: &str, user_ctx: &UserContext) -> Result<Option<Memory>> {
        self.for_context(user_ctx)?
            .get_memory(id)
            .await
            .context("get_memory failed")
    }

    /// Update a memory's content (creates a history record).
    pub async fn update(&self, id: &str, content: String, user_ctx: &UserContext) -> Result<Memory> {
        let memory = self.for_context(user_ctx)?
            .update_memory(id, content)
            .await
            .context("update_memory failed")?;
        self.notify_committed_memory_change();
        Ok(memory)
    }

    /// Delete a single memory only after resolving the complete caller owner.
    pub async fn delete(&self, id: &str, user_ctx: &UserContext) -> Result<()> {
        self.for_context(user_ctx)?
            .delete_memory(id)
            .await
            .context("delete_memory failed")?;
        self.notify_committed_memory_change();
        Ok(())
    }

    /// Delete all memories for a scope filter.
    pub async fn delete_all(
        &self,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<u64> {

        let count = self.for_context(user_ctx)?
            .delete_all_memories(None, agent_id, session_id)
            .await
            .context("delete_all_memories failed")?;
        if count > 0 { self.notify_committed_memory_change(); }
        Ok(count)
    }

    /// Get the audit history for a memory.
    pub async fn history(&self, memory_id: &str, user_ctx: &UserContext) -> Result<Vec<MemoryHistory>> {
        self.for_context(user_ctx)?
            .get_memory_history(memory_id)
            .await
            .context("get_memory_history failed")
    }

    /// Compress memories older than `older_than_days` into a single summary.
    pub async fn compress(
        &self,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        older_than_days: u32,
    ) -> Result<Option<Memory>> {

        let memory = self.for_context(user_ctx)?
            .compress_memories(None, agent_id, session_id, older_than_days)
            .await
            .context("compress_memories failed")?;
        if memory.is_some() {
            self.notify_committed_memory_change();
        }
        Ok(memory)
    }

    /// Earliest future expiry from authoritative durable memory state.
    pub(crate) async fn next_expiry_due_at(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        self.ensure_scope_open()?;
        let _operation = self.admit_operation("next_expiry_due_at")?;
        super::controls::next_expiry_due_at(self).await
    }

    /// Expire stale memories (those past their `valid_until` TTL). Returns expired count.
    pub(crate) async fn expire_stale(&self) -> Result<u64> {
        let _operation = self.admit_operation("expire_stale")?;
        let count = self.resume_memory_purge().await?;
        if count > 0 { self.notify_committed_memory_change(); }
        Ok(count)
    }

    // ── Knowledge Graph ───────────────────────────────────────────────────────

    pub async fn knowledge_graph(&self, user_ctx: &UserContext) -> Result<KnowledgeGraph> {
        let _operation = self.admit_operation("knowledge_graph")?;
        self.admin_storage(user_ctx)?.get_graph().await.context("get_graph failed")
    }

    pub async fn create_entity(&self, entity: Entity, user_ctx: &UserContext) -> Result<Entity> {
        let _operation = self.admit_operation("create_entity")?;
        self.admin_storage(user_ctx)?
            .create_entity(entity)
            .await
            .context("create_entity failed")
    }

    pub async fn create_entities(&self, entities: Vec<Entity>, user_ctx: &UserContext) -> Result<Vec<Entity>> {
        let _operation = self.admit_operation("create_entities")?;
        self.admin_storage(user_ctx)?
            .create_entities(entities)
            .await
            .context("create_entities failed")
    }

    pub async fn get_entity(&self, name: &str, user_ctx: &UserContext) -> Result<Option<Entity>> {
        let _operation = self.admit_operation("get_entity")?;
        self.admin_storage(user_ctx)?
            .get_entity(name)
            .await
            .context("get_entity failed")
    }

    pub async fn search_entities(&self, query: &str, user_ctx: &UserContext) -> Result<Vec<Entity>> {
        let _operation = self.admit_operation("search_entities")?;
        self.admin_storage(user_ctx)?
            .search_entities(query)
            .await
            .context("search_entities failed")
    }

    pub async fn semantic_search_entities(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
        user_ctx: &UserContext,
    ) -> Result<Vec<SemanticSearchResult>> {
        let _operation = self.admit_operation("semantic_search_entities")?;
        self.admin_storage(user_ctx)?
            .semantic_search(query, limit, threshold)
            .await
            .context("semantic_search failed")
    }

    pub async fn add_entity_observations(
        &self,
        name: &str,
        observations: Vec<String>,
        user_ctx: &UserContext,
    ) -> Result<Entity> {
        let _operation = self.admit_operation("add_entity_observations")?;
        self.admin_storage(user_ctx)?
            .add_observations(name, observations)
            .await
            .context("add_observations failed")
    }

    pub async fn delete_entity(&self, name: &str, user_ctx: &UserContext) -> Result<()> {
        let _operation = self.admit_operation("delete_entity")?;
        self.admin_storage(user_ctx)?
            .delete_entity(name)
            .await
            .context("delete_entity failed")
    }

    pub async fn create_relation(&self, relation: Relation, user_ctx: &UserContext) -> Result<Relation> {
        let _operation = self.admit_operation("create_relation")?;
        self.admin_storage(user_ctx)?
            .create_relation(relation)
            .await
            .context("create_relation failed")
    }

    pub async fn create_relations(&self, relations: Vec<Relation>, user_ctx: &UserContext) -> Result<Vec<Relation>> {
        let _operation = self.admit_operation("create_relations")?;
        self.admin_storage(user_ctx)?
            .create_relations(relations)
            .await
            .context("create_relations failed")
    }

    pub async fn get_relations(&self, entity_name: &str, user_ctx: &UserContext) -> Result<Vec<Relation>> {
        let _operation = self.admit_operation("get_relations")?;
        self.admin_storage(user_ctx)?
            .get_relations(entity_name)
            .await
            .context("get_relations failed")
    }

    pub async fn delete_relation(&self, from: &str, to: &str, relation_type: &str, user_ctx: &UserContext) -> Result<()> {
        let _operation = self.admit_operation("delete_relation")?;
        self.admin_storage(user_ctx)?
            .delete_relation(from, to, relation_type)
            .await
            .context("delete_relation failed")
    }

    pub async fn expand_neighbors(
        &self,
        entity_name: &str,
        depth: u8,
        limit: usize,
        user_ctx: &UserContext,
    ) -> Result<KnowledgeGraph> {
        let _operation = self.admit_operation("expand_neighbors")?;
        self.admin_storage(user_ctx)?
            .expand_neighbors(entity_name, depth, limit)
            .await
            .context("expand_neighbors failed")
    }

    pub async fn find_path(&self, from: &str, to: &str, max_depth: u8, user_ctx: &UserContext) -> Result<Vec<Vec<String>>> {
        let _operation = self.admit_operation("find_path")?;
        self.admin_storage(user_ctx)?
            .find_path(from, to, max_depth)
            .await
            .context("find_path failed")
    }

    pub async fn get_related(
        &self,
        entity_name: &str,
        relation_type: Option<&str>,
        direction: &str,
        limit: usize,
        user_ctx: &UserContext,
    ) -> Result<Vec<Entity>> {
        let _operation = self.admit_operation("get_related")?;
        self.admin_storage(user_ctx)?
            .get_related(entity_name, relation_type, direction, limit)
            .await
            .context("get_related failed")
    }

    // ── TaskStreams ───────────────────────────────────────────────────────────

    pub async fn create_task_stream(&self, stream: TaskStream, user_ctx: &UserContext) -> Result<TaskStream> {
        self.for_context(user_ctx)?
            .create_task_stream(stream)
            .await
            .context("create_task_stream failed")
    }

    pub async fn get_task_stream(
        &self,
        name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
    ) -> Result<Option<TaskStream>> {
        self.for_context(user_ctx)?
            .get_task_stream(name, None, agent_id)
            .await
            .context("get_task_stream failed")
    }

    pub async fn add_to_task_stream(
        &self,
        stream_name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        memory: Memory,
    ) -> Result<Memory> {
        let memory = self.for_context(user_ctx)?
            .add_to_task_stream(stream_name, None, agent_id, memory)
            .await
            .context("add_to_task_stream failed")?;
        self.notify_committed_memory_change();
        Ok(memory)
    }

    /// Get a model-aware context window for a TaskStream, respecting the token budget.
    pub async fn task_stream_context(
        &self,
        stream_name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        model_id: &str,
        max_tokens: Option<u64>,
    ) -> Result<ContextWindow> {
        self.for_context(user_ctx)?
            .get_context_for_task(stream_name, None, agent_id, model_id, max_tokens)
            .await
            .context("get_context_for_task failed")
    }

    pub async fn list_task_streams(
        &self,
        agent_id: Option<&str>,
        user_ctx: &UserContext,
    ) -> Result<Vec<TaskStream>> {
        self.for_context(user_ctx)?
            .list_task_streams(agent_id, None)
            .await
            .context("list_task_streams failed")
    }

    pub async fn archive_task_stream(
        &self,
        name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
    ) -> Result<TaskStream> {
        self.for_context(user_ctx)?
            .archive_task_stream(name, None, agent_id)
            .await
            .context("archive_task_stream failed")
    }

    pub async fn auto_summarize_task_stream(
        &self,
        stream_name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        model_id: &str,
    ) -> Result<Option<Memory>> {
        let memory = self.for_context(user_ctx)?
            .auto_summarize_task_stream(stream_name, None, agent_id, model_id)
            .await
            .context("auto_summarize_task_stream failed")?;
        if memory.is_some() {
            self.notify_committed_memory_change();
        }
        Ok(memory)
    }

    // ── Mindmaps ─────────────────────────────────────────────────────────────

    pub async fn create_mindmap(&self, mindmap: MindMap, user_ctx: &UserContext) -> Result<MindMap> {
        let _operation = self.admit_operation("create_mindmap")?;
        self.admin_storage(user_ctx)?
            .create_mindmap(mindmap)
            .await
            .context("create_mindmap failed")
    }

    pub async fn get_mindmap(&self, name: &str, user_id: Option<&str>, user_ctx: &UserContext) -> Result<Option<MindMap>> {
        let _operation = self.admit_operation("get_mindmap")?;
        self.admin_storage(user_ctx)?
            .get_mindmap(name, user_id, None)
            .await
            .context("get_mindmap failed")
    }

    pub async fn list_mindmaps(
        &self,
        user_id: Option<&str>,
        agent_id: Option<&str>,
        user_ctx: &UserContext,
    ) -> Result<Vec<MindMap>> {
        let _operation = self.admit_operation("list_mindmaps")?;
        self.admin_storage(user_ctx)?
            .list_mindmaps(user_id, agent_id)
            .await
            .context("list_mindmaps failed")
    }

    pub async fn add_mindmap_node(
        &self,
        mindmap_name: &str,
        user_id: Option<&str>,
        node: MindMapNode,
        user_ctx: &UserContext,
    ) -> Result<MindMap> {
        let _operation = self.admit_operation("add_mindmap_node")?;
        self.admin_storage(user_ctx)?
            .add_mindmap_node(mindmap_name, user_id, None, node)
            .await
            .context("add_mindmap_node failed")
    }

    pub async fn add_mindmap_edge(
        &self,
        mindmap_name: &str,
        user_id: Option<&str>,
        edge: MindMapEdge,
        user_ctx: &UserContext,
    ) -> Result<MindMap> {
        let _operation = self.admit_operation("add_mindmap_edge")?;
        self.admin_storage(user_ctx)?
            .add_mindmap_edge(mindmap_name, user_id, None, edge)
            .await
            .context("add_mindmap_edge failed")
    }

    pub async fn delete_mindmap_node(
        &self,
        mindmap_name: &str,
        user_id: Option<&str>,
        node_id: &str,
        user_ctx: &UserContext,
    ) -> Result<MindMap> {
        let _operation = self.admit_operation("delete_mindmap_node")?;
        self.admin_storage(user_ctx)?
            .delete_mindmap_node(mindmap_name, user_id, None, node_id)
            .await
            .context("delete_mindmap_node failed")
    }

    pub async fn delete_mindmap(&self, name: &str, user_id: Option<&str>, user_ctx: &UserContext) -> Result<()> {
        let _operation = self.admit_operation("delete_mindmap")?;
        self.admin_storage(user_ctx)?
            .delete_mindmap(name, user_id, None)
            .await
            .context("delete_mindmap failed")
    }

    // ── Storage arc accessor (for MCP server handler) ─────────────────────────

    /// Bind a non-deserializable complete owner captured by the host.
    pub fn for_owner(&self, principal: &CredentialPrincipal) -> Result<OwnedMemory<'_>> {
        // A retained facade must not retain permission after its lifecycle
        // authority releases the memory scope.
        self.ensure_scope_open()?;
        let lease = self.admit_operation("owner_memory_access")?;
        Ok(OwnedMemory { service: self, principal: principal.clone(), _lease: lease })
    }

    pub fn for_context(&self, context: &UserContext) -> Result<OwnedMemory<'_>> {
        self.ensure_scope_open()?;
        self.for_owner(&CredentialPrincipal::from_verified_context(context)?)
    }

    /// Global administrative access requires installation/service authority;
    /// role strings or user-supplied query filters do not establish it.
    pub fn admin_storage(&self, context: &UserContext) -> Result<Arc<dyn MemoryStorage>> {
        self.ensure_scope_open()?;
        anyhow::ensure!(context.trusted_host_id().is_some(), "Trusted host memory administration required");
        Ok(Arc::clone(&self.storage))
    }

    /// Config accessor.
    pub fn config(&self) -> &MemoryConfig {
        &self.config
    }
}

async fn shutdown_failed_owned_memory_client(
    db: &Surreal<Any>,
    error: anyhow::Error,
) -> anyhow::Error {
    match db.shutdown().await {
        Ok(()) => error,
        Err(cleanup_error) => error.context(format!(
            "memory client initialization failed and owned client shutdown also failed: {cleanup_error}"
        )),
    }
}

async fn close_undelivered_memory_service(service: MemoryService) {
    let report = service.close_scope().await;
    if let Some(error) = report.failure {
        tracing::error!(
            ownership = ?report.ownership,
            error = %error,
            "undelivered memory service cleanup failed"
        );
    }
}

async fn close_memory_resources(
    lifecycle_scope: ResourceScope,
    storage: Arc<SurrealStorage>,
    owned_client: Option<Surreal<Any>>,
    ownership: MemoryStorageOwnership,
) -> MemoryCloseReport {
    let lifecycle_report = lifecycle_scope.close().await;
    let scope_result = storage.close().await;
    let engine_result = match owned_client {
        Some(db) => db.shutdown().await.map_err(|error| error.to_string()),
        None => Ok(()),
    };
    let lifecycle_failure = (!lifecycle_report.failures.is_empty()
        || !lifecycle_report.unresolved_effects.is_empty())
        .then(|| format!(
            "memory lifecycle drain reported failures [{}] and unresolved effects [{}]",
            lifecycle_report.failures.join("; "),
            lifecycle_report.unresolved_effects.join("; "),
        ));
    let failure = match (&scope_result, &engine_result, lifecycle_failure) {
        (Ok(()), Ok(()), None) => None,
        (Err(scope_error), Ok(()), None) => Some(scope_error.to_string()),
        (Ok(()), Err(engine_error), None) => Some(engine_error.clone()),
        (Err(scope_error), Err(engine_error), None) => Some(format!(
            "memory scope release failed: {scope_error}; owned engine shutdown failed: {engine_error}"
        )),
        (Ok(()), Ok(()), Some(lifecycle_error)) => Some(lifecycle_error),
        (Err(scope_error), Ok(()), Some(lifecycle_error)) => Some(format!(
            "memory lifecycle drain failed: {lifecycle_error}; memory scope release failed: {scope_error}"
        )),
        (Ok(()), Err(engine_error), Some(lifecycle_error)) => Some(format!(
            "memory lifecycle drain failed: {lifecycle_error}; owned engine shutdown failed: {engine_error}"
        )),
        (Err(scope_error), Err(engine_error), Some(lifecycle_error)) => Some(format!(
            "memory lifecycle drain failed: {lifecycle_error}; memory scope release failed: {scope_error}; owned engine shutdown failed: {engine_error}"
        )),
    };
    MemoryCloseReport {
        ownership,
        scope_released: scope_result.is_ok(),
        engine_close_acknowledged: matches!(ownership, MemoryStorageOwnership::UarOwned)
            && engine_result.is_ok(),
        failure,
    }
}
