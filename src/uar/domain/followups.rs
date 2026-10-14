//! Durable owner-controlled follow-up decisions and conversation inbox records.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::collaboration::{PrivateRevisionRef, RepresentationGrantRef};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuietHours { pub start: String, pub end: String, pub timezone: String }

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FollowupRequest {
    pub source_instance_id: String,
    pub destination_instance_id: String,
    pub conversation_id: String,
    pub event_kinds: Vec<String>,
    #[serde(default)] pub enabled: bool,
    pub quiet_hours: Option<QuietHours>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FollowupAuthority {
    pub binding: PrivateRevisionRef,
    pub representation_revision: u64,
    pub representation_grants: Vec<RepresentationGrantRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FollowupDisposition { Queued, Deferred, Suppressed, Delivered }

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FollowupDecision {
    pub id: String,
    pub rule_revision: u64,
    pub occurrence_id: String,
    pub source_sequence: u64,
    pub source_instance_id: String,
    pub event_kind: String,
    pub occurred_at: DateTime<Utc>,
    pub disposition: FollowupDisposition,
    pub reason: String,
    pub available_at: Option<DateTime<Utc>>,
    pub decided_at: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FollowupRule {
    pub id: String,
    pub owner_key: String,
    pub catalog_owner_id: String,
    pub owner_grant: crate::uar::security::claims::FollowupOwnerGrant,
    pub workspace_id: String,
    pub revision: u64,
    pub enabled: bool,
    pub revoked: bool,
    pub source_instance_id: String,
    pub destination_instance_id: String,
    pub conversation_id: String,
    pub source_authority: FollowupAuthority,
    pub destination_authority: FollowupAuthority,
    pub event_kinds: Vec<String>,
    pub quiet_hours: Option<QuietHours>,
    pub cursor: Option<u64>,
    pub decisions: Vec<FollowupDecision>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl std::fmt::Debug for FollowupRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FollowupRule").field("id", &self.id).field("revision", &self.revision).finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FollowupInboxItem {
    pub id: String,
    pub owner_key: String,
    pub workspace_id: String,
    pub conversation_id: String,
    pub rule_id: String,
    pub rule_revision: u64,
    pub occurrence_id: String,
    pub source_instance_id: String,
    pub event_kind: String,
    pub occurred_at: DateTime<Utc>,
    pub delivered_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowupView {
    pub id: String,
    pub workspace_id: String,
    pub revision: u64,
    pub enabled: bool,
    pub revoked: bool,
    pub source_instance_id: String,
    pub destination_instance_id: String,
    pub conversation_id: String,
    pub event_kinds: Vec<String>,
    pub quiet_hours: Option<QuietHours>,
    pub cursor: Option<u64>,
    pub decisions: Vec<FollowupDecisionView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowupDecisionView {
    pub id: String,
    pub rule_revision: u64,
    pub disposition: FollowupDisposition,
    pub reason: String,
    pub available_at: Option<DateTime<Utc>>,
}
impl From<&FollowupRule> for FollowupView {
    fn from(r: &FollowupRule) -> Self {
        Self { id: r.id.clone(), workspace_id: r.workspace_id.clone(), revision: r.revision,
            enabled: r.enabled, revoked: r.revoked, source_instance_id: r.source_instance_id.clone(),
            destination_instance_id: r.destination_instance_id.clone(), conversation_id: r.conversation_id.clone(),
            event_kinds: r.event_kinds.clone(), quiet_hours: r.quiet_hours.clone(), cursor: r.cursor,
            decisions: r.decisions.iter().map(|d| FollowupDecisionView { id: d.id.clone(), rule_revision: d.rule_revision,
                disposition: d.disposition.clone(), reason: d.reason.clone(), available_at: d.available_at }).collect() }
    }
}
