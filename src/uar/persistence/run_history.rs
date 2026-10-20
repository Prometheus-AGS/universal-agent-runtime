//! Restart-durable public run history: run metadata plus the events already
//! published to the bounded in-process history. The adopted restart-durability
//! contract covers acknowledged pre-termination events, so every published
//! event is written through, not only a terminal snapshot.
//!
//! Ownership persists as the established namespace key
//! (`ActorOwner::presentation_owner_key`), never as a deserialized principal;
//! readers recompute the key from the live middleware-verified context and
//! compare on every read. `None` is the anonymous namespace.
use serde::{Deserialize, Serialize};

/// Persisted form of the public `Run` record within its ownership namespace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunHistoryRecord {
    pub owner_key: Option<String>,
    pub run_id: String,
    /// Serialized public wire record (`crate::uar::domain::runs::Run`),
    /// including the terminal status once reached.
    pub run: serde_json::Value,
    pub revision: u64,
}

/// One published stream event, ordered by `event_id` within its run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunHistoryEvent {
    pub run_id: String,
    pub event_id: u64,
    /// Serialized `crate::uar::domain::events::NormalizedEvent` exactly as it
    /// was published to the in-process history buffer.
    pub event: serde_json::Value,
}

/// Length-prefixed record key so identifiers cannot blur key boundaries.
pub(crate) fn record_key(run_id: &str) -> String {
    format!("{}:{run_id}", run_id.len())
}

pub(crate) fn event_key(run_id: &str, event_id: u64) -> String {
    format!("{}:{event_id}", record_key(run_id))
}
