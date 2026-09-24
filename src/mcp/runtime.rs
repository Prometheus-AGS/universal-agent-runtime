//! Host-owned MCP readiness and lazy startup, downstream of policy projection.
//!
//! Cached catalogs describe tools, not executable grants. A prepared server pins
//! its cache generation; the first governed call waits for a matching live
//! connection and rejects a changed catalog rather than executing stale metadata.

use std::collections::{BTreeMap, HashMap};
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
use super::config::{McpHttpHeaderValue, McpServerEntry, expand_from_environment};
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

/// Parsed request-scoped HTTP headers. Values remain private and Debug never
/// exposes them.
#[derive(Clone)]
pub(crate) struct RunHttpHeaders {
    bearer: Option<secrecy::SecretString>,
    custom: HashMap<reqwest_mcp::header::HeaderName, reqwest_mcp::header::HeaderValue>,
}

impl fmt::Debug for RunHttpHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RunHttpHeaders([redacted])")
    }
}

impl RunHttpHeaders {
    pub(crate) fn parse(
        values: &std::collections::BTreeMap<String, secrecy::SecretString>,
    ) -> anyhow::Result<Self> {
        use secrecy::ExposeSecret;
        let mut bearer = None;
        let mut custom = HashMap::new();
        for (name, value) in values {
            let name = reqwest_mcp::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| anyhow::anyhow!("MCP header name is invalid"))?;
            anyhow::ensure!(
                name != reqwest_mcp::header::HOST && name.as_str() != "mcp-session-id",
                "MCP transport-owned header is forbidden"
            );
            if name == reqwest_mcp::header::AUTHORIZATION {
                let token = value
                    .expose_secret()
                    .strip_prefix("Bearer ")
                    .ok_or_else(|| anyhow::anyhow!("MCP authorization must use Bearer scheme"))?;
                anyhow::ensure!(!token.is_empty(), "MCP bearer token is empty");
                anyhow::ensure!(bearer.is_none(), "MCP authorization header is duplicated");
                bearer = Some(secrecy::SecretString::from(token.to_owned()));
            } else {
                let value =
                    reqwest_mcp::header::HeaderValue::from_bytes(value.expose_secret().as_bytes())
                        .map_err(|_| anyhow::anyhow!("MCP header value is invalid"))?;
                anyhow::ensure!(
                    custom.insert(name, value).is_none(),
                    "MCP header is duplicated"
                );
            }
        }
        Ok(Self { bearer, custom })
    }

    pub(crate) fn from_configuration(
        configuration: &McpServerEntry,
        environment: &McpBindingEnvironment,
        overlay: Option<&std::collections::BTreeMap<String, secrecy::SecretString>>,
    ) -> anyhow::Result<Self> {
        let McpServerEntry::RemoteHttp { headers, .. } = configuration else {
            anyhow::bail!("MCP HTTP headers require a remote HTTP server");
        };
        let mut resolved = std::collections::BTreeMap::new();
        for (name, value) in headers {
            let value = match value {
                McpHttpHeaderValue::Literal(value) => secrecy::SecretString::from(
                    expand_from_environment(value, environment.variables())?,
                ),
                McpHttpHeaderValue::SecretRef { secret_ref, .. } => {
                    let variable = secret_ref
                        .strip_prefix("env:")
                        .ok_or_else(|| anyhow::anyhow!("MCP header secret reference is invalid"))?;
                    let value = environment
                        .variables()
                        .get(std::ffi::OsStr::new(variable))
                        .filter(|value| !value.is_empty())
                        .ok_or_else(|| {
                            anyhow::anyhow!("MCP header secret reference is unresolved")
                        })?
                        .to_str()
                        .ok_or_else(|| anyhow::anyhow!("MCP header secret is not UTF-8"))?;
                    secrecy::SecretString::from(value.to_owned())
                }
            };
            resolved.insert(name.clone(), value);
        }
        if let Some(overlay) = overlay {
            for (name, value) in overlay {
                anyhow::ensure!(
                    !resolved
                        .keys()
                        .any(|existing| existing.eq_ignore_ascii_case(name)),
                    "MCP header is configured by both the destination and run grant"
                );
                resolved.insert(name.clone(), value.clone());
            }
        }
        Self::parse(&resolved)
    }

    pub(crate) fn apply(
        &self,
        mut config: rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig,
    ) -> rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig {
        use secrecy::ExposeSecret;
        if let Some(bearer) = &self.bearer {
            config = config.auth_header(bearer.expose_secret().to_owned());
        }
        if !self.custom.is_empty() {
            config = config.custom_headers(self.custom.clone());
        }
        config
    }
}

/// One immutable credential lifetime shared by a run's root and narrowed
/// local children. It contains no credential bytes and cannot be renewed in
/// place; a trusted host must establish a fresh binding at a run boundary.
#[derive(Clone)]
pub(crate) struct RunMcpCredentialLease {
    expires_at_unix: u64,
    revoked: tokio_util::sync::CancellationToken,
}

impl fmt::Debug for RunMcpCredentialLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RunMcpCredentialLease")
            .field("expires_at_unix", &self.expires_at_unix)
            .field("revoked", &self.revoked.is_cancelled())
            .finish()
    }
}

impl RunMcpCredentialLease {
    pub(crate) fn new(expires_at_unix: u64) -> Self {
        Self {
            expires_at_unix,
            revoked: tokio_util::sync::CancellationToken::new(),
        }
    }

