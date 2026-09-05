//! Owner-isolated live MCP bindings with caller-owned, single-flight refresh.
//!
//! This is a trusted-host cache, not an execution or credential grant. The
//! connector must launch from the supplied immutable environment and credential
//! revision, not re-read ambient state. Dropping that connector future must
//! cancel its partially established transport. No refresh task is detached.

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use thiserror::Error;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::uar::domain::events::{McpServerState, McpStateReason};
use crate::uar::runtime::actor::messages::ActorOwner;
use crate::uar::tools::descriptor::ToolSource;

use super::catalog::{ServerAuthentication, ServerConfigHash, ServerDefinition, ServerSource};
use super::config::{McpServerEntry, expand_from_environment};
use super::lifecycle::{McpLifecycle, McpLifecycleSubscription};
use super::projection::ServerToolCatalog;
use super::registry::McpRegistry;

/// Exact host-selected environment, including inherited variables and cwd.
///
/// Preserve non-UTF-8 values; lossy conversion could alias distinct launches.
/// The connector must use this full map with environment inheritance disabled.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct McpBindingEnvironment {
    directory: PathBuf,
    variables: BTreeMap<OsString, OsString>,
}

impl McpBindingEnvironment {
    /// Capture resolved launch inputs. This performs no ambient environment I/O.
    ///
    /// # Errors
    /// Rejects relative cwd, whose meaning could change between lookup and spawn.
    pub fn new(
        directory: PathBuf,
        variables: BTreeMap<OsString, OsString>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(directory.is_absolute(), "MCP binding cwd must be absolute");
        Ok(Self {
            directory,
            variables,
        })
    }

    /// Apply declared environment overrides against one captured parent map.
    /// Sibling overrides do not depend on hash-map insertion order.
    ///
    /// # Errors
    /// Rejects relative cwd and unresolved/non-UTF-8 interpolated values. Other
    /// inherited OS-string values remain byte-exact, including non-UTF-8 values.
    pub fn resolve(
        directory: PathBuf,
        mut inherited: BTreeMap<OsString, OsString>,
        configuration: &McpServerEntry,
    ) -> anyhow::Result<Self> {
        let (McpServerEntry::Stdio { env, .. } | McpServerEntry::RemoteHttp { env, .. }) =
            configuration;
        let overrides = env
            .iter()
            .map(|(key, value)| {
                Ok((
                    OsString::from(key),
                    OsString::from(expand_from_environment(value, &inherited)?),
                ))
            })
            .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
        inherited.extend(overrides);
        Self::new(directory, inherited)
    }

    /// Host-resolved working directory; not a filesystem permission grant.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Full launch environment, including secrets. Never emit this in telemetry.
    pub fn variables(&self) -> &BTreeMap<OsString, OsString> {
        &self.variables
    }
}

impl fmt::Debug for McpBindingEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("McpBindingEnvironment([redacted])")
    }
}

/// Exact reuse identity. Private fields prevent omission of owner or revisions.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct McpBindingKey {
    owner: ActorOwner,
    server: String,
    source: ServerSource,
    config: ServerConfigHash,
    required: bool,
    authentication: ServerAuthentication,
    environment: Arc<McpBindingEnvironment>,
}

impl fmt::Debug for McpBindingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpBindingKey")
            .field("server", &self.server)
            .finish_non_exhaustive()
    }
}

/// One immutable connection request, created after host policy projection.
#[derive(Clone)]
pub struct McpBindingRequest {
    key: McpBindingKey,
    definition: Arc<ServerDefinition>,
}

impl McpBindingRequest {
    /// Associate a selected definition with verified principal/tenant identity.
    /// Authentication revisions come from the definition, never an MCP response.
    pub fn new(
        owner: ActorOwner,
        definition: Arc<ServerDefinition>,
        environment: Arc<McpBindingEnvironment>,
    ) -> Self {
        let key = McpBindingKey {
            owner,
            server: definition.name().to_owned(),
            source: definition.source().clone(),
            config: definition.config_hash().clone(),
            required: definition.is_required(),
            authentication: definition.authentication().clone(),
            environment,
        };
        Self { key, definition }
    }

    /// Identity used for lookup and explicit invalidation.
    pub fn key(&self) -> &McpBindingKey {
        &self.key
    }

