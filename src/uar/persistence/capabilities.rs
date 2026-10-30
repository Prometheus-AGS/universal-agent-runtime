//! Durable capability names and the typed refusal for a missing capability.
//!
//! The names are the `PersistenceLayer::supports_*` flag names, so the
//! capability matrix keys and the `capability` field of a refusal use one
//! vocabulary.

/// Stable error code returned to HTTP clients when the active store lacks a
/// durable capability an operation needs.
pub const CAPABILITY_UNSUPPORTED: &str = "capability_unsupported";

pub const DURABLE_AGENT_INSTANCES: &str = "supports_durable_agent_instances";
pub const DURABLE_OBSERVERS: &str = "supports_durable_observers";
pub const DURABLE_ACCOUNTING: &str = "supports_durable_accounting";
pub const CHANNEL_OBSERVERS: &str = "supports_channel_observers";
/// Follow-up and schedule rules live in the collaboration catalog, which has no
/// durable form on the in-memory backend.
pub const DURABLE_COLLABORATION_CATALOG: &str = "durable_collaboration_catalog";

/// JSON error envelope for a capability refusal:
/// `{"error":{"code":"capability_unsupported","capability":"<name>"}}`.
#[must_use]
pub fn refusal_body(capability: &str) -> serde_json::Value {
    serde_json::json!({"error": {"code": CAPABILITY_UNSUPPORTED, "capability": capability}})
}
