# Assessment — uar-carryover-audit

**Date:** 2026-07-06
**Method:** direct inspection only — `grep`/`git log`/reading source
for every candidate in `goals.md`, no assumption carried forward from
`current-waypoint.json`'s existing claims. This phase exists
specifically because two of those claims were already found stale
during seeding; the goal here is to verify everything else the same
way, not repeat the mistake.

**Baseline health check**: `cargo check --lib` clean (same 2
pre-existing, unrelated `tool_router` dead_code warnings as every
recent phase).

## Carryover item disposition

### CH-06 — per-agent/per-task budget configuration surface: **CONFIRMED STILL OPEN**

`LlmBudgetConfig` (`src/config.rs:1471`) has exactly two fields:
`global_limit: f64` and `model_limits: HashMap<String, f64>`
(per-model). No per-agent or per-task field exists anywhere in the
struct, and no other budget-scoping code was found via `grep` across
`src/uar/settings/` or `src/config.rs`. The carried claim ("aggregation
done; only global limit configurable") is accurate as stated.

### CH-08 — activation-outcome correlation: **CONFIRMED STILL OPEN, exactly as carried**

`src/uar/runtime/skills/service.rs:472` has an explicit, already-honest
comment: `record_skill_activation` (recall half) is called per matched
skill; `record_skill_activation_outcome` (outcome half,
`src/uar/telemetry/metrics.rs:245`) is defined but **never called
anywhere** (confirmed via `grep -rn` across `src/`). The comment
itself already documents why: correlating activation against a run's
later tool-call stream is a separate, harder problem, deliberately
scope-cut. Nothing to correct here — this carried item was accurate.

### CH-07 — durable (non-session-scoped) cost/spend history: **CONFIRMED STILL OPEN**

`src/uar/runtime/cost_budget.rs`'s own module doc comment (line 11)
states persistence is "intentionally out of [scope]". `CostBudgetTracker`
holds spend/limits in a plain `HashMap` behind `Arc<RwLock<Inner>>` —
in-process only, lost on restart. No SurrealDB/Postgres table or
persistence call found for cost history anywhere in `src/`. Carried
item accurate.

### R2 (cancel semantics) — **RESOLVED, stale in `productDecisionsRequired`**

Fully implemented: `src/uar/runtime/manager.rs` has a per-run
`CancellationToken` (`run_cancellations: Arc<RwLock<HashMap<String,
CancellationToken>>>`), `cancel_run()`, `cancel_run_if_no_subscribers()`
(last-subscriber-drop semantics — exactly what
`openspec/changes/add-run-cancellation/proposal.md`'s "Decision R2"
specifies), and a `root_cancellation_token()` tied to graceful
shutdown. The decision was made and shipped; this is not an open
product question.

### R3 (eval scope) — **RESOLVED, stale in `productDecisionsRequired`**

`src/uar/eval/` has 8 real source files (`cli.rs`, `targeted.rs`,
`integration_tests.rs`, and others); `openspec/changes/eval-targeted-suites/`
and several other `eval-*` changes are landed (confirmed present in
`openspec/changes/` and referenced in prior phases' `changes_completed_note`
history). A build-your-own eval harness is exactly the decision that
was made and executed — this question was resolved, likely during
`uar-next-harness` or an earlier phase, without the waypoint's
`productDecisionsRequired` list being pruned afterward.

### R4 (guardrail build-vs-buy) — **RESOLVED, stale in `productDecisionsRequired`**

`GuardrailsConfig` (`src/config.rs:1907`) has real, non-stub fields
(`input_screening_enabled`, `block_on_injection`, `block_on_pii`);
`openspec/changes/guardrail-pii-block/` and
`openspec/changes/mount-governance-guardrails/` are both landed. "Build"
was the decision, and it shipped.

**All three `productDecisionsRequired` entries are stale** — none are
still open questions. This list should be cleared, not carried into
whatever phase comes after this one.

### Unformatted code — **CARRIED FILE NAMES ARE STALE, UNDERLYING PROBLEM IS REAL AND CURRENT**

Neither `routes.rs` nor `ingestion_worker.rs` (the two files the
carried note names) appears in a fresh `cargo fmt --check`. That check
instead found **21 diffs across 12 different files**:
`src/server.rs` (1), `src/uar/compiler/conformance.rs` (7),
`src/uar/eval/integration_tests.rs` (1), `src/uar/guardrails.rs` (1),
`src/uar/mcp_server.rs` (1), `src/uar/memory/mcp_server.rs` (1),
`src/uar/runtime/skills/wasm_runtime.rs` (2), `tests/agent_templates_test.rs`
(1), `tests/bdd.rs` (3), `tests/integration/live/load_test.rs` (1).
This is real, current drift — likely accumulated incrementally across
several recent phases/`spawn_task` sessions each adding a few
unformatted lines rather than one large event. A single `cargo fmt`
pass (formatting-only, zero behavior change) would clear all 21 at
once.

### `task_7c2fd7ee` (SurrealQL `type::thing()` fix) — **CONFIRMED STILL OPEN, unchanged**

All 3 call sites still use `type::thing()`:
`src/uar/persistence/providers/surreal.rs:524`,
`src/uar/compiler/storage/surreal.rs:71,109`. Identical to the state
found during `uar-security-deps-and-hygiene`'s `surrealdb-upgrade`
change. The separately-`spawn_task`-started session has not yet landed
a fix (or hasn't touched these files if it has committed elsewhere).

### `task_188b4179` (`VectorMatcher::embed_batch` placeholder embeddings) — **CONFIRMED STILL OPEN, unchanged**

`src/uar/runtime/matching/vector.rs`'s `embed_batch` still has the real
model-forward call commented out (`// UNCOMMENT ONCE COMPILED TO VERIFY
SIGNATURE` / `// let output = model.forward(input_ids, ...);`) — tensors
are built from real tokenization but never actually run through the
model, so embeddings are still placeholders. Unchanged from the
carried description.

## Spec Gap Summary

No canonical spec documents an expected "when is a
`productDecisionsRequired`/`carryOverDebt` entry considered resolved
and removed" process — this assessment found 5 stale entries across
two lists (H3, H8 recorders — corrected during seeding; R2, R3, R4 —
corrected here) purely through direct re-verification, not because any
process caught the drift. Worth a process note for whoever runs
`/kbd-reflect` on this phase: the fix isn't a new doc, it's discipline
— prune these lists as items resolve, in the same reflection that
resolves them, rather than leaving pruning to a future phase's spot
checks.

## Goal Progress

| Goal | Status | Reason |
|---|---|---|
| G0 Audit the carryOverDebt list | **MET** (as an assessment goal — every candidate now has a directly-verified status) | 3 items confirmed genuinely open (CH-06, CH-08, CH-07) with real code gaps; 3 product decisions confirmed resolved and stale in the tracking list (R2, R3, R4); the fmt-drift note confirmed stale in specifics but real in substance (12 files, not the 2 named); both spawn_task bugs confirmed still open and unchanged. |

## Candidate work surfaced for `/kbd-plan`

Genuinely open, agent-actionable, non-duplicative-with-other-sessions
items to choose from:

1. **CH-06**: add per-agent/per-task budget configuration (real feature
   work — a new config surface + enforcement plumbing in
   `cost_budget.rs`/`config.rs`).
2. **CH-08**: wire activation-outcome correlation (real feature work —
   requires correlating skill activation against a run's later
   tool-call stream; the module's own comment already flags this as
   "harder" than a simple recorder call).
3. **CH-07**: add durable cost/spend history (real feature work — a
   persistence layer for `cost_budget.rs`'s currently in-memory state).
4. **fmt pass**: a single, mechanical, zero-risk `cargo fmt` across the
   12 drifted files (S-complexity, high confidence, good "clear the
   decks" first change if bundled with any of 1–3).

**Explicitly not this phase's to plan**: `task_7c2fd7ee` and
`task_188b4179` remain owned by their separate `spawn_task` sessions —
confirming they're still open here is informational (so `/kbd-plan`
doesn't accidentally step on persistence/matching code those sessions
may be actively touching), not an invitation to fix them from this
phase.

## Sycophancy self-check

- S-02: this assessment does not claim any of CH-06/CH-08/CH-07 are
  "almost done" or "simple fixes" — CH-08 and CH-07 in particular are
  flagged as requiring real design work (event correlation, persistence
  schema), not just wiring.
- S-03: the corrected `productDecisionsRequired` finding (all 3 stale)
  is a stronger, more surprising conclusion than the seed's original
  hypothesis (which only flagged 2 stale `carryOverDebt` entries) —
  stated plainly rather than downplayed to match the seed's framing.
- S-07: no scope creep — every item here is either from `goals.md`'s
  own candidate list or (the R2/R3/R4 findings) a natural byproduct of
  checking items already on that list; nothing new was invented.

ASSESSMENT COMPLETE
