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
use crate::mcp::config::{McpServerEntry, RemoteHttpGrantPolicy, expand_from_environment};
use crate::mcp::runtime::{
    McpRunResources, McpRuntimeManager, RunHttpHeaders, RunMcpConnection, RunMcpConnector,
    RunMcpCredentialLease, RunMcpGrantControl,
};
use crate::uar::runtime::actor::messages::ActorOwner;

#[derive(Clone, Deserialize)]
pub struct RunMcpServerInput {
    pub name: String,
    #[serde(default)]
    pub url: Option<SecretString>,
    #[serde(default)]
    pub headers: BTreeMap<String, SecretString>,
    #[serde(default)]
    pub grant: Option<RunMcpGrantInput>,
}

#[derive(Clone, Deserialize)]
pub struct RunMcpGrantInput {
    pub credential_revision: String,
    #[serde(default)]
    pub scopes: BTreeSet<String>,
    pub expires_at_unix: u64,
    #[serde(default)]
    pub headers: BTreeMap<String, SecretString>,
}

impl std::fmt::Debug for RunMcpGrantInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunMcpGrantInput")
            .field("credential_revision", &"[REDACTED]")
            .field("scopes", &self.scopes)
            .field("expires_at_unix", &self.expires_at_unix)
            .field("header_names", &self.headers.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Debug for RunMcpServerInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunMcpServerInput")
            .field("name", &self.name)
            .field("url", &self.url.as_ref().map(|_| "[REDACTED]"))
            .field("header_names", &self.headers.keys().collect::<Vec<_>>())
            .field("grant", &self.grant)
            .finish()
    }
}