    /// Exact declaration the connector must establish.
    pub fn definition(&self) -> &Arc<ServerDefinition> {
        &self.definition
    }

    /// Snapshot the connector must use, without ambient fallback.
    pub fn environment(&self) -> &McpBindingEnvironment {
        &self.key.environment
    }

    /// Verified user and tenant namespace; never supplied by model arguments.
    pub fn owner(&self) -> &ActorOwner {
        &self.key.owner
    }
}

impl fmt::Debug for McpBindingRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpBindingRequest")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

/// Secret-free cache outcome, shared by all callers of one connection attempt.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum McpBindingError {
    /// Authentication must complete before connection reuse or startup.
    #[error("MCP server {server:?} requires authentication")]
    AuthenticationRequired { server: String },
    /// The host connector failed. Raw errors may contain credential-bearing URLs.
    #[error("MCP server {server:?} could not establish a connection")]
    ConnectionFailed { server: String },
    /// The connector returned a missing, foreign or differently configured server.
    #[error("MCP connector returned a mismatched binding for server {server:?}")]
    InvalidBinding { server: String },
    /// Partial discovery cannot justify lazy startup or a step's tool inventory.
    #[error("MCP server {server:?} has no complete matching tool catalog")]
    IncompleteCatalog { server: String },
    /// A newer generation superseded the attempt; the caller must re-project.
    #[error("MCP binding for server {server:?} was invalidated")]
    Invalidated { server: String },
    /// The caller owning the refresh dropped its future; retry starts fresh.
    #[error("MCP binding refresh for server {server:?} was cancelled")]
    Cancelled { server: String },
    /// Shutdown revokes both cached bindings and outstanding refreshes.
    #[error("MCP binding cache is shutting down")]
    ShuttingDown,
}

/// Host connector output after successful, complete tool discovery.
/// The cache checks server identity, completeness and every compiled descriptor.
pub struct ConnectedMcpServer {
    registry: Option<McpRegistry>,
    catalog: ServerToolCatalog,
}

impl ConnectedMcpServer {
    /// Transfer an exclusively owned registry and its discovery snapshot.
    /// The complete flag must reflect all pagination, not merely a first page.
    pub fn new(registry: McpRegistry, catalog: ServerToolCatalog) -> Self {
        Self {
            registry: Some(registry),
            catalog,
        }
    }

    fn into_parts(mut self) -> (McpRegistry, ServerToolCatalog) {
        // Only this consuming method can take the privately owned registry.
        (
            self.registry.take().expect("connected registry is owned"),
            self.catalog.clone(),
        )
    }
}

impl fmt::Debug for ConnectedMcpServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectedMcpServer")
            .field("catalog", &self.catalog)
            .finish_non_exhaustive()
    }
}

impl Drop for ConnectedMcpServer {
    fn drop(&mut self) {
        if let Some(registry) = &self.registry {
            registry.begin_shutdown();
        }
    }
}

/// Generation pinned during host preflight. Old steps cannot revive a revoked
/// binding by looking up the same key after invalidation.
#[derive(Debug, Clone)]
pub struct McpBindingTicket {
    key: McpBindingKey,
    generation: Uuid,
}

/// One owned live connection. Retain this lease for the entire tool call.
///
/// A registry reference is host-internal capability, not model authorization.
/// Do not mutate or upsert this registry; replace it through cache invalidation.
/// Required/optional handling and complete tool discovery belong to lifecycle.
pub struct McpBinding {
    request: Arc<McpBindingRequest>,
    registry: McpRegistry,
    catalog: ServerToolCatalog,
    revoked: CancellationToken,
}

impl McpBinding {
    /// Inputs captured when this exact connection was established.
    pub fn request(&self) -> &McpBindingRequest {
        &self.request
    }

    /// Complete descriptors discovered on this binding, not from another owner.
    pub fn catalog(&self) -> &ServerToolCatalog {
        &self.catalog
    }

    /// Borrow the connection only while its owning lease remains valid.
    ///
    /// # Errors
    /// Rejects a lease revoked by invalidation or shutdown. Revocation also
    /// reaches previously borrowed registry views through registry shutdown.
    pub fn registry(&self) -> Result<&McpRegistry, McpBindingError> {
        if self.revoked.is_cancelled() {
            return Err(McpBindingError::Invalidated {
                server: self.request.key.server.clone(),
            });
        }
        Ok(&self.registry)
    }

