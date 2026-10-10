//! Durable logical-agent state. Root runs and live actors are separate objects.

use std::collections::HashSet;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::uar::domain::collaboration::{
    ImmutableDefinitionRef, PrivateRevisionRef, RepresentationGrantRef,
};

pub const AGENT_INSTANCE_SCHEMA_VERSION: u32 = 1;
pub const MAX_INSTANCE_COMMAND_BYTES: usize = 256 * 1024;
pub const MAX_INSTANCE_OUTCOME_BYTES: usize = 1024 * 1024;
pub const MAX_INSTANCE_INBOX: u32 = 64;
pub const MAX_INSTANCE_COMMAND_RETENTION: u32 = 128;
pub const MAX_INSTANCE_EVENTS: u32 = 512;
pub const MAX_INSTANCE_RESTART_ATTEMPTS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceActivationProfile {
    Request,
    OnDemand,
    Resident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceLifecycle {
    Dormant,
    Active,
    Draining,
    Disabled,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceRecovery {
    Ready,
    PendingReconciliation,
    EffectUncertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceCommandKind {
    Turn,
    Activate,
    Passivate,
    Drain,
    Disable,
    Restart,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceCommandStatus {
    Accepted,
    Running,
    Completed,
    Failed,
    Cancelled,
    Uncertain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentInstanceLimits {
    pub max_inbox: u32,
    pub retained_commands: u32,
    pub retained_events: u32,
    pub max_restart_attempts: u32,
    pub idle_timeout_secs: u64,
}

impl Default for AgentInstanceLimits {
    fn default() -> Self {
        Self {
            max_inbox: 32,
            retained_commands: 64,
            retained_events: 256,
            max_restart_attempts: 3,
            idle_timeout_secs: 300,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentInstanceCommand {
    pub command_id: String,
    pub request_digest: String,
    pub kind: InstanceCommandKind,
    /// Private durable input. Never project this field to ordinary status APIs.
    pub payload: Option<Value>,
    pub payload_ref: Option<String>,
    pub status: InstanceCommandStatus,
    pub attempt_id: Option<String>,
    pub root_run_id: Option<String>,
    pub accepted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Private terminal result. Public administration projects a safe summary.
    pub outcome: Option<Value>,
}

impl fmt::Debug for AgentInstanceCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AgentInstanceCommand")
            .field("command_id", &self.command_id)
            .field("kind", &self.kind)
            .field("status", &self.status)
            .field("attempt_id", &self.attempt_id)
            .field("root_run_id", &self.root_run_id)
            .field("payload", &self.payload.as_ref().map(|_| "<redacted>"))
            .field("outcome", &self.outcome.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentInstanceAttempt {
    pub command_id: String,
    pub attempt_id: String,
    pub root_run_id: String,
    pub epoch: u64,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentInstanceEvent {
    pub sequence: u64,
    pub kind: String,
    pub command_id: Option<String>,
    pub attempt_id: Option<String>,
    pub root_run_id: Option<String>,
    pub epoch: u64,
    pub committed_at: DateTime<Utc>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentInstanceRecord {
    pub schema_version: u32,
    pub instance_id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub definition: ImmutableDefinitionRef,
    pub effective_binding: PrivateRevisionRef,
    /// Private authority is mutable without rewriting the pinned deployment.
    #[serde(default)]
    pub representation_revision: u64,
    /// References remain attached after revocation; empty cannot grant fallback.
    #[serde(default)]
    pub representation_grants: Vec<RepresentationGrantRef>,
    /// Stable conversation identity shared by distinct fresh-root turns.
    pub session_id: String,
    pub profile: InstanceActivationProfile,
    pub lifecycle: InstanceLifecycle,
    pub recovery: InstanceRecovery,
    pub limits: AgentInstanceLimits,
    pub revision: u64,
    pub epoch: u64,
    pub inbox: Vec<AgentInstanceCommand>,
    pub active_attempt: Option<AgentInstanceAttempt>,
    pub events: Vec<AgentInstanceEvent>,
    pub next_event_sequence: u64,
    /// This mutation's complete semantic events, including any trimmed from the
    /// bounded UI projection. The outbox writes them with the source CAS.
    #[serde(skip)]
    pub newly_appended_events: Vec<AgentInstanceEvent>,
    pub restart_attempts: u32,
    pub last_error_code: Option<String>,
    /// Exact operator/effect receipt required to settle an uncertain outcome.
    pub reconciliation_receipt: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl fmt::Debug for AgentInstanceRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AgentInstanceRecord")
            .field("instance_id", &self.instance_id)
            .field("owner_id", &self.owner_id)
            .field("workspace_id", &self.workspace_id)
            .field("profile", &self.profile)
            .field("lifecycle", &self.lifecycle)
            .field("recovery", &self.recovery)
            .field("revision", &self.revision)
            .field("epoch", &self.epoch)
            .field("inbox_depth", &self.inbox.len())
            .field("active_attempt", &self.active_attempt)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AgentInstanceStoreError {
    #[error("durable agent instances are unsupported by this persistence provider")]
    Unsupported,
    #[error("agent instance is outside the verified owner or workspace")]
    ScopeMismatch,
    #[error("agent instance already exists")]
    AlreadyExists,
    #[error("agent instance does not exist")]
    NotFound,
    #[error("agent instance revision or activation epoch changed")]
    Conflict,
    #[error("agent instance immutable identity or binding changed")]
    ImmutableField,
    #[error("agent instance record is malformed or exceeds its bounded limits")]
    InvalidRecord,
    #[error("agent instance revision or activation epoch is exhausted")]
    RevisionExhausted,
}

impl AgentInstanceRecord {
    /// Check persisted scope and bounded state before trusting a read or write.
    pub fn validate(
        &self,
        owner_id: &str,
        workspace_id: &str,
    ) -> Result<(), AgentInstanceStoreError> {
        if self.owner_id != owner_id || self.workspace_id != workspace_id {
            return Err(AgentInstanceStoreError::ScopeMismatch);
        }
        if self.schema_version != AGENT_INSTANCE_SCHEMA_VERSION
            || self.instance_id.is_empty()
            || self.owner_id.is_empty()
            || self.workspace_id.is_empty()
            || self.session_id.is_empty()
            || self.definition.id.is_empty()
            || self.definition.version.is_empty()
            || self.definition.digest.is_empty()
            || self.effective_binding.id.is_empty()
            || self.effective_binding.digest.is_empty()
            || self.revision > i64::MAX as u64
            || self.epoch > i64::MAX as u64
            || self.representation_revision > i64::MAX as u64
            || self.representation_grants.len() > 128
            || self.representation_grants.iter().any(|reference| {
                reference.grant_id.is_empty() || reference.revision == 0
                    || reference.constraint_digest.is_empty()
            })
            || self.limits.max_inbox == 0
            || self.limits.max_inbox > MAX_INSTANCE_INBOX
            // A full turn queue can have one pending lifecycle receipt and
            // still needs room to record a superseding Cancel receipt.
            || self.limits.retained_commands < self.limits.max_inbox.saturating_add(2)
            || self.limits.retained_commands > MAX_INSTANCE_COMMAND_RETENTION
            || self.limits.retained_events == 0
            || self.limits.retained_events > MAX_INSTANCE_EVENTS
            || self.limits.max_restart_attempts == 0
            || self.limits.max_restart_attempts > MAX_INSTANCE_RESTART_ATTEMPTS
            || self.inbox.len() > self.limits.retained_commands as usize
            || self.events.len() > self.limits.retained_events as usize
            || self.restart_attempts > self.limits.max_restart_attempts
            || self
                .last_error_code
                .as_ref()
                .is_some_and(|code| code.len() > 128)
            || self
                .reconciliation_receipt
                .as_ref()
                .is_some_and(|receipt| receipt.is_empty() || receipt.len() > 256)
            || self.updated_at < self.created_at
        {
            return Err(AgentInstanceStoreError::InvalidRecord);
        }
        let mut command_ids = HashSet::new();
        let mut pending_turn_count = 0;
        for command in &self.inbox {
            let payload_bytes = command
                .payload
                .as_ref()
                .map(|payload| serde_json::to_vec(payload).map(|bytes| bytes.len()))
                .transpose()
                .map_err(|_| AgentInstanceStoreError::InvalidRecord)?
                .unwrap_or(0);
            let outcome_bytes = command
                .outcome
                .as_ref()
                .map(|outcome| serde_json::to_vec(outcome).map(|bytes| bytes.len()))
                .transpose()
                .map_err(|_| AgentInstanceStoreError::InvalidRecord)?
                .unwrap_or(0);
            if command.command_id.is_empty()
                || !command_ids.insert(&command.command_id)
                || command.request_digest.len() != 64
                || !command
                    .request_digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
                || payload_bytes > MAX_INSTANCE_COMMAND_BYTES
                || outcome_bytes > MAX_INSTANCE_OUTCOME_BYTES
                || command
                    .payload_ref
                    .as_ref()
                    .is_some_and(|reference| reference.len() > 256)
                || command.updated_at < command.accepted_at
            {
                return Err(AgentInstanceStoreError::InvalidRecord);
            }
            if command.kind == InstanceCommandKind::Turn
                && matches!(
                    command.status,
                    InstanceCommandStatus::Accepted | InstanceCommandStatus::Running
                )
            {
                pending_turn_count += 1;
            }
        }
        if pending_turn_count > self.limits.max_inbox as usize {
            return Err(AgentInstanceStoreError::InvalidRecord);
        }
        if let Some(active) = &self.active_attempt {
            if active.epoch != self.epoch
                || !self.inbox.iter().any(|command| {
                    command.command_id == active.command_id
                        && command.attempt_id.as_deref() == Some(active.attempt_id.as_str())
                        && command.root_run_id.as_deref() == Some(active.root_run_id.as_str())
                        && command.status == InstanceCommandStatus::Running
                })
            {
                return Err(AgentInstanceStoreError::InvalidRecord);
            }
        }
        let mut sequence = None;
        for event in &self.events {
            if event.epoch > self.epoch
                || event.kind.is_empty()
                || event.kind.len() > 128
                || sequence.is_some_and(|previous| event.sequence <= previous)
            {
                return Err(AgentInstanceStoreError::InvalidRecord);
            }
            sequence = Some(event.sequence);
        }
        if sequence.is_some_and(|last| last >= self.next_event_sequence) {
            return Err(AgentInstanceStoreError::InvalidRecord);
        }
        Ok(())
    }

    /// Check one whole-record conditional mutation, including immutable refs.
    pub fn validate_next(&self, next: &Self) -> Result<(), AgentInstanceStoreError> {
        next.validate(&self.owner_id, &self.workspace_id)?;
        if self.instance_id != next.instance_id
            || self.definition != next.definition
            || self.effective_binding != next.effective_binding
            || self.session_id != next.session_id
            || self.created_at != next.created_at
        {
            return Err(AgentInstanceStoreError::ImmutableField);
        }
        if next.revision
            != self
                .revision
                .checked_add(1)
                .ok_or(AgentInstanceStoreError::RevisionExhausted)?
            || (next.epoch != self.epoch
                && next.epoch
                    != self
                        .epoch
                        .checked_add(1)
                        .ok_or(AgentInstanceStoreError::RevisionExhausted)?)
            || next.next_event_sequence < self.next_event_sequence
            || next.updated_at < self.updated_at
            || next.representation_revision < self.representation_revision
            || (next.representation_grants != self.representation_grants
                && next.representation_revision != self.representation_revision.checked_add(1)
                    .ok_or(AgentInstanceStoreError::RevisionExhausted)?)
        {
            return Err(AgentInstanceStoreError::InvalidRecord);
        }
        if self.recovery == InstanceRecovery::EffectUncertain
            && next.recovery != InstanceRecovery::EffectUncertain
            && (next.reconciliation_receipt.is_none()
                || next.reconciliation_receipt == self.reconciliation_receipt)
        {
            return Err(AgentInstanceStoreError::InvalidRecord);
        }
        let old_retained_start = self
            .inbox
            .iter()
            .position(|command| {
                next.inbox
                    .iter()
                    .any(|candidate| candidate.command_id == command.command_id)
            })
            .unwrap_or(self.inbox.len());
        for dropped in &self.inbox[..old_retained_start] {
            if matches!(
                dropped.status,
                InstanceCommandStatus::Accepted
                    | InstanceCommandStatus::Running
                    | InstanceCommandStatus::Uncertain
            ) {
                return Err(AgentInstanceStoreError::InvalidRecord);
            }
        }
        let old_retained = &self.inbox[old_retained_start..];
        if next.inbox.len() < old_retained.len() {
            return Err(AgentInstanceStoreError::InvalidRecord);
        }
        for (previous, current) in old_retained.iter().zip(next.inbox.iter()) {
            if current.command_id != previous.command_id {
                return Err(AgentInstanceStoreError::InvalidRecord);
            }
            if current.request_digest != previous.request_digest
                || current.kind != previous.kind
                || current.payload != previous.payload
                || current.payload_ref != previous.payload_ref
                || current.accepted_at != previous.accepted_at
                || (matches!(
                    previous.status,
                    InstanceCommandStatus::Completed
                        | InstanceCommandStatus::Failed
                        | InstanceCommandStatus::Cancelled
                ) && current != previous)
                || (previous.status == InstanceCommandStatus::Uncertain
                    && current.status != InstanceCommandStatus::Uncertain
                    && next.reconciliation_receipt == self.reconciliation_receipt)
            {
                return Err(AgentInstanceStoreError::InvalidRecord);
            }
        }
        let mut appended_sequence = self.next_event_sequence;
        for event in &next.newly_appended_events {
            if event.sequence != appended_sequence {
                return Err(AgentInstanceStoreError::InvalidRecord);
            }
            appended_sequence = appended_sequence
                .checked_add(1)
                .ok_or(AgentInstanceStoreError::RevisionExhausted)?;
        }
        if appended_sequence != next.next_event_sequence {
            return Err(AgentInstanceStoreError::InvalidRecord);
        }
        for event in next
            .events
            .iter()
            .filter(|event| event.sequence >= self.next_event_sequence)
        {
            if !next.newly_appended_events.contains(event) {
                return Err(AgentInstanceStoreError::InvalidRecord);
            }
        }
        for previous in &self.events {
            if let Some(current) = next
                .events
                .iter()
                .find(|event| event.sequence == previous.sequence)
            {
                if current != previous {
                    return Err(AgentInstanceStoreError::InvalidRecord);
                }
            }
        }
        Ok(())
    }
}
