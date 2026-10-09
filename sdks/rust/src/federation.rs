//! Explicit selected-instance admission; discovery does not authorize effects.
use super::Client;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FederationRequirement {
    pub required_capabilities: Vec<String>,
    pub team_endpoint: bool,
    pub distributed_team_members: bool,
    pub automatic_takeover: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FederationAdmission {
    pub instance: Value,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    /// This snapshot cannot be forwarded as proof for any effect.
    pub grants_effect_authority: bool,
}
impl Client {
    /// Query the configured, authenticated runtime, not an advertised agent URL.
    /// The resource owner still rechecks principal, workspace, approvals and
    /// its current execution fence at each effect. No run is created here.
    pub async fn admit_federated_instance(&self, required: &FederationRequirement) -> Result<FederationAdmission> {
        let url = self.base_url();
        let loopback = url.host_str().is_some_and(|host| matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]"));
        if !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some() ||
            (url.scheme() != "https" && !(url.scheme() == "http" && loopback)) {
            return Err(Error::Config("federation_endpoint_denied: use authenticated HTTPS or loopback".into()));
        }
        if self.api_key.as_ref().is_none_or(|credential| credential.trim().is_empty()) {
            return Err(Error::Config("federation_authentication_required".into()));
        }
        let response: Value = self.json(self.request(reqwest::Method::GET, "/api/uar/capabilities")?).await?;
        let profile = response.get("federation").ok_or_else(|| Error::Config("federation_profile_unavailable".into()))?;
        if profile.get("profile").and_then(Value::as_str) != Some("urn:prometheus:uar:federation:1") ||
            profile.get("registryAdvertisementGrantsAuthority").and_then(Value::as_bool) != Some(false) {
            return Err(Error::Config("federation_trust_contract_unsupported".into()));
        }
        let capabilities: Vec<String> = serde_json::from_value(response.get("capabilities").cloned().unwrap_or(Value::Null))?;
        if required.required_capabilities.iter().any(|capability| !capabilities.contains(capability)) {
            return Err(Error::Config("federation_required_capability_unavailable".into()));
        }
        for (requested, field) in [(required.team_endpoint, "teamEndpoint"),
            (required.distributed_team_members, "distributedTeamMembers"),
            (required.automatic_takeover, "automaticTakeover")] {
            if requested && profile.get(field).and_then(Value::as_bool) != Some(true) {
                return Err(Error::Config(format!("federation_required_feature_unavailable:{field}")));
            }
        }
        if required.team_endpoint && profile.get("resourceSideTeamFencing").and_then(Value::as_bool) != Some(true) {
            return Err(Error::Config("federation_resource_fencing_unavailable".into()));
        }
        let instance = response.get("instance").cloned().ok_or_else(|| Error::Config("federation_instance_identity_required".into()))?;
        Ok(FederationAdmission { instance, endpoint: self.base_url().to_string(), capabilities, grants_effect_authority: false })
    }
}
