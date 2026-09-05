//! Required/optional MCP preparation after authority and policy selection.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;

use crate::uar::runtime::actor::messages::ActorOwner;

use super::binding_cache::{McpBindingEnvironment, McpBindingError, McpBindingRequest};
use super::projection::{McpProjectionError, McpServerProjection, McpStepProjection};
use super::runtime::{McpRuntimeError, McpRuntimeManager, PreparedMcpServer};

/// Availability failures safe to report without launch inputs or credentials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Error)]
#[serde(rename_all = "snake_case")]
pub enum McpServerFailure {
    /// A declared override could not resolve against the captured environment.
    #[error("environment resolution failed; check the server's declared environment variables")]
    Environment,
    /// The host requires a new authenticated binding before using this server.
    #[error("authentication is required; authenticate the server and prepare a new run")]
    Authentication,
    /// The selected transport could not establish a service connection.
    #[error("connection failed; check the configured server command or endpoint")]
    Connection,
    /// Discovery did not yield a complete, valid descriptor catalog.
    #[error("tool discovery failed; check the server's tools/list response and schemas")]
    Discovery,
    /// Readiness exceeded the configured deadline.
    #[error("readiness timed out; check server responsiveness and the readiness timeout")]
    Timeout,
}

/// One omitted optional server. Contains no raw transport errors or secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct McpPreflightWarning {
    server: String,
    reason: McpServerFailure,
}

impl McpPreflightWarning {
    /// Selected server whose tools were omitted; no lower origin replaced it.
    pub fn server(&self) -> &str {
        &self.server
    }

    /// Bounded, actionable failure classification.
    pub const fn reason(&self) -> McpServerFailure {
        self.reason
    }
}

/// Failure that prevents returning a usable MCP preflight result.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum McpPreflightError {
    /// The requesting run ended; no further dependency startup is admitted.
    #[error("MCP preflight cancelled with its run")]
    Cancelled,
    /// Required declarations cannot be silently omitted.
    #[error("required MCP server {server:?} could not be prepared: {reason}")]
    RequiredServer {
        server: String,
        reason: McpServerFailure,
    },
    /// Revocation, cancellation, shutdown or a violated binding invariant.
    #[error("MCP preflight for server {server:?} stopped: {source}")]
    Interrupted {
        server: String,
        source: McpRuntimeError,
    },
    /// Exact tool projection failed after availability filtering.
    #[error(transparent)]
    Projection(#[from] McpProjectionError),
}

/// Consistent tools and bindings after excluding only failed optional servers.
#[derive(Debug, Clone)]
pub struct McpPreflight {
    owner: ActorOwner,
    projection: McpStepProjection,
    servers: BTreeMap<String, PreparedMcpServer>,
    warnings: Vec<McpPreflightWarning>,
}

impl McpPreflight {
    /// Verified principal whose exact cache namespace supplied every binding.
    pub fn owner(&self) -> &ActorOwner {
        &self.owner
    }

    /// Exact selected tools. Failed optional servers contribute no descriptors.
    pub fn projection(&self) -> &McpStepProjection {
        &self.projection
    }

    /// Matching generation-pinned bindings for downstream governed calls.
    pub fn servers(&self) -> &BTreeMap<String, PreparedMcpServer> {
        &self.servers
    }

    /// Named availability failures to surface in the run's diagnostics.
    pub fn warnings(&self) -> &[McpPreflightWarning] {
        &self.warnings
    }

    /// Dispatch an already-governed name through its exact prepared binding.
    /// No legacy/global registry lookup or name-derived server identity occurs.
    /// The host must also restrict the name to its frozen model-visible step.
    ///
    /// # Errors
    /// Rejects absent tools/bindings and propagates readiness, generation,
    /// catalog, timeout and transport failures without replaying the call.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let tool = self.projection.tools().get(name).ok_or_else(|| {
            anyhow::anyhow!("MCP tool {name:?} is not in the prepared projection")
        })?;
        let server = self
            .servers
            .get(tool.server().name())
            .ok_or_else(|| anyhow::anyhow!("MCP tool {name:?} has no prepared server binding"))?;
        Ok(server.call_tool(tool, arguments).await?)
    }

    /// Capture concrete, narrowed transports for host-authorized delegation.
    /// Only in-process tools are retained from the companion registry; its
    /// legacy MCP transports and connection recipes cannot enter this grant.
    /// This explicit handoff waits for lazy bindings without changing normal
    /// preflight behavior. The child receives no permission to reconnect.
    ///
    /// # Errors
    /// Fails on readiness, changed discovery, revocation or descriptor collision.
    /// A server that was optional during preflight cannot disappear silently
    /// once its prepared tools are being granted to a child.
    pub async fn freeze_bindings(
        &self,
        companion: &super::registry::McpRegistry,
    ) -> anyhow::Result<super::registry::McpRegistry> {
        let no_servers = std::collections::HashSet::new();
        let mut combined = companion.filtered(Some(&no_servers), None);
        let allowed_tools = self
            .projection
            .tools()
            .keys()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        // Keep all leases through discovery and final revocation checks.
        let mut leases = Vec::with_capacity(self.servers.len());
        for prepared in self.servers.values() {
            let binding = prepared.wait_until_ready().await?;
            let selected = binding.registry()?.filtered(None, Some(&allowed_tools));
            combined = combined.merge(&selected)?;
            leases.push(binding);
        }
        let frozen = combined.freeze_bindings().await?;
        for binding in &leases {
            binding.registry()?;
        }
        frozen.require_bound_servers(self.servers.keys().map(String::as_str))?;
        Ok(frozen)
    }
}

