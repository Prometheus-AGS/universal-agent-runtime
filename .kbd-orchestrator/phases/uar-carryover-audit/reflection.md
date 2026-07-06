# Phase Reflection: uar-carryover-audit

**Project:** universal-agent-runtime
**Date:** 2026-07-06
**Phase completion:** 100%
**Changes completed:** 4 / 4

## What diverged from plan, first

This phase existed specifically *because* of divergence between carried
state and reality (`uar-frontend-typecheck-cleanup`'s reflection
recommended feature scope, but 2 of the 3 items spot-checked at that
point turned out already done). That pattern continued through this
phase's own execution, in a good way — deeper investigation kept
finding the *real* shape of each item was different from its
assessed/carried description, and each time the correction was made
and disclosed rather than smoothed over:

- **CH-06's actual gap was narrower and more specific than assessed.**
  `assessment.md` said "no per-agent/per-task field exists anywhere."
  True at the `LlmBudgetConfig` (global config) layer, false at the
  `AgentDescriptorIR` (per-agent spec) layer — `BudgetsSection.max_cost_per_session_usd`
  was already declared, parsed, and completeness-checked. Then, while
  *implementing* that narrower fix, a second surprise: `AgentPolicy`
  (the actual runtime-facing type) has no `budgets` field at all — the
  value only reaches the runtime via `extensions["budgets"]` as JSON,
  an already-established, intentional convention documented in
  `to_artifact.rs`'s own module doc, not something this change
  invented. Two corrections in sequence, both caught before landing
  wrong code, both disclosed in the change's own `proposal.md`.
- **CH-08's design required tracing actual data flow, not just
  wiring an existing function call.** The plan's initial framing
  ("call `record_skill_activation_outcome`") undersold the real work:
  correlating "was skill X's tool used" required (a) capturing
  skill→MCP-server ownership *before* registries merge (the merged
  registry loses that distinction), and (b) reverse-resolving invoked
  tool names back to their owning server via the registry's own
  `resolve_mcp_tool` — reusing existing authoritative logic rather than
  reimplementing tool-name-sanitization rules that turned out to use
  `__` as a separator, not the `::` the plan's prose loosely implied.
- **CH-07's placement mattered more than expected.** The first draft of
  the correlation/persistence wiring was nested inside an `if let
  Some(cost) = cost_usd_estimate && ... { }` branch — technically
  compiled, but silently made outcome-correlation and cost-history
  depend on cost-tracking being enabled, which has nothing to do with
  either feature. Caught by re-reading the surrounding control flow
  before committing, moved to the single run-end point common to every
  terminal branch (cancelled / has-usage / no-usage).

As with every phase since `uar-spec-v2-and-polish`: none of this
phase's 4 OpenSpec change directories were run through `/opsx:verify` +
`/opsx:archive` — same already-decided pattern (artifact-refiner
formally retired via `uar-security-deps-and-hygiene`'s decision
record), not new debt.

## Artifact Quality Summary

| Metric                             | Value                                 |
| ----------------------------------- | -------------------------------------- |
| Changes with QA (artifact-refiner)  | 0/4                                     |
| First-pass pass rate                | N/A — gate not applicable this phase   |
| Changes requiring refinement        | 0                                      |
| Total refinement iterations         | 0                                      |

No artifact-refiner MCP tool available (formally retired,
`uar-security-deps-and-hygiene`). Replacement verification: `cargo
check`/`cargo test --lib`/`cargo clippy` for every change, plus a real
round-trip integration test against a live embedded SurrealKV instance
for `ch07-durable-cost-history` given its persistence-layer risk
(matching `surrealdb-upgrade`'s established precedent for
persistence-touching changes).

### Recurring Constraint Violations

None — no constraint-based QA gate ran this phase.

## Goals

| Goal | Status | Notes |
|---|---|---|
| G0 Audit the carryOverDebt list, then execute what's genuinely open | **MET** | All 3 real gaps (CH-06, CH-08, CH-07) landed with working, tested implementations — not just type-checked stubs. The audit itself corrected 5 stale entries across two lists (`carryOverDebt`: H3, H8 recorders; `productDecisionsRequired`: R2, R3, R4) before any code was written, preventing wasted effort on already-solved problems. |

Fully MET. No scope cuts — all 4 planned changes landed. `plan.md`'s
own scope was itself revised once (CH-06 narrowed) as a direct result
of doing the work carefully rather than executing the original framing
blindly; that revision is a sign the process worked, not a failure to
plan correctly upfront.

## Delivered Changes

- `fmt-drift-cleanup` — `15457ed` — by: claude-code
- `ch06-wire-agent-cost-budget` — `e68e2fb` — by: claude-code
- `ch08-activation-outcome-correlation` — `e68e2fb` — by: claude-code
- `ch07-durable-cost-history` — `ac75ee6` — by: claude-code

All 4 committed to `main` (not pushed — no push instruction given).
Full checkpoint green at the end of both Round 2 and Round 3: `cargo
test --lib` 372/372 (363 pre-phase baseline + 9 new tests: 4 CH-06, 4
CH-08, 1 CH-07 round-trip), `cargo clippy` zero new warnings under both
default and `postgres-backend` feature sets (confirmed via `git stash`
A/B comparison each time, not assumed).

## Technical Debt Introduced

None. Every change closed a real, previously-disclosed gap with a
working implementation:

- CH-06's `agent_cost_limit_from_extensions` reads an established,
  documented convention (`extensions["budgets"]`), not a new one.
- CH-08 explicitly excludes prompt-overlay-only skills from outcome
  tracking rather than inventing a proxy signal — a disclosed
  limitation, not a shortcut.
