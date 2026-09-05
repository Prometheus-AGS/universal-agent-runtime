//! Immutable host-owned MCP definitions, separate from executable connections.
//!
//! The host assigns source and authentication metadata; neither is accepted
//! from a server's tool annotations. Multiple sources may declare one name.
//! A projection must resolve those candidates by authority before binding any
//! connection. Catalog construction never connects, expands environment
//! variables, provisions a command, or grants access to a server.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::Arc;

use super::config::McpServerEntry;
use sha2::{Digest, Sha256};

/// Trust order for server declarations, from lowest to highest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServerAuthority {
    /// A declaration contributed by an activated skill.
    Skill,
    /// Operator configuration owned by the runtime host.
    Global,
}

/// Provenance assigned by the host, not a declaration's self-reported rank.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ServerSource {
    /// The operator's global configuration, including persisted administration.
    Global,
    /// A declaration from this skill's installed configuration.
    Skill { skill_id: String },
}

impl ServerSource {
    /// Derive authority from provenance so the two cannot contradict each other.
    #[must_use]
    pub const fn authority(&self) -> ServerAuthority {
        match self {
            Self::Global => ServerAuthority::Global,
            Self::Skill { .. } => ServerAuthority::Skill,
        }
    }
}

/// Host-observed authentication metadata, never a credential or access token.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ServerAuthentication {
    /// No authentication result is available yet; do not assume anonymous access.
    Unknown,
    /// The host has established that this binding requires no credentials.
    NotRequired,
    /// Authentication must complete before this server can be used.
    Required,
    /// An opaque host credential-revision identity, not its secret material.
    Authenticated { binding_id: String },
}

impl fmt::Debug for ServerAuthentication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unknown => "Unknown",
            Self::NotRequired => "NotRequired",
            Self::Required => "Required",
            Self::Authenticated { .. } => "Authenticated { binding_id: [redacted] }",
        })
    }
}

/// Sandbox requirement derived from the launch configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerSandboxPolicy {
    /// Remote HTTP has no local child process to sandbox.
    NotApplicable,
    /// The operator did not request isolation for the stdio process.
    Unrestricted,
    /// An OS-backed launcher is required; currently rejected at admission.
    Required,
}

/// Opaque identity of the declared transport and launch inputs.
///
/// This is not an environment snapshot or an authentication identity. Binding
/// keys must include those separately. The digest includes configured secret
/// values, so it deliberately has no serializable or printable representation.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ServerConfigHash(String);

impl fmt::Debug for ServerConfigHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ServerConfigHash([redacted])")
    }
}

/// One immutable declaration; its configuration is available only by reference.
#[derive(Clone)]
pub struct ServerDefinition {
    name: String,
    source: ServerSource,
    configuration: McpServerEntry,
    config_hash: ServerConfigHash,
    required: bool,
    authentication: ServerAuthentication,
}

impl ServerDefinition {
    /// Capture a declaration and its host-assigned metadata without doing I/O.
    ///
    /// # Errors
    /// Rejects empty identities and unsupported sandbox requests. In particular,
    /// a disabled or not-yet-connected declaration cannot hide an inert sandbox.
    pub fn new(
        name: String,
        source: ServerSource,
        configuration: McpServerEntry,
        required: bool,
        authentication: ServerAuthentication,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(!name.trim().is_empty(), "MCP server name is required");
        if let ServerSource::Skill { skill_id } = &source {
            anyhow::ensure!(
                !skill_id.trim().is_empty(),
                "MCP server {name:?} requires a source skill identity"
            );
        }
        if let ServerAuthentication::Authenticated { binding_id } = &authentication {
            anyhow::ensure!(
                !binding_id.trim().is_empty(),
                "MCP server {name:?} requires a credential binding identity"
            );
        }
        configuration.validate_sandbox_policy(&name)?;
        let config_hash = hash_configuration(&configuration);
        Ok(Self {
            name,
            source,
            configuration,
            config_hash,
            required,
            authentication,
        })
    }

    /// Source-local server name, before provider tool namespacing.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Host-assigned origin of this declaration.
    #[must_use]
    pub fn source(&self) -> &ServerSource {
        &self.source
    }

    /// Authority is always derived from the captured source.
    #[must_use]
    pub fn authority(&self) -> ServerAuthority {
        self.source.authority()
    }

    /// Trusted launch input; may contain secrets and must not enter telemetry.
    #[must_use]
    pub fn configuration(&self) -> &McpServerEntry {
        &self.configuration
    }

    /// Stable declared-input identity, independent of source and map order.
    #[must_use]
    pub fn config_hash(&self) -> &ServerConfigHash {
        &self.config_hash
    }

