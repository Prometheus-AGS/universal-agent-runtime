//! Background workers for the memory system.
//!
//! - **TTL worker**: periodically expires stale memories (those past `valid_until`).
//! - **Compression worker**: periodically compresses old memories into summaries.

use std::sync::Arc;
use tokio::time::{Duration, interval};

use super::service::MemoryService;

/// Run the TTL expiry worker. Checks for expired memories every `interval_secs` seconds.
///
/// Intended to be spawned as a background task:
/// ```rust,ignore
/// tokio::spawn(background::run_ttl_worker(memory_svc, shutdown_rx));
/// ```
pub async fn run_ttl_worker(
    service: Arc<MemoryService>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
    interval_secs: u64,
) {
    let mut ticker = interval(Duration::from_secs(interval_secs));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                match service.expire_stale().await {
                    Ok(count) if count > 0 => {
                        tracing::info!(expired = count, "Memory TTL worker: expired stale memories");
                    }
                    Ok(_) => {} // no-op
                    Err(e) => {
                        tracing::warn!(error = %e, "Memory TTL worker error (continuing)");
                    }
                }
            }
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    tracing::info!("Memory TTL worker shutting down");
                    break;
                }
            }
        }
    }
}