- CH-07's dual-backend implementation (SurrealDB + Postgres) keeps
  feature parity with this project's established pattern rather than
  only covering the default backend.

## Debt Found, Not Introduced By This Phase (disclosed for the record)

- `task_7c2fd7ee` (SurrealQL `type::thing()` fix) and `task_188b4179`
  (`VectorMatcher::embed_batch` placeholder embeddings) — both
  reconfirmed still open and unchanged during assessment. Owned by
  separate `spawn_task` sessions; this phase deliberately did not
  touch either, to avoid duplicating or conflicting with that
  in-flight work.
- 215 pre-existing `bun run lint` problems and 174 pre-existing
  `prettier --check` failures — quantified in the prior phase
  (`uar-frontend-typecheck-cleanup`), unrelated to and untouched by
  this phase (backend-only scope).
- No canonical doc explains the `AgentPolicy.extensions` convention
  (which sections live there vs. get a typed field) beyond
  `to_artifact.rs`'s own module comment — worth a short mention in
  `docs/` if a future phase adds more extension-backed config, so this
  pattern doesn't require re-discovery every time.

## Architecture Integrity

- No violations of the Prometheus Base Rules Set identified:
  - Rule 5/6 (truth over fluency, evidence before conclusions): both
    of CH-06's mid-implementation corrections and CH-08's design
    refinement were caught by reading actual code before writing more
    of it, not assumed from the plan's prose.
  - Rule 3/31 (surgical, small, reviewable changes): each of the 4
    changes touched only what its own `proposal.md` names; CH-07's
    dual-backend footprint (schema + trait + 2 provider impls) is
    larger than the others but is exactly the minimum needed for
    stated feature parity, not scope creep.
  - Rule 30 (tests are part of completion): every change verified via
    real command execution; CH-07 specifically got a real database
    round-trip, not a mock.
  - Rule 29 (strong typing): `CostEntry`/`agent_cost_limit_from_extensions`
    are properly typed at their boundaries; the one JSON `Value`
    traversal (`extensions["budgets"]`) is bounded to a single small
    helper function and matches an existing, intentional convention in
    the codebase rather than introducing untyped access more broadly.
- `.kbd-orchestrator/constraints.md` does not exist in this repo (same
  as every prior phase's reflection) — no separate machine-checkable
  constraint file beyond the rule set above.

## Cross-Tool Coordination Notes

Single-tool phase (`sourceTool: claude-code` throughout). No conflicts
with the two in-flight `spawn_task` sessions — this phase's 4 changes
touched `manager.rs`, `persistence/`, `cost_budget`-adjacent code, and
formatting; neither spawn_task's target files (`surreal.rs`'s
`type::thing()` sites, `vector.rs`'s `embed_batch`) were modified here,
confirmed by direct inspection before starting, not assumed.

- **Progress tracking**: `progress.json`/`current-waypoint.json` stayed
  in sync throughout; each round's code commit was paired with a
  corresponding state-update commit at natural checkpoints.
- **Handoff quality**: `assess.handoff.json`, `plan.handoff.json`, and
  `execute.handoff.json` were all written at their respective stage
  boundaries the first time through — no retroactive handoff writes
  needed this phase (unlike two phases prior).

## Lessons Learned

- **A carried "done"/"still open" claim that hasn't been re-verified in
  N phases should be treated as a hypothesis, not a fact — even inside
  a single phase's own execution.** This phase's central lesson from
  its own seeding (don't trust stale carryover) applied again
  *mid-execution*: CH-06 and CH-08's plan-time framings were themselves
  slightly wrong once real code was read closely, and catching that
  before writing the wrong fix cost minutes, not a redo.
- **When a design decision references "an existing pattern," go find
  and read that pattern before assuming it applies the way described.**
  CH-08's tool-namespacing separator (`__`, not the plan's loosely-described
  `::`) and CH-06's extensions-JSON convention were both confirmed by
  reading the actual authoritative code (`McpRegistry::from_config`,
  `to_artifact.rs`) rather than inferring from adjacent documentation
  or comments alone.
- **A wrongly-placed correct-looking line of code is a real risk, not
  just a style nit.** CH-07's correlation logic compiling fine inside
  the cost-tracking-enabled branch could have shipped a subtly broken
  feature (outcome tracking silently disabled whenever cost tracking
  is off) that no test would have caught unless it specifically
  exercised that combination. Re-reading the surrounding control flow
  before committing — not just confirming the new code compiles and
  its own unit test passes — is what caught it.

## Next Phase Focus

This phase closes cleanly with no items of its own re-carried. What
remains open, all pre-existing and already tracked elsewhere:

1. **`task_7c2fd7ee`** and **`task_188b4179`** — check status of the
   separate `spawn_task` sessions before scoping any phase touching
   persistence (`surreal.rs`) or vector matching (`vector.rs`).
2. **215 `bun run lint` problems / 174 `prettier --check` failures** —
   quantified but unaddressed frontend hygiene debt, candidate for a
   future frontend-focused phase if one comes up.
3. **`docs/` gap on the `AgentPolicy.extensions` convention** — minor,
   worth folding into a future phase that touches agent-spec docs
   rather than justifying its own phase.

Given this phase delivered 3 real, tested features (CH-06/07/08) and
found no further hygiene backlog worth a dedicated phase, recommend the
next phase pursue genuine new feature/roadmap scope — the same
recommendation `uar-frontend-typecheck-cleanup`'s reflection made,
which now actually has real, delivered ground under it rather than an
open question mark.

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess` invocation.
