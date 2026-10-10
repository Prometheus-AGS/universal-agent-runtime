//! Durable team execution identities, reservations and selected context.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamReservation {
    pub tokens: u64,
    pub cost_microunits: u64,
    pub elapsed_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdmitTeamTaskRequest {
    pub command_id: String,
    pub expected_team_revision: u64,
    pub expected_task_revision: u64,
    pub member_id: String,
    pub reservation: TeamReservation,
    #[serde(default)]
    pub context_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DispatchTeamAttemptRequest {
    pub command_id: String,
    pub expected_team_revision: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamExecutionAttempt {
    pub id: String,
    pub run_id: String,
    #[serde(default)]
    pub root_id: String,
    #[serde(default)]
    pub approval_scope_id: String,
    #[serde(default)]
    pub queue_sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation_of_wait_id: Option<String>,
    pub owner_id: String,
    pub workspace_id: String,
    pub team_id: String,
    pub task_id: String,
    pub member_id: String,
    pub member_revision: u64,
    pub ownership_epoch: u64,
    pub binding_revision: u64,
    pub execution_epoch: u64,
    #[serde(default)]
    pub execution_fence: Option<TeamExecutionFence>,
    #[serde(default)]
    pub effective_models: Vec<EffectiveTeamModelReceipt>,
    #[serde(default = "uncertain_disposition")]
    pub effect_disposition: String,
    #[serde(default = "unknown_accounting")]
    pub accounting_state: String,
    pub status: String,
    #[serde(default)]
    pub execution_outcome: Option<String>,
    pub reservation: TeamReservation,
    pub context_artifact_ids: Vec<String>,
    pub usage: Option<TeamReservation>,
    pub usage_revision: u64,
    pub output: Option<Value>,
    pub state_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<TeamExecutionDiagnostic>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamArtifact {
    pub id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub team_id: String,
    pub task_id: String,
    pub member_id: String,
    pub attempt_id: String,
    pub content: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamControlRequest {
    pub command_id: String,
    pub expected_team_revision: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamExecutionSummary {
    pub attempts: Vec<TeamExecutionAttempt>,
    pub committed: TeamReservation,
    pub reserved: TeamReservation,
    pub uncertain_attempts: Vec<String>,
    #[serde(default)]
    pub command_receipts: Vec<super::team_wait::TeamPeerCommandReceipt>,
    #[serde(default)]
    pub waits: Vec<super::team_wait::TeamWait>,
    #[serde(default)]
    pub continuations: Vec<super::team_wait::ContinuationReceipt>,
    #[serde(default)]
    pub context_receipts: Vec<super::team_context::TeamContextReceipt>,
    pub budget: Value,
    pub limits: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamExecutionCommandReceipt {
    pub owner_id: String,
    pub workspace_id: String,
    pub team_id: String,
    pub command_id: String,
    pub operation: String,
    pub request_digest: String,
    pub attempt_id: Option<String>,
    #[serde(default)]
    pub recovery_attempt_ids: Vec<String>,
    pub committed_at: DateTime<Utc>,
}

/// Trusted endpoint profile revision; distinct from immutable definition references.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamProfileRef {
    pub id: String,
    pub revision: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamModelRoute {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default, schemars::JsonSchema)]
#[serde(tag = "mode", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TeamReasoningRequest {
    #[default]
    Off,
    Explicit {
        effort: crate::config::ReasoningEffort,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamModelSettingsRequest {
    pub route: TeamModelRoute,
    pub profile: TeamProfileRef,
    pub expected_settings_revision: u64,
    #[serde(default)]
    pub reasoning: TeamReasoningRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamLimitMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TeamFitDisposition {
    #[serde(rename_all = "camelCase")]
    SettingsOnly {
        guaranteed_fit: bool,
        reason: String,
    },
    #[serde(rename_all = "camelCase")]
    GuaranteedFit {
        guaranteed_fit: bool,
        budget_contract: TeamProfileRef,
        count_evidence_ref: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectiveTeamModelReceipt {
    pub route: TeamModelRoute,
    pub wire_model_alias: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_identity: Option<TeamPricingIdentity>,
    pub endpoint_kind: String,
    pub profile: TeamProfileRef,
    pub settings_revision: u64,
    pub requested_reasoning: TeamReasoningRequest,
    pub effective_reasoning: TeamReasoningRequest,
    pub support: String,
    pub support_evidence_ref: String,
    pub limits: TeamLimitMetadata,
    pub fit: TeamFitDisposition,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamExecutionDiagnostic {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    pub retryable: bool,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected_diagnostic_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collaboration_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "disposition", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TeamModelSettingsResult {
    Accepted {
        effective: EffectiveTeamModelReceipt,
    },
    Refused {
        requested: TeamModelSettingsRequest,
        diagnostic: TeamExecutionDiagnostic,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderTeamModelSettingsRecord {
    pub route: TeamModelRoute,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_identity: Option<TeamPricingIdentity>,
    pub profile: TeamProfileRef,
    pub settings_revision: u64,
    pub reasoning: TeamReasoningRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RebindTeamModelRequest {
    pub command_id: String,
    pub binding_id: String,
    pub expected_binding_revision: u64,
    pub route: TeamModelRoute,
    pub profile: TeamProfileRef,
    pub settings_revision: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamExecutionFence {
    pub catalog_id: String,
    pub service_instance_id: String,
    pub incarnation_id: String,
    pub epoch: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", from = "TeamExecutionClaimWire")]
pub struct TeamExecutionClaim {
    #[serde(flatten)]
    pub fence: TeamExecutionFence,
    pub state: String,
    pub acquired_at: DateTime<Utc>,
    pub audit_receipt_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<DateTime<Utc>>,
}

// Decode the closed flat wire record explicitly: flatten cannot combine with
// deny_unknown_fields on the embedded fence's deserializer.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TeamExecutionClaimWire {
    catalog_id: String,
    service_instance_id: String,
    incarnation_id: String,
    epoch: u64,
    state: String,
    acquired_at: DateTime<Utc>,
    audit_receipt_id: String,
    #[serde(default)]
    released_at: Option<DateTime<Utc>>,
}

impl From<TeamExecutionClaimWire> for TeamExecutionClaim {
    fn from(wire: TeamExecutionClaimWire) -> Self {
        Self {
            fence: TeamExecutionFence {
                catalog_id: wire.catalog_id,
                service_instance_id: wire.service_instance_id,
                incarnation_id: wire.incarnation_id,
                epoch: wire.epoch,
            },
            state: wire.state,
            acquired_at: wire.acquired_at,
            audit_receipt_id: wire.audit_receipt_id,
            released_at: wire.released_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamExecutionReclaimRequest {
    pub command_id: String,
    pub catalog_id: String,
    pub expected_epoch: u64,
    pub replacement_service_instance_id: String,
    pub reason: String,
    pub fencing_evidence_ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamExecutionReclaimReceipt {
    pub command_id: String,
    pub request_digest: String,
    pub authenticated_actor_id: String,
    pub reason: String,
    pub previous_fence: TeamExecutionFence,
    pub replacement_fence: TeamExecutionFence,
    pub authorization_decision_ref: String,
    pub fencing_evidence_ref: String,
    pub transferred_queued_attempt_ids: Vec<String>,
    pub uncertain_attempt_ids: Vec<String>,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamExecutionFencingEvidence {
    pub id: String,
    pub fence: TeamExecutionFence,
    pub root_and_children_stopped: bool,
    pub source: String,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamExecutionOwnershipView {
    pub claim: Option<TeamExecutionClaim>,
    pub current_fence: TeamExecutionFence,
    pub owns_execution: bool,
}

fn uncertain_disposition() -> String {
    "uncertain".to_owned()
}
fn unknown_accounting() -> String {
    "reserved-unknown".to_owned()
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamPricingIdentity {
    pub provider_id: String,
    pub model_id: String,
    pub catalog_revision: String,
}