    fn revoke(&self) {
        self.revoked.cancel();
        self.registry.begin_shutdown();
    }
}

impl fmt::Debug for McpBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpBinding")
            .field("key", &self.request.key)
            .field("revoked", &self.revoked.is_cancelled())
            .finish_non_exhaustive()
    }
}

impl Drop for McpBinding {
    fn drop(&mut self) {
        self.revoke();
    }
}

type RefreshResult = Result<Arc<McpBinding>, McpBindingError>;

struct Refresh {
    generation: Uuid,
    cancellation: CancellationToken,
    completion: watch::Sender<Option<RefreshResult>>,
}

struct Entry {
    generation: Uuid,
    lifecycle: McpLifecycle,
    ready: Option<Arc<McpBinding>>,
    refresh: Option<Arc<Refresh>>,
    catalog: Option<ServerToolCatalog>,
}

impl Entry {
    fn new(request: &McpBindingRequest) -> Self {
        let generation = Uuid::new_v4();
        let lifecycle = McpLifecycle::new(
            request.key.server.clone(),
            generation,
            matches!(
                request.key.authentication,
                ServerAuthentication::Unknown | ServerAuthentication::Required
            ),
        );
        Self {
            generation,
            lifecycle,
            ready: None,
            refresh: None,
            catalog: None,
        }
    }
}

#[derive(Default)]
struct CacheState {
    closed: bool,
    entries: HashMap<McpBindingKey, Entry>,
    // Keep cancelled transports owned until their asynchronous close is joined.
    retired: Vec<Arc<McpBinding>>,
}

/// Shared across runs, but never across distinct owner/config/auth/env keys.
///
/// Ready lookups take a read lock. Refresh publication, invalidation and RAII
/// cancellation take short write locks, never held across connector I/O.
#[derive(Clone, Default)]
pub struct McpBindingCache {
    state: Arc<RwLock<CacheState>>,
}

impl fmt::Debug for McpBindingCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self
            .state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f.debug_struct("McpBindingCache")
            .field("closed", &state.closed)
            .field("entry_count", &state.entries.len())
            .field("retired_count", &state.retired.len())
            .finish_non_exhaustive()
    }
}

impl McpBindingCache {
    /// Pin the generation before inspecting a catalog or preparing a step.
    ///
    /// # Errors
    /// Rejects new tickets after shutdown has closed admission.
    pub fn pin(&self, request: &McpBindingRequest) -> Result<McpBindingTicket, McpBindingError> {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.closed {
            return Err(McpBindingError::ShuttingDown);
        }
        let entry = state
            .entries
            .entry(request.key.clone())
            .or_insert_with(|| Entry::new(request));
        Ok(McpBindingTicket {
            key: request.key.clone(),
            generation: entry.generation,
        })
    }

    /// Observe only this exact verified owner/config/auth/environment binding.
    /// Creates a dormant/auth-required entry, never starts a server.
    ///
    /// # Errors
    /// Rejects shutdown or generation invalidation racing observation admission.
    pub fn observe(
        &self,
        request: &McpBindingRequest,
    ) -> Result<McpLifecycleSubscription, McpBindingError> {
        let ticket = self.pin(request)?;
        let state = self
            .state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        check_ticket(&state, &ticket)?;
        Ok(state
            .entries
            .get(&ticket.key)
            .expect("validated binding ticket")
            .lifecycle
            .subscribe())
    }

    /// Read complete discovery for exactly this owner and generation.
    ///
    /// # Errors
    /// Rejects stale tickets and shutdown; never substitutes a newer catalog.
    pub fn catalog(
        &self,
        ticket: &McpBindingTicket,
    ) -> Result<Option<ServerToolCatalog>, McpBindingError> {
        let state = self
            .state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        check_ticket(&state, ticket)?;
        Ok(state
            .entries
            .get(&ticket.key)
            .and_then(|entry| entry.catalog.clone()))
    }

