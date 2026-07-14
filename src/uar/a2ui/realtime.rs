//! A2UI live-update backbone (Change 20, `a2ui-realtime-backbone-from-flint-realtime-fabric`).
//!
//! Scope correction found during implementation: the plan frames this as
//! "wire `flint-realtime-fabric` as the SSE/fan-out backbone for A2UI live
//! updates," but auditing the existing code first found:
//!
//! 1. **No A2UI emission call site exists yet.** `super::protocol`'s
//!    `A2uiMessage`/`CreateMessage`/`ComponentsMessage`/`DataMessage`/`DeleteMessage`
//!    DTOs are `Deserialize`-only, `pub(crate)`, and referenced nowhere else
//!    in the crate (`#[expect(dead_code, ...)]` on the module confirms
//!    they're contract-validation scaffolding, not a live code path). The
//!    orchestrator does not currently create or update A2UI surfaces during
//!    a run.
//! 2. **Multi-client fan-out ("convergence") already exists.**
//!    [`crate::uar::runtime::manager::RunManager`] broadcasts every
//!    [`NormalizedEvent`] to all subscribers of a run via a `tokio::sync::broadcast`
//!    channel (`RunManager::subscribe`) — multiple SSE clients on the same
//!    `run_id` already converge on the same event stream. This module does
//!    not need to reimplement that.
//! 3. **The actual gap is durable, replayable fan-out for "late-join
//!    reattach."** A `broadcast` channel has a bounded buffer and no
//!    replay-from-start semantics — a client that connects after events
//!    have already fired gets nothing before its subscribe point, and a
//!    slow client can miss events entirely (`RecvError::Lagged`). This is
//!    exactly the capability `flint-realtime-fabric` (a durable pub/sub
//!    broker, `frf-sdk-rust`'s `FrfClient::subscribe(..., Offset::BEGINNING)`)
//!    is for.
//!
//! So this module's real, buildable scope is: (a) a conversion from A2UI
//! wire messages to [`StatePatchOp`]s so *whenever* a future orchestrator
//! call site does emit A2UI surface changes, they compose with the
//! existing `NormalizedEvent::StatePatch`/`RunManager` broadcast pipeline
//! for free; and (b) an [`A2uiReplayBackbone`] trait with a real, tested
//! in-process implementation ([`InMemoryReplayBackbone`]) for late-join
//! replay. The `flint-realtime-fabric`-backed implementation is designed
//! but not wired as a live Cargo dependency this pass — see "Deferred"
//! below.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::uar::domain::events::StatePatchOp;

/// The 4 A2UI v0.9 wire-message kinds, per `docs/protocols/a2ui-profile.md`
/// and `super::protocol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A2uiWireKind {
    CreateSurface,
    UpdateComponents,
    UpdateDataModel,
    DeleteSurface,
}

/// Converts a single A2UI wire message into a [`StatePatchOp`] rooted at
/// `/a2ui/surfaces/{surface_id}`, so it can be emitted as a
/// `NormalizedEvent::StatePatch` and ride the existing
/// [`crate::uar::runtime::manager::RunManager`] broadcast/SSE pipeline.
///
/// Each A2UI message becomes exactly one coarse-grained patch op (not a
/// field-by-field JSON-Patch decomposition of the surface tree) — the
/// message's own `kind` plus its full `payload` is enough for a consumer to
/// reconstruct the surface, and A2UI's own `GenericBinder` (client-side)
/// already owns fine-grained reactive diffing once the message arrives.
/// Re-deriving that here would duplicate logic that belongs to the A2UI
/// client library, not this backend bridge.
#[must_use]
pub fn surface_message_to_state_patch(
    surface_id: &str,
    kind: A2uiWireKind,
    payload: Value,
) -> StatePatchOp {
    match kind {
        A2uiWireKind::CreateSurface => StatePatchOp {
            op: "add".to_string(),
            path: format!("/a2ui/surfaces/{surface_id}"),
            value: Some(payload),
        },
        A2uiWireKind::UpdateComponents => StatePatchOp {
            op: "replace".to_string(),
            path: format!("/a2ui/surfaces/{surface_id}/components"),
            value: Some(payload),
        },
        A2uiWireKind::UpdateDataModel => StatePatchOp {
            op: "replace".to_string(),
            path: format!("/a2ui/surfaces/{surface_id}/dataModel"),
            value: Some(payload),
        },
        A2uiWireKind::DeleteSurface => StatePatchOp {
            op: "remove".to_string(),
            path: format!("/a2ui/surfaces/{surface_id}"),
            value: None,
        },
    }
}

/// A durable-replay backbone for A2UI state patches, keyed by `run_id`.
/// Distinct from `RunManager`'s live broadcast: this trait's job is
/// specifically "can a client that joins *after* some patches were already
/// published still receive them," which a `tokio::sync::broadcast` channel
/// cannot do once its buffer has advanced past a slow/late subscriber.
pub trait A2uiReplayBackbone: Send + Sync {
    /// Publishes a patch for `run_id`, retaining it for later replay.
    fn publish(&self, run_id: &str, op: StatePatchOp);

