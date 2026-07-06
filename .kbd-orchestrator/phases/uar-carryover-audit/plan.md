PLAN: uar-carryover-audit
Project: universal-agent-runtime
Date: 2026-07-06
OpenSpec available: YES
Changes to implement: 4

## Framing

`assessment.md` confirmed 3 genuinely open feature gaps (CH-06, CH-08,
CH-07) plus one zero-risk mechanical fix (fmt drift). While drafting
this plan, deeper investigation of CH-06 found it's smaller and more
precisely scoped than the assessment stated — worth correcting here
rather than carrying an inaccurate scope into execute:

- **CH-06 correction**: `assessment.md` said "no per-agent/per-task
  field exists anywhere." That's true of the *global* config layer
  (`LlmBudgetConfig` in `config.rs`), but `AgentDescriptorIR`'s
  `BudgetsSection` (`src/uar/compiler/ir.rs:611`) already declares
  `max_cost_per_session_usd: Option<f64>` — parsed, even
  completeness-checked (`completeness.rs:102`) — and the runtime's
  `CostBudgetTracker` already *records* spend against
  `BudgetScope::Agent` on every cost event
  (`manager.rs:1568`, via `cost_scope_agent_id`). The only missing
  piece is wiring: nothing ever reads
  `artifact.policy.budgets.max_cost_per_session_usd` and calls
  `cost_budget.set_limit(BudgetScope::Agent, agent_id, ...)`. This
  makes CH-06 a small, low-risk wiring change, not new config-surface
  design. **"Per-task" is dropped from CH-06's scope** — the
  codebase's own comment (`manager.rs:1563`) states this runtime has
  no task entity distinct from a run; there's nothing to wire a
  per-task limit to.

CH-08 and CH-07 remain real design work (event correlation strategy;
persistence schema), sized and sequenced accordingly. The fmt change is
sequenced first as a zero-risk "clear the decks" step, consistent with
`plan.md` patterns from prior phases.

## CHANGE LIST (ordered)

1. **fmt-drift-cleanup**: run `cargo fmt` across the 12 drifted files
   - Scope: formatting only, no behavior change
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: XS · Complexity score: Trivial · Model class: small
   - Customer value: LOW (pure hygiene) but zero risk
   - Details: `server.rs`, `uar/compiler/conformance.rs` (×7 sites),
     `uar/eval/integration_tests.rs`, `uar/guardrails.rs`,
     `uar/mcp_server.rs`, `uar/memory/mcp_server.rs`,
     `uar/runtime/skills/wasm_runtime.rs` (×2), `tests/agent_templates_test.rs`,
     `tests/bdd.rs` (×3), `tests/integration/live/load_test.rs`. Run
     `cargo fmt` (whole workspace, not per-file, to avoid missing any
     the assessment's snapshot didn't catch), then `cargo fmt --check`
     to confirm zero diffs remain.
   - Verify: `cargo fmt --check` clean; `cargo check`/`cargo test --lib`
     unaffected (formatting cannot change behavior, but confirm anyway).

2. **ch06-wire-agent-cost-budget**: wire the already-declared
   `max_cost_per_session_usd` into the already-existing per-agent
   `CostBudgetTracker` scope
   - Scope: `src/uar/runtime/manager.rs` (where `artifact.policy` and
     `cost_scope_agent_id` are already in scope, near
     `with_global_cost_budget`/the Round-2-noted `manager.rs:1135`
     region)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM (closes a real, user-visible gap — an
     agent author who sets `budgets.max_cost_per_session_usd` in their
     agent spec today has that value silently ignored)
   - Details: When an agent's `artifact.policy.budgets` has
     `max_cost_per_session_usd: Some(limit)`, call
     `cost_budget.set_limit(BudgetScope::Agent, &agent_id, BudgetLimit {
     limit_usd: limit, warn_at: 0.8 })` once per run setup (or once per
     agent, cached — decide based on whether `set_limit` is cheap
     enough to call idempotently every run; it's a single `HashMap`
     insert behind a `RwLock`, so per-run is likely fine and simpler
     than adding a "have we set this agent's limit yet" cache). No new
     config field, no spec/IR change — purely wiring already-declared
     config to the already-recording tracker.
   - Verify: a unit test asserting that setting
     `budgets.max_cost_per_session_usd` on an agent causes
     `cost_budget.record(Agent, agent_id, cost)` to return
     `BudgetStatus::Warning`/`Exceeded` at the right thresholds (mirror
     the existing `accumulates_and_crosses_thresholds` test pattern in
     `cost_budget.rs`); `cargo test --lib` full suite green.