    /// Reuse a ready binding or share one caller-owned connection attempt.
    ///
    /// The connector receives the exact snapshot, must return an exclusively
    /// owned single-server registry, and must unwind partial resources on drop.
    /// It owns connect/list timeouts and credential resolution against the pinned
    /// identity. This function does not spawn, retry or substitute global inputs.
    ///
    /// # Errors
    /// Returns authentication, connector, identity, invalidation or cancellation
    /// errors. Every waiter on the same attempt receives the same result.
    pub async fn get_or_connect<F, Fut>(
        &self,
        request: Arc<McpBindingRequest>,
        connect: F,
    ) -> RefreshResult
    where
        F: FnOnce(Arc<McpBindingRequest>) -> Fut,
        Fut: Future<Output = Result<ConnectedMcpServer, McpBindingError>>,
    {
        let ticket = self.pin(&request)?;
        self.get_or_connect_pinned(&ticket, request, connect).await
    }

    /// Obtain readiness without allowing an already-prepared step to advance
    /// its binding generation implicitly.
    ///
    /// # Errors
    /// Returns stale-ticket errors in addition to the ordinary binding errors.
    pub async fn get_or_connect_pinned<F, Fut>(
        &self,
        ticket: &McpBindingTicket,
        request: Arc<McpBindingRequest>,
        connect: F,
    ) -> RefreshResult
    where
        F: FnOnce(Arc<McpBindingRequest>) -> Fut,
        Fut: Future<Output = Result<ConnectedMcpServer, McpBindingError>>,
    {
        if ticket.key != request.key {
            return Err(McpBindingError::InvalidBinding {
                server: request.key.server.clone(),
            });
        }
        if matches!(
            request.key.authentication,
            ServerAuthentication::Unknown | ServerAuthentication::Required
        ) {
            return Err(McpBindingError::AuthenticationRequired {
                server: request.key.server.clone(),
            });
        }
        {
            let state = self
                .state
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            check_ticket(&state, ticket)?;
            if let Some(binding) = state
                .entries
                .get(&request.key)
                .and_then(|entry| entry.ready.as_ref())
            {
                return Ok(Arc::clone(binding));
            }
        }
        let (refresh, leader) = {
            let mut state = self
                .state
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            check_ticket(&state, ticket)?;
            let entry = state
                .entries
                .entry(request.key.clone())
                .or_insert_with(|| Entry::new(&request));
            if let Some(binding) = &entry.ready {
                return Ok(Arc::clone(binding));
            }
            match &entry.refresh {
                Some(refresh) => (Arc::clone(refresh), false),
                None => {
                    let (completion, _) = watch::channel(None);
                    let refresh = Arc::new(Refresh {
                        generation: entry.generation,
                        cancellation: CancellationToken::new(),
                        completion,
                    });
                    entry.refresh = Some(Arc::clone(&refresh));
                    entry
                        .lifecycle
                        .transition(entry.generation, McpServerState::Connecting, None);
                    (refresh, true)
                }
            }
        };
        if !leader {
            return wait_for_refresh(&refresh).await;
        }

        // Installed before invoking the closure, so panic or future cancellation
        // clears the in-flight marker and notifies all of this attempt's waiters.
        let mut guard = RefreshGuard {
            state: Arc::clone(&self.state),
            key: request.key.clone(),
            refresh: Arc::clone(&refresh),
            finished: false,
        };
        let connected = tokio::select! {
            biased;
            _ = refresh.cancellation.cancelled() => {
                Err(McpBindingError::Invalidated { server: request.key.server.clone() })
            }
            result = connect(Arc::clone(&request)) => result,
        };
        let result = match connected {
            Err(error) => Err(error),
            Ok(connected) => {
                let (registry, catalog) = connected.into_parts();
                let binding = Arc::new(McpBinding {
                    request: Arc::clone(&request),
                    registry,
                    catalog,
                    revoked: CancellationToken::new(),
                });
                if registry_matches(&binding) {
                    Ok(binding)
                } else {
                    binding.revoke();
                    self.state
                        .write()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .retired
                        .push(binding);
                    Err(McpBindingError::InvalidBinding {
                        server: request.key.server.clone(),
                    })
                }
            }
        };
        guard.finish(result)
    }

    /// Revoke one identity, including retained views, and advance its generation.
    /// An outstanding attempt must unwind before a replacement starts.
    pub fn invalidate(&self, key: &McpBindingKey) {
        self.invalidate_matching(|candidate| candidate == key);
    }

    /// Revoke every configuration/credential/environment revision of one owner.
    pub fn invalidate_owner(&self, owner: &ActorOwner) {
        self.invalidate_matching(|candidate| &candidate.owner == owner);
    }

