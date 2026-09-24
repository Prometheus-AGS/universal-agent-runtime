use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use url::{Host, Url};

use super::HostInputError;
use crate::mcp::binding_cache::{McpBindingCache, McpBindingEnvironment};
use crate::mcp::catalog::{McpCatalog, ServerAuthentication, ServerDefinition, ServerSource};
use crate::mcp::config::McpServerEntry;
use crate::mcp::runtime::{McpRunResources, McpRuntimeManager, RunHttpHeaders, RunMcpConnector};
use crate::uar::runtime::actor::messages::ActorOwner;

#[derive(Clone, Deserialize)]
pub struct RunMcpServerInput {
    pub name: String,
    pub url: SecretString,
    #[serde(default)]
    pub headers: BTreeMap<String, SecretString>,
}

impl std::fmt::Debug for RunMcpServerInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunMcpServerInput")
            .field("name", &self.name)
            .field("url", &"[REDACTED]")
            .field("header_names", &self.headers.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Clone)]
struct RunMcpServer {
    name: String,
    url: SecretString,
    secret_values: Vec<SecretString>,
    headers: RunHttpHeaders,
}

/// Complete request-owned MCP universe. It deliberately has no Serialize impl.
#[derive(Clone, Default)]
pub struct RunMcpServers(Vec<RunMcpServer>);

impl std::fmt::Debug for RunMcpServers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunMcpServers")
            .field("server_names", &self.names())
            .finish()
    }
}

impl RunMcpServers {
    pub fn from_inputs(inputs: Vec<RunMcpServerInput>) -> Result<Self, HostInputError> {
        if inputs.is_empty() {
            return Err(HostInputError::new(
                "run_mcp_server_invalid",
                "mcp_servers must not be empty",
            ));
        }
        let mut names = BTreeSet::new();
        let mut servers = Vec::with_capacity(inputs.len());
        for input in inputs {
            let name = input.name.trim();
            if name.is_empty() || !names.insert(name.to_owned()) {
                return Err(HostInputError::new(
                    "run_mcp_server_invalid",
                    "MCP server names must be non-empty and unique",
                ));
            }
            let url = Url::parse(input.url.expose_secret()).map_err(|_| {
                HostInputError::new("run_mcp_server_invalid", "MCP server URL is invalid")
            })?;
            if !is_loopback_host(&url)
                || url.scheme() != "http"
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err(HostInputError::new(
                    "run_mcp_server_invalid",
                    "MCP server URL must be loopback HTTP without userinfo",
                ));
            }
            let headers = RunHttpHeaders::parse(&input.headers).map_err(|_| {
                HostInputError::new("run_mcp_server_invalid", "MCP server headers are invalid")
            })?;
            let mut secret_values = vec![input.url.clone()];
            secret_values.extend(input.headers.values().cloned());
            servers.push(RunMcpServer {
                name: name.to_owned(),
                url: input.url,
                secret_values,
                headers,
            });
        }
        Ok(Self(servers))
    }

    #[must_use]
    pub fn names(&self) -> BTreeSet<String> {
        self.0.iter().map(|server| server.name.clone()).collect()
    }

    pub(crate) fn scrubber(&self) -> super::RunSecretScrubber {
        super::RunSecretScrubber::from_values(
            self.0
                .iter()
                .flat_map(|server| server.secret_values.iter().cloned())
                .collect(),
        )
    }

    pub(crate) fn resources(
        &self,
        owner: ActorOwner,
        working_directory: std::path::PathBuf,
    ) -> Result<McpRunResources, HostInputError> {
        let mut definitions = Vec::with_capacity(self.0.len());
        let mut headers = HashMap::new();
        for server in &self.0 {
            let configuration = McpServerEntry::RemoteHttp {
                url: server.url.expose_secret().to_owned(),
                env: HashMap::new(),
            };
            let authentication = ServerAuthentication::Authenticated {
                binding_id: uuid::Uuid::new_v4().to_string(),
            };
            definitions.push(
                ServerDefinition::new(
                    server.name.clone(),
                    ServerSource::Global,
                    configuration,
                    true,
                    authentication,
                )
                .map_err(|_| {
                    HostInputError::new(
                        "run_mcp_server_invalid",
                        "MCP server definition is invalid",
                    )
                })?,
            );
            headers.insert(server.name.clone(), server.headers.clone());
        }
        let catalog = McpCatalog::from_definitions(definitions).map_err(|_| {
            HostInputError::new("run_mcp_server_invalid", "MCP server catalog is invalid")
        })?;
        let environment =
            McpBindingEnvironment::new(working_directory, BTreeMap::new()).map_err(|_| {
                HostInputError::new("working_directory_invalid", "working directory is invalid")
            })?;
        let runtime = McpRuntimeManager::new(
            McpBindingCache::default(),
            Arc::new(RunMcpConnector::new(headers)),
            Duration::from_secs(15),
            Duration::from_secs(60),
        )
        .map_err(|_| {
            HostInputError::new(
                "run_mcp_server_invalid",
                "MCP runtime configuration is invalid",
            )
        })?;
        Ok(McpRunResources::new_run_scoped(
            owner,
            runtime,
            Arc::new(catalog),
            Arc::new(environment),
            self.names(),
        ))
    }
}

pub(super) fn is_loopback_host(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => IpAddr::V4(address).is_loopback(),
        Some(Host::Ipv6(address)) => IpAddr::V6(address).is_loopback(),
        None => false,
    }
}
