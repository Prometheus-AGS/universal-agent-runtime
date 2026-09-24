//! Non-persistable resources admitted from a trusted embedding host.

mod credentials;
mod history;
mod mcp;

pub use credentials::{RunCredentialInput, RunCredentials, RunProviderKind};
pub use history::{HistorySeedStatus, HostHistoryInput};
pub use mcp::{RunMcpServerInput, RunMcpServers};

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
    #[serde(default)]
    pub artifact_inline: bool,
}

impl HostResourcesMarker {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.credential_providers.is_empty() && self.mcp_servers.is_empty()
    }
}
