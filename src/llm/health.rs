//! Provider health tracking for failover-aware routing (CH-03).
//!
//! Tracks per-provider consecutive-failure counts and puts a provider into a
//! cooldown window once `error_threshold` consecutive failures are recorded,
//! per [`crate::config::FailoverConfig`]. [`ModelRouter::route`](super::router::ModelRouter::route)
//! and [`ProviderRegistry::resolve_to_llm_config`](super::registry::ProviderRegistry::resolve_to_llm_config)
//! both consult [`ProviderHealthMonitor::is_available`] to skip providers currently in cooldown.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, Default)]
struct ProviderHealthState {
    consecutive_errors: u32,
    cooldown_until: Option<Instant>,
}

impl ProviderHealthState {
    fn is_healthy(&self, now: Instant) -> bool {
        self.cooldown_until.is_none_or(|until| now >= until)
    }
}

/// A point-in-time view of one provider's health, for API/console surfacing.
#[derive(Debug, Clone, Copy)]
pub struct ProviderHealthSnapshot {
    pub healthy: bool,
    pub consecutive_errors: u32,
    pub cooldown_remaining_secs: Option<u64>,
}

/// Tracks provider health via consecutive-failure cooldowns.
///
/// Shared (via `Arc`) between [`ProviderRegistry`](super::registry::ProviderRegistry),
/// [`ModelRouter`](super::router::ModelRouter), and every [`Orchestrator`](super::orchestrator::Orchestrator)
/// instance so a failure observed on one run's driver call immediately affects
/// routing decisions for the next.
#[derive(Debug, Default)]
pub struct ProviderHealthMonitor {
    state: RwLock<HashMap<String, ProviderHealthState>>,
}

impl ProviderHealthMonitor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful call to `provider_id` — clears its error streak and cooldown.
    pub async fn record_success(&self, provider_id: &str) {
        let mut state = self.state.write().await;
        state
            .entry(provider_id.to_string())
            .or_default()
            .consecutive_errors = 0;
        if let Some(entry) = state.get_mut(provider_id) {
            entry.cooldown_until = None;
        }
        drop(state);
        crate::uar::telemetry::metrics::record_provider_health(provider_id, true);
    }

    /// Record a failed call to `provider_id`. Once `consecutive_errors` reaches
    /// `error_threshold`, the provider enters cooldown for `cooldown_secs`.
    pub async fn record_failure(
        &self,
        provider_id: &str,
        error_threshold: u32,
        cooldown_secs: u64,
    ) {
        let mut state = self.state.write().await;
        let entry = state.entry(provider_id.to_string()).or_default();
        entry.consecutive_errors += 1;
        let tripped = entry.consecutive_errors >= error_threshold.max(1);
        if tripped {
            entry.cooldown_until = Some(Instant::now() + Duration::from_secs(cooldown_secs));
        }
        let healthy = entry.is_healthy(Instant::now());
        drop(state);
        crate::uar::telemetry::metrics::record_provider_health(provider_id, healthy);
    }

    /// Whether `provider_id` is currently outside its cooldown window.
    /// Unknown providers (never recorded) are considered available.
    pub async fn is_available(&self, provider_id: &str) -> bool {
        let state = self.state.read().await;
        state
            .get(provider_id)
            .is_none_or(|entry| entry.is_healthy(Instant::now()))
    }

    /// Snapshot every tracked provider's health, for API/console surfacing.
    pub async fn snapshot(&self) -> HashMap<String, ProviderHealthSnapshot> {
        let state = self.state.read().await;
        let now = Instant::now();
        state
            .iter()
            .map(|(id, entry)| {
                let cooldown_remaining_secs = entry
                    .cooldown_until
                    .filter(|until| *until > now)
                    .map(|until| (until - now).as_secs());
                (
                    id.clone(),
                    ProviderHealthSnapshot {
                        healthy: entry.is_healthy(now),
                        consecutive_errors: entry.consecutive_errors,
                        cooldown_remaining_secs,
                    },
                )
            })
            .collect()
    }

    /// Spawn the periodic sweep loop: every `interval_secs`, refresh the health
    /// gauge for each tracked provider (expired cooldowns read as healthy again
    /// without waiting for the next call to that provider). Exits when `token`
    /// is cancelled.
    pub fn spawn_monitor_loop(
        self: Arc<Self>,
        interval_secs: u64,
        token: CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs.max(1)));
            loop {
                tokio::select! {
                    _ = ticker.tick() => self.sweep().await,
                    () = token.cancelled() => break,
                }
            }
        })
    }

    async fn sweep(&self) {
        let snapshot = self.snapshot().await;
        for (provider_id, status) in snapshot {
            crate::uar::telemetry::metrics::record_provider_health(&provider_id, status.healthy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unknown_provider_is_available() {
        let monitor = ProviderHealthMonitor::new();
        assert!(monitor.is_available("openai").await);
    }

    #[tokio::test]
    async fn trips_cooldown_after_threshold_failures() {
        let monitor = ProviderHealthMonitor::new();
        monitor.record_failure("openai", 3, 60).await;
        assert!(monitor.is_available("openai").await, "below threshold");
        monitor.record_failure("openai", 3, 60).await;
        monitor.record_failure("openai", 3, 60).await;
        assert!(!monitor.is_available("openai").await, "threshold reached");
    }

    #[tokio::test]
    async fn success_clears_cooldown() {
        let monitor = ProviderHealthMonitor::new();
        for _ in 0..3 {
            monitor.record_failure("openai", 3, 60).await;
        }
        assert!(!monitor.is_available("openai").await);
        monitor.record_success("openai").await;
        assert!(monitor.is_available("openai").await);
    }

    #[tokio::test]
    async fn snapshot_reports_cooldown_remaining() {
        let monitor = ProviderHealthMonitor::new();
        for _ in 0..2 {
            monitor.record_failure("openai", 2, 60).await;
        }
        let snap = monitor.snapshot().await;
        let status = snap.get("openai").expect("tracked");
        assert!(!status.healthy);
        assert_eq!(status.consecutive_errors, 2);
        assert!(status.cooldown_remaining_secs.unwrap_or(0) > 0);
    }
}
