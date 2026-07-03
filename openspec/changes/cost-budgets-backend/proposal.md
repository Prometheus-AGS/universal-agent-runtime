# CH-06 cost-budgets-backend

## Why

`CostBudgetTracker` (`src/uar/runtime/cost_budget.rs`) already existed —
per-scope spend accumulation, warn/exceeded thresholds, the
`uar_cost_budget_spent_usd` gauge — but was never constructed anywhere
outside its own unit tests. `LlmConfig.budget` (`global_limit`,
`model_limits`, `enforcement`) was likewise a dead config section. And
`manager.rs`'s per-run cost estimate always passed a literal `0` for
cache-read tokens, overcharging every run that had a cache hit relative to
`ModelCost::compute`'s discounted cache-read rate.

## What changed

- `RunManager` owns a shared `CostBudgetTracker` (`cost_budget` field,
  always present — unconfigured scopes just have an unlimited default
  `BudgetLimit`).
- `RunManager::with_global_cost_budget` seeds the `Global` scope's limit
  from `LlmConfig.budget.global_limit` at boot (`server.rs`).
- At run end, spend is recorded against `Run`, `Session`, and `Agent` scopes
  (keyed by `run_id`, `session.id()`, `agent_id`) plus `Global` — `Task` is
  intentionally omitted, this runtime has no task entity distinct from a run.
- A new `NormalizedEvent::BudgetAlert` domain event (`src/uar/domain/events.rs`)
  is emitted for the first scope (in the order above) that crosses its
  warning or hard-limit threshold, mirrored to the SSE/AG-UI surface as
  `agui.budget.alert` (`src/uar/api/sse.rs`).
- Closed a pre-existing gap: `crate::normalized::NormalizedEvent::Usage`
  already carried `cached_tokens`, but `manager.rs` discarded it and passed
  a literal `0` into `estimate_cost`. Now accumulated into
  `total_cache_read_tokens` and passed through, so cache-read discount
  pricing (`ModelCost::compute`) is reflected in both the per-run cost
  estimate and the budget spend recorded against it.

## Scope notes

- Per-agent/per-task budget *configuration* (as opposed to per-agent/per-task
  spend *aggregation*, which is wired) has no config surface yet — only the
  global limit from `LlmConfig.budget` is seeded. Setting a per-agent limit
  requires calling `CostBudgetTracker::set_limit` directly; there's no admin
  API/config schema for it yet. Follow-up if per-agent budgets become a
  product requirement.
- Console/dashboard surfacing beyond the SSE `BudgetAlert` event and the
  Prometheus `uar_cost_budget_spent_usd` gauge (already scraped via
  `/metrics`) is CH-07's job (cost-dashboard).
