//! Retained subscription state and acknowledged ownership of bus workers.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use futures::FutureExt as _;
use serde::Serialize;
use tokio::sync::{broadcast, watch};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use super::{EntityTopic, LiveAction, LiveEvent};

/// Current subscription generation. Late observers receive the current state.
#[derive(Clone, Debug, Serialize)]
pub struct TopicState {
    /// Increases only when an explicit reconnect is requested.
    pub generation: u64,
    /// True only after the database acknowledged subscription registration.
    pub ready: bool,
    /// Unavailability or finalization reason, without connection credentials.
    pub reason: Option<String>,
}

/// Owns bus workers and one retained, cancellation-safe shutdown result.
pub struct BusLifecycle {
    admission: Mutex<bool>,
    tasks: TaskTracker,
    stop: CancellationToken,
    reconnect: watch::Sender<u64>,
    states: watch::Sender<HashMap<EntityTopic, TopicState>>,
    senders: Arc<HashMap<EntityTopic, broadcast::Sender<LiveEvent>>>,
    closing: AtomicBool,
    worker_failed: AtomicBool,
    closed: watch::Sender<Option<Result<(), String>>>,
    failures: Mutex<Vec<String>>,
}

impl std::fmt::Debug for BusLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BusLifecycle")
            .field("states", &*self.states.borrow())
            .field("closing", &self.closing.load(Ordering::Acquire))
            .finish()
    }
}

impl BusLifecycle {
    pub(crate) fn new(senders: Arc<HashMap<EntityTopic, broadcast::Sender<LiveEvent>>>) -> Arc<Self> {
        let initial = senders.keys().map(|topic| (*topic, TopicState {
            generation: 0, ready: false, reason: None,
        })).collect();
        Arc::new(Self {
            admission: Mutex::new(true), tasks: TaskTracker::new(),
            stop: CancellationToken::new(), reconnect: watch::channel(0).0,
            states: watch::channel(initial).0, senders,
            closing: AtomicBool::new(false), worker_failed: AtomicBool::new(false), closed: watch::channel(None).0,
            failures: Mutex::new(Vec::new()),
        })
    }

    pub(crate) fn spawn(self: &Arc<Self>, work: impl Future<Output = anyhow::Result<()>> + Send + 'static) {
        let gate = self.admission.lock().unwrap_or_else(|error| error.into_inner());
        if !*gate { return; }
        let owner = Arc::clone(self);
        self.tasks.spawn(async move {
            let failure = match std::panic::AssertUnwindSafe(work).catch_unwind().await {
                Ok(Ok(())) => None,
                Ok(Err(error)) => Some(error.to_string()),
                Err(_) => Some("subscription worker panicked".to_owned()),
            };
            if let Some(failure) = failure {
                owner.worker_failed.store(true, Ordering::Release);
                owner.failures.lock().unwrap_or_else(|error| error.into_inner()).push(failure);
                let states = owner.states.borrow().clone();
                for (topic, state) in states {
                    owner.transition(topic, state.generation, false, Some("subscription worker failed".to_owned()));
                }
            }
        });
    }

    pub(crate) fn stop_token(&self) -> CancellationToken { self.stop.clone() }
    pub(crate) fn reconnect_events(&self) -> watch::Receiver<u64> { self.reconnect.subscribe() }
    /// Read the retained reconnect generation for host-side CAS control.
    pub(crate) fn generation(&self) -> u64 { *self.reconnect.borrow() }

    /// Current retained state, including unavailability before subscription.
    pub fn states(&self) -> watch::Receiver<HashMap<EntityTopic, TopicState>> { self.states.subscribe() }

    pub(crate) fn record_failure(&self, failure: impl Into<String>) {
        self.failures.lock().unwrap_or_else(|error| error.into_inner()).push(failure.into());
    }

