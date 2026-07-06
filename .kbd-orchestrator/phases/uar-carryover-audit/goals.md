# Goals

Phase: **uar-carryover-audit**

Seeded at the user's explicit choice ("Fresh assess first") after
`uar-frontend-typecheck-cleanup`'s reflection recommended returning to
feature scope, but two of the three carried-over debt items checked
during that recommendation turned out to be stale:

- `current-waypoint.json`'s `carryOverDebt` flags **H3
  (`emit-runtime-step-events`)** as "unbuilt planned change (top
  priority)" — it is not. `openspec/changes/emit-runtime-step-events/`
  has a full proposal + design + tasks.md (all tasks checked except one
  manual live-env step, explicitly marked non-blocking), and the actual
  code (`src/normalized.rs`, `src/llm/orchestrator.rs`,
  `src/uar/domain/events.rs`, `src/uar/runtime/manager.rs`) is present
  and wired, confirmed via `git log` (commits `2444b8c`/`9c45aae`/`f43e403`,
  already ancestors of `HEAD`).
- The same list flags **"Sandbox + MCP-status metric recorders still
  dead (H8 partial)"** — also not true.
  `crate::uar::telemetry::metrics::set_mcp_server_status` and
  `record_sandbox_*` are called from real code paths
  (`src/mcp/registry.rs`, `src/llm/orchestrator.rs`), confirmed via
  `grep`.

**This phase has no single fixed goal yet.** Its job is to run
`/kbd-assess` against the full carry-over list below, determine what's
actually still open vs. already resolved by other work (this repo has
several `spawn_task` sessions and phases running concurrently), and
produce a grounded `assessment.md` that `/kbd-plan` can act on —
exactly the process gap this seed exists to close.

## Candidates to verify (do not trust without direct evidence)

- **CH-06** per-agent/per-task budget configuration surface
  (aggregation reportedly done; only global limit configurable) — a
  quick `grep` found no per-agent/per-task budget code; **plausibly
  still genuinely open**, but confirm directly rather than assume.
- **CH-08** activation-outcome correlation (recall half wired, outcome
  half unsolved) — not yet checked this session.
- **Durable (non-session-scoped) cost/spend history** for the CH-07
  dashboard — not yet checked this session.
- **R2/R3/R4 product decisions required** (cancel semantics, eval
  scope, guardrail build-vs-buy) — `openspec/changes/` already contains
  `add-run-cancellation`, `eval-*` (multiple), and
  `mount-governance-guardrails`/`guardrail-pii-block` entries; these
  decisions may already be resolved by that landed work. Check before
  assuming they're still open questions.
- **Unformatted code** — the carried note names `routes.rs` +
  `ingestion_worker.rs` specifically; a fresh `cargo fmt --check` this
  session found **neither file** in the diff, but found 21 diffs across
  12 *other* files instead (`server.rs`, `uar/compiler/conformance.rs`
  ×7, `uar/eval/integration_tests.rs`, `uar/guardrails.rs`,
  `uar/mcp_server.rs`, `uar/memory/mcp_server.rs`,
  `uar/runtime/skills/wasm_runtime.rs` ×2, plus 3 test files). The
  carried note is stale in specifics but the underlying problem
  (unformatted code drifting onto `main`) is real and current — worth
  a `cargo fmt` pass regardless of which files it turns out to be.
- **`task_188b4179`** (`VectorMatcher::embed_batch` placeholder
  embeddings) and **`task_7c2fd7ee`** (SurrealQL `type::thing()` fix) —
  both were separately `spawn_task`-started by the user in a prior
  session; check their current status (may already be resolved) before
  scoping any related work here, to avoid duplicating effort.

## Already confirmed resolved (do not re-verify, do not re-carry)

- H3 `emit-runtime-step-events` — done, see above.
- H8 sandbox/MCP-status metric recorders — done, see above.
- 96-alert Dependabot backlog, `benches/hot_path.rs` never run,
  `tests/uar_integration.rs`/`tests/bdd.rs` compile failures,
  `write-position-reminder.sh` schema mismatch, artifact-refiner QA
  gate decision, 17 `bun run typecheck` errors — all resolved across
  `uar-security-deps-and-hygiene` and `uar-frontend-typecheck-cleanup`.
  The `carryOverDebt` list in `current-waypoint.json` has not been
  pruned to reflect this; part of this phase's assess output should be
  a cleaned-up carry-over list for future phases to seed from, so this
  staleness problem doesn't compound further.

## Success criteria

- `assessment.md` states, for every item above, a directly-verified
  status (DONE / OPEN / PARTIAL) with the evidence used — not a
  restatement of the carried claim.
- The output gives `/kbd-plan` enough to scope a real, non-stale set of
  changes — whether that's CH-06, CH-08, cost history, a product
  decision, the fmt pass, or some combination.
- `current-waypoint.json`'s `carryOverDebt` list is corrected (stale
  entries removed or marked resolved) as part of this phase's own
  output, so the next phase doesn't inherit the same staleness.

---

## Instructions

Review and refine the goals above before running `/kbd-assess`. When
ready:

```
/kbd-assess uar-carryover-audit
```
