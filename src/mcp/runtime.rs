//! Host-owned MCP readiness and lazy startup, downstream of policy projection.
//!
//! Cached catalogs describe tools, not executable grants. A prepared server pins
//! its cache generation; the first governed call waits for a matching live
//! connection and rejects a changed catalog rather than executing stale metadata.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

use crate::uar::runtime::actor::messages::ActorOwner;

use super::binding_cache::{
    ConnectedMcpServer, McpBinding, McpBindingCache, McpBindingEnvironment, McpBindingError,
    McpBindingRequest, McpBindingTicket,
};
use super::catalog::{McpCatalog, ServerAuthentication, ServerDefinition, ServerSource};
use super::config::McpServerEntry;
use super::lifecycle::McpLifecycleSubscription;
use super::preflight::{McpPreflight, McpPreflightError, prepare_servers};
use super::projection::{McpServerProjection, ProjectedMcpTool, ServerToolCatalog};
use super::registry::McpRegistry;
use super::stdio_process::StdioProcessSupervisor;

/// Trusted host adapter for transport establishment and complete discovery.
/// Implementations must use the request's snapshot and credential revision,
/// preserve reconnect inputs, and cancel partial resources when dropped.
#[async_trait]
pub trait McpConnector: Send + Sync {
    /// Establish an owned single-server registry and discover every tool page.
    ///
    /// # Errors
    /// Report secret-free failures; missing tools/list pages are not success.
    async fn connect(
        &self,
        request: Arc<McpBindingRequest>,
    ) -> Result<ConnectedMcpServer, McpBindingError>;

    /// Close admission and join cleanup, including cancelled partial attempts.
    ///
    /// # Errors
    /// Report a cleanup failure instead of claiming all resources were reaped.
    async fn shutdown(&self) -> anyhow::Result<()>;
}

/// Concrete stdio host adapter. HTTP requires its own snapshot-aware adapter;
/// this type intentionally does not claim to handle remote declarations.
#[derive(Debug, Default)]
pub struct StdioMcpConnector {
    processes: StdioProcessSupervisor,
}

/// Default host connector for configured stdio and remote HTTP declarations.
/// Every connection is built from the immutable binding request; the connector
/// never re-reads process environment or configuration during a run.
#[derive(Debug, Default)]
pub struct ConfiguredMcpConnector {
    processes: StdioProcessSupervisor,
}

#[async_trait]
impl McpConnector for ConfiguredMcpConnector {
    async fn connect(
        &self,
        request: Arc<McpBindingRequest>,
    ) -> Result<ConnectedMcpServer, McpBindingError> {
        match request.definition().configuration() {
            McpServerEntry::Stdio { .. } => {
                McpRegistry::connect_stdio_binding(request, self.processes.clone()).await
            }
            McpServerEntry::RemoteHttp { .. } => McpRegistry::connect_http_binding(request).await,
        }
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        self.processes.shutdown().await?;
        Ok(())
    }
}

#[async_trait]
impl McpConnector for StdioMcpConnector {
    async fn connect(
        &self,
        request: Arc<McpBindingRequest>,
    ) -> Result<ConnectedMcpServer, McpBindingError> {
        McpRegistry::connect_stdio_binding(request, self.processes.clone()).await
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        self.processes.shutdown().await?;
        Ok(())
    }
}

/// Failure before or during a projected host tool call, without secret payloads.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum McpRuntimeError {
    /// Invalidated, unauthenticated, failed or closed binding.
    #[error(transparent)]
    Binding(#[from] McpBindingError),
    /// Newly discovered metadata differs from the step's prepared catalog.
    #[error("MCP server {server:?} changed its tool catalog; prepare a new step")]
    CatalogChanged { server: String },
    /// The supplied call target is not the exact projected descriptor.
    #[error("MCP tool {tool:?} is not projected for server {server:?}")]
    ToolNotProjected { server: String, tool: String },
    /// One total budget covers readiness plus execution on the call path.
    #[error("MCP server {server:?} exceeded its {operation} timeout")]
    TimedOut {
        server: String,
        operation: &'static str,
    },
    /// The transport call failed; no automatic replay is authorized here.
    #[error("MCP tool {tool:?} failed on server {server:?}")]
    ToolFailed { server: String, tool: String },
}

