//! Cost settlement writes the durable ledger and raises budget alerts.
//!
//! Before this was a free function it lived inside the completed-run branch of
//! the run manager, so a cancelled run's spend never reached the ledger.

#![cfg(feature = "in-memory-backend")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use universal_agent_runtime::llm::catalog::estimate_cost;
use universal_agent_runtime::uar::domain::events::{NormalizedEvent, RuntimeEventSink};
use universal_agent_runtime::uar::persistence::PersistenceLayer;
use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
use universal_agent_runtime::uar::runtime::cost_budget::{
    BudgetLimit, BudgetScope, CostBudgetTracker,
};
use universal_agent_runtime::uar::runtime::run_cost::{RunCostInputs, settle_run_cost};

const MODEL: &str = "openai/gpt-4.1-mini";

#[derive(Default)]
struct CollectingSink(Mutex<Vec<NormalizedEvent>>);

#[async_trait]
impl RuntimeEventSink for CollectingSink {
    async fn emit(&self, event: NormalizedEvent) {
        self.0.lock().unwrap().push(event);
    }
}

fn inputs(model: &str, tracking: bool) -> RunCostInputs<'_> {
    RunCostInputs {
        run_id: "run-1",
        session_id: "session-1",
        agent_id: "agent-1",
        model,
        input_tokens: 1_000,
        output_tokens: 500,
        cache_read_tokens: 0,
        cost_tracking_enabled: tracking,
    }
}

/// The ledger write is spawned, so poll briefly for it.
async fn rows(db: &InMemoryProvider, scope: &str, id: &str, want: usize) -> Vec<f64> {
    for _ in 0..100 {
        let found = db.list_cost_history(scope, id).await.unwrap();
        if found.len() >= want {
            return found.iter().map(|e| e.cost_usd).collect();
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Vec::new()
}

#[tokio::test]
async fn a_run_with_usage_writes_one_ledger_row_per_scope() {
    let expected = estimate_cost(MODEL, 1_000, 500, 0).expect("model is priced in the catalog");
    let db = Arc::new(InMemoryProvider::new());
    let persistence: Arc<dyn PersistenceLayer> = db.clone();
    let sink = CollectingSink::default();

    let cost = settle_run_cost(
        inputs(MODEL, true),
        &CostBudgetTracker::new(),
        Some(&persistence),
        &sink,
    )
    .await;

    assert_eq!(cost, Some(expected));
    for (scope, id) in [
        ("run", "run-1"),
        ("session", "session-1"),
        ("agent", "agent-1"),
        ("global", "global"),
    ] {
        assert_eq!(rows(&db, scope, id, 1).await, vec![expected], "{scope}");
    }
    assert!(sink.0.lock().unwrap().is_empty(), "no budget, no alert");
}

#[tokio::test]
async fn cost_tracking_off_records_nothing() {
    let db = Arc::new(InMemoryProvider::new());
    let persistence: Arc<dyn PersistenceLayer> = db.clone();

    let cost = settle_run_cost(
        inputs(MODEL, false),
        &CostBudgetTracker::new(),
        Some(&persistence),
        &CollectingSink::default(),
    )
    .await;

    assert_eq!(cost, None);
    assert!(
        db.list_cost_history("run", "run-1")
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn an_unpriced_model_records_nothing() {
    let db = Arc::new(InMemoryProvider::new());
    let persistence: Arc<dyn PersistenceLayer> = db.clone();

    let cost = settle_run_cost(
        inputs("nobody/not-a-model", true),
        &CostBudgetTracker::new(),
        Some(&persistence),
        &CollectingSink::default(),
    )
    .await;

    assert_eq!(cost, None);
    assert!(
        db.list_cost_history("run", "run-1")
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn an_exceeded_budget_raises_exactly_one_alert() {
    let budget = CostBudgetTracker::new();
    budget
        .set_limit(
            BudgetScope::Run,
            "run-1",
            BudgetLimit {
                limit_usd: 0.000_001,
                warn_at: 0.8,
            },
        )
        .await;
    // The driver wrapper charges each call as it happens; settlement only reads.
    budget.record(BudgetScope::Run, "run-1", 0.001).await;
    let sink = CollectingSink::default();

    settle_run_cost(inputs(MODEL, true), &budget, None, &sink).await;

    let events = sink.0.lock().unwrap();
    assert_eq!(events.len(), 1);
    match &events[0] {
        NormalizedEvent::BudgetAlert {
            run_id,
            scope,
            scope_id,
            exceeded,
            ..
        } => {
            assert_eq!(run_id, "run-1");
            assert_eq!(scope, "run");
            assert_eq!(scope_id, "run-1");
            assert!(*exceeded);
        }
        other => panic!("expected BudgetAlert, got {other:?}"),
    }
}

#[tokio::test]
async fn settling_without_persistence_still_returns_the_cost() {
    let cost = settle_run_cost(
        inputs(MODEL, true),
        &CostBudgetTracker::new(),
        None,
        &CollectingSink::default(),
    )
    .await;
    assert!(cost.is_some());
}
