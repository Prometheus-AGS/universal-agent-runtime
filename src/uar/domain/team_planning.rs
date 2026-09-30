//! Durable, owner-scoped team planning records. Execution belongs to later C09 work.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::collaboration::ImmutableDefinitionRef;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMemberSpec {
    pub role: String,
    pub kind: String,
    pub min: u32,
    pub max: u32,
    pub definition: ImmutableDefinitionRef,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamDefinitionSummary {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub instructions: Option<crate::uar::domain::team_context::TeamInstructions>,
    pub id: String,
    pub version: String,
    pub digest: String,
    pub title: String,
    pub purpose: String,
    pub package: ImmutableDefinitionRef,
    pub members: Vec<TeamMemberSpec>,
    pub budget: Value,
    pub limits: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemberSlotRequest {
    pub role: String,
    pub count: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateTeamRequest {
    pub command_id: String,
    pub deployment_binding_id: String,
    pub team_definition: ImmutableDefinitionRef,
    pub input: Value,
    #[serde(default)]
    pub member_slots: Vec<MemberSlotRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateTeamTaskRequest {
    pub command_id: String,
    pub task_id: String,
    pub expected_team_revision: u64,
    pub title: String,
    pub role: String,
    pub input: Value,
    pub output_contract: Value,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssignTeamTaskRequest {
    pub command_id: String,
    pub expected_team_revision: u64,
    pub expected_task_revision: u64,
    pub member_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssignTeamReviewerRequest {
    pub command_id: String,
    pub expected_team_revision: u64,
    pub expected_task_revision: u64,
    pub member_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionTeamTaskRequest {
    pub command_id: String,
    pub expected_team_revision: u64,
    pub expected_task_revision: u64,
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamBindingRef {
    pub id: String,
    pub revision: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub id: String,
    pub role: String,
    pub kind: String,
    pub ordinal: u32,
    pub definition: ImmutableDefinitionRef,
    pub revision: u64,
    pub status: String,
}

/// A durable, non-executable assignment constraint. Current host and Cedar grants
/// must be recomputed before a turn or effect can be admitted.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentAuthority {
    pub binding_id: String,
    pub binding_revision: u64,
    pub workspace_id: String,
    pub role: String,
    pub member_id: String,
    pub ownership_epoch: u64,
    pub can_execute: bool,
    pub can_use_tools: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamTask {
    pub id: String,
    pub title: String,
    pub role: String,
    pub input: Value,
    pub output_contract: Value,
    pub output: Option<Value>,
    pub depends_on: Vec<String>,
    pub status: String,
    pub revision: u64,
    #[serde(default)]
    pub assignee_member_id: Option<String>,
    #[serde(default)]
    pub ownership_epoch: u64,
    #[serde(default)]
    pub assignment_authority: Option<AssignmentAuthority>,
    #[serde(default)]
    pub reviewer_member_id: Option<String>,
    #[serde(default)]
    pub reviewer_epoch: u64,
    #[serde(default)]
    pub state_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamInstance {
    pub id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub revision: u64,
    pub status: String,
    pub definition: ImmutableDefinitionRef,
    pub package: ImmutableDefinitionRef,
    pub binding: TeamBindingRef,
    pub input: Value,
    pub members: Vec<TeamMember>,
    pub tasks: Vec<TeamTask>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamPlanningCommandReceipt {
    pub owner_id: String,
    pub workspace_id: String,
    pub command_id: String,
    pub request_digest: String,
    pub operation: String,
    pub team_id: String,
    pub task_id: Option<String>,
    pub committed_at: DateTime<Utc>,
}