3. **ch08-activation-outcome-correlation**: wire the "outcome" half of
   skill activation tracking
   - Scope: `src/uar/runtime/manager.rs` (matched-skill tool-ownership
     tracking, run-end correlation), `src/uar/telemetry/metrics.rs`
     (already has `record_skill_activation_outcome`, just needs a
     caller)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium · Model class: mid
     (real design decision, not mechanical)
   - Customer value: MEDIUM (closes the precision/recall pair the
     `service.rs:472` comment already flags as half-done; lets
     operators see not just "skill X was activated" but "skill X's
     tools were actually used")
   - Details: Each matched skill's `mcp_config` (when present)
     introduces one or more MCP server names into the per-run merged
     registry (`McpRegistry::from_config` → `registries_to_merge` →
     `final_mcp`, `manager.rs:845-855`) — tools from that server are
     namespaced `{server_name}::{tool_name}` (`src/mcp/registry.rs`'s
     `namespaced_name` convention). Correlation design: **before**
     merging, record each matched skill's introduced server name(s)
     (`skill_id → Vec<server_name>`). During/after the run, collect the
     set of actually-invoked tool names (already flowing through
     `NormalizedEvent::ToolCallComplete` on the same emitter used for
     `SkillActivated`). At run completion, for each matched skill that
     introduced at least one server: `success = true` if any invoked
     tool name starts with one of that skill's server-name prefixes,
     else `false`; call `record_skill_activation_outcome(skill_id,
     success)`. Skills with no `mcp_config` (prompt-overlay-only
     skills) have no distinguishable "used" signal at this layer —
     **explicitly exclude them from outcome tracking** rather than
     inventing a proxy signal; disclose this as a known limitation in
     the change's `proposal.md`, don't silently paper over it.
   - Verify: a unit/integration test with a fake matched skill +
     synthetic tool-call event confirming `record_skill_activation_outcome`
     fires with the correct `success` value; confirm prompt-overlay-only
     skills are correctly excluded (no call, not a `false` call).

4. **ch07-durable-cost-history**: persist `CostBudgetTracker`'s
   in-memory spend to SurrealDB
   - Scope: `migrations/surrealdb/schema.surql` (new table),
     `src/uar/persistence/mod.rs` (new `PersistenceLayer` trait
     methods), `src/uar/persistence/providers/surreal.rs` +
     `src/uar/persistence/providers/postgres.rs` (implementations, per
     this project's dual-backend pattern — check whether Postgres
     needs a matching `migrations/*.sql` file too), `src/uar/runtime/manager.rs`
     (call the new persist method alongside `cost_budget.record()`)
   - Depends on: NONE (independent of changes 2–3, but naturally lands
     after them since it persists the same spend events change 2 makes
     more meaningful)
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium · Model class: mid
     (schema design + dual-backend implementation, not just wiring)
   - Customer value: MEDIUM (the CH-07 cost dashboard currently loses
     all spend history on every restart — durable history is what
     makes a "dashboard" meaningful over time rather than a
     since-last-restart snapshot)
   - Details: New table `cost_ledger` (fields: `scope: string,
     scope_id: string, cost_usd: float, recorded_at: datetime` +
     an id/index, following `schema.surql`'s existing
     `SCHEMAFULL`/`DEFINE FIELD`/`DEFINE INDEX` conventions). Add
     `PersistenceLayer::record_cost_entry(scope, scope_id, cost_usd)`
     and `list_cost_history(scope, scope_id, since) -> Vec<CostEntry>`
     (or similar — exact query shape is an execute-time decision, not
     pre-decided here). Wire from `manager.rs`'s existing cost-recording
     block (`manager.rs:1560-1600`, same block change 2 touches) —
     fire-and-forget (`tokio::spawn`) rather than blocking the
     hot path on a DB write, matching this project's async-patterns
     skill guidance for non-critical-path writes.
   - Verify: `cargo test --lib` (new persistence method tests, mirroring
     existing `surreal.rs`/`postgres.rs` test patterns);
     live-server smoke check (embedded SurrealKV) confirming a
     recorded cost entry round-trips through a real query — given this
     touches the persistence layer, treat this as its own dedicated
     checkpoint like `surrealdb-upgrade` did, not folded into the
     shared Round 2 checkpoint below.

## EXECUTION ROUND ORDER

- **Round 1** (sequenced first, zero risk): `fmt-drift-cleanup`
- **Round 2** (parallel, one shared checkpoint): `ch06-wire-agent-cost-budget`,
  `ch08-activation-outcome-correlation`
- **Round 3** (sequenced, own dedicated checkpoint — touches
  persistence): `ch07-durable-cost-history`

Round 2's shared checkpoint: `cargo test --lib` full suite green,
`cargo clippy` zero new warnings. Round 3 gets its own checkpoint
(persistence-layer changes get individual verification in this
project's established pattern — see `surrealdb-upgrade`/`rmcp-pin-bump`
from `uar-security-deps-and-hygiene`) including a live-server smoke
check.

## COMMANDS TO RUN

```
/opsx:new fmt-drift-cleanup
/opsx:new ch06-wire-agent-cost-budget
/opsx:new ch08-activation-outcome-correlation
/opsx:new ch07-durable-cost-history
```

Per this project's established practice, these
`openspec/changes/<id>/proposal.md` + `tasks.md` pairs are written
per-change at execute time, not pre-scaffolded during planning.

## Sycophancy self-check

- S-02: CH-08's known limitation (prompt-overlay-only skills excluded
  from outcome tracking) is stated up front as a real, disclosed gap,
  not smoothed over as "fully solved."
- S-03: at least 2 explicit trade-offs stated — CH-06's
  per-run-vs-cached `set_limit` call (simplicity chosen over a
  premature cache), and CH-07's fire-and-forget-vs-blocking persistence
  write (throughput chosen over strict durability-before-continue).
- S-07: no scope creep — CH-06's scope was *corrected* (narrowed,
  actually) from the assessment's framing based on new evidence found
  during planning, not expanded; "per-task" was explicitly dropped as
  inapplicable rather than force-fit.

PLAN COMPLETE
