use std::collections::{BTreeMap, BTreeSet};

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use url::Url;

use super::HostInputError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunProviderKind {
    OpenaiCompatible,
    Anthropic,
}

impl RunProviderKind {
    const fn protocol(self) -> &'static str {
        match self {
            Self::OpenaiCompatible => "openai_compatible",
            Self::Anthropic => "anthropic",
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct RunCredentialInput {
    pub provider_id: String,
    pub provider_kind: String,
    pub base_url: SecretString,
    pub api_key: SecretString,
    #[serde(default)]
    pub default_model: Option<String>,
}

impl std::fmt::Debug for RunCredentialInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunCredentialInput")
            .field("provider_id", &self.provider_id)
            .field("provider_kind", &self.provider_kind)
            .field("base_url", &"[REDACTED]")
            .field("api_key", &"[REDACTED]")
            .field("default_model", &self.default_model)
            .finish()
    }
}

#[derive(Clone)]
struct RunCredential {
    provider_id: String,
    provider_kind: RunProviderKind,
    base_url: SecretString,
    api_key: SecretString,
    default_model: Option<String>,
}

/// Complete run-local provider set. It deliberately has no Serialize impl.
#[derive(Clone, Default)]
pub struct RunCredentials(BTreeMap<String, RunCredential>);

impl std::fmt::Debug for RunCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunCredentials")
            .field("provider_ids", &self.provider_ids())
            .finish()
    }
}

impl RunCredentials {
    pub fn from_inputs(inputs: Vec<RunCredentialInput>) -> Result<Self, HostInputError> {
        if inputs.is_empty() {
            return Err(HostInputError::new(
                "run_credential_invalid",
                "run_credentials must not be empty",
            ));
        }
        let mut credentials = BTreeMap::new();
        for input in inputs {
            let provider_id = input.provider_id.trim();
            if provider_id.is_empty()
                || provider_id.contains('/')
                || input.api_key.expose_secret().is_empty()
            {
                return Err(HostInputError::new(
                    "run_credential_invalid",
                    "run credential provider, endpoint, or key is invalid",
                ));
            }
            validate_endpoint(input.base_url.expose_secret())?;
            let provider_kind = match input.provider_kind.as_str() {
                "openai_compatible" => RunProviderKind::OpenaiCompatible,
                "anthropic" => RunProviderKind::Anthropic,
                _ => {
                    return Err(HostInputError::new(
                        "run_credential_provider_kind_unsupported",
                        "run credential provider kind is unsupported",
                    ));
                }
            };
            if input
                .default_model
                .as_deref()
                .is_some_and(|model| model.trim().is_empty() || model.contains('/'))
            {
                return Err(HostInputError::new(
                    "run_credential_invalid",
                    "run credential default_model is invalid",
                ));
            }
            let credential = RunCredential {
                provider_id: provider_id.to_owned(),
                provider_kind,
                base_url: input.base_url,
                api_key: input.api_key,
                default_model: input.default_model,
            };
            if credentials
                .insert(provider_id.to_owned(), credential)
                .is_some()
            {
                return Err(HostInputError::new(
                    "run_credential_invalid",
                    "run credential provider ids must be unique",
                ));
            }
        }
        Ok(Self(credentials))
    }

    #[must_use]
    pub fn provider_ids(&self) -> BTreeSet<String> {
        self.0.keys().cloned().collect()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn contains(&self, provider_id: &str) -> bool {
        self.0.contains_key(provider_id)
    }

    pub(crate) fn config_for(
        &self,
        provider_id: &str,
        requested_model: Option<&str>,
        mut base: crate::config::LlmConfig,
    ) -> Result<crate::config::LlmConfig, HostInputError> {
        let credential = self.0.get(provider_id).ok_or_else(|| {
            HostInputError::new(
                "run_credential_provider_unavailable",
                "selected provider has no run credential",
            )
        })?;
        let model = requested_model
            .filter(|model| !model.trim().is_empty())
            .map(str::to_owned)
            .or_else(|| credential.default_model.clone())
            .ok_or_else(|| {
                HostInputError::new(
                    "run_credential_invalid",
                    "selected provider requires a model",
                )
            })?;
        let model = model
            .rsplit_once('/')
            .map_or(model.as_str(), |(_, model)| model)
            .to_owned();
        base.model = model;
        base.resolved_provider_id = Some(credential.provider_id.clone());
        base.host_provider_kind = Some(credential.provider_kind.protocol().to_owned());
        base.host_supplied_connection = true;
        base.api_key = Some(credential.api_key.expose_secret().to_owned());
        base.api_key_env = None;
        base.base_url = Some(credential.base_url.expose_secret().to_owned());
        base.provider_keys.clear();
        Ok(base)
    }

    #[must_use]
    pub(crate) fn scrub(&self, text: &str) -> String {
        self.scrubber().scrub(text)
    }

    pub(crate) fn scrubber(&self) -> super::RunSecretScrubber {
        super::RunSecretScrubber::from_values(
            self.0
                .values()
                .flat_map(|credential| [credential.api_key.clone(), credential.base_url.clone()])
                .collect(),
        )
    }
}

fn validate_endpoint(value: &str) -> Result<(), HostInputError> {
    let url = Url::parse(value).map_err(|_| {
        HostInputError::new(
            "run_credential_invalid",
            "run credential endpoint is invalid",
        )
    })?;
    if !url.username().is_empty() || url.password().is_some() || url.host_str().is_none() {
        return Err(HostInputError::new(
            "run_credential_invalid",
            "run credential endpoint is invalid",
        ));
    }
    let allowed =
        url.scheme() == "https" || (url.scheme() == "http" && super::mcp::is_loopback_host(&url));
    if !allowed {
        return Err(HostInputError::new(
            "run_credential_invalid",
            "run credential endpoint must use HTTPS or loopback HTTP",
        ));
    }
    if crate::llm::local_only::enabled()
        && !url
            .host_str()
            .is_some_and(crate::llm::local_only::host_is_local)
    {
        return Err(HostInputError::new(
            "run_credential_invalid",
            "run credential endpoint is unavailable in local-only mode",
        ));
    }
    Ok(())
}
