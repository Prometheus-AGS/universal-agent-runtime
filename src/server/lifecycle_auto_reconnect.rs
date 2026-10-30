//! Standalone-server recovery for remote transports.
//!
//! The patched SurrealDB SDK never reconnects by itself: after a dropped socket
//! it rejects every operation until the host calls `reconnect` (#441). A sidecar
//! host owns that decision through its control pipe. A standalone server has no
//! such host, so this driver issues the same explicit, generation-checked
//! reconnect through `RuntimeHandle::reconnect` until the resource recovers.

use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use super::lifecycle_reconnect::{ReconnectTransport, TransportAvailability};
use crate::uar::runtime::lifecycle::{RuntimeHandle, RuntimeSupervisor};

/// How often the driver reads the retained transport state.
const TRANSPORT_POLL_INTERVAL: Duration = Duration::from_secs(1);
/// First delay after a failed reconnect; a restarting server may not listen yet.
const INITIAL_RETRY_DELAY: Duration = Duration::from_millis(250);
/// Upper bound for the doubling delay while the server stays unreachable.
const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);

/// Register a supervised driver for `resource_id`. Its scope depends on
/// `dependencies`, so shutdown stops the driver before the resources it
/// recovers.
///
/// # Errors
///
/// Returns an error when the supervisor is closing or rejects the scope.
pub(super) async fn start(
    supervisor: &RuntimeSupervisor,
    resource_id: &'static str,
    dependencies: [&'static str; 2],
    transport: Arc<ReconnectTransport>,
) -> anyhow::Result<()> {
    let scope = supervisor
        .register(format!("{resource_id}-auto-reconnect"), "uar-host")
        .await?;
    scope.declare_dependencies(dependencies)?;
    let stop = scope.cancellation_token();
    scope.spawn("auto-reconnect", drive(supervisor.handle(), resource_id, transport, stop))?;
    scope.mark_ready()?;
    Ok(())
}

/// Watch `transport` and reconnect `resource_id` whenever it becomes
/// unavailable, until the transport is released or `stop` is cancelled.
///
/// Recovery goes through the registered reconnect handler, so it shares the
/// sidecar's gate, generation checks and subscription acknowledgement.
///
/// # Errors
///
/// Never returns an error; reconnect failures are logged and retried.
async fn drive(
    runtime: RuntimeHandle,
    resource_id: &'static str,
    transport: Arc<ReconnectTransport>,
    stop: CancellationToken,
) -> anyhow::Result<()> {
    loop {
        match transport.snapshot().state {
            TransportAvailability::Unavailable => {
                if !recover(&runtime, resource_id, &transport, &stop).await {
                    return Ok(());
                }
            }
            TransportAvailability::Released => return Ok(()),
            TransportAvailability::Available | TransportAvailability::ObservationUnavailable => {}
        }
        tokio::select! {
            () = stop.cancelled() => return Ok(()),
            () = tokio::time::sleep(TRANSPORT_POLL_INTERVAL) => {}
        }
    }
}

/// Retry with a capped doubling delay. Returns `false` when the driver should
/// stop because the transport was released or shutdown began.
async fn recover(
    runtime: &RuntimeHandle,
    resource_id: &'static str,
    transport: &ReconnectTransport,
    stop: &CancellationToken,
) -> bool {
    tracing::warn!(
        name: "persistence.transport.lost",
        resource = resource_id,
        "Remote transport lost; reconnecting"
    );
    let mut delay = INITIAL_RETRY_DELAY;
    let mut attempt: u64 = 0;
    loop {
        attempt = attempt.saturating_add(1);
        let snapshot = transport.snapshot();
        if matches!(snapshot.state, TransportAvailability::Released) {
            return false;
        }
        // The handler must succeed even when the transport is already back:
        // subscription recovery for the new generation can still fail.
        let result = match snapshot.generation {
            Some(generation) => runtime.reconnect(resource_id, generation).await,
            None => Err(anyhow::anyhow!("transport generation is unavailable")),
        };
        match result {
            Ok(generation) => {
                tracing::info!(
                    name: "persistence.transport.recovered",
                    resource = resource_id,
                    generation,
                    attempt,
                    "Remote transport recovered"
                );
                return true;
            }
            Err(error) => tracing::warn!(
                name: "persistence.transport.reconnect_failed",
                resource = resource_id,
                attempt,
                retry_in_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                error = %error,
                "Remote transport reconnect failed"
            ),
        }
        tokio::select! {
            () = stop.cancelled() => return false,
            () = tokio::time::sleep(delay) => {}
        }
        delay = delay.saturating_mul(2).min(MAX_RETRY_DELAY);
    }
}
