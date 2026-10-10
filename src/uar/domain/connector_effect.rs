//! Scoped connector intents. Credentials are opaque references resolved only by a trusted host.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorProvider {
    Github,
    Notion,
    Slack,
    Jira,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorAction {
    Read,
    Draft,
    Write,
    Send,
    Publish,
}

/// Publication authority chosen by the trusted host for a GitHub binding.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorApprovalMode {
    #[default]
    ExplicitCustomer,
    StandingPolicy,
}

impl ConnectorApprovalMode {
    fn is_explicit_customer(&self) -> bool {
        *self == Self::ExplicitCustomer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectorBindingRequest {
    pub command_id: String,
    pub id: String,
    pub expected_revision: Option<u64>,
    pub provider: ConnectorProvider,
    #[serde(default, skip_serializing_if = "ConnectorApprovalMode::is_explicit_customer")]
    pub approval_mode: ConnectorApprovalMode,
    pub target: String,
    pub site: Option<String>,
    pub allowed_actions: BTreeSet<ConnectorAction>,
    pub allowed_egress_labels: BTreeSet<String>,
    pub credential_ref: String,
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorBinding {
    pub id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub revision: u64,
    pub provider: ConnectorProvider,
    #[serde(default)]
    pub approval_mode: ConnectorApprovalMode,
    pub target: String,
    pub site: Option<String>,
    pub allowed_actions: BTreeSet<ConnectorAction>,
    pub allowed_egress_labels: BTreeSet<String>,
    pub credential_ref: String,
    pub revoked: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectorEffectRequest {
    pub command_id: String,
    pub binding_id: String,
    pub expected_binding_revision: u64,
    pub action: ConnectorAction,
    pub egress_labels: BTreeSet<String>,
    pub decision_ref: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorEffect {
    pub id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub binding_id: String,
    pub binding_revision: u64,
    pub provider: ConnectorProvider,
    pub target: String,
    pub action: ConnectorAction,
    pub egress_labels: BTreeSet<String>,
    pub decision_ref: Option<String>,
    pub payload_digest: String,
    pub payload: Value,
    pub status: String,
    pub dispatch_id: Option<String>,
    pub receipt: Option<ConnectorEffectReceipt>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorEffectReceipt {
    pub disposition: String,
    pub external_id: Option<String>,
    pub evidence_ref: Option<String>,
    pub result: Option<Value>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorEffectView {
    pub id: String,
    pub binding_id: String,
    pub binding_revision: u64,
    pub provider: ConnectorProvider,
    pub target: String,
    pub action: ConnectorAction,
    pub egress_labels: BTreeSet<String>,
    pub decision_ref: Option<String>,
    pub payload_digest: String,
    pub status: String,
    pub dispatch_id: Option<String>,
    pub receipt: Option<ConnectorEffectReceipt>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&ConnectorEffect> for ConnectorEffectView {
    fn from(effect: &ConnectorEffect) -> Self {
        Self {
            id: effect.id.clone(),
            binding_id: effect.binding_id.clone(),
            binding_revision: effect.binding_revision,
            provider: effect.provider,
            target: effect.target.clone(),
            action: effect.action,
            egress_labels: effect.egress_labels.clone(),
            decision_ref: effect.decision_ref.clone(),
            payload_digest: effect.payload_digest.clone(),
            status: effect.status.clone(),
            dispatch_id: effect.dispatch_id.clone(),
            receipt: effect.receipt.clone(),
            created_at: effect.created_at,
            updated_at: effect.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectorOutcomeRequest {
    pub command_id: String,
    pub dispatch_id: String,
    pub disposition: String,
    pub external_id: Option<String>,
    pub evidence_ref: Option<String>,
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorCommandReceipt {
    pub request_digest: String,
    pub effect_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedConnectorRequest {
    pub effect_id: String,
    pub dispatch_id: String,
    pub provider: ConnectorProvider,
    pub credential_ref: String,
    pub origin: String,
    pub method: String,
    pub path: String,
    pub body: Option<Value>,
    pub headers: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CancelConnectorEffectRequest {
    pub command_id: String,
}
