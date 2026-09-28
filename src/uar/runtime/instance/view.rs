//! Public metadata projections. Private prompts, outcomes and grants stay in storage.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::uar::persistence::agent_instances::{
    AgentInstanceCommand, AgentInstanceEvent, AgentInstanceRecord,
};
pub use crate::uar::persistence::agent_instances::{
    AgentInstanceLimits, InstanceActivationProfile, InstanceCommandKind, InstanceCommandStatus,
    InstanceLifecycle, InstanceRecovery,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInstanceCommandView {
    pub command_id: String,
    pub kind: InstanceCommandKind,
    pub status: InstanceCommandStatus,
    pub attempt_id: Option<String>,
    pub root_run_id: Option<String>,
    pub accepted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&AgentInstanceCommand> for AgentInstanceCommandView {
    fn from(command: &AgentInstanceCommand) -> Self {
        Self {
            command_id: command.command_id.clone(),
            kind: command.kind,
            status: command.status,
            attempt_id: command.attempt_id.clone(),
            root_run_id: command.root_run_id.clone(),
            accepted_at: command.accepted_at,
            updated_at: command.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInstanceEventView {
    pub sequence: u64,
    pub kind: String,
    pub command_id: Option<String>,
    pub attempt_id: Option<String>,
    pub root_run_id: Option<String>,
    pub epoch: u64,
    pub committed_at: DateTime<Utc>,
}

impl From<&AgentInstanceEvent> for AgentInstanceEventView {
    fn from(event: &AgentInstanceEvent) -> Self {
        Self {
            sequence: event.sequence,
            kind: event.kind.clone(),
            command_id: event.command_id.clone(),
            attempt_id: event.attempt_id.clone(),
            root_run_id: event.root_run_id.clone(),
            epoch: event.epoch,
            committed_at: event.committed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInstanceView {
    pub instance_id: String,
    pub workspace_id: String,
    pub definition_id: String,
    pub definition_version: String,
    pub definition_digest: String,
    pub binding_id: String,
    pub binding_revision: u64,
    pub binding_digest: String,
    pub session_id: String,
    pub profile: InstanceActivationProfile,
    pub limits: AgentInstanceLimits,
    pub lifecycle: InstanceLifecycle,
    pub recovery: InstanceRecovery,
    pub revision: u64,
    pub epoch: u64,
    pub queue_depth: usize,
    pub active_run_id: Option<String>,
    pub active_attempt_id: Option<String>,
    pub active_command_id: Option<String>,
    pub restart_attempts: u32,
    pub last_error_code: Option<String>,
    pub reconciliation_receipt: Option<String>,
    pub next_event_sequence: u64,
    pub commands: Vec<AgentInstanceCommandView>,
    pub events: Vec<AgentInstanceEventView>,
}

impl From<&AgentInstanceRecord> for AgentInstanceView {
    fn from(record: &AgentInstanceRecord) -> Self {
        Self {
            instance_id: record.instance_id.clone(),
            workspace_id: record.workspace_id.clone(),
            definition_id: record.definition.id.clone(),
            definition_version: record.definition.version.clone(),
            definition_digest: record.definition.digest.clone(),
            binding_id: record.effective_binding.id.clone(),
            binding_revision: record.effective_binding.revision,
            binding_digest: record.effective_binding.digest.clone(),
            session_id: record.session_id.clone(),
            profile: record.profile,
            limits: record.limits.clone(),
            lifecycle: record.lifecycle,
            recovery: record.recovery,
            revision: record.revision,
            epoch: record.epoch,
            queue_depth: record
                .inbox
                .iter()
                .filter(|command| {
                    matches!(
                        command.status,
                        InstanceCommandStatus::Accepted | InstanceCommandStatus::Running
                    )
                })
                .count(),
            active_run_id: record
                .active_attempt
                .as_ref()
                .map(|attempt| attempt.root_run_id.clone()),
            active_attempt_id: record
                .active_attempt
                .as_ref()
                .map(|attempt| attempt.attempt_id.clone()),
            active_command_id: record
                .active_attempt
                .as_ref()
                .map(|attempt| attempt.command_id.clone()),
            restart_attempts: record.restart_attempts,
            last_error_code: record.last_error_code.clone(),
            reconciliation_receipt: record.reconciliation_receipt.clone(),
            next_event_sequence: record.next_event_sequence,
            commands: record.inbox.iter().map(Into::into).collect(),
            events: record.events.iter().map(Into::into).collect(),
        }
    }
}
