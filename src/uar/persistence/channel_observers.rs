//! Separate channel-source profile. C07 logical-instance observers are unchanged.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const CHANNEL_OBSERVER_PROFILE: &str = "uar.channel-source/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChannelProjectionClass {
    #[default]
    MetadataOnly,
    PolicyFiltered,
}

impl ChannelProjectionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata_only",
            Self::PolicyFiltered => "policy_filtered",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelSourceScope {
    pub provider: String,
    pub account: String,
    pub workspace: String,
    pub room: String,
    pub thread: Option<String>,
    pub sender: String,
}

impl ChannelSourceScope {
    pub fn valid(&self) -> bool {
        [
            &self.provider,
            &self.account,
            &self.workspace,
            &self.room,
            &self.sender,
        ]
        .into_iter()
        .all(|value| !value.trim().is_empty())
            && self.thread.as_ref().is_none_or(|value| !value.trim().is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelSubscription {
    pub profile: String,
    pub subscription_id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub observer_instance_id: String,
    pub source: ChannelSourceScope,
    pub grant_issuer: String,
    pub grant_id: String,
    pub revision: u64,
    pub cursor: Option<String>,
    pub paused: bool,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelInboxEntry {
    pub subscription_id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub occurrence_id: String,
    pub native_message_id: String,
    pub source_tenant_id: String,
    pub subscriber_cursor_id: String,
    pub delivery_id: String,
    pub route_id: String,
    pub route_revision: String,
    pub binding_revision: String,
    pub policy_revision: String,
    pub original_actor: String,
    pub original_principal: String,
    pub payload_sha256: String,
    #[serde(default)]
    pub classification: ChannelProjectionClass,
    /// Present only after a fresh recipient-delivery release for this text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_projection: Option<String>,
    pub status: ChannelInboxStatus,
    pub gate_receipt_id: Option<String>,
    pub admitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelInboxStatus {
    PendingAuthority,
    Admitted,
    Withheld,
    Uncertain,
    Acknowledged,
}

/// Content-free delivery state for the authenticated operator inventory.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelDeliveryMetadata {
    pub delivery_id: String,
    pub occurrence_id: String,
    pub subscriber_cursor_id: String,
    pub status: ChannelInboxStatus,
    pub admitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ChannelInboxEntry> for ChannelDeliveryMetadata {
    fn from(entry: ChannelInboxEntry) -> Self {
        Self {
            delivery_id: entry.delivery_id,
            occurrence_id: entry.occurrence_id,
            subscriber_cursor_id: entry.subscriber_cursor_id,
            status: entry.status,
            admitted_at: entry.admitted_at,
            updated_at: entry.updated_at,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelObserverStoreError {
    #[error("channel observer persistence is unsupported")]
    Unsupported,
    #[error("channel observer record conflicts with existing identity")]
    Conflict,
    #[error("channel observer record is outside its owner or workspace")]
    ScopeMismatch,
}