    /// Whether failure to bind this server must abort the requesting preflight.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.required
    }

    /// Host authentication observation captured with this definition.
    #[must_use]
    pub fn authentication(&self) -> &ServerAuthentication {
        &self.authentication
    }

    /// Derive the policy from the same configuration that supplies the hash.
    #[must_use]
    pub const fn sandbox_policy(&self) -> ServerSandboxPolicy {
        match &self.configuration {
            McpServerEntry::Stdio {
                sandboxed: true, ..
            } => ServerSandboxPolicy::Required,
            McpServerEntry::Stdio {
                sandboxed: false, ..
            } => ServerSandboxPolicy::Unrestricted,
            McpServerEntry::RemoteHttp { .. } => ServerSandboxPolicy::NotApplicable,
        }
    }

    fn equivalent_to(&self, other: &Self) -> bool {
        self.name == other.name
            && self.source == other.source
            && self.config_hash == other.config_hash
            && self.required == other.required
            && self.authentication == other.authentication
    }
}

impl fmt::Debug for ServerDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerDefinition")
            .field("name", &self.name)
            .field("source", &self.source)
            .field("required", &self.required)
            .field("authentication", &self.authentication)
            .field("sandbox_policy", &self.sandbox_policy())
            .finish_non_exhaustive()
    }
}

/// Immutable, deterministically ordered candidates, without connection state.
///
/// # Examples
/// Construct global and skill definitions with their actual host provenance,
/// then pass them to `from_definitions`. Use `candidates` during projection;
/// their presence alone is not permission to start or invoke a server.
#[derive(Clone, Default)]
pub struct McpCatalog {
    definitions: BTreeMap<String, BTreeMap<ServerSource, Arc<ServerDefinition>>>,
}

impl McpCatalog {
    /// Freeze all declarations, coalescing identical repeats from one source.
    ///
    /// # Errors
    /// Conflicting declarations for one name from the same source are rejected
    /// instead of choosing a winner based on arrival order. Distinct sources
    /// remain separate candidates for authority-aware projection.
    pub fn from_definitions(
        definitions: impl IntoIterator<Item = ServerDefinition>,
    ) -> anyhow::Result<Self> {
        let mut catalog = Self::default();
        for definition in definitions {
            let candidates = catalog
                .definitions
                .entry(definition.name.clone())
                .or_default();
            if let Some(existing) = candidates.get(&definition.source) {
                anyhow::ensure!(
                    existing.equivalent_to(&definition),
                    "conflicting MCP definitions for server {:?} from {:?}",
                    definition.name,
                    definition.source,
                );
            } else {
                candidates.insert(definition.source.clone(), Arc::new(definition));
            }
        }
        Ok(catalog)
    }

    /// Distinct server names in stable order, without selecting a declaration.
    pub fn server_names(&self) -> impl Iterator<Item = &str> {
        self.definitions.keys().map(String::as_str)
    }

    /// All origins for a name; the caller must apply authority and run policy.
    pub fn candidates(&self, name: &str) -> impl Iterator<Item = &Arc<ServerDefinition>> {
        self.definitions
            .get(name)
            .into_iter()
            .flat_map(|sources| sources.values())
    }

    /// Every declaration ordered by server name, then source identity.
    pub fn definitions(&self) -> impl Iterator<Item = &Arc<ServerDefinition>> {
        self.definitions
            .values()
            .flat_map(|sources| sources.values())
    }
}

impl fmt::Debug for McpCatalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpCatalog")
            .field("server_count", &self.definitions.len())
            .field("definition_count", &self.definitions().count())
            .finish_non_exhaustive()
    }
}

fn hash_configuration(configuration: &McpServerEntry) -> ServerConfigHash {
    let mut hasher = Sha256::new();
    hasher.update(b"uar.mcp.server-config.v1\0");
    match configuration {
        McpServerEntry::Stdio {
            command,
            args,
            env,
            sandboxed,
        } => {
            hasher.update([0]);
            hash_field(&mut hasher, command);
            hasher.update((args.len() as u64).to_be_bytes());
            for argument in args {
                hash_field(&mut hasher, argument);
            }
            hash_environment(&mut hasher, env);
            hasher.update([u8::from(*sandboxed)]);
        }
        McpServerEntry::RemoteHttp { url, env } => {
            hasher.update([1]);
            hash_field(&mut hasher, url);
            hash_environment(&mut hasher, env);
        }
    }
    ServerConfigHash(
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

fn hash_environment(hasher: &mut Sha256, environment: &HashMap<String, String>) {
    let ordered = environment.iter().collect::<BTreeMap<_, _>>();
    hasher.update((ordered.len() as u64).to_be_bytes());
    for (key, value) in ordered {
        hash_field(hasher, key);
        hash_field(hasher, value);
    }
}

fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}
