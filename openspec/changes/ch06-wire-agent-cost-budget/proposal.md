# ch06-wire-agent-cost-budget

## Why

Carried as "CH-06 per-agent/per-task budget configuration surface
(aggregation done; only global limit configurable)" across several
phases. `uar-carryover-audit`'s assessment confirmed the gap but
initially framed it as a missing config surface. Deeper investigation
during planning found it's narrower than that:

- `AgentDescriptorIR.BudgetsSection` (`src/uar/compiler/ir.rs:611`)
  already declares `max_cost_per_session_usd: Option<f64>` — parsed
  from the agent spec, even completeness-checked
  (`completeness.rs:102`).
- `CostBudgetTracker` (`src/uar/runtime/cost_budget.rs`) already
  *records* spend against `BudgetScope::Agent` on every cost event
  (`manager.rs`, via `cost_scope_agent_id`) — the tracking dimension
  already exists.
- The actual gap: nothing ever read the declared limit and called
  `cost_budget.set_limit(BudgetScope::Agent, agent_id, ...)`. A
  further wrinkle found while implementing: `AgentPolicy` (the
  runtime-facing domain type `manager.rs` actually reads) has **no**
  `budgets` field at all — `to_artifact.rs`'s own module doc explains
  why: sections with "no direct home on `AgentArtifact` (governance,
  A2A, **budgets**, ...) are preserved losslessly under
  `AgentArtifact.extensions` as JSON." So the limit isn't reachable via
  a typed `artifact.policy.budgets...` path — it has to be read out of
  `artifact.extensions["budgets"]` as JSON, an already-established,
  intentional convention in this codebase, not a new one introduced
  here.

"Per-task" is dropped from this change's scope entirely: `manager.rs`'s
own comment (near the existing `BudgetScope::Task` handling) states
this runtime has no task entity distinct from a run, so there is
nothing to wire a per-task limit to.

## What changed

- Added `agent_cost_limit_from_extensions(&HashMap<String, Value>) ->
  Option<f64>` (`src/uar/runtime/manager.rs`) — reads
  `extensions["budgets"]["max_cost_per_session_usd"]`, returning `None`
  for an absent key, a `null` "budgets" value (every agent gets one,
  per `to_artifact.rs`'s unconditional `stash()` even when the IR
  section is `None`), a missing/`null` field, or a non-numeric value.
- In `RunManager::start_run`, call `cost_budget.set_limit(Agent,
  agent_id, BudgetLimit { limit_usd, warn_at: 0.8 })` when the helper
  returns `Some`. Called once per run rather than cached — `set_limit`
  is a single `HashMap` insert behind a `RwLock`; a "have we configured
  this agent yet" cache would be premature complexity for that cost.

## Verification

- 4 new unit tests (`cost_budget_wiring_tests` in `manager.rs`) cover:
  a declared limit extracts correctly; an absent `"budgets"` key
  returns `None`; a `null` `"budgets"` value (the common case — every
  agent without an explicit budgets section) returns `None`; a present
  `"budgets"` object missing the specific field, or with that field
  explicitly `null`, both return `None`.
- Deliberately did **not** duplicate `cost_budget.rs`'s own
  `accumulates_and_crosses_thresholds` test — that already proves
  `record()`'s `Warning`/`Exceeded` behavior once a limit is set; this
  change's own tests cover exactly the new logic (JSON extraction), not
  already-tested tracker behavior.
- `cargo test --lib cost_budget`: 6/6 green (2 existing
  `cost_budget::tests` + 4 new `manager::cost_budget_wiring_tests`).
- `cargo check --lib`: clean (same 2 pre-existing unrelated warnings).
