use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::uar::service_instance::EffectiveServiceBinding;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskReceipt {
    pub admission_id: String,
    pub task_id: String,
    pub native_task_id: String,
    pub run_id: String,
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub runtime_epoch: String,
    pub revision: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u64>,
    pub state: String,
    pub retention: RetentionReceipt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_service_binding: Option<EffectiveServiceBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<TaskDiagnostic>,
    pub cancellation: CancellationReceipt,
    pub detached: bool,
    pub detached_observers: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    pub unsupported_semantics: Vec<&'static str>,
    pub links: TaskLinks,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetentionReceipt {
    pub mode: &'static str,
    pub terminal_ttl_seconds: u64,
    pub terminal_record_cap: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeDescriptor {
    pub event_cursor_profile: &'static str,
    pub profile: &'static str,
    pub runtime_epoch: String,
    pub recovery: &'static str,
    pub retention: RetentionReceipt,
    pub steer_supported: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CancellationReceipt {
    pub requested: bool,
    pub acknowledged: bool,
    pub terminal: bool,
    pub cleanup_uncertain: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskDiagnostic {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskLinks {
    pub status: String,
    pub stream: String,
    pub tool_approval: String,
    pub cancel: String,
    pub detach: String,
    pub steer: String,
}
