//! Cost settlement for a finished run.
//!
//! Both terminal paths of a run (completed and cancelled) settle the cost of
//! the model calls that reported usage: estimate the USD cost from the pricing
//! catalog, record the metric, write the durable cost-ledger rows and surface a
//! `BudgetAlert` for the first scope that crossed its threshold.

use std::sync::Arc;

use crate::uar::domain::events::{NormalizedEvent, RuntimeEventSink};
use crate::uar::persistence::PersistenceLayer;
use crate::uar::runtime::cost_budget::{BudgetScope, BudgetStatus, CostBudgetTracker};

/// Everything `settle_run_cost` needs about one finished run.
#[derive(Debug, Clone, Copy)]
pub struct RunCostInputs<'a> {
    pub run_id: &'a str,
    pub session_id: &'a str,
    pub agent_id: &'a str,
    /// `provider/model` as resolved for the run.
    pub model: &'a str,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    /// `run_llm_config.cost_tracking`; when false nothing is estimated or recorded.
    pub cost_tracking_enabled: bool,
}

/// Estimate the run's cost, write the durable ledger rows and emit a
/// `BudgetAlert` when a scope crossed its threshold.
///
/// Returns the estimated cost, or `None` when cost tracking is disabled or the
/// model is unpriced. Ledger writes are fire-and-forget so the hot path never
/// blocks on a database write. The caller emits the terminal event afterwards,
/// so a `BudgetAlert` always precedes it.
pub async fn settle_run_cost(
    inputs: RunCostInputs<'_>,
    cost_budget: &CostBudgetTracker,
    persistence: Option<&Arc<dyn PersistenceLayer>>,
    sink: &dyn RuntimeEventSink,
) -> Option<f64> {
    let cost = if inputs.cost_tracking_enabled {
        crate::llm::catalog::estimate_cost(
            inputs.model,
            u64::from(inputs.input_tokens),
            u64::from(inputs.output_tokens),
            u64::from(inputs.cache_read_tokens),
        )
    } else {
        None
    }?;
    let (provider, model_id) = inputs.model.split_once('/')?;
    crate::uar::telemetry::metrics::record_llm_cost(provider, model_id, cost);

    // Driver wrappers already charged every model call. Surface a `BudgetAlert`
    // for the first scope (in priority order) that crosses its configured
    // threshold. Unconfigured scopes have an unlimited `BudgetLimit::default()`,
    // so reading the status does not charge the final request again.
    // `BudgetScope::Task` is intentionally omitted: this runtime has no task
    // entity distinct from a run.
    let scopes: [(BudgetScope, &str); 4] = [
        (BudgetScope::Run, inputs.run_id),
        (BudgetScope::Session, inputs.session_id),
        (BudgetScope::Agent, inputs.agent_id),
        (BudgetScope::Global, "global"),
    ];
    let mut alert: Option<(BudgetScope, String, f64, f64, bool)> = None;
    for (scope, scope_id) in scopes {
        let status = cost_budget.status(scope, scope_id).await;
        if let Some(db) = persistence.cloned() {
            let scope_str = scope.as_str().to_string();
            let scope_id_owned = scope_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = db
                    .record_cost_entry(&scope_str, &scope_id_owned, cost)
                    .await
                {
                    tracing::warn!(error = %e, scope = %scope_str, "Failed to persist cost ledger entry");
                }
            });
        }
        if alert.is_none()
            && let BudgetStatus::Warning {
                spent_usd,
                limit_usd,
            }
            | BudgetStatus::Exceeded {
                spent_usd,
                limit_usd,
            } = status
        {
            alert = Some((
                scope,
                scope_id.to_string(),
                spent_usd,
                limit_usd,
                status.is_exceeded(),
            ));
        }
    }
    if let Some((scope, scope_id, spent_usd, limit_usd, exceeded)) = alert {
        sink.emit(NormalizedEvent::BudgetAlert {
            run_id: inputs.run_id.to_string(),
            scope: scope.as_str().to_string(),
            scope_id,
            spent_usd,
            limit_usd,
            exceeded,
        })
        .await;
    }
    Some(cost)
}
