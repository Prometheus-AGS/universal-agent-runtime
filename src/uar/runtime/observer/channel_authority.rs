//! UAR obtains current Gate authority as the authenticated execution owner.
//! Incoming route claims and earlier receipts are never an execution permit.

use std::time::Duration;

use reqwest::Url;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_json::Value;

const CONTRACT: &str = "afc.channel-authority/1";
const PROTOCOL: &str = "afc.channel-effect/1";

pub struct ChannelGateClient {
    client: reqwest::Client,
    base: Url,
    bearer: SecretString,
}

impl std::fmt::Debug for ChannelGateClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelGateClient").field("configured", &true).finish()
    }
}

#[derive(Debug, Deserialize)]
struct GateDecision {
    contract: String,
    protocol: String,
    effect_id: uuid::Uuid,
    occurrence_id: String,
    action: String,
    disposition: String,
    receipt_id: uuid::Uuid,
}

impl ChannelGateClient {
    /// The host supplies these only to the sidecar process; they never enter API JSON.
    pub fn from_process_environment() -> Option<Self> {
        let mut base = Url::parse(&std::env::var("UAR_CHANNEL_GATE_URL").ok()?).ok()?;
        let loopback = matches!(base.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
        if base.scheme() != "https" && !(base.scheme() == "http" && loopback) {
            return None;
        }
        let bearer = std::env::var("UAR_CHANNEL_GATE_BEARER_TOKEN").ok()?;
        if bearer.trim().is_empty() || !base.username().is_empty() || base.password().is_some()
            || base.query().is_some() || base.fragment().is_some() {
            return None;
        }
        if !base.path().ends_with('/') {
            let path = format!("{}/", base.path());
            base.set_path(&path);
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build().ok()?;
        Some(Self { client, base, bearer: SecretString::from(bearer) })
    }

    /// Evaluate and revalidate one exact action under Gate's current grant and policy.
    /// A repeated release reports uncertain, never a fresh permit.
    pub async fn release(&self, request: &Value) -> Result<String, &'static str> {
        let effect_id = request.get("effect_id").and_then(Value::as_str).ok_or("channel_effect_invalid")?;
        let occurrence_id = request.get("occurrence_id").and_then(Value::as_str).ok_or("channel_effect_invalid")?;
        let action = request.get("action").and_then(Value::as_str).ok_or("channel_effect_invalid")?;
        let parsed_effect = uuid::Uuid::parse_str(effect_id).map_err(|_| "channel_effect_invalid")?;
        for (stage, expected) in [("evaluate", "eligible"), ("release", "released")] {
            let url = self.base.join(&format!("authority/channels/{stage}")).map_err(|_| "channel_gate_configuration_invalid")?;
            let response = self.client.post(url)
                .bearer_auth(self.bearer.expose_secret())
                .json(request)
                .send().await.map_err(|_| if stage == "release" { "channel_effect_uncertain" } else { "channel_gate_unreachable" })?;
            if !response.status().is_success() {
                return Err(if stage == "release" && response.status().is_server_error() {
                    "channel_effect_uncertain"
                } else if response.status().is_server_error() {
                    "channel_gate_unreachable"
                } else if response.status().as_u16() == 409 {
                    "channel_effect_uncertain"
                } else { "channel_gate_rejected" });
            }
            let decision: GateDecision = response.json().await.map_err(|_| if stage == "release" { "channel_effect_uncertain" } else { "channel_gate_response_invalid" })?;
            if decision.contract != CONTRACT || decision.protocol != PROTOCOL
                || decision.effect_id != parsed_effect || decision.occurrence_id != occurrence_id
                || decision.action != action {
                return Err("channel_gate_response_mismatch");
            }
            if decision.disposition != expected {
                return Err(if decision.disposition == "uncertain" { "channel_effect_uncertain" } else { "channel_effect_withheld" });
            }
            if stage == "release" {
                return Ok(decision.receipt_id.to_string());
            }
        }
        Err("channel_gate_response_invalid")
    }
}