pub(crate) async fn prepare_servers(
    runtime: &McpRuntimeManager,
    projection: &McpServerProjection,
    owner: &ActorOwner,
    inherited: &McpBindingEnvironment,
    observation: Option<(
        &Arc<super::run_events::McpRunEvents>,
        &tokio_util::sync::CancellationToken,
    )>,
) -> Result<McpPreflight, McpPreflightError> {
    let mut servers = BTreeMap::new();
    let mut warnings = Vec::new();
    let mut omitted = BTreeSet::new();
    for (name, definition) in projection.servers() {
        if observation.is_some_and(|(_, token)| token.is_cancelled()) {
            return Err(McpPreflightError::Cancelled);
        }
        // Each override resolves against the same parent snapshot, never the
        // result of another server's overrides or fresh process-global state.
        let environment = McpBindingEnvironment::resolve(
            inherited.directory().to_path_buf(),
            inherited.variables().clone(),
            definition.configuration(),
        );
        let prepared = match environment {
            Ok(environment) => {
                let request = Arc::new(McpBindingRequest::new(
                    owner.clone(),
                    Arc::clone(definition),
                    Arc::new(environment),
                ));
                let outcome = match observation {
                    Some((events, cancellation)) => {
                        runtime
                            .prepare_observed(request, events, cancellation)
                            .await
                    }
                    None => runtime.prepare(request).await,
                };
                match outcome {
                    Ok(server) => Ok(server),
                    Err(_) if observation.is_some_and(|(_, token)| token.is_cancelled()) => {
                        return Err(McpPreflightError::Cancelled);
                    }
                    Err(error) => match availability_failure(&error) {
                        Some(reason) => Err(reason),
                        None => {
                            return Err(McpPreflightError::Interrupted {
                                server: name.clone(),
                                source: error,
                            });
                        }
                    },
                }
            }
            Err(_) => Err(McpServerFailure::Environment),
        };
        match prepared {
            Ok(server) => {
                servers.insert(name.clone(), server);
            }
            Err(reason) if definition.is_required() => {
                return Err(McpPreflightError::RequiredServer {
                    server: name.clone(),
                    reason,
                });
            }
            Err(reason) => {
                tracing::warn!(server = %name, reason = %reason,
                    "Optional MCP server unavailable; its tools are omitted from this step");
                omitted.insert(name.clone());
                warnings.push(McpPreflightWarning {
                    server: name.clone(),
                    reason,
                });
            }
        }
    }
    let projection = projection
        .without_unavailable_optional_servers(&omitted)?
        .with_tools(servers.values().map(|server| server.catalog().clone()))?;
    Ok(McpPreflight {
        owner: owner.clone(),
        projection,
        servers,
        warnings,
    })
}

fn availability_failure(error: &McpRuntimeError) -> Option<McpServerFailure> {
    match error {
        McpRuntimeError::Binding(McpBindingError::AuthenticationRequired { .. }) => {
            Some(McpServerFailure::Authentication)
        }
        McpRuntimeError::Binding(McpBindingError::ConnectionFailed { .. }) => {
            Some(McpServerFailure::Connection)
        }
        McpRuntimeError::Binding(McpBindingError::IncompleteCatalog { .. }) => {
            Some(McpServerFailure::Discovery)
        }
        McpRuntimeError::TimedOut { .. } => Some(McpServerFailure::Timeout),
        // Optional availability is not permission to continue through a stale
        // generation or a connector's identity/catalog invariant violation.
        McpRuntimeError::Binding(
            McpBindingError::InvalidBinding { .. }
            | McpBindingError::Invalidated { .. }
            | McpBindingError::Cancelled { .. }
            | McpBindingError::ShuttingDown,
        )
        | McpRuntimeError::CatalogChanged { .. }
        | McpRuntimeError::ToolNotProjected { .. }
        | McpRuntimeError::ToolFailed { .. } => None,
    }
}
