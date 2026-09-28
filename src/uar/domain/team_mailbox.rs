//! Durable team inbox records. A requested turn is not an admitted model run.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TeamMessageMode {
    QueueOnly,
    TriggerTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TeamMessageStatus {
    Accepted,
    Delivered,
    Processed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnqueueTeamMessageRequest {
    pub command_id: String,
    pub message_id: String,
    pub recipient_member_id: String,
    pub task_id: Option<String>,
    pub expected_task_epoch: Option<u64>,
    pub mode: TeamMessageMode,
    pub content: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamInboxMessage {
    pub message_id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub team_id: String,
    /// Public enqueue is operator-authored. Member senders require a host-resolved identity.
    pub sender_owner_id: String,
    pub recipient_member_id: String,
    pub recipient_member_revision: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_epoch: Option<u64>,
    pub mode: TeamMessageMode,
    pub content: Value,
    pub status: TeamMessageStatus,
    pub accepted_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_turn_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMailboxCommandReceipt {
    pub owner_id: String,
    pub workspace_id: String,
    pub command_id: String,
    pub request_digest: String,
    pub operation: String,
    pub team_id: String,
    pub message_id: String,
    pub committed_at: DateTime<Utc>,
}
