//! Redaction-only data from an admitted request; never authentication authority.

use crate::uar::runtime::turn::host::RunSecretScrubber;

/// Finite, in-memory credential contribution. It cannot be deserialized or used
/// to obtain identity, host authority, a transport, or an executable credential.
#[derive(Clone, Debug, Default)]
pub struct AuthenticatedCredentialCapture(RunSecretScrubber);

impl AuthenticatedCredentialCapture {
    pub(crate) fn bearer(token: &str) -> Self {
        Self(RunSecretScrubber::from_values(vec![
            token.to_owned().into(),
            format!("Bearer {token}").into(),
        ]))
    }

    pub(crate) fn api_key(key: &str) -> Self {
        Self(RunSecretScrubber::from_values(vec![key.to_owned().into()]))
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    pub(crate) fn into_scrubber(self) -> RunSecretScrubber {
        self.0
    }
}
