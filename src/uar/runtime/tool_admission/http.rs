//! Authenticated loopback adapter for a paired host's private admission route.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use url::{Host, Url};

use super::{
    AdmissionCancellationOutcome, AdmissionCancellationReason, AdmissionTerminalOutcome,
    AdmittedToolInvocation, HostAdmissionBinding, HostAdmissionPreparation, HostAdmissionReceipt,
    HostToolAdmissionPort, LocalAdmissionDisposition, PreparedToolInvocation,
    TOOL_ADMISSION_PROTOCOL_VERSION,
};

/// Request-owned paired-host admission endpoint. Secret fields never implement
/// `Serialize` or expose their values through `Debug`.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunToolAdmissionInput {
    pub version: u32,
    pub host_epoch: String,
    pub url: SecretString,
    #[serde(default)]
    pub headers: BTreeMap<String, SecretString>,
}

impl std::fmt::Debug for RunToolAdmissionInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunToolAdmissionInput")
            .field("version", &self.version)
            .field("host_epoch", &self.host_epoch)
            .field("url", &"[REDACTED]")
            .field("header_names", &self.headers.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// HTTP implementation of the private admission port. It is scoped to one
/// run-owned Boss bridge and shares that bridge's bearer authentication.
pub struct HttpHostToolAdmissionPort {
    binding: HostAdmissionBinding,
    prepare_url: Url,
    resolve_url: Url,
    cancel_url: Url,
    finish_url: Url,
    headers: HeaderMap,
    client: reqwest::Client,
}

impl std::fmt::Debug for HttpHostToolAdmissionPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpHostToolAdmissionPort")
            .field("binding", &self.binding)
            .field("endpoint", &"[REDACTED]")
            .field("header_names", &self.headers.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl HttpHostToolAdmissionPort {
    /// Validate one authenticated loopback binding without retaining input
    /// secrets in serializable state.
    pub fn from_input(input: RunToolAdmissionInput) -> anyhow::Result<Self> {
        anyhow::ensure!(
            input.version == TOOL_ADMISSION_PROTOCOL_VERSION,
            "Paired host does not support tool admission protocol v1"
        );
        anyhow::ensure!(
            !input.host_epoch.trim().is_empty() && input.host_epoch.len() <= 128,
            "Paired host admission epoch is invalid"
        );
        let base = Url::parse(input.url.expose_secret())?;
        anyhow::ensure!(
            base.scheme() == "http"
                && base.username().is_empty()
                && base.password().is_none()
                && base.query().is_none()
                && base.fragment().is_none()
                && is_loopback(&base)
                && base.path().trim_end_matches('/') == "/uar/admission/v1",
            "Paired host admission URL must be the private loopback v1 route"
        );
        let mut headers = HeaderMap::new();
        for (name, value) in input.headers {
            let name = HeaderName::from_bytes(name.as_bytes())?;
            anyhow::ensure!(
                name == reqwest::header::AUTHORIZATION,
                "Paired host admission accepts only its authorization header"
            );
            headers.insert(name, HeaderValue::from_str(value.expose_secret())?);
        }
        anyhow::ensure!(
            headers.contains_key(reqwest::header::AUTHORIZATION),
            "Paired host admission requires authorization"
        );
        let mut prepare_url = base.clone();
        prepare_url.set_path("/uar/admission/v1/prepare");
        let mut resolve_url = base.clone();
        resolve_url.set_path("/uar/admission/v1/resolve");
        let mut cancel_url = base;
        cancel_url.set_path("/uar/admission/v1/cancel");
        let mut finish_url = cancel_url.clone();
        finish_url.set_path("/uar/admission/v1/finish");
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            binding: HostAdmissionBinding {
                version: input.version,
                host_epoch: input.host_epoch,
            },
            prepare_url,
            resolve_url,
            cancel_url,
            finish_url,
            headers,
            client,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrepareRequest<'a> {
    invocation: &'a PreparedToolInvocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResolveRequest<'a> {
    admission_id: &'a str,
    invocation_id: &'a str,
    local_disposition: LocalAdmissionDisposition,
    approved: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CancelRequest<'a> {
    admission_id: &'a str,
    invocation_id: &'a str,
    reason: AdmissionCancellationReason,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FinishRequest<'a> {
    admission_id: &'a str,
    invocation_id: &'a str,
    outcome: AdmissionTerminalOutcome,
}

#[async_trait]
impl HostToolAdmissionPort for HttpHostToolAdmissionPort {
    fn binding(&self) -> HostAdmissionBinding {
        self.binding.clone()
    }

    async fn prepare(
        &self,
        invocation: Arc<PreparedToolInvocation>,
    ) -> anyhow::Result<HostAdmissionPreparation> {
        let response = self
            .client
            .post(self.prepare_url.clone())
            .headers(self.headers.clone())
            .json(&PrepareRequest {
                invocation: invocation.as_ref(),
            })
            .send()
            .await?;
        anyhow::ensure!(
            response.status().is_success(),
            "Paired host rejected tool admission (HTTP {})",
            response.status().as_u16()
        );
        let preparation = response.json::<HostAdmissionPreparation>().await?;
        anyhow::ensure!(
            preparation.managed_mcp_metadata,
            "Paired host returned an unmanaged admission preparation"
        );
        Ok(preparation)
    }

    async fn resolve(
        &self,
        invocation: Arc<PreparedToolInvocation>,
        preparation: HostAdmissionPreparation,
        local_disposition: LocalAdmissionDisposition,
        approved: bool,
    ) -> anyhow::Result<Option<AdmittedToolInvocation>> {
        let response = self
            .client
            .post(self.resolve_url.clone())
            .headers(self.headers.clone())
            .json(&ResolveRequest {
                admission_id: &preparation.admission_id,
                invocation_id: &preparation.invocation_id,
                local_disposition,
                approved,
            })
            .send()
            .await?;
        anyhow::ensure!(
            response.status().is_success(),
            "Paired host rejected tool admission resolution (HTTP {})",
            response.status().as_u16()
        );
        if !approved {
            return Ok(None);
        }
        let receipt = response.json::<HostAdmissionReceipt>().await?;
        anyhow::ensure!(
            receipt.managed_mcp_metadata,
            "Paired host returned an unmanaged admission receipt"
        );
        Ok(Some(AdmittedToolInvocation {
            prepared: invocation,
            local_disposition,
            host_receipt: receipt,
        }))
    }

    async fn cancel(
        &self,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
        reason: AdmissionCancellationReason,
    ) -> anyhow::Result<AdmissionCancellationOutcome> {
        let response = self
            .client
            .post(self.cancel_url.clone())
            .headers(self.headers.clone())
            .json(&CancelRequest {
                admission_id,
                invocation_id: &invocation.invocation_id,
                reason,
            })
            .send()
            .await?;
        anyhow::ensure!(
            response.status().is_success(),
            "Paired host rejected tool admission cancellation (HTTP {})",
            response.status().as_u16()
        );
        Ok(response.json::<AdmissionCancellationOutcome>().await?)
    }

    async fn finish(
        &self,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
        outcome: AdmissionTerminalOutcome,
    ) -> anyhow::Result<()> {
        let response = self
            .client
            .post(self.finish_url.clone())
            .headers(self.headers.clone())
            .json(&FinishRequest {
                admission_id,
                invocation_id: &invocation.invocation_id,
                outcome,
            })
            .send()
            .await?;
        anyhow::ensure!(
            response.status().is_success(),
            "Paired host rejected tool admission terminal receipt (HTTP {})",
            response.status().as_u16()
        );
        Ok(())
    }
}

fn is_loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    }
}
