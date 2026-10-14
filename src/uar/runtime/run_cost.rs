//! Cost reporting for a finished run; provider calls settle in the host driver.
//!
//! Durable settlement belongs to individual calls. This terminal helper awaits
//! their accounting state and reports the live root/team total without a second
//! append-only charge. A pending liability is independent of execution success.

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
    /// `run_llm_config.cost_tracking`; controls terminal cost display, not durable admission.
    pub cost_tracking_enabled: bool,
}

/// Report per-call cost and emit a budget alert plus unresolved-accounting diagnostic.
///
/// Returns the estimated cost, or `None` when cost tracking is disabled or the
/// model is unpriced. This does not charge again: the host driver owns stable
/// per-call settlements. Execution completion and accounting completion are separate.
pub async fn settle_run_cost(
    inputs: RunCostInputs<'_>,
    cost_budget: &CostBudgetTracker,
    persistence: Option<&Arc<dyn PersistenceLayer>>,
    sink: &dyn RuntimeEventSink,
) -> Option<f64> {
    // Await inspection before terminal publication. A failed read is unresolved,
    // never evidence that the pending pre-dispatch liability disappeared.
    let owner = cost_budget.accounting_owner(inputs.run_id);
    // Explicit untracked/unlimited development mode has no accounting claim.
    if owner.is_none() && !inputs.cost_tracking_enabled { return None; }
    let accounting = match (persistence, owner) {
        (Some(db), Some(owner)) => db.list_accounting_calls(&owner, inputs.run_id).await,
        _ => Err(crate::uar::persistence::accounting::AccountingError::Unsupported.into()),
    };
    let settled = match &accounting {
        Ok(calls) => calls.iter().all(|call| call.state == crate::uar::persistence::accounting::AccountingState::Settled),
        Err(_) => false,
    };
    if !settled {
        sink.emit(NormalizedEvent::PresentationDiagnostic {
            run_id: inputs.run_id.to_owned(), code: "accounting_unresolved".into(),
            message: "Execution outcome is independent of accounting; durable usage remains pending reconciliation".into(),
        }).await;
    }
    if !inputs.cost_tracking_enabled { return None; }
    let usage = cost_budget.run_usage(inputs.run_id);
    let cost = if usage.usage_reports > 0 && !usage.unpriced_usage { Some(usage.cost_usd) } else { None }?;
    if let Some((provider,model_id)) = inputs.model.split_once('/') {
        crate::uar::telemetry::metrics::record_llm_cost(provider,model_id,cost);
    }

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