    /// Returns every patch published for `run_id` so far, in publish order.
    /// A late-joining client calls this once on connect (its "reattach"),
    /// then switches to the live broadcast for anything published after.
    fn replay(&self, run_id: &str) -> Vec<StatePatchOp>;
}

/// In-process, single-instance implementation of [`A2uiReplayBackbone`].
/// Retains the full patch history per run in memory for the process's
/// lifetime — real and useful for tests and a single-server deployment,
/// but explicitly **not** durable across process restarts or usable across
/// multiple server instances. That cross-instance/durable case is exactly
/// what the `flint-realtime-fabric`-backed implementation (deferred, see
/// module docs) is for.
#[derive(Debug, Default)]
pub struct InMemoryReplayBackbone {
    history: Mutex<HashMap<String, Vec<StatePatchOp>>>,
}

impl InMemoryReplayBackbone {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}

impl A2uiReplayBackbone for InMemoryReplayBackbone {
    fn publish(&self, run_id: &str, op: StatePatchOp) {
        self.history
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(run_id.to_string())
            .or_default()
            .push(op);
    }

    fn replay(&self, run_id: &str) -> Vec<StatePatchOp> {
        self.history
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(run_id)
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_surface_becomes_an_add_op_at_the_surface_path() {
        let op = surface_message_to_state_patch(
            "surface-1",
            A2uiWireKind::CreateSurface,
            serde_json::json!({ "catalogId": "urn:uar:a2ui:catalog:1" }),
        );
        assert_eq!(op.op, "add");
        assert_eq!(op.path, "/a2ui/surfaces/surface-1");
        assert!(op.value.is_some());
    }

    #[test]
    fn update_components_becomes_a_replace_op_at_the_components_subpath() {
        let op = surface_message_to_state_patch(
            "surface-1",
            A2uiWireKind::UpdateComponents,
            serde_json::json!({ "components": [] }),
        );
        assert_eq!(op.op, "replace");
        assert_eq!(op.path, "/a2ui/surfaces/surface-1/components");
    }

    #[test]
    fn update_data_model_becomes_a_replace_op_at_the_data_model_subpath() {
        let op = surface_message_to_state_patch(
            "surface-1",
            A2uiWireKind::UpdateDataModel,
            serde_json::json!({ "foo": "bar" }),
        );
        assert_eq!(op.op, "replace");
        assert_eq!(op.path, "/a2ui/surfaces/surface-1/dataModel");
    }

    #[test]
    fn delete_surface_becomes_a_remove_op_with_no_value() {
        let op =
            surface_message_to_state_patch("surface-1", A2uiWireKind::DeleteSurface, Value::Null);
        assert_eq!(op.op, "remove");
        assert_eq!(op.path, "/a2ui/surfaces/surface-1");
        assert!(op.value.is_none());
    }

    #[test]
    fn replay_backbone_returns_published_patches_in_order_per_run() {
        let backbone = InMemoryReplayBackbone::new();
        let op1 = surface_message_to_state_patch(
            "s1",
            A2uiWireKind::CreateSurface,
            serde_json::json!({}),
        );
        let op2 = surface_message_to_state_patch(
            "s1",
            A2uiWireKind::UpdateComponents,
            serde_json::json!({}),
        );
        backbone.publish("run-a", op1.clone());
        backbone.publish("run-a", op2.clone());
        backbone.publish("run-b", op1.clone());

        let replayed = backbone.replay("run-a");
        assert_eq!(replayed, vec![op1, op2]);
    }

    #[test]
    fn replay_backbone_returns_empty_for_a_run_with_no_published_patches() {
        let backbone = InMemoryReplayBackbone::new();
        assert!(backbone.replay("nonexistent-run").is_empty());
    }

    /// Demonstrates "multi-client convergence" (the first of Change 20's 2
    /// BDD scenarios) at the unit level: multiple independent readers of
    /// the same backbone's `replay()` for a run see the identical patch
    /// sequence. The full BDD scenario (`tests/bdd/features/a2ui-live-update.feature`)
    /// exercises this through `RunManager`'s real SSE broadcast instead,
    /// since that's the actual live-client transport; this test isolates
    /// the backbone's own guarantee.
    #[test]
    fn two_independent_readers_converge_on_the_same_patch_sequence() {
        let backbone = InMemoryReplayBackbone::new();
        let op = surface_message_to_state_patch(
            "s1",
            A2uiWireKind::CreateSurface,
            serde_json::json!({}),
        );
        backbone.publish("run-a", op.clone());

        let reader_one = backbone.replay("run-a");
        let reader_two = backbone.replay("run-a");
        assert_eq!(reader_one, reader_two);
        assert_eq!(reader_one, vec![op]);
    }
}
