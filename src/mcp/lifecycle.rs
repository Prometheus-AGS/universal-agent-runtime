//! Ordered, owner-scoped MCP lifecycle observations with bounded retention.

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock, RwLock, Weak};

use tokio::sync::broadcast;
use uuid::Uuid;

use crate::uar::domain::events::{
    McpServerLifecycle, McpServerState, McpStateReason, NormalizedEvent,
};

const LIFECYCLE_EVENT_CAPACITY: usize = 128;

struct LifecycleState {
    latest: McpServerLifecycle,
    sender: broadcast::Sender<NormalizedEvent>,
}

type BindingHealth = BTreeMap<String, BTreeMap<Uuid, bool>>;

fn binding_health() -> &'static RwLock<BindingHealth> {
    static HEALTH: OnceLock<RwLock<BindingHealth>> = OnceLock::new();
    HEALTH.get_or_init(|| RwLock::new(BTreeMap::new()))
}

fn record_binding_health(server: &str, binding_id: Uuid, healthy: bool) {
    let mut health = binding_health()
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let bindings = health.entry(server.to_owned()).or_default();
    bindings.insert(binding_id, healthy);
    let any_ready = bindings.values().any(|ready| *ready);
    // Serialize aggregate computation and publication; otherwise an older
    // writer could publish after a newer state released this lock.
    crate::uar::telemetry::metrics::set_mcp_server_status(server, any_ready);
}

impl Drop for LifecycleState {
    fn drop(&mut self) {
        let mut health = binding_health()
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(bindings) = health.get_mut(&self.latest.server) else {
            return;
        };
        bindings.remove(&self.latest.binding_id);
        let any_ready = bindings.values().any(|ready| *ready);
        if bindings.is_empty() {
            health.remove(&self.latest.server);
        }
        crate::uar::telemetry::metrics::set_mcp_server_status(&self.latest.server, any_ready);
    }
}

/// Publisher is retained inside one exact owner/config/auth/environment entry.
#[derive(Clone)]
pub(crate) struct McpLifecycle(Arc<RwLock<LifecycleState>>);

impl McpLifecycle {
    pub(crate) fn new(server: String, generation: Uuid, authentication_required: bool) -> Self {
        let (sender, _) = broadcast::channel(LIFECYCLE_EVENT_CAPACITY);
        let state = if authentication_required {
            McpServerState::AuthRequired
        } else {
            McpServerState::Disabled
        };
        let latest = McpServerLifecycle {
            event_id: Uuid::new_v4(),
            binding_id: Uuid::new_v4(),
            generation,
            sequence: 0,
            timestamp: chrono::Utc::now(),
            server,
            state,
            reason: authentication_required.then_some(McpStateReason::AuthenticationRequired),
        };
        record_binding_health(&latest.server, latest.binding_id, false);
        Self(Arc::new(RwLock::new(LifecycleState { latest, sender })))
    }

    pub(crate) fn transition(
        &self,
        generation: Uuid,
        next: McpServerState,
        reason: Option<McpStateReason>,
    ) {
        let mut state = self
            .0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.latest.generation != generation
            || state.latest.state == McpServerState::ShuttingDown
            || (state.latest.state == next && state.latest.reason == reason)
        {
            return;
        }
        publish(&mut state, generation, next, reason);
    }

    pub(crate) fn advance(&self, generation: Uuid, reason: McpStateReason) {
        let mut state = self
            .0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        publish(
            &mut state,
            generation,
            McpServerState::Disabled,
            Some(reason),
        );
    }

    pub(crate) fn cancel_connecting(&self, generation: Uuid) {
        let mut state = self
            .0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.latest.generation == generation && state.latest.state == McpServerState::Connecting
        {
            publish(
                &mut state,
                generation,
                McpServerState::Failed,
                Some(McpStateReason::Cancelled),
            );
        }
    }

    pub(crate) fn subscribe(&self) -> McpLifecycleSubscription {
        let state = self
            .0
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        McpLifecycleSubscription {
            initial: Some(normalized(&state.latest)),
            receiver: state.sender.subscribe(),
            lifecycle: Arc::downgrade(&self.0),
        }
    }
}

fn publish(
    state: &mut LifecycleState,
    generation: Uuid,
    next: McpServerState,
    reason: Option<McpStateReason>,
) {
    state.latest = McpServerLifecycle {
        event_id: Uuid::new_v4(),
        binding_id: state.latest.binding_id,
        generation,
        sequence: state.latest.sequence.saturating_add(1),
        timestamp: chrono::Utc::now(),
        server: state.latest.server.clone(),
        state: next,
        reason,
    };
    // Publication and the existing compatibility recorder share the state lock,
    // so one generation cannot report Ready after a newer invalidation event.
    record_binding_health(
        &state.latest.server,
        state.latest.binding_id,
        next == McpServerState::Ready,
    );
    let _ = state.sender.send(normalized(&state.latest));
}

fn normalized(lifecycle: &McpServerLifecycle) -> NormalizedEvent {
    NormalizedEvent::McpServerStateChanged {
        run_id: None,
        lifecycle: lifecycle.clone(),
    }
}

/// Exact-binding subscription admitted by the cache using host owner identity.
/// There is no global cross-owner event receiver. Observation grants no tools.
pub struct McpLifecycleSubscription {
    initial: Option<NormalizedEvent>,
    receiver: broadcast::Receiver<NormalizedEvent>,
    lifecycle: Weak<RwLock<LifecycleState>>,
}

impl std::fmt::Debug for McpLifecycleSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpLifecycleSubscription")
            .finish_non_exhaustive()
    }
}

impl McpLifecycleSubscription {
    pub(crate) fn pending_count(&self) -> usize {
        self.receiver.len().min(LIFECYCLE_EVENT_CAPACITY) + usize::from(self.initial.is_some())
    }

    pub(crate) fn try_recv(&mut self) -> Result<NormalizedEvent, broadcast::error::TryRecvError> {
        if let Some(initial) = self.initial.take() {
            return Ok(initial);
        }
        self.receiver.try_recv()
    }

    /// Read the admission snapshot, then each subsequent ordered transition.
    /// Event IDs remain unchanged across delivery and resynchronization.
    ///
    /// # Errors
    /// Lagged reports skipped records explicitly; use resynchronize before
    /// treating the next observation as a complete current-state projection.
    pub async fn recv(&mut self) -> Result<NormalizedEvent, broadcast::error::RecvError> {
        if let Some(initial) = self.initial.take() {
            return Ok(initial);
        }
        self.receiver.recv().await
    }

    /// Atomically replace the observation cursor and return its latest state.
    /// This is a snapshot, not a claim that missed transitions were replayed.
    /// Returns None after the binding's publisher has been dropped.
    pub fn resynchronize(&mut self) -> Option<NormalizedEvent> {
        let lifecycle = self.lifecycle.upgrade()?;
        let state = lifecycle
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.receiver = state.sender.subscribe();
        self.initial = None;
        Some(normalized(&state.latest))
    }
}
