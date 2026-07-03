## 1. Tracker (shipped prior turn)

- [x] 1.1 `CostBudgetTracker`/`BudgetScope`/`BudgetLimit`/`BudgetStatus`
      (`src/uar/runtime/cost_budget.rs`) — accumulation, warn/exceeded
      thresholds, `uar_cost_budget_spent_usd` gauge. 2 unit tests green.

## 2. Wire into the runtime (this pass)

- [x] 2.1 `RunManager.cost_budget` field + `with_global_cost_budget` builder
      seeding the `Global` scope from `LlmConfig.budget.global_limit`.
- [x] 2.2 Per-run spend recorded against `Run`/`Session`/`Agent`/`Global`
      scopes alongside the existing `record_llm_cost` call.
- [x] 2.3 `NormalizedEvent::BudgetAlert` domain event + `agui.budget.alert`
      SSE mapping for the first scope that crosses a threshold.
- [x] 2.4 Closed the `manager.rs` cache-read-tokens gap: `cached_tokens`
      from the driver-level `Usage` event is now accumulated
      (`total_cache_read_tokens`) and passed to `estimate_cost` instead of a
      literal `0`.

## 3. Verify

- [x] 3.1 `cargo check --lib` green.
- [x] 3.2 `cargo test --lib cost_budget` — 2/2 green.
- [x] 3.3 Full-suite batch checkpoint: 318/318 lib tests green.

## 4. Follow-ups (disclosed, not this pass)

- [ ] Per-agent/per-task budget *configuration* surface (aggregation is
      wired; only the global limit is seeded from config today).
