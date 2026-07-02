//! Per-scope cost budgets + spend aggregation (CH-06, fable §R11 / §7).
//!
//! Builds on the existing opt-in cost estimation (`LlmConfig.cost_tracking`,
//! `estimate_cost`, the `uar_llm_cost_usd` metric). This module adds the
//! *accounting* layer the runtime was missing: aggregate spend per run / task /
//! session / agent, declare budget envelopes, and emit threshold-crossing
//! events so the UI and operators can react before a runaway burns the budget.
//!
//! In-memory and lock-light — a single-process aggregator suitable for the
//! runtime hot path. Durable roll-ups (SurrealDB/Postgres) can layer on top by
//! subscribing to the emitted events; that persistence is intentionally out of
//! scope here.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

/// The dimension a budget / spend record is keyed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BudgetScope {
    Run,
    Task,
    Session,
    Agent,
    /// Whole-deployment ("house") spend.
    Global,
}

impl BudgetScope {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Task => "task",
            Self::Session => "session",
            Self::Agent => "agent",
            Self::Global => "global",
        }
    }
}

/// A configured budget limit for one `(scope, scope_id)`.
#[derive(Debug, Clone, Copy)]
pub struct BudgetLimit {
    /// Hard ceiling in USD. Spend beyond this reports `Exceeded`.
    pub limit_usd: f64,
    /// Fraction (0.0–1.0) at which a `Warning` fires (e.g. 0.8 = 80%).
    pub warn_at: f64,
}

impl Default for BudgetLimit {
    fn default() -> Self {
        Self {
            limit_usd: f64::INFINITY,
            warn_at: 0.8,
        }
    }
}

/// Result of recording spend against a budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetStatus {
    /// Under the warning threshold.
    Ok,
    /// At/over `warn_at * limit` but under `limit`.
    Warning { spent_usd: f64, limit_usd: f64 },
    /// At/over the hard limit.
    Exceeded { spent_usd: f64, limit_usd: f64 },
}

impl BudgetStatus {
    #[must_use]
    pub fn is_exceeded(self) -> bool {
        matches!(self, Self::Exceeded { .. })
    }
}

#[derive(Debug, Default)]
struct Inner {
    /// (scope, scope_id) -> accumulated USD spend.
    spend: HashMap<(BudgetScope, String), f64>,
    /// (scope, scope_id) -> configured limit.
    limits: HashMap<(BudgetScope, String), BudgetLimit>,
}

/// Thread-safe cost-budget tracker. Cloneable handle over shared state.
#[derive(Debug, Clone, Default)]
pub struct CostBudgetTracker {
    inner: Arc<RwLock<Inner>>,
}

impl CostBudgetTracker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set (or replace) the budget limit for a `(scope, scope_id)`.
    pub async fn set_limit(&self, scope: BudgetScope, scope_id: &str, limit: BudgetLimit) {
        self.inner
            .write()
            .await
            .limits
            .insert((scope, scope_id.to_string()), limit);
    }

    /// Record `cost_usd` of spend against a `(scope, scope_id)` and return the
    /// resulting status. Emits a `cost.budget.*` tracing event on threshold
    /// crossings so downstream (UI/alerts) can react; also updates the
    /// `uar_cost_budget_spent_usd` gauge.
    pub async fn record(&self, scope: BudgetScope, scope_id: &str, cost_usd: f64) -> BudgetStatus {
        let key = (scope, scope_id.to_string());
        let mut guard = self.inner.write().await;
        let spent = {
            let e = guard.spend.entry(key.clone()).or_insert(0.0);
            *e += cost_usd.max(0.0);
            *e
        };
        let limit = guard.limits.get(&key).copied().unwrap_or_default();
        drop(guard);

        metrics::gauge!(
            "uar_cost_budget_spent_usd",
            "scope" => scope.as_str(),
        )
        .set(spent);

        let status = if spent >= limit.limit_usd {
            tracing::warn!(
                name: "cost.budget.exceeded",
                scope = scope.as_str(), scope_id, spent_usd = spent, limit_usd = limit.limit_usd,
                "cost budget exceeded"
            );
            BudgetStatus::Exceeded {
                spent_usd: spent,
                limit_usd: limit.limit_usd,
            }
        } else if limit.limit_usd.is_finite() && spent >= limit.warn_at * limit.limit_usd {
            tracing::warn!(
                name: "cost.budget.warning",
                scope = scope.as_str(), scope_id, spent_usd = spent, limit_usd = limit.limit_usd,
                "cost budget warning threshold crossed"
            );
            BudgetStatus::Warning {
                spent_usd: spent,
                limit_usd: limit.limit_usd,
            }
        } else {
            BudgetStatus::Ok
        };
        status
    }

    /// Current accumulated spend for a `(scope, scope_id)`.
    pub async fn spent(&self, scope: BudgetScope, scope_id: &str) -> f64 {
        self.inner
            .read()
            .await
            .spend
            .get(&(scope, scope_id.to_string()))
            .copied()
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn accumulates_and_crosses_thresholds() {
        let t = CostBudgetTracker::new();
        t.set_limit(
            BudgetScope::Task,
            "t1",
            BudgetLimit {
                limit_usd: 1.0,
                warn_at: 0.8,
            },
        )
        .await;

        assert_eq!(
            t.record(BudgetScope::Task, "t1", 0.5).await,
            BudgetStatus::Ok
        );
        assert!(matches!(
            t.record(BudgetScope::Task, "t1", 0.35).await,
            BudgetStatus::Warning { .. }
        ));
        assert!(t.record(BudgetScope::Task, "t1", 0.2).await.is_exceeded());
        assert!((t.spent(BudgetScope::Task, "t1").await - 1.05).abs() < 1e-9);
    }

    #[tokio::test]
    async fn no_limit_never_warns() {
        let t = CostBudgetTracker::new();
        assert_eq!(
            t.record(BudgetScope::Global, "g", 1000.0).await,
            BudgetStatus::Ok
        );
    }
}