/// Shared host manager for exact-key connection reuse and lazy preparation.
#[derive(Clone)]
pub struct McpRuntimeManager {
    cache: McpBindingCache,
    connector: Arc<dyn McpConnector>,
    readiness_timeout: Duration,
    call_timeout: Duration,
}

impl fmt::Debug for McpRuntimeManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpRuntimeManager")
            .field("cache", &self.cache)
            .field("readiness_timeout", &self.readiness_timeout)
            .field("call_timeout", &self.call_timeout)
            .finish_non_exhaustive()
    }
}

/// Root-host capture supplied alongside an already-resolved run policy.
/// The catalog/environment are immutable; runtime clones share the same cache
/// and transport supervisor. This is not a serializable request payload or a
/// child reconnect grant. The application host retains shutdown ownership.
#[derive(Debug, Clone)]
pub struct McpRunResources {
    owner: ActorOwner,
    runtime: McpRuntimeManager,
    catalog: Arc<McpCatalog>,
    environment: Arc<McpBindingEnvironment>,
}

impl McpRunResources {
    /// Capture trusted root inputs without connecting, reading ambient state,
    /// assigning declaration authority or changing the caller's run policy.
    pub fn new(
        owner: ActorOwner,
        runtime: McpRuntimeManager,
        catalog: Arc<McpCatalog>,
        environment: Arc<McpBindingEnvironment>,
    ) -> Self {
        Self {
            owner,
            runtime,
            catalog,
            environment,
        }
    }

    /// Principal whose credentials and environment the host captured.
    pub fn owner(&self) -> &ActorOwner {
        &self.owner
    }

    /// Shared runtime; requests do not construct a fresh connection cache.
    pub fn runtime(&self) -> &McpRuntimeManager {
        &self.runtime
    }

    /// Exact global/skill declarations, including host-assigned auth revisions.
    pub fn catalog(&self) -> &Arc<McpCatalog> {
        &self.catalog
    }

    /// Exact launch inputs. Never serialize this into a turn or event.
    pub fn environment(&self) -> &Arc<McpBindingEnvironment> {
        &self.environment
    }
}

impl McpRuntimeManager {
    /// Subscribe before preflight/readiness to retain that binding's transitions.
    ///
    /// # Errors
    /// Rejects shutdown or a generation invalidated during admission.
    pub fn observe(
        &self,
        request: &McpBindingRequest,
    ) -> Result<McpLifecycleSubscription, McpBindingError> {
        self.cache.observe(request)
    }

    /// Prepare exact authority-selected servers for one verified owner.
    /// Resolve each declaration against the same captured parent environment.
    /// Required availability failures abort; optional failures warn and remove
    /// their tools. Complete cached skill/child catalogs retain lazy startup.
    ///
    /// # Errors
    /// Required server failure, interrupted/invalidated binding, or invalid tool
    /// projection. Optional status never suppresses an ownership invariant.
    pub async fn preflight(
        &self,
        projection: &McpServerProjection,
        owner: &ActorOwner,
        inherited: &McpBindingEnvironment,
    ) -> Result<McpPreflight, McpPreflightError> {
        prepare_servers(self, projection, owner, inherited, None).await
    }

    pub(crate) async fn preflight_observed(
        &self,
        projection: &McpServerProjection,
        owner: &ActorOwner,
        inherited: &McpBindingEnvironment,
        events: &Arc<super::run_events::McpRunEvents>,
        cancellation: &tokio_util::sync::CancellationToken,
    ) -> Result<McpPreflight, McpPreflightError> {
        prepare_servers(
            self,
            projection,
            owner,
            inherited,
            Some((events, cancellation)),
        )
        .await
    }

