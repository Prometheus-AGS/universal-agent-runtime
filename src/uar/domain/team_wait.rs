//! Attempt-derived peer authority and durable continuation contracts.
use super::team_execution::{TeamExecutionFence, TeamProfileRef, TeamReservation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttemptAuthority {
    pub owner_id: String,
    pub workspace_id: String,
    pub team_id: String,
    pub task_id: String,
    pub member_id: String,
    pub attempt_id: String,
    pub run_id: String,
    pub task_ownership_epoch: u64,
    pub member_revision: u64,
    pub binding: TeamProfileRef,
    pub execution_fence: TeamExecutionFence,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TargetOutcome {
    pub task_id: String,
    pub member_id: String,
    pub attempt_id: String,
    pub execution_outcome: String,
    pub effect_disposition: String,
    pub artifact_ids: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttributedPayload {
    pub text: String,
    pub artifact_ids: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DirectRecipient {
    pub member_id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub task_id: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamSendRequest {
    pub command_id: String,
    pub recipient: DirectRecipient,
    pub payload: AttributedPayload,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DelegatedTask {
    pub task_id: String,
    pub role: String,
    pub input: Value,
    pub output_contract: Value,
    pub depends_on: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamDelegateRequest {
    pub command_id: String,
    pub recipient_member_id: String,
    pub expected_team_revision: u64,
    pub task: DelegatedTask,
    pub payload: AttributedPayload,
    pub reservation: TeamReservation,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamWaitRequest {
    pub command_id: String,
    pub target_task_ids: Vec<String>,
    pub predicate: String,
    pub continuation_input: AttributedPayload,
    pub continuation_reservation: TeamReservation,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamPeerCommandReceipt {
    pub command_id: String,
    pub request_digest: String,
    pub scope: TeamPeerScope,
    pub operation: String,
    pub sender_member_id: String,
    pub sender_attempt_id: String,
    pub accepted_at: DateTime<Utc>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub message_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub task_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub attempt_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub wait_id: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamPeerScope {
    pub owner_id: String,
    pub workspace_id: String,
    pub team_id: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamPeerMessage {
    pub message_id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub team_id: String,
    pub sender_member_id: String,
    pub sender_attempt_id: String,
    pub recipient_member_id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub recipient_task_id: Option<String>,
    pub mode: String,
    pub payload: AttributedPayload,
    pub accepted_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MessageDelivery {
    pub message_id: String,
    pub recipient_member_id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub recipient_task_id: Option<String>,
    pub status: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub selected_attempt_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub selected_at: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub consumed_at: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub rejection_code: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamWait {
    pub wait_id: String,
    pub authority: AttemptAuthority,
    pub target_task_ids: Vec<String>,
    pub predicate: String,
    pub continuation_input: AttributedPayload,
    pub continuation_reservation: TeamReservation,
    pub state: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub continuation_attempt_id: Option<String>,
    pub wake_outcomes: Vec<TargetOutcome>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub reason_code: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnexecutedTeamToolCall {
    pub tool_call_id: String,
    pub disposition: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KernelTeamYield {
    pub kind: String,
    pub wait_id: String,
    pub yielding_attempt_id: String,
    pub durable_receipt_id: String,
    pub remaining_tool_calls: Vec<UnexecutedTeamToolCall>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContinuationReceipt {
    pub wait_id: String,
    pub uniqueness_key: String,
    pub previous_attempt_id: String,
    pub authority: AttemptAuthority,
    pub root_id: String,
    pub approval_scope_id: String,
    pub authorization_revision: u64,
    pub task_id: String,
    pub continuation_attempt_id: String,
    pub run_id: String,
    pub execution_fence: TeamExecutionFence,
    pub reservation: TeamReservation,
    pub selected_artifact_ids: Vec<String>,
    pub target_outcomes: Vec<TargetOutcome>,
    pub committed_at: DateTime<Utc>,
}

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}
