//! Durable, metadata-only observers of committed logical-agent transitions.

use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const OBSERVER_SCHEMA_VERSION: u32 = 1;
pub const MAX_OBSERVER_SOURCES: usize = 64;
pub const MAX_OBSERVER_CONVERSATIONS: usize = 64;
pub const MAX_OBSERVER_INBOX: u32 = 256;
pub const MAX_OBSERVER_RETRIES: u32 = 10;
pub const MAX_OBSERVER_GAPS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObserverProjection {
    ControlMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverLimits {
    pub max_inbox: u32,
    pub max_retries: u32,
    pub retained_acknowledged: u32,
}

impl Default for ObserverLimits {
    fn default() -> Self {
        Self {
            max_inbox: 128,
            max_retries: 3,
            retained_acknowledged: 64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObserverAdmissionStatus {
    Admitted,
    Acknowledged,
    Retry,
    DeadLetter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverAdmission {
    pub occurrence_id: String,
    pub source_instance_id: String,
    pub source_sequence: u64,
    /// Immutable metadata snapshot for recovery after source outbox retention.
    pub occurrence: ObserverOccurrence,
    pub observer_command_id: String,
    pub status: ObserverAdmissionStatus,
    pub attempts: u32,
    pub last_error_code: Option<String>,
    pub admitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverGap {
    pub source_instance_id: String,
    pub missing_from: u64,
    pub missing_through: u64,
    pub detected_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverSubscription {
    pub schema_version: u32,
    pub subscription_id: String,
    pub owner_id: String,
    pub owner_user_id: String,
    pub owner_tenant_id: Option<String>,
    pub workspace_id: String,
    pub observer_instance_id: String,
    pub source_instance_ids: Vec<String>,
    pub conversation_ids: Option<Vec<String>>,
    pub projection: ObserverProjection,
    pub revision: u64,
    pub paused: bool,
    pub revoked: bool,
    pub cursors: BTreeMap<String, u64>,
    pub inbox: Vec<ObserverAdmission>,
    pub limits: ObserverLimits,
    pub gaps: Vec<ObserverGap>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverOccurrence {
    pub occurrence_id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub source_instance_id: String,
    pub conversation_id: String,
    pub sequence: u64,
    pub kind: String,
    pub command_id: Option<String>,
    pub attempt_id: Option<String>,
    pub root_run_id: Option<String>,
    pub epoch: u64,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverOccurrenceBounds {
    pub low: Option<u64>,
    pub high: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum ObserverStoreError {
    #[error("durable observers are unsupported by this persistence provider")]
    Unsupported,
    #[error("observer subscription is outside the verified owner or workspace")]
    ScopeMismatch,
    #[error("observer subscription already exists")]
    AlreadyExists,
    #[error("observer subscription does not exist")]
    NotFound,
    #[error("observer subscription revision changed")]
    Conflict,
    #[error("observer subscription immutable identity changed")]
    ImmutableField,
    #[error("observer subscription is malformed or exceeds bounded limits")]
    InvalidRecord,
    #[error("observer subscription revision is exhausted")]
    RevisionExhausted,
}

impl ObserverSubscription {
    pub fn validate(&self, owner_id: &str, workspace_id: &str) -> Result<(), ObserverStoreError> {
        if self.owner_id != owner_id || self.workspace_id != workspace_id {
            return Err(ObserverStoreError::ScopeMismatch);
        }
        if self.schema_version != OBSERVER_SCHEMA_VERSION
            || self.subscription_id.is_empty()
            || self.owner_id.is_empty()
            || self.owner_user_id.trim().is_empty()
            || self.owner_user_id == "anonymous"
            || self.owner_id != owner_key(&self.owner_user_id, self.owner_tenant_id.as_deref())
            || self.workspace_id.is_empty()
            || self.observer_instance_id.is_empty()
            || self.source_instance_ids.is_empty()
            || self.source_instance_ids.len() > MAX_OBSERVER_SOURCES
            || self.source_instance_ids.iter().any(String::is_empty)
            || self.source_instance_ids.iter().collect::<HashSet<_>>().len()
                != self.source_instance_ids.len()
            || self.source_instance_ids.contains(&self.observer_instance_id)
            || self.limits.max_inbox == 0
            || self.limits.max_inbox > MAX_OBSERVER_INBOX
            || self.limits.max_retries == 0
            || self.limits.max_retries > MAX_OBSERVER_RETRIES
            || self.limits.retained_acknowledged > self.limits.max_inbox
            || self.inbox.len() > self.limits.max_inbox as usize
            || self.gaps.len() > MAX_OBSERVER_GAPS
            || self.updated_at < self.created_at
        {
            return Err(ObserverStoreError::InvalidRecord);
        }
        if let Some(conversations) = &self.conversation_ids {
            if conversations.is_empty()
                || conversations.len() > MAX_OBSERVER_CONVERSATIONS
                || conversations.iter().any(String::is_empty)
                || conversations.iter().collect::<HashSet<_>>().len() != conversations.len()
            {
                return Err(ObserverStoreError::InvalidRecord);
            }
        }
        if self.cursors.keys().any(|source| !self.source_instance_ids.contains(source)) {
            return Err(ObserverStoreError::InvalidRecord);
        }
        let mut admission_ids = HashSet::new();
        let mut command_ids = HashSet::new();
        for entry in &self.inbox {
            if entry.occurrence_id.is_empty()
                || entry.observer_command_id.is_empty()
                || entry.occurrence.occurrence_id != entry.occurrence_id
                || entry.occurrence.source_instance_id != entry.source_instance_id
                || entry.occurrence.sequence != entry.source_sequence
                || entry.occurrence.owner_id != self.owner_id
                || entry.occurrence.workspace_id != self.workspace_id
                || entry.attempts > self.limits.max_retries
                || entry.updated_at < entry.admitted_at
                || !admission_ids.insert(&entry.occurrence_id)
                || !command_ids.insert(&entry.observer_command_id)
            {
                return Err(ObserverStoreError::InvalidRecord);
            }
        }
        for gap in &self.gaps {
            if gap.source_instance_id.is_empty()
                || gap.missing_from > gap.missing_through
                || gap.acknowledged_at.is_some_and(|at| at < gap.detected_at)
            {
                return Err(ObserverStoreError::InvalidRecord);
            }
        }
        Ok(())
    }

    pub fn validate_next(&self, next: &Self) -> Result<(), ObserverStoreError> {
        next.validate(&self.owner_id, &self.workspace_id)?;
        if self.subscription_id != next.subscription_id
            || self.observer_instance_id != next.observer_instance_id
            || self.owner_user_id != next.owner_user_id
            || self.owner_tenant_id != next.owner_tenant_id
            || self.created_at != next.created_at
        {
            return Err(ObserverStoreError::ImmutableField);
        }
        if next.revision != self.revision.checked_add(1).ok_or(ObserverStoreError::RevisionExhausted)?
            || next.updated_at < self.updated_at
            || (self.revoked && !next.revoked)
            || self.cursors.iter().any(|(source, cursor)| {
                next.source_instance_ids.contains(source)
                    && next.cursors.get(source).is_none_or(|next_cursor| next_cursor < cursor)
            })
        {
            return Err(ObserverStoreError::InvalidRecord);
        }
        for previous in &self.inbox {
            if let Some(current) = next.inbox.iter().find(|entry| entry.occurrence_id == previous.occurrence_id) {
                if current.source_instance_id != previous.source_instance_id
                    || current.source_sequence != previous.source_sequence
                    || current.occurrence != previous.occurrence
                    || current.observer_command_id != previous.observer_command_id
                    || current.admitted_at != previous.admitted_at
                    || current.updated_at < previous.updated_at
                    || current.attempts < previous.attempts
                    || (matches!(previous.status, ObserverAdmissionStatus::Acknowledged | ObserverAdmissionStatus::DeadLetter)
                        && current.status != previous.status)
                {
                    return Err(ObserverStoreError::InvalidRecord);
                }
            } else if !matches!(previous.status, ObserverAdmissionStatus::Acknowledged) {
                return Err(ObserverStoreError::InvalidRecord);
            }
        }
        if next.inbox.iter().any(|entry| {
            !self.inbox.iter().any(|previous| previous.occurrence_id == entry.occurrence_id)
                && !next.source_instance_ids.contains(&entry.source_instance_id)
        }) {
            return Err(ObserverStoreError::InvalidRecord);
        }
        for previous in &self.gaps {
            if let Some(current) = next.gaps.iter().find(|gap| gap.source_instance_id == previous.source_instance_id && gap.missing_from == previous.missing_from) {
                if current.missing_through < previous.missing_through
                    || (previous.acknowledged_at.is_some()
                        && current.missing_through != previous.missing_through)
                    || current.detected_at != previous.detected_at
                    || (previous.acknowledged_at.is_some() && current.acknowledged_at != previous.acknowledged_at)
                {
                    return Err(ObserverStoreError::InvalidRecord);
                }
            } else if previous.acknowledged_at.is_none() {
                return Err(ObserverStoreError::InvalidRecord);
            }
        }
        Ok(())
    }
}

fn owner_key(user_id: &str, tenant_id: Option<&str>) -> String {
    match tenant_id {
        Some(tenant) => format!("v1:t:{}:{}:s:{}:{}", tenant.len(), tenant, user_id.len(), user_id),
        None => format!("v1:s:{}:{}", user_id.len(), user_id),
    }
}