    /// Revoke every owner/config/auth/environment revision of one server.
    pub fn invalidate_server(&self, server: &str) {
        self.invalidate_matching(|candidate| candidate.server == server);
    }

    /// Retire a disconnected transport but preserve its complete catalog for a
    /// later lazy start. Existing prepared steps are invalidated, not retargeted.
    ///
    /// # Errors
    /// Rejects stale tickets and shutdown. The host must re-prepare before use.
    pub fn retire_connection(&self, ticket: &McpBindingTicket) -> Result<(), McpBindingError> {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        check_ticket(&state, ticket)?;
        if let Some(entry) = state.entries.get_mut(&ticket.key) {
            entry.lifecycle.transition(
                entry.generation,
                McpServerState::ShuttingDown,
                Some(McpStateReason::Retired),
            );
            entry.generation = Uuid::new_v4();
            entry
                .lifecycle
                .advance(entry.generation, McpStateReason::Retired);
            if let Some(refresh) = &entry.refresh {
                refresh.cancellation.cancel();
            }
            if let Some(binding) = entry.ready.take() {
                binding.revoke();
                state.retired.push(binding);
            }
        }
        Ok(())
    }

    fn invalidate_matching(&self, matches: impl Fn(&McpBindingKey) -> bool) {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut retired = Vec::new();
        for (key, entry) in &mut state.entries {
            if !matches(key) {
                continue;
            }
            entry.lifecycle.transition(
                entry.generation,
                McpServerState::ShuttingDown,
                Some(McpStateReason::Invalidated),
            );
            entry.generation = Uuid::new_v4();
            entry
                .lifecycle
                .advance(entry.generation, McpStateReason::Invalidated);
            entry.catalog = None;
            if let Some(refresh) = &entry.refresh {
                refresh.cancellation.cancel();
            }
            if let Some(binding) = entry.ready.take() {
                binding.revoke();
                retired.push(binding);
            }
        }
        state.retired.extend(retired);
    }

    /// Join retired transports outside the cache lock; safe to cancel and resume.
    /// Newly retired bindings remain queued for the next call.
    pub async fn reap_retired(&self) {
        let retired = self
            .state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retired
            .clone();
        for binding in &retired {
            binding.registry.shutdown().await;
        }
        self.state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retired
            .retain(|binding| !retired.iter().any(|closed| Arc::ptr_eq(binding, closed)));
    }

    /// Permanently close admission, unwind refreshes, then join all transports.
    /// Retired handles stay in shared state if this future is cancelled.
    pub async fn shutdown(&self) {
        let refreshes = {
            let mut state = self
                .state
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.closed = true;
            let mut refreshes = Vec::new();
            let mut retired = Vec::new();
            for entry in state.entries.values_mut() {
                entry
                    .lifecycle
                    .transition(entry.generation, McpServerState::ShuttingDown, None);
                entry.generation = Uuid::new_v4();
                entry.catalog = None;
                if let Some(refresh) = &entry.refresh {
                    refresh.cancellation.cancel();
                    refreshes.push(Arc::clone(refresh));
                }
                if let Some(binding) = entry.ready.take() {
                    binding.revoke();
                    retired.push(binding);
                }
            }
            state.retired.extend(retired);
            refreshes
        };
        for refresh in refreshes {
            let _ = wait_for_refresh(&refresh).await;
        }
        self.reap_retired().await;
    }
}

struct RefreshGuard {
    state: Arc<RwLock<CacheState>>,
    key: McpBindingKey,
    refresh: Arc<Refresh>,
    finished: bool,
}

