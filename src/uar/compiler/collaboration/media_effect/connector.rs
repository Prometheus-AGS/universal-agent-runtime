use std::{collections::{BTreeMap, btree_map::Entry}, fmt};

use serde::{Deserialize, Serialize};

use super::super::validation::request_digest;
use crate::uar::domain::media_effect::{MediaAction, MediaEffect, MediaEffectStatus, MediaTarget};

/// Single-use work claim returned to the trusted host after durable admission.
/// Content is supplied again at claim time and never written to the catalog.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaDispatchClaim {
    pub effect: MediaEffect,
    pub content: Option<String>,
}

impl fmt::Debug for MediaDispatchClaim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaDispatchClaim")
            .field("effect", &self.effect)
            .finish_non_exhaustive()
    }
}

/// Host implementation of a media peer. The implementation resolves any
/// processor credentials privately and keeps speech and video credentials
/// separate. Neither the claim nor the outcome carries credential material.
/// Reconciliation must be a read of evidence and must never invoke the effect.
#[async_trait::async_trait]
pub trait MediaHostConnector {
    async fn invoke(&mut self, claim: MediaDispatchClaim) -> MediaConnectorResult;
    async fn reconcile(&self, attempt_id: &str) -> MediaConnectorResult;
}

/// Result visible to the trusted host. Evidence references are opaque IDs.
/// A lost response remains uncertain even if the peer applied the effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum MediaConnectorResult {
    Applied { evidence_ref: String },
    NotApplied { evidence_ref: String },
    Unknown,
}

/// Deterministic, network-free peer for use in a separate fixture process.
/// A peer instance survives UAR restarts and remembers session digests and
/// typed results keyed by opaque IDs.
#[derive(Debug, Default)]
pub struct MockMediaPeer {
    sessions: BTreeMap<String, MockMediaSession>,
    attempts_by_effect: BTreeMap<String, String>,
    results_by_attempt: BTreeMap<String, MediaConnectorResult>,
    invocations: usize,
    applied: usize,
    lose_next_result: bool,
}

#[derive(Debug, Clone, Copy)]
enum MockMediaSession {
    JoinedRoom,
    CoordinatorCall,
    HumanCall,
}

impl MockMediaPeer {
    /// Return the total calls received by the peer, including duplicate calls.
    #[must_use]
    pub fn invocations(&self) -> usize {
        self.invocations
    }

    /// Return the number of externally visible effects the peer applied.
    #[must_use]
    pub fn applied(&self) -> usize {
        self.applied
    }

    /// Drop the next invocation response after recording the peer's result.
    pub fn lose_next_result(&mut self) {
        self.lose_next_result = true;
    }

    fn apply_action(&mut self, effect: &MediaEffect, session_key: String) -> bool {
        use MediaAction::{EndCall, EndMedia, Join, Publish, SendContent, StartCall, Transfer};
        use MockMediaSession::{CoordinatorCall, HumanCall, JoinedRoom};

        match (effect.permission.action, &effect.scope.target, self.sessions.entry(session_key)) {
            (Join, MediaTarget::Room(_), Entry::Vacant(entry)) => {
                entry.insert(JoinedRoom);
                true
            }
            (StartCall, MediaTarget::Recipient(_), Entry::Vacant(entry)) => {
                entry.insert(CoordinatorCall);
                true
            }
            (Publish | SendContent, MediaTarget::Room(_), Entry::Occupied(entry)) => {
                matches!(entry.get(), JoinedRoom)
            }
            (EndMedia, MediaTarget::Room(_), Entry::Occupied(entry))
                if matches!(entry.get(), JoinedRoom) =>
            {
                entry.remove();
                true
            }
            (Transfer, MediaTarget::Recipient(_), Entry::Occupied(mut entry))
                if matches!(entry.get(), CoordinatorCall) =>
            {
                entry.insert(HumanCall);
                true
            }
            (EndCall, MediaTarget::Recipient(_), Entry::Occupied(entry))
                if matches!(entry.get(), CoordinatorCall | HumanCall) =>
            {
                entry.remove();
                true
            }
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl MediaHostConnector for MockMediaPeer {
    async fn invoke(&mut self, claim: MediaDispatchClaim) -> MediaConnectorResult {
        self.invocations += 1;
        let lost = std::mem::take(&mut self.lose_next_result);
        let Some(attempt_id) = claim.effect.attempt_id.as_ref() else {
            return MediaConnectorResult::Unknown;
        };
        if claim.effect.status != MediaEffectStatus::Unknown {
            return MediaConnectorResult::Unknown;
        }
        if let Some(prior) = self.attempts_by_effect.get(&claim.effect.id) {
            if lost || prior != attempt_id {
                return MediaConnectorResult::Unknown;
            }
            return self.reconcile(attempt_id).await;
        }
        if self.results_by_attempt.contains_key(attempt_id) {
            return MediaConnectorResult::Unknown;
        }
        let Ok(session_key) = request_digest(&(&claim.effect.authority, &claim.effect.scope)) else {
            return MediaConnectorResult::Unknown;
        };
        // The mock's external evidence is keyed by the opaque effect UUID.
        let evidence_ref = claim.effect.id.clone();
        let result = if self.apply_action(&claim.effect, session_key) {
            self.applied += 1;
            MediaConnectorResult::Applied { evidence_ref }
        } else {
            MediaConnectorResult::NotApplied { evidence_ref }
        };
        self.attempts_by_effect.insert(claim.effect.id.clone(), attempt_id.clone());
        self.results_by_attempt.insert(attempt_id.clone(), result.clone());
        if lost {
            MediaConnectorResult::Unknown
        } else {
            result
        }
    }

    async fn reconcile(&self, attempt_id: &str) -> MediaConnectorResult {
        self.results_by_attempt.get(attempt_id).cloned().unwrap_or(MediaConnectorResult::Unknown)
    }
}
