//! Durable feedback identity, review links and independent human admissions.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackSource {
    Direct,
    Bossfang,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObserveFeedbackRequest {
    pub command_id: String,
    pub source: FeedbackSource,
    pub source_event_id: String,
    pub feedback: String,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LinkFeedbackWorkflowRequest {
    pub command_id: String,
    pub expected_revision: u64,
    pub workflow_run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttachFeedbackReviewRequest {
    pub command_id: String,
    pub expected_revision: u64,
    pub role: String,
    pub artifact_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeedbackPolicyRequest {
    pub command_id: String,
    pub id: String,
    pub expected_revision: Option<u64>,
    pub source: FeedbackSource,
    pub connector_binding_id: String,
    pub egress_label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackStandingPolicy {
    pub id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub revision: u64,
    pub source: FeedbackSource,
    pub connector_binding_id: String,
    pub egress_label: String,
    pub enabled: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthorizeFeedbackIssueRequest {
    pub command_id: String,
    pub expected_revision: u64,
    pub policy_id: String,
    pub expected_policy_revision: u64,
    pub sanitized_issue: serde_json::Value,
    pub sanitization_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdmitFeedbackImplementationRequest {
    pub command_id: String,
    pub expected_revision: u64,
    pub artifact_ids: BTreeSet<String>,
    pub decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackApproval {
    #[serde(default)]
    pub operator_id: Option<String>,
    #[serde(default)]
    pub connector_binding_revision: Option<u64>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub action: Option<super::connector_effect::ConnectorAction>,
    pub id: String,
    pub authority: String,
    pub policy_id: Option<String>,
    pub policy_revision: Option<u64>,
    pub connector_binding_id: Option<String>,
    pub egress_label: Option<String>,
    pub artifact_id: String,
    pub artifact_digest: String,
    pub sanitized_payload_digest: String,
    pub sanitization_ref: String,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackImplementationDecision {
    pub id: String,
    pub operator_id: String,
    pub decision: String,
    pub artifact_ids: BTreeSet<String>,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackIntake {
    pub id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub revision: u64,
    pub source: FeedbackSource,
    pub source_event_id: String,
    pub feedback: String,
    #[serde(default)]
    pub requested_target: Option<String>,
    pub feedback_digest: String,
    pub fingerprint: String,
    pub status: String,
    pub workflow_run_id: Option<String>,
    pub classification: Option<String>,
    pub duplicate_of: Option<String>,
    pub review_artifacts: BTreeMap<String, String>,
    #[serde(default)]
    pub issue_draft: Option<FeedbackIssueDraft>,
    #[serde(default)]
    pub connector_effect_id: Option<String>,
    #[serde(default)]
    pub external_issue_id: Option<String>,
    pub issue_approval: Option<FeedbackApproval>,
    pub implementation: Option<FeedbackImplementationDecision>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackCommandReceipt {
    pub request_digest: String,
    pub subject_id: String,
}

/// Host materialization of a committed workflow draft. This does not start an executor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackIssueDraft {
    pub artifact_id: String,
    pub artifact_digest: String,
    pub payload_digest: String,
    pub connector_binding_id: String,
    pub connector_binding_revision: u64,
    pub target: String,
    pub egress_label: String,
    pub sanitized_issue: serde_json::Value,
    pub sanitization_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DraftFeedbackIssueRequest {
    pub command_id: String,
    pub expected_revision: u64,
    pub connector_binding_id: String,
    pub expected_binding_revision: u64,
    pub sanitized_issue: serde_json::Value,
    pub sanitization_ref: String,
    pub egress_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExplicitFeedbackIssueApprovalRequest {
    pub command_id: String,
    pub expected_revision: u64,
    pub connector_binding_id: String,
    pub expected_binding_revision: u64,
    pub target: String,
    pub artifact_id: String,
    pub artifact_digest: String,
    pub payload_digest: String,
    pub sanitized_issue: serde_json::Value,
    pub sanitization_ref: String,
    pub egress_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeedbackControlRequest {
    pub command_id: String,
    pub expected_revision: u64,
    pub decision: String,
}