impl RefreshGuard {
    fn finish(&mut self, result: RefreshResult) -> RefreshResult {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let current = !state.closed
            && state.entries.get(&self.key).is_some_and(|entry| {
                entry.generation == self.refresh.generation
                    && entry
                        .refresh
                        .as_ref()
                        .is_some_and(|refresh| Arc::ptr_eq(refresh, &self.refresh))
            });
        let result = if current {
            result
        } else {
            if let Ok(binding) = result {
                binding.revoke();
                state.retired.push(binding);
            }
            Err(if state.closed {
                McpBindingError::ShuttingDown
            } else {
                McpBindingError::Invalidated {
                    server: self.key.server.clone(),
                }
            })
        };
        if let Some(entry) = state.entries.get_mut(&self.key)
            && entry
                .refresh
                .as_ref()
                .is_some_and(|refresh| Arc::ptr_eq(refresh, &self.refresh))
        {
            entry.refresh = None;
            if current {
                let (next, reason) = match &result {
                    Ok(binding) => {
                        binding
                            .registry
                            .attach_lifecycle(entry.lifecycle.clone(), entry.generation);
                        (McpServerState::Ready, None)
                    }
                    Err(error) => lifecycle_failure(error),
                };
                entry.lifecycle.transition(entry.generation, next, reason);
            }
            if let Ok(binding) = &result {
                entry.catalog = Some(binding.catalog.clone());
                entry.ready = Some(Arc::clone(binding));
            }
        }
        self.refresh.completion.send_replace(Some(result.clone()));
        self.finished = true;
        result
    }
}

impl Drop for RefreshGuard {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.finish(Err(McpBindingError::Cancelled {
                server: self.key.server.clone(),
            }));
        }
    }
}

pub(crate) fn lifecycle_failure(
    error: &McpBindingError,
) -> (McpServerState, Option<McpStateReason>) {
    let reason = match error {
        McpBindingError::AuthenticationRequired { .. } => {
            return (
                McpServerState::AuthRequired,
                Some(McpStateReason::AuthenticationRequired),
            );
        }
        McpBindingError::ConnectionFailed { .. } => McpStateReason::ConnectionFailed,
        McpBindingError::IncompleteCatalog { .. } => McpStateReason::IncompleteCatalog,
        McpBindingError::InvalidBinding { .. } => McpStateReason::InvalidBinding,
        McpBindingError::Invalidated { .. } => McpStateReason::Invalidated,
        McpBindingError::Cancelled { .. } => McpStateReason::Cancelled,
        McpBindingError::ShuttingDown => return (McpServerState::ShuttingDown, None),
    };
    (McpServerState::Failed, Some(reason))
}

async fn wait_for_refresh(refresh: &Refresh) -> RefreshResult {
    let mut completion = refresh.completion.subscribe();
    loop {
        // Clone and release watch's read guard before awaiting a change. Inspect
        // the current value even on a late subscription (already marked seen).
        let result = completion.borrow_and_update().clone();
        if let Some(result) = result {
            return result;
        }
        if completion.changed().await.is_err() {
            return Err(McpBindingError::ShuttingDown);
        }
    }
}

fn registry_matches(binding: &McpBinding) -> bool {
    let request = &binding.request;
    let catalog_key = McpBindingRequest::new(
        request.key.owner.clone(),
        Arc::clone(binding.catalog.definition()),
        Arc::clone(&request.key.environment),
    );
    let descriptors = binding.registry.descriptors();
    if catalog_key.key != request.key
        || !binding.catalog.is_complete()
        || descriptors.len() != binding.catalog.tools().len()
        || descriptors.iter().any(|tool| {
            !binding
                .catalog
                .tools()
                .get(&tool.provider_name)
                .is_some_and(|cached| cached.equivalent_to(tool))
        })
    {
        return false;
    }
    let entries = binding.registry.server_entries();
    if entries.len() != 1
        || binding.registry.has_frozen_bindings()
        || binding.registry.server_names() != vec![request.key.server.clone()]
        || binding.registry.descriptors().iter().any(|tool| {
            tool.source != ToolSource::Mcp || tool.server.as_deref() != Some(&request.key.server)
        })
    {
        return false;
    }
    entries
        .get(&request.key.server)
        .is_some_and(|configuration| {
            ServerDefinition::new(
                request.key.server.clone(),
                request.key.source.clone(),
                configuration.clone(),
                request.definition.is_required(),
                request.key.authentication.clone(),
            )
            .is_ok_and(|actual| actual.config_hash() == &request.key.config)
        })
}

fn check_ticket(state: &CacheState, ticket: &McpBindingTicket) -> Result<(), McpBindingError> {
    if state.closed {
        return Err(McpBindingError::ShuttingDown);
    }
    if !state
        .entries
        .get(&ticket.key)
        .is_some_and(|entry| entry.generation == ticket.generation)
    {
        return Err(McpBindingError::Invalidated {
            server: ticket.key.server.clone(),
        });
    }
    Ok(())
}
