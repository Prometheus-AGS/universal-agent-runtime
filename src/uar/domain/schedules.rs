//! Protected schedule intent and inspectable occurrence receipts.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Deliberately bounded calendar input; arbitrary cron/RRULE is not accepted.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum ScheduleRecurrence {
    Once { at: DateTime<Utc> },
    Daily { time: String, timezone: String },
    Weekly { weekdays: Vec<u8>, time: String, timezone: String },
}

/// Authenticated request. Owner authority is never accepted in this document.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScheduleRequest {
    pub instance_id: String,
    pub recurrence: ScheduleRecurrence,
    pub prompt: String,
}

/// A current-revision pending occurrence has no execution authority yet.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScheduleOccurrence {
    pub id: String,
    pub revision: u64,
    pub due_at: DateTime<Utc>,
    pub command_id: String,
}

/// Durable guard survives edits, approvals, uncertainty and process death.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScheduleGuard {
    pub occurrence: ScheduleOccurrence,
    pub instance_id: String,
    pub owner_grant: crate::uar::security::claims::ScheduleOwnerGrant,
    pub execution_fence: crate::uar::domain::team_execution::TeamExecutionFence,
    /// Retain admitted input across subsequent intent edits; never project it.
    pub prompt: String,
    pub dispatch_started: bool,
    pub command_observed: bool,
    pub run_id: Option<String>,
    pub claimed_at: DateTime<Utc>,
    pub last_error: Option<String>,
}

/// Bounded metadata records why a candidate was never executed.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScheduleSkip {
    pub revision: u64,
    pub reason: String,
    pub first: String,
    pub last: String,
    pub count: u64,
}

/// A terminal disposition is retained independently of the instance inbox.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScheduleReceipt {
    pub occurrence: ScheduleOccurrence,
    pub run_id: Option<String>,
    pub disposition: String,
    pub resolved_at: DateTime<Utc>,
    pub instance_id: String,
    pub owner_grant: crate::uar::security::claims::ScheduleOwnerGrant,
    pub inbox_released: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleReceiptView {
    pub occurrence: ScheduleOccurrence,
    pub run_id: Option<String>,
    pub disposition: String,
    pub resolved_at: DateTime<Utc>,
}

/// Stored only in the trusted catalog, never deserialized from an API request.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DurableSchedule {
    pub id: String,
    pub owner_key: String,
    pub owner_grant: crate::uar::security::claims::ScheduleOwnerGrant,
    pub workspace_id: String,
    pub instance_id: String,
    pub recurrence: ScheduleRecurrence,
    pub prompt: String,
    pub revision: u64,
    pub paused: bool,
    pub cancelled: bool,
    pub effective_at: DateTime<Utc>,
    pub next_local_date: Option<NaiveDate>,
    pub once_scanned: bool,
    pub pending: Option<ScheduleOccurrence>,
    pub guard: Option<ScheduleGuard>,
    pub skipped: Vec<ScheduleSkip>,
    pub receipts: Vec<ScheduleReceipt>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl std::fmt::Debug for DurableSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DurableSchedule").field("id", &self.id)
            .field("revision", &self.revision).finish_non_exhaustive()
    }
}

/// Status deliberately excludes private input and the persisted owner grant.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleView {
    pub id: String,
    pub workspace_id: String,
    pub instance_id: String,
    pub recurrence: ScheduleRecurrence,
    pub revision: u64,
    pub paused: bool,
    pub cancelled: bool,
    pub next_local_date: Option<NaiveDate>,
    pub pending: Option<ScheduleOccurrence>,
    pub active: Option<ScheduleOccurrence>,
    pub active_run_id: Option<String>,
    pub last_error: Option<String>,
    pub skipped: Vec<ScheduleSkip>,
    pub receipts: Vec<ScheduleReceiptView>,
}

impl From<&DurableSchedule> for ScheduleView {
    fn from(value: &DurableSchedule) -> Self {
        Self {
            id: value.id.clone(), workspace_id: value.workspace_id.clone(),
            instance_id: value.instance_id.clone(), recurrence: value.recurrence.clone(),
            revision: value.revision, paused: value.paused, cancelled: value.cancelled,
            next_local_date: value.next_local_date, pending: value.pending.clone(),
            active: value.guard.as_ref().map(|g| g.occurrence.clone()),
            active_run_id: value.guard.as_ref().and_then(|g| g.run_id.clone()),
            last_error: value.guard.as_ref().and_then(|g| g.last_error.clone()),
            skipped: value.skipped.clone(), receipts: value.receipts.iter().map(|r| ScheduleReceiptView {
                occurrence: r.occurrence.clone(), run_id: r.run_id.clone(), disposition: r.disposition.clone(), resolved_at: r.resolved_at,
            }).collect(),
        }
    }
}