    pub(crate) fn transition(&self, topic: EntityTopic, generation: u64, ready: bool, reason: Option<String>) {
        self.states.send_modify(|states| {
            states.insert(topic, TopicState { generation, ready, reason: reason.clone() });
        });
        if let Some(sender) = self.senders.get(&topic) {
            let _ = sender.send(LiveEvent {
                action: LiveAction::Reset, topic, id: String::new(),
                data: serde_json::json!({"version": 1, "generation": generation, "ready": ready, "reason": reason}),
            });
        }
    }

    /// Wait for acknowledged subscriptions; a failed attempt returns immediately.
    pub async fn ready(&self) -> anyhow::Result<()> {
        let mut states = self.states.subscribe();
        loop {
            {
                let current = states.borrow_and_update();
                if current.values().all(|state| state.ready) { return Ok(()); }
                if let Some(reason) = current.values().find_map(|state| state.reason.as_ref()) {
                    anyhow::bail!("subscription unavailable: {reason}");
                }
            }
            states.changed().await.map_err(|_| anyhow::anyhow!("subscription state owner ended"))?;
        }
    }

    /// A reconnect acknowledgement belongs to the requested generation, never
    /// to a retained Ready value left by an earlier connection.
    pub async fn ready_generation(&self, generation: u64) -> anyhow::Result<()> {
        let mut states = self.states.subscribe();
        loop {
            {
                let current = states.borrow_and_update();
                anyhow::ensure!(!self.worker_failed.load(Ordering::Acquire),
                    "subscription worker failed; reconnect requires a new owned worker scope");
                if current.values().all(|state| state.generation == generation && state.ready) { return Ok(()); }
                if let Some(reason) = current.values().filter(|state| state.generation == generation)
                    .find_map(|state| state.reason.as_ref()) {
                    anyhow::bail!("subscription generation {generation} unavailable: {reason}");
                }
                anyhow::ensure!(!self.stop.is_cancelled(), "subscription scope is closed");
            }
            states.changed().await.map_err(|_| anyhow::anyhow!("subscription state owner ended"))?;
        }
    }

    /// Request one connection attempt for the observed generation.
    pub fn request_reconnect(&self, expected_generation: u64) -> anyhow::Result<u64> {
        let gate = self.admission.lock().unwrap_or_else(|error| error.into_inner());
        anyhow::ensure!(*gate, "subscription admission is closed");
        anyhow::ensure!(!self.worker_failed.load(Ordering::Acquire), "subscription worker ended; a new owned worker scope is required");
        let current = *self.reconnect.borrow();
        anyhow::ensure!(current == expected_generation, "stale subscription generation");
        let next = current.checked_add(1).ok_or_else(|| anyhow::anyhow!("subscription generation exhausted"))?;
        self.reconnect.send_replace(next);
        Ok(next)
    }

    /// Close admission, stop readers, and await all acknowledged cleanup.
    pub async fn shutdown(self: &Arc<Self>) -> anyhow::Result<()> {
        if !self.closing.swap(true, Ordering::AcqRel) {
            {
                let mut gate = self.admission.lock().unwrap_or_else(|error| error.into_inner());
                *gate = false;
                self.tasks.close();
                self.stop.cancel();
            }
            let owner = Arc::clone(self);
            tokio::spawn(async move {
                owner.tasks.wait().await;
                let failures = owner.failures.lock().unwrap_or_else(|error| error.into_inner());
                let result = if failures.is_empty() { Ok(()) } else { Err(failures.join("; ")) };
                owner.states.send_modify(|states| {
                    for state in states.values_mut() {
                        state.ready = false;
                        state.reason = Some("subscription closed".to_owned());
                    }
                });
                owner.closed.send_replace(Some(result));
            });
        }
        let mut result = self.closed.subscribe();
        loop {
            if let Some(result) = result.borrow_and_update().clone() {
                return result.map_err(anyhow::Error::msg);
            }
            result.changed().await.map_err(|_| anyhow::anyhow!("subscription finalizer ended without acknowledgement"))?;
        }
    }
}
