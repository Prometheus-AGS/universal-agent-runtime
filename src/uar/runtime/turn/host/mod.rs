//! Non-persistable resources admitted from a trusted embedding host.

mod credentials;
mod history;
mod mcp;

pub use credentials::{RunCredentialInput, RunCredentials, RunProviderKind};
pub use history::{HistorySeedStatus, HostHistoryInput};
pub use mcp::{RunMcpServerInput, RunMcpServers};

/// Secret-free identity of one administrator-registered run grant. Persisted
/// run markers use this to admit a same-owner renewal without retaining URLs,
/// headers, credential bytes, or a sensitive configuration hash.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HostMcpGrantMarker {
    pub server: String,
    pub destination_id: String,
    pub trusted_host: String,
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub scopes: std::collections::BTreeSet<String>,
    pub credential_revision: String,
    pub expires_at_unix: u64,
}

impl HostMcpGrantMarker {
    /// Renewal can rotate or shorten a credential while preserving its host,
    /// destination and authority. Extending expiry requires a new revision.
    pub(crate) fn accepts_renewal(&self, renewed: &Self) -> bool {
        self.server == renewed.server
            && self.destination_id == renewed.destination_id
            && self.trusted_host == renewed.trusted_host
            && renewed.scopes.is_subset(&self.scopes)
            && (renewed.credential_revision != self.credential_revision
                || renewed.expires_at_unix <= self.expires_at_unix)
    }
}

/// Exact in-memory values removed from errors before they reach events or logs.
/// This type has no Serialize implementation and its Debug output is redacted.
#[derive(Clone, Default)]
pub struct RunSecretScrubber(Vec<secrecy::SecretString>);

impl std::fmt::Debug for RunSecretScrubber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RunSecretScrubber([redacted])")
    }
}

impl RunSecretScrubber {
    pub(crate) fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    pub(crate) fn from_values(values: Vec<secrecy::SecretString>) -> Self {
        Self(values)
    }

    #[must_use]
    pub fn scrub(&self, text: &str) -> String {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        use secrecy::ExposeSecret;
        let mut scrubbed = text.to_owned();
        for value in &self.0 {
            let value = value.expose_secret();
            if value.is_empty() {
                continue;
            }
            scrubbed = scrubbed.replace(value, "[REDACTED]");
            scrubbed = scrubbed.replace(&STANDARD.encode(value), "[REDACTED]");
        }
        scrubbed
    }
}

/// Stable API error returned before a run is admitted. Messages never include
/// request-owned secret values.
#[derive(Debug, Clone)]
pub struct HostInputError {
    pub code: &'static str,
    pub message: &'static str,
}

impl HostInputError {
    pub const fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }
}

impl std::fmt::Display for HostInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message)
    }
}

impl std::error::Error for HostInputError {}

/// Safe marker retained in the run record so continuation routes can require
/// the host to reattach resources without retaining any secret material.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HostResourcesMarker {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub credential_providers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_grants: Vec<HostMcpGrantMarker>,
    #[serde(default)]
    pub artifact_inline: bool,
}

impl HostResourcesMarker {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.credential_providers.is_empty()
            && self.mcp_servers.is_empty()
            && self.mcp_grants.is_empty()
    }
}
