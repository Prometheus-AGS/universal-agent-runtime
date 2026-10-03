//! Versioned workflow plans and durable, owner-scoped execution records.
use super::{
    collaboration::ImmutableDefinitionRef, team_execution::TeamReservation,
    team_planning::TeamBindingRef,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const WORKFLOW_CAPABILITY: &str = "prometheus.workflow-execution/1.0.0";
pub const WORKFLOW_EXTENSION: &str = "prometheus.workflow-execution";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowStepSelection {
    pub step_id: String,
    pub member_id: String,
    pub reservation: TeamReservation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartWorkflowRequest {
    pub command_id: String,
    pub workflow_definition: ImmutableDefinitionRef,
    pub team_id: String,
    pub expected_team_revision: u64,
    pub expected_binding_revision: u64,
    pub input: Value,
    pub steps: Vec<WorkflowStepSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowDecisionRequest {
    pub command_id: String,
    pub expected_run_revision: u64,
    pub wait_id: String,
    pub decision: String,
    pub artifact_id: String,
    pub artifact_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowControlRequest {
    pub command_id: String,
    pub expected_run_revision: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCompiledStep {
    pub id: String,
    pub role: String,
    pub instructions: String,
    pub input_mapping: BTreeMap<String, String>,
    pub output: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCompiledPlan {
    pub interpretation_version: String,
    pub digest: String,
    pub input: Value,
    pub output: Value,
    pub steps: Vec<WorkflowCompiledStep>,
    pub max_activations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDiagnostic {
    pub field: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDefinitionSummary {
    pub identity: ImmutableDefinitionRef,
    pub package: ImmutableDefinitionRef,
    pub title: String,
    pub input: Value,
    pub output: Value,
    pub steps: Vec<WorkflowCompiledStep>,
    pub supported: bool,
    pub diagnostics: Vec<WorkflowDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowArtifact {
    pub id: String,
    pub digest: String,
    pub content: Value,
    pub attempt_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStepState {
    pub step_id: String,
    pub task_id: String,
    pub member_id: String,
    pub member_revision: u64,
    pub member_definition: ImmutableDefinitionRef,
    pub reservation: TeamReservation,
    pub status: String,
    pub attempt_id: Option<String>,
    pub artifact: Option<WorkflowArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowWait {
    pub id: String,
    pub artifact_id: String,
    pub artifact_digest: String,
    pub decisions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDecisionReceipt {
    pub id: String,
    pub command_id: String,
    pub wait_id: String,
    pub decision: String,
    pub artifact_id: String,
    pub artifact_digest: String,
    pub run_revision: u64,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowAccounting {
    pub committed: TeamReservation,
    pub reserved: TeamReservation,
    pub unresolved_attempt_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRun {
    pub id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub revision: u64,
    pub status: String,
    pub definition: ImmutableDefinitionRef,
    pub package: ImmutableDefinitionRef,
    pub definition_snapshot: Value,
    pub plan: WorkflowCompiledPlan,
    pub team_id: String,
    pub team_definition: ImmutableDefinitionRef,
    pub binding: TeamBindingRef,
    pub input: Value,
    pub steps: Vec<WorkflowStepState>,
    pub wait: Option<WorkflowWait>,
    pub decision: Option<WorkflowDecisionReceipt>,
    pub state_reason: Option<String>,
    pub accounting: WorkflowAccounting,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCommandReceipt {
    pub request_digest: String,
    pub operation: String,
    pub run_id: String,
    pub result: WorkflowRun,
    pub completed: bool,
    pub team_control: Option<super::team_execution::TeamControlRequest>,
}
