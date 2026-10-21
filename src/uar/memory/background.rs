//! Event-driven background work for the memory system.
//!
//! The TTL worker wakes only for a committed-memory change, its next durable
//! business expiry, or shutdown. It never uses a periodic probe to decide that
//! expiry or cleanup completed.

use std::sync::Arc;
use std::time::Duration;

use super::service::MemoryService;

/// Run the durable TTL worker.
///
/// `legacy_interval_secs` remains for source compatibility with callers from
/// before lifecycle scheduling. It is intentionally ignored: a configured
/// duration cannot establish correctness across execution environments.
pub async fn run_ttl_worker(
    service: Arc<MemoryService>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
    _legacy_interval_secs: u64,
) {
    let mut changes = service.subscribe_committed_memory_changes();

    loop {
        if *shutdown.borrow_and_update() {
            return;
        }
        let due = match service.next_expiry_due_at().await {
            Ok(due) => due,
            Err(error) => {
                tracing::warn!(error = %error, "Memory TTL due-time read failed; awaiting an authoritative change or shutdown");
                tokio::select! {
                    changed = changes.changed() => {
                        if changed.is_err() { return; }
                    }
                    changed = shutdown.changed() => {
                        if changed.is_err() || *shutdown.borrow() { return; }
                    }
                }
                continue;
            }
        };

        let Some(due) = due else {
            tokio::select! {
                changed = changes.changed() => {
                    if changed.is_err() { return; }
                }
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() { return; }
                }
            }
            continue;
        };

        let remaining = (due - chrono::Utc::now()).to_std().unwrap_or(Duration::ZERO);
        tokio::select! {
            () = tokio::time::sleep(remaining) => {
                // Awaiting this operation is the acknowledgement that all
                // accepted expiry cleanup settled before the next schedule read.
                match service.expire_stale().await {
                    Ok(count) if count > 0 => tracing::info!(expired = count, "Memory TTL expiry settled"),
                    Ok(_) => {
                        // The durable row can remain past due when another
                        // worker already settled it. Do not reread it in a
                        // zero-duration loop; wait for a committed change or
                        // shutdown before calculating a new deadline.
                        if await_change_or_shutdown(&mut changes, &mut shutdown).await { return; }
                    }
                    Err(error) => {
                        tracing::warn!(error = %error, "Memory TTL expiry operation failed; awaiting an authoritative change or shutdown");
                        if await_change_or_shutdown(&mut changes, &mut shutdown).await { return; }
                    }
                }
            }
            changed = changes.changed() => {
                if changed.is_err() { return; }
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { return; }
            }
        }
    }
}

/// Returns true when this worker must terminate. A failed expiry operation is
/// not retried from elapsed time; a committed change or explicit shutdown is
/// the only event that permits another durable due-time read.
async fn await_change_or_shutdown(
    changes: &mut tokio::sync::watch::Receiver<u64>,
    shutdown: &mut tokio::sync::watch::Receiver<bool>,
) -> bool {
    tokio::select! {
        changed = changes.changed() => changed.is_err(),
        changed = shutdown.changed() => changed.is_err() || *shutdown.borrow(),
    }
}