    pub(crate) fn authorize(&self, server: &str) -> Result<(), McpBindingError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(u64::MAX);
        if self.revoked.is_cancelled() || now >= self.expires_at_unix {
            self.revoked.cancel();
            return Err(McpBindingError::AuthenticationRequired {
                server: server.to_owned(),
            });
        }
        Ok(())
    }

    pub(crate) fn revoke(&self) {
        self.revoked.cancel();
    }
}

#[derive(Clone)]
pub(crate) struct RunMcpConnection {
    headers: BTreeMap<String, secrecy::SecretString>,
    credential: Option<RunMcpCredentialLease>,
}

impl fmt::Debug for RunMcpConnection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RunMcpConnection")
            .field("header_count", &self.headers.len())
            .field("credential", &self.credential)
            .finish()
    }
}

impl RunMcpConnection {
    pub(crate) fn new(
        headers: BTreeMap<String, secrecy::SecretString>,
        credential: Option<RunMcpCredentialLease>,
    ) -> Self {
        Self {
            headers,
            credential,
        }
    }
}

/// Revocation handle retained by the authenticated run host. It can only
/// revoke already-admitted server leases and cannot add or renew authority.
#[derive(Clone, Default)]
pub(crate) struct RunMcpGrantControl {
    credentials: Arc<HashMap<String, RunMcpCredentialLease>>,
}

impl fmt::Debug for RunMcpGrantControl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RunMcpGrantControl")
            .field("server_count", &self.credentials.len())
            .finish()
    }
}

impl RunMcpGrantControl {
    pub(crate) fn from_connections(connections: &HashMap<String, RunMcpConnection>) -> Self {
        Self {
            credentials: Arc::new(
                connections
                    .iter()
                    .filter_map(|(name, connection)| {
                        connection
                            .credential
                            .clone()
                            .map(|credential| (name.clone(), credential))
                    })
                    .collect(),
            ),
        }
    }

    pub(crate) fn revoke(&self, server: &str) -> bool {
        let Some(credential) = self.credentials.get(server) else {
            return false;
        };
        credential.revoke();
        true
    }

    fn revoke_all(&self) {
        for credential in self.credentials.values() {
            credential.revoke();
        }
    }
}

/// Run-owned connector. It can only connect the exact HTTP names captured in
/// the request and has no process supervisor or global configuration access.
pub(crate) struct RunMcpConnector {
    connections: HashMap<String, RunMcpConnection>,
    grants: RunMcpGrantControl,
}

impl fmt::Debug for RunMcpConnector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunMcpConnector")
            .field("server_count", &self.connections.len())
            .finish()
    }
}

impl RunMcpConnector {
    pub(crate) fn new(
        connections: HashMap<String, RunMcpConnection>,
        grants: RunMcpGrantControl,
    ) -> Self {
        Self {
            connections,
            grants,
        }
    }
}

#[async_trait]
impl McpConnector for RunMcpConnector {
    async fn connect(
        &self,
        request: Arc<McpBindingRequest>,
    ) -> Result<ConnectedMcpServer, McpBindingError> {
        let name = request.definition().name();
        let connection =
            self.connections
                .get(name)
                .ok_or_else(|| McpBindingError::InvalidBinding {
                    server: name.to_owned(),
                })?;
        let headers = RunHttpHeaders::from_configuration(
            request.definition().configuration(),
            request.environment(),
            Some(&connection.headers),
        )
        .map_err(|_| McpBindingError::InvalidBinding {
            server: name.to_owned(),
        })?;
        McpRegistry::connect_http_binding_with_headers(
            request,
            &headers,
            connection.credential.clone(),
        )
        .await
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        self.grants.revoke_all();
        Ok(())
    }
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
            McpServerEntry::RemoteHttp { .. } => {
                let headers = RunHttpHeaders::from_configuration(
                    request.definition().configuration(),
                    request.environment(),
                    None,
                )
                .map_err(|_| McpBindingError::InvalidBinding {
                    server: request.definition().name().to_owned(),
                })?;
                McpRegistry::connect_http_binding_with_headers(request, &headers, None).await
            }
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
    run_scoped_names: Option<Arc<std::collections::BTreeSet<String>>>,
    run_grants: Option<RunMcpGrantControl>,
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
            run_scoped_names: None,
            run_grants: None,
        }
    }

    /// Capture a request-owned HTTP universe. The names are the complete MCP
    /// authority for this root and its local children.
    pub(crate) fn new_run_scoped(
        owner: ActorOwner,
        runtime: McpRuntimeManager,
        catalog: Arc<McpCatalog>,
        environment: Arc<McpBindingEnvironment>,
        names: std::collections::BTreeSet<String>,
        run_grants: RunMcpGrantControl,
    ) -> Self {
        Self {
            owner,
            runtime,
            catalog,
            environment,
            run_scoped_names: Some(Arc::new(names)),
            run_grants: Some(run_grants),
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

    /// Exact request-owned server names, or `None` for the host's shared
    /// configured catalog.
    pub(crate) fn run_scoped_names(&self) -> Option<&Arc<std::collections::BTreeSet<String>>> {
        self.run_scoped_names.as_ref()
    }

    pub(crate) fn run_grants(&self) -> Option<&RunMcpGrantControl> {
        self.run_grants.as_ref()
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

    /// Publish that a run credential must be refreshed without selecting or
    /// constructing a replacement credential.
    pub(crate) fn require_authentication(&self, server: &str) {
        self.cache.require_authentication(server);
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
