//! Typed host media intents and durable effect records. Media bytes and
//! credentials are never part of the collaboration catalog.

use std::{collections::BTreeSet, fmt};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaAction {
    Join,
    Publish,
    SendContent,
    EndMedia,
    StartCall,
    EndCall,
    Transfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaModality {
    Audio,
    Video,
    Text,
}

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[expect(missing_debug_implementations, reason = "media target identifiers are private")]
pub enum MediaTarget {
    Room(String),
    Recipient(String),
}

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaScope {
    pub session_id: String,
    pub target: MediaTarget,
    pub purpose: String,
}

impl fmt::Debug for MediaScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaScope").finish_non_exhaustive()
    }
}

/// The exact combination approved by the user. Matching independent sets of
/// actions and modalities would authorize combinations never confirmed.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaPermission {
    pub action: MediaAction,
    pub modality: MediaModality,
    pub capability: String,
}

impl fmt::Debug for MediaPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaPermission")
            .field("action", &self.action)
            .field("modality", &self.modality)
            .finish_non_exhaustive()
    }
}

/// Resolved from host authentication and workspace context at ingress.
/// Persisted for equality checks, never accepted as a request field.
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAuthorityScope {
    pub owner_id: String,
    pub workspace_id: String,
    pub tenant_id: Option<String>,
    pub host_id: String,
}

impl fmt::Debug for MediaAuthorityScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaAuthorityScope").finish_non_exhaustive()
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[expect(missing_debug_implementations, reason = "confirmation requests carry private actor and scope identifiers")]
pub struct ConfirmMediaGrantRequest {
    pub command_id: String,
    pub coordinator_id: String,
    pub scope: MediaScope,
    pub permissions: BTreeSet<MediaPermission>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[expect(missing_debug_implementations, reason = "delegation requests carry private actor and grant identifiers")]
pub struct DelegateMediaGrantRequest {
    pub command_id: String,
    pub grant_id: String,
    pub worker_id: String,
    pub permissions: BTreeSet<MediaPermission>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[expect(missing_debug_implementations, reason = "effect requests carry private scope and content metadata")]
pub struct PrepareMediaEffectRequest {
    pub command_id: String,
    pub grant_id: String,
    /// The trusted host's authenticated coordinator or worker identity.
    pub actor_id: String,
    pub delegation_id: Option<String>,
    pub scope: MediaScope,
    pub permission: MediaPermission,
    pub content_digest: Option<String>,
}

/// Content is supplied again at dispatch and compared to the prepared digest.
/// This request has no serialization or diagnostic representation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[expect(missing_debug_implementations, reason = "dispatch requests can contain raw meeting content")]
pub struct DispatchMediaEffectRequest {
    pub content: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[expect(missing_debug_implementations, reason = "command identifiers are kept out of diagnostics")]
pub struct MediaCommandRequest {
    pub command_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaOutcome {
    Applied,
    NotApplied,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[expect(missing_debug_implementations, reason = "outcome requests carry private attempt and evidence identifiers")]
pub struct MediaOutcomeRequest {
    pub command_id: String,
    pub attempt_id: String,
    pub outcome: MediaOutcome,
    pub evidence_ref: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaConfirmation {
    pub id: String,
    pub authority: MediaAuthorityScope,
    pub coordinator_id: String,
    pub scope: MediaScope,
    pub permissions: BTreeSet<MediaPermission>,
    pub expires_at: DateTime<Utc>,
    pub confirmed_at: DateTime<Utc>,
}

impl fmt::Debug for MediaConfirmation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaConfirmation")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// Revocation is mutable. Scope and expiry are read from the immutable
/// confirmation identified by `confirmation_id`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaGrant {
    pub id: String,
    pub confirmation_id: String,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Delegation narrows the referenced grant. It inherits target, session,
/// tenant, host, and purpose from the parent confirmation.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDelegation {
    pub id: String,
    pub grant_id: String,
    pub worker_id: String,
    pub permissions: BTreeSet<MediaPermission>,
    pub expires_at: DateTime<Utc>,
}

impl fmt::Debug for MediaDelegation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaDelegation")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaEffectStatus {
    Prepared,
    Cancelled,
    /// A durable attempt exists and may have reached the external peer.
    Unknown,
    Applied,
    NotApplied,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaEffect {
    /// Stable idempotency key returned on preparation and kept across retries.
    pub id: String,
    pub authority: MediaAuthorityScope,
    pub actor_id: String,
    pub grant_id: String,
    pub confirmation_id: String,
    pub delegation_id: Option<String>,
    pub scope: MediaScope,
    pub permission: MediaPermission,
    pub content_digest: Option<String>,
    pub status: MediaEffectStatus,
    pub attempt_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl fmt::Debug for MediaEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaEffect")
            .field("id", &self.id)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

/// An append-only record of a media effect transition. No peer response body,
/// provider credentials, or conversation content enters the receipt.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaReceipt {
    pub effect_id: String,
    pub actor_id: String,
    pub grant_id: String,
    pub confirmation_id: String,
    pub attempt_id: Option<String>,
    pub disposition: MediaEffectStatus,
    pub evidence_ref: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

impl fmt::Debug for MediaReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaReceipt")
            .field("effect_id", &self.effect_id)
            .field("disposition", &self.disposition)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCommandReceipt {
    pub request_digest: String,
    pub resource_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaEffectView {
    pub effect: MediaEffect,
    pub receipts: Vec<MediaReceipt>,
}
