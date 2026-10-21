//! One explicit recovery authority for a transport and its owned subscriptions.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::Mutex;

use crate::uar::realtime::lifecycle::BusLifecycle;

type ReconnectFuture = Pin<Box<dyn Future<Output = anyhow::Result<u64>> + Send>>;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum TransportAvailability {
    Available,
    Unavailable,
    Released,
    ObservationUnavailable,
}

#[derive(Clone, Serialize)]
pub(super) struct TransportSnapshot {
    pub generation: Option<u64>,
    pub state: TransportAvailability,
}

/// Callbacks retain the same authenticated client used by the subscription bus.
/// Reading state is observational; only the explicit handler replaces transport.
pub(super) struct ReconnectTransport {
    snapshot: Box<dyn Fn() -> TransportSnapshot + Send + Sync>,
    reconnect: Box<dyn Fn(u64) -> ReconnectFuture + Send + Sync>,
}

impl ReconnectTransport {
    pub fn new<S, R, F>(snapshot: S, reconnect: R) -> Arc<Self>
    where
        S: Fn() -> TransportSnapshot + Send + Sync + 'static,
        R: Fn(u64) -> F + Send + Sync + 'static,
        F: Future<Output = anyhow::Result<u64>> + Send + 'static,
    {
        Arc::new(Self { snapshot: Box::new(snapshot), reconnect: Box::new(move |generation| Box::pin(reconnect(generation))) })
    }

    pub fn snapshot(&self) -> TransportSnapshot { (self.snapshot)() }
}

pub(super) struct ReconnectPair {
    gate: Mutex<()>,
    transport: Option<Arc<ReconnectTransport>>,
    subscriptions: Arc<BusLifecycle>,
}

impl ReconnectPair {
    pub fn new(transport: Option<Arc<ReconnectTransport>>, subscriptions: Arc<BusLifecycle>) -> Arc<Self> {
        Arc::new(Self { gate: Mutex::new(()), transport, subscriptions })
    }

    /// Both resource aliases enter this same gate. RuntimeHandle owns this
    /// operation independently of the control response waiter.
    pub async fn reconnect(&self, transport_command: bool, expected_generation: u64) -> anyhow::Result<u64> {
        let _gate = self.gate.lock().await;
        let subscription_generation = self.subscriptions.generation();
        if !transport_command {
            anyhow::ensure!(expected_generation == subscription_generation, "stale subscription generation");
        }
        let transport_generation = if let Some(transport) = &self.transport {
            let snapshot = transport.snapshot();
            let generation = snapshot.generation.ok_or_else(|| anyhow::anyhow!("transport generation is unavailable"))?;
            if transport_command {
                anyhow::ensure!(expected_generation == generation, "stale transport generation");
            }
            let generation = match snapshot.state {
                TransportAvailability::Available => generation,
                TransportAvailability::Unavailable => (transport.reconnect)(generation).await?,
                TransportAvailability::Released => anyhow::bail!("transport is released"),
                TransportAvailability::ObservationUnavailable => anyhow::bail!("transport state is unavailable"),
            };
            let acknowledged = transport.snapshot();
            anyhow::ensure!(acknowledged.generation == Some(generation)
                && matches!(acknowledged.state, TransportAvailability::Available),
                "transport reconnect did not retain the acknowledged available generation");
            Some(generation)
        } else {
            anyhow::ensure!(!transport_command, "resource has no independently controlled transport");
            None
        };

        // Transport availability alone never acknowledges the pair. Every
        // owned topic must register successfully for this exact new generation.
        let next = self.subscriptions.request_reconnect(subscription_generation)?;
        self.subscriptions.ready_generation(next).await?;
        if let (Some(transport), Some(generation)) = (&self.transport, transport_generation) {
            let observed = transport.snapshot();
            anyhow::ensure!(observed.generation == Some(generation)
                && matches!(observed.state, TransportAvailability::Available),
                "transport changed before subscription recovery acknowledgement");
        }
        if transport_command {
            transport_generation.ok_or_else(|| anyhow::anyhow!("transport generation is unavailable"))
        } else { Ok(next) }
    }
}

#[cfg(feature = "surreal-backend")]
pub(super) fn surreal_snapshot(
    generation: anyhow::Result<u64>,
    state: anyhow::Result<tokio::sync::watch::Receiver<surrealdb::TransportState>>,
) -> TransportSnapshot {
    let generation = generation.ok();
    let state = match state {
        Ok(state) => match *state.borrow() {
            surrealdb::TransportState::Available => TransportAvailability::Available,
            surrealdb::TransportState::Unavailable => TransportAvailability::Unavailable,
            surrealdb::TransportState::Released => TransportAvailability::Released,
        },
        Err(_) => TransportAvailability::ObservationUnavailable,
    };
    TransportSnapshot { generation, state }
}