#[derive(Clone)]
struct RunMcpServer {
    name: String,
    configuration: McpServerEntry,
    authentication: ServerAuthentication,
    secret_values: Vec<SecretString>,
    headers: BTreeMap<String, SecretString>,
    credential: Option<RunMcpCredentialLease>,
    grant_marker: Option<super::HostMcpGrantMarker>,
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
    pub fn from_inputs(
        inputs: Vec<RunMcpServerInput>,
        user: &crate::uar::security::claims::UserContext,
        catalog: &McpCatalog,
        environment: &McpBindingEnvironment,
    ) -> Result<Self, HostInputError> {
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
            servers.push(match (input.url, input.grant) {
                (Some(url), None) => direct_loopback_server(name, url, input.headers)?,
                (None, Some(grant)) if input.headers.is_empty() => {
                    trusted_destination_server(name, grant, user, catalog, environment)?
                }
                _ => {
                    return Err(HostInputError::new(
                        "run_mcp_server_invalid",
                        "MCP server must select one loopback URL or one trusted destination grant",
                    ));
                }
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

    pub(crate) fn grant_markers(&self) -> Vec<super::HostMcpGrantMarker> {
        self.0
            .iter()
            .filter_map(|server| server.grant_marker.clone())
            .collect()
    }

    pub(crate) fn resources(
        &self,
        owner: ActorOwner,
        working_directory: std::path::PathBuf,
        inherited_environment: &McpBindingEnvironment,
    ) -> Result<McpRunResources, HostInputError> {
        let mut definitions = Vec::with_capacity(self.0.len());
        let mut connections = HashMap::new();
        for server in &self.0 {
            definitions.push(
                ServerDefinition::new(
                    server.name.clone(),
                    ServerSource::Global,
                    server.configuration.clone(),
                    true,
                    server.authentication.clone(),
                )
                .map_err(|_| {
                    HostInputError::new(
                        "run_mcp_server_invalid",
                        "MCP server definition is invalid",
                    )
                })?,
            );
            connections.insert(
                server.name.clone(),
                RunMcpConnection::new(server.headers.clone(), server.credential.clone()),
            );
        }
        let catalog = McpCatalog::from_definitions(definitions).map_err(|_| {
            HostInputError::new("run_mcp_server_invalid", "MCP server catalog is invalid")
        })?;
        let environment = McpBindingEnvironment::new(
            working_directory,
            inherited_environment.variables().clone(),
        )
        .map_err(|_| {
            HostInputError::new("working_directory_invalid", "working directory is invalid")
        })?;
        let grants = RunMcpGrantControl::from_connections(&connections);
        let runtime = McpRuntimeManager::new(
            McpBindingCache::default(),
            Arc::new(RunMcpConnector::new(connections, grants.clone())),
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
            grants,
        ))
    }
}

fn direct_loopback_server(
    name: &str,
    url: SecretString,
    headers: BTreeMap<String, SecretString>,
) -> Result<RunMcpServer, HostInputError> {
    let parsed = Url::parse(url.expose_secret())
        .map_err(|_| HostInputError::new("run_mcp_server_invalid", "MCP server URL is invalid"))?;
    if !is_loopback_host(&parsed)
        || parsed.scheme() != "http"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(HostInputError::new(
            "run_mcp_server_invalid",
            "MCP server URL must be loopback HTTP without userinfo",
        ));
    }
    RunHttpHeaders::parse(&headers).map_err(|_| {
        HostInputError::new("run_mcp_server_invalid", "MCP server headers are invalid")
    })?;
    let mut secret_values = vec![url.clone()];
    secret_values.extend(headers.values().cloned());
    Ok(RunMcpServer {
        name: name.to_owned(),
        configuration: McpServerEntry::RemoteHttp {
            url: url.expose_secret().to_owned(),
            env: HashMap::new(),
            headers: HashMap::new(),
            grant_policy: None,
        },
        authentication: ServerAuthentication::Authenticated {
            binding_id: uuid::Uuid::new_v4().to_string(),
        },
        secret_values,
        headers,
        credential: None,
        grant_marker: None,
    })
}

fn trusted_destination_server(
    name: &str,
    grant: RunMcpGrantInput,
    user: &crate::uar::security::claims::UserContext,
    catalog: &McpCatalog,
    environment: &McpBindingEnvironment,
) -> Result<RunMcpServer, HostInputError> {
    let host_id = trusted_host_id(user).ok_or_else(|| {
        HostInputError::new(
            "run_mcp_grant_forbidden",
            "remote MCP grants require an authenticated trusted host",
        )
    })?;
    let definition = catalog
        .candidates(name)
        .find(|definition| matches!(definition.source(), ServerSource::Global))
        .ok_or_else(|| {
            HostInputError::new(
                "run_mcp_destination_unregistered",
                "remote MCP destination is not administrator registered",
            )
        })?;
    let McpServerEntry::RemoteHttp {
        url, grant_policy, ..
    } = definition.configuration()
    else {
        return Err(HostInputError::new(
            "run_mcp_destination_invalid",
            "trusted MCP destination must use remote HTTP",
        ));
    };
    let policy = grant_policy.as_ref().ok_or_else(|| {
        HostInputError::new(
            "run_mcp_grant_forbidden",
            "remote MCP destination does not accept run grants",
        )
    })?;
    validate_grant_policy(host_id, &grant, policy)?;
    let resolved_environment = McpBindingEnvironment::resolve(
        environment.directory().to_path_buf(),
        environment.variables().clone(),
        definition.configuration(),
    )
    .map_err(|_| {
        HostInputError::new(
            "run_mcp_destination_invalid",
            "remote MCP destination environment is unavailable",
        )
    })?;
    let expanded_url =
        expand_from_environment(url, resolved_environment.variables()).map_err(|_| {
            HostInputError::new(
                "run_mcp_destination_invalid",
                "remote MCP destination URL is unavailable",
            )
        })?;
    validate_remote_destination(&expanded_url, policy)?;
    RunHttpHeaders::from_configuration(
        definition.configuration(),
        &resolved_environment,
        Some(&grant.headers),
    )
    .map_err(|_| {
        HostInputError::new(
            "run_mcp_server_invalid",
            "MCP destination and grant headers are invalid or conflict",
        )
    })?;
    let mut secret_values = vec![SecretString::from(expanded_url)];
    secret_values.extend(grant.headers.values().cloned());
    let owner = ActorOwner::from_verified_context(user).map_err(|_| {
        HostInputError::new(
            "run_mcp_grant_forbidden",
            "remote MCP grant owner is invalid",
        )
    })?;
    let credential = RunMcpCredentialLease::new(grant.expires_at_unix);
    let grant_marker = super::HostMcpGrantMarker {
        server: name.to_owned(),
        destination_id: policy.destination_id.clone(),
        trusted_host: host_id.to_owned(),
        scopes: grant.scopes.clone(),
        credential_revision: grant.credential_revision.clone(),
        expires_at_unix: grant.expires_at_unix,
    };
    Ok(RunMcpServer {
        name: name.to_owned(),
        configuration: definition.configuration().clone(),
        authentication: ServerAuthentication::Authenticated {
            binding_id: format!(
                "run-grant:{host_id}:{}:{name}:{}",
                owner.presentation_owner_key(),
                grant.credential_revision
            ),
        },
        secret_values,
        headers: grant.headers,
        credential: Some(credential),
        grant_marker: Some(grant_marker),
    })
}

fn trusted_host_id(user: &crate::uar::security::claims::UserContext) -> Option<&str> {
    let roles = user.claims.roles.as_deref().unwrap_or_default();
    if roles.iter().any(|role| role == "host-session") {
        return Some("sidecar-launch-host");
    }
    roles
        .iter()
        .any(|role| role == "uar:mcp:delegate")
        .then(|| user.claims.uar_instance_id.as_deref())
        .flatten()
}

fn validate_grant_policy(
    host_id: &str,
    grant: &RunMcpGrantInput,
    policy: &RemoteHttpGrantPolicy,
) -> Result<(), HostInputError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(u64::MAX);
    if !policy.trusted_hosts.contains(host_id)
        || grant.credential_revision.trim().is_empty()
        || grant.credential_revision.len() > 128
        || !grant
            .credential_revision
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
        || !policy.required_scopes.is_subset(&grant.scopes)
    {
        return Err(HostInputError::new(
            "run_mcp_grant_forbidden",
            "remote MCP grant is not valid for this host, destination, scope, or time",
        ));
    }
    if grant.expires_at_unix <= now {
        return Err(HostInputError::new(
            "run_mcp_grant_authentication_required",
            "remote MCP grant is expired and must be renewed",
        ));
    }
    if grant.scopes.iter().any(|scope| scope.trim().is_empty()) {
        return Err(HostInputError::new(
            "run_mcp_grant_forbidden",
            "remote MCP grant contains an invalid scope",
        ));
    }
    Ok(())
}

fn validate_remote_destination(
    raw_url: &str,
    policy: &RemoteHttpGrantPolicy,
) -> Result<(), HostInputError> {
    let url = Url::parse(raw_url).map_err(|_| {
        HostInputError::new(
            "run_mcp_destination_invalid",
            "remote MCP destination URL is invalid",
        )
    })?;
    let private_development = policy.allow_private_http && is_private_host(&url);
    if (url.scheme() != "https" && !(url.scheme() == "http" && private_development))
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(HostInputError::new(
            "run_mcp_destination_invalid",
            "remote MCP destination requires HTTPS or authorized private development HTTP",
        ));
    }
    Ok(())
}

fn is_private_host(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address.is_loopback() || address.is_private(),
        Some(Host::Ipv6(address)) => address.is_loopback() || address.is_unique_local(),
        None => false,
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