    pub(crate) async fn prepare_observed(
        &self,
        request: Arc<McpBindingRequest>,
        events: &Arc<super::run_events::McpRunEvents>,
        cancellation: &tokio_util::sync::CancellationToken,
    ) -> Result<PreparedMcpServer, McpRuntimeError> {
        let mut prepared = events
            .forward(
                self,
                &request,
                self.prepare(Arc::clone(&request)),
                Some(cancellation),
            )
            .await?;
        prepared.events = Some(Arc::clone(events));
        Ok(prepared)
    }

    /// Attach a host connector and positive configured readiness/call budgets.
    ///
    /// # Errors
    /// Rejects zero budgets, which cannot provide a usable readiness contract.
    pub fn new(
        cache: McpBindingCache,
        connector: Arc<dyn McpConnector>,
        readiness_timeout: Duration,
        call_timeout: Duration,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !readiness_timeout.is_zero(),
            "MCP readiness timeout must be positive"
        );
        anyhow::ensure!(!call_timeout.is_zero(), "MCP call timeout must be positive");
        Ok(Self {
            cache,
            connector,
            readiness_timeout,
            call_timeout,
        })
    }

    /// Prepare a selected server's complete catalog for step projection.
    /// Global servers are eager. Skill/child servers defer startup only when
    /// the same owner/config/auth/environment generation has complete discovery.
    ///
    /// # Errors
    /// Discovery failure, authentication requirement, shutdown or invalidation.
    /// Required/optional policy is deliberately handled by the preflight caller.
    pub async fn prepare(
        &self,
        request: Arc<McpBindingRequest>,
    ) -> Result<PreparedMcpServer, McpRuntimeError> {
        let ticket = self.cache.pin(&request)?;
        if matches!(
            request.definition().authentication(),
            ServerAuthentication::Unknown | ServerAuthentication::Required
        ) {
            return Err(McpBindingError::AuthenticationRequired {
                server: request.definition().name().to_owned(),
            }
            .into());
        }
        let cached = self.cache.catalog(&ticket)?;
        let catalog = match cached {
            Some(catalog) if !matches!(request.definition().source(), ServerSource::Global) => {
                catalog
            }
            _ => self
                .connect(&ticket, Arc::clone(&request))
                .await?
                .catalog()
                .clone(),
        };
        Ok(PreparedMcpServer {
            manager: self.clone(),
            request,
            ticket,
            catalog,
            events: None,
        })
    }

    async fn connect(
        &self,
        ticket: &McpBindingTicket,
        request: Arc<McpBindingRequest>,
    ) -> Result<Arc<McpBinding>, McpRuntimeError> {
        let server = request.definition().name().to_owned();
        let connector = Arc::clone(&self.connector);
        tokio::time::timeout(
            self.readiness_timeout,
            self.cache
                .get_or_connect_pinned(ticket, request, move |request| async move {
                    connector.connect(request).await
                }),
        )
        .await
        .map_err(|_| McpRuntimeError::TimedOut {
            server,
            operation: "readiness",
        })?
        .map_err(McpRuntimeError::from)
    }

    /// Revoke cached connections and await both refresh and transport cleanup.
    ///
    /// # Errors
    /// Returns connector cleanup failures, including partial process attempts.
    pub async fn shutdown(&self) -> anyhow::Result<()> {
        self.cache.shutdown().await;
        self.connector.shutdown().await
    }

    /// Revoke every cached revision of an administratively changed server and
    /// join transports already retired by the generation change.
    pub async fn invalidate_server(&self, server: &str) {
        self.cache.invalidate_server(server);
        self.cache.reap_retired().await;
    }
}

/// Complete preflight catalog and generation retained by one prepared step.
#[derive(Clone)]
pub struct PreparedMcpServer {
    manager: McpRuntimeManager,
    request: Arc<McpBindingRequest>,
    ticket: McpBindingTicket,
    catalog: ServerToolCatalog,
    events: Option<Arc<super::run_events::McpRunEvents>>,
}

