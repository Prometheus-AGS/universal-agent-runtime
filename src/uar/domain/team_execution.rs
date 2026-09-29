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
#[serde(rename_all = "camelCase")]
pub struct TeamExecutionAttempt {
    pub id: String,
    pub run_id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub team_id: String,
    pub task_id: String,
    pub member_id: String,
    pub member_revision: u64,
    pub ownership_epoch: u64,
    pub binding_revision: u64,
    pub execution_epoch: u64,
    pub status: String,
    #[serde(default)]
    pub execution_outcome: Option<String>,
    pub reservation: TeamReservation,
    pub context_artifact_ids: Vec<String>,
    pub usage: Option<TeamReservation>,
    pub usage_revision: u64,
    pub output: Option<Value>,
    pub state_reason: Option<String>,
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