impl fmt::Debug for PreparedMcpServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedMcpServer")
            .field("catalog", &self.catalog)
            .field("ticket", &self.ticket)
            .finish_non_exhaustive()
    }
}

impl PreparedMcpServer {
    /// Snapshot to feed into authority-checked step tool projection.
    pub fn catalog(&self) -> &ServerToolCatalog {
        &self.catalog
    }

    /// Wait for or lazily start exactly the binding prepared for this step.
    ///
    /// # Errors
    /// Rejects timeout, invalidation and discovery changes before tool execution.
    pub async fn wait_until_ready(&self) -> Result<Arc<McpBinding>, McpRuntimeError> {
        match &self.events {
            Some(events) => {
                events
                    .forward(&self.manager, &self.request, self.ready_binding(), None)
                    .await
            }
            None => self.ready_binding().await,
        }
    }

    async fn ready_binding(&self) -> Result<Arc<McpBinding>, McpRuntimeError> {
        let binding = self
            .manager
            .connect(&self.ticket, Arc::clone(&self.request))
            .await?;
        let discovered = binding.catalog();
        if discovered.tools().len() != self.catalog.tools().len()
            || self.catalog.tools().iter().any(|(name, descriptor)| {
                !discovered
                    .tools()
                    .get(name)
                    .is_some_and(|actual| descriptor.equivalent_to(actual))
            })
        {
            return Err(McpRuntimeError::CatalogChanged {
                server: self.request.definition().name().to_owned(),
            });
        }
        Ok(binding)
    }

    /// Execute an already-governed projected call after readiness, within one
    /// total call budget. Policy, argument validation and approval must have run
    /// in the trusted host before entering this transport method.
    ///
    /// # Errors
    /// Rejects a foreign/changed projection, readiness failure, timeout or tool
    /// failure. A failed call is never replayed: it may have mutated remotely.
    pub async fn call_tool(
        &self,
        tool: &ProjectedMcpTool,
        arguments: Value,
    ) -> Result<Value, McpRuntimeError> {
        match &self.events {
            Some(events) => {
                events
                    .forward(
                        &self.manager,
                        &self.request,
                        self.call_projected(tool, arguments),
                        None,
                    )
                    .await
            }
            None => self.call_projected(tool, arguments).await,
        }
    }

    async fn call_projected(
        &self,
        tool: &ProjectedMcpTool,
        arguments: Value,
    ) -> Result<Value, McpRuntimeError> {
        let server = self.request.definition().name().to_owned();
        let name = &tool.descriptor().provider_name;
        if !same_definition(self.request.definition(), tool.server())
            || !self
                .catalog
                .tools()
                .get(name)
                .is_some_and(|cached| cached.equivalent_to(tool.descriptor()))
        {
            return Err(McpRuntimeError::ToolNotProjected {
                server,
                tool: name.clone(),
            });
        }
        tokio::time::timeout(self.manager.call_timeout, async {
            let binding = self.ready_binding().await?;
            binding
                .registry()?
                .call_namespaced_tool(name, arguments)
                .await
                .map_err(|_| McpRuntimeError::ToolFailed {
                    server: server.clone(),
                    tool: name.clone(),
                })
        })
        .await
        .map_err(|_| McpRuntimeError::TimedOut {
            server,
            operation: "call (including readiness)",
        })?
    }

    /// Retire this connection after a host-observed disconnect, keeping only
    /// complete discovery for the next preparation. This step becomes stale.
    ///
    /// # Errors
    /// Rejects already-invalidated generations and shutdown.
    pub async fn retire_connection(&self) -> Result<(), McpBindingError> {
        self.manager.cache.retire_connection(&self.ticket)?;
        self.manager.cache.reap_retired().await;
        Ok(())
    }
}

fn same_definition(expected: &ServerDefinition, actual: &ServerDefinition) -> bool {
    expected.name() == actual.name()
        && expected.source() == actual.source()
        && expected.config_hash() == actual.config_hash()
        && expected.is_required() == actual.is_required()
        && expected.authentication() == actual.authentication()
}
