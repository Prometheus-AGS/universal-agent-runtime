# Reflection — uar-next-harness

**Date:** 2026-07-04
**Status at reflection:** 16/24 tracked changes complete. Reflecting mid-phase
at explicit operator direction (G1-G3 were the operator's scoped ask for this
turn; G4/G5 were deliberately deferred, not abandoned, and are carried
forward into a new phase rather than left dangling in this one).

## Goal achievement

| Goal | Status | Notes |
|---|---|---|
| **G1 — Foundation (P0)** | **MET** | CH-01 a2a-grpc-enable, CH-02 postgres-credential-store, CH-03 provider-health-failover, CH-04 prompt-dialect-engine all done and verified (330/330 lib tests). MemPalace enablement (D-B) and dependency unpinning (D-D) were deliberate scope cuts documented in plan.md, not gaps. Docker Compose full stack already existed (`docker-compose.{dev,prod,test,override,prod.postgres}.yaml`) from prior work. Cedar `is_tool_allowed` is now called from the orchestrator's tool-approval gate (`manager.rs`), not just HTTP-only as an earlier phase's carryover claimed — confirmed resolved, not carried further. |
| **G2 — Intelligence (P1)** | **MET** | CH-09 capability-registry-benchmarks, CH-11 rag-hardening (in-process per D-A; Knowledge Service extraction deliberately deferred), CH-05 per-model-context-strategy (real Summarize/Hierarchical, not the stub it initially appeared to be), CH-06 cost-budgets-backend, CH-08 skill-activation-metrics all done. |
| **G3 — UX & Integration (P2)** | **MET** | CH-16 skill-pack-bundling, CH-18 librefang-a2a-agui-bridge (D-C scope: UAR-side only), CH-21 agui-spec-alignment (mostly complete — disclosed scope note: self-hosted vocabulary-conformance test in place of the literal AG-UI dojo/client), CH-07 cost-dashboard, CH-10 model-comparison-dashboard all done and verified against a live server with Playwright (not just build-green). Web UI config redesign and agent state visualization were satisfied by prior phases (`direct-entity-migration-*`, `runtime-console-ux`), not part of this phase's own ledger. |
| **G4 — Specification & Distribution (P2)** | **NOT MET** | CH-12 agent-spec-v2, CH-13 compiler-v2-stages, CH-14 conformance-testing, CH-15 agent-template-library, CH-17 eval-targeted-suites — none started. This is the largest remaining chunk: a genuinely sequential chain (spec RFC → compiler stages → {conformance, templates} → evals), matching plan.md's own "Round 3" grouping. |
| **G5 — Polish & Release (P3)** | **NOT MET** | CH-19 docs-overhaul-deploy-guide, CH-20 perf-security-load — none started. Both explicitly depend on Rounds 1-3 having landed (they audit/document what exists), which is now true for G1-G3 but not G4. |

## Delivered changes this reconciliation pass (2026-07-03/04)

`progress.json`'s changes_completed count was stale at 7/24 going into this
pass (CH-02/04/05/06/08/21 were already committed in earlier turns but never
counted). This pass:

- **Wired 5 changes that were committed-but-unused dead code or genuinely
  unstarted**: CH-02 (Postgres credential store — added migration + parity
  tests + docs), CH-03 (provider health monitoring + failover — built a new
  `ProviderHealthMonitor` and wired it through router/registry/orchestrator/
  server; genuinely unstarted before this pass), CH-04 (prompt dialect
  engine — wired into orchestrator request assembly via a new
  `LlmRequest.extra_params` seam), CH-06 (cost budget tracking — wired
  `CostBudgetTracker` into `RunManager` + a new `BudgetAlert` event), CH-08
  (skill activation metrics — wired `record_skill_activation` into
  `match_skills`).
- **Verified CH-21 as mostly complete** (disclosed scope note, not a gap).
- **Implemented CH-05 for real** after an initial verification found it was
  a stub disguised as done: `strategy.rs`'s `Summarize`/`Hierarchical` now
  call the existing LLM-backed summarizer instead of a sliding-window
  fallback, plus a new `Auto` variant for model-aware selection.
- **Built CH-07 and CH-10 from scratch**, including two backend gaps that
  blocked them (`BudgetAlert` wasn't wired into the entity-graph SSE
  surface; CH-09's benchmark data wasn't exposed via any REST endpoint).
- **Fixed a real environment blocker**: the `prometheus-entity-management`
  frontend submodule was uninitialized (empty) for this entire phase —
  initializing and building it fixed `cargo build`'s embedded frontend step
  (which had been failing since before this phase started).
- **Fixed a bug that fix surfaced**: the submodule's `useEntityList`/
  `useEntity` hooks are deprecated in its own 2.0 in favor of a
  transport-registry pattern this app never adopted; 6 call sites
  (`use-models.ts` and 5 others) silently returned empty data. Migrated all
  6 to a new shared `useGraphEntities`/`useGraphEntity` pair. Verified live:
  the models catalog went from showing 0 models to 5331 models across 150
  providers, and CH-10's compare feature was exercised end-to-end with real
  data.

## Artifact Quality Summary

No `artifact-refiner` QA gate was run for any change this phase (no
`.refiner/artifacts/` logs exist for this phase's changes) — this is a
carried-over gap from at least 3 prior phases per this project's own
carryover notes ("Artifact-refiner QA gate automation — missed this phase;
automate for next"), not something newly introduced. Every change in this
phase was instead verified via: `cargo check`/`cargo test --lib` (330/330
green), `bun run build` (green), and for the two frontend dashboards, a live
server + Playwright smoke test with console-error checking — a real,
if manual, substitute for automated QA, but not the artifact-refiner
pipeline the process calls for.

## Technical debt

- **Artifact-refiner QA gate still not automated** (carried 4+ phases now).
- **`bun run typecheck` has 17 pre-existing errors unrelated to this
  phase's work**: Base UI `Select`'s `string | null` nullability (affects
  `agent-editor.tsx`, `agents-page.tsx`, `models-page.tsx`'s pre-existing
  `AddModelDialog`), `react-resizable-panels` API drift (`resizable.tsx`),
  `recharts` type-export drift (`chart.tsx`'s `TooltipValueType` import),
  and a couple of narrower type-strictness issues (`knowledge-page.tsx`,
  `use-thread-graph-sync.ts`). None of these were introduced by this phase
  and none affect runtime behavior confirmed via the live server checks
  above, but `bun run typecheck` is not currently a clean gate for this repo.
- **CH-01's gRPC transport** has no dedicated MATRIX.md coverage gap
  remaining (closed in an earlier round) — noted only because an earlier
  batch-verification pass had flagged it as open; confirmed closed by
  `tasks.md` before this pass, so no action needed.
- **Session-scoped cost/budget data**: CH-07's cost dashboard reads the
  in-memory entity graph — there is no persisted run-history REST endpoint.
  Spend/alerts shown cover runs completed since the page was last loaded.
  A durable roll-up is a reasonable follow-up (`CostBudgetTracker`'s own doc
  comment anticipates this).
- **CH-08's activation-outcome correlation** (did the model actually use an
  activated skill's tools) was explicitly scope-cut — only the
  activation-recall half was wired.
- **CH-06's per-agent/per-task budget *configuration*** (as opposed to
  aggregation, which is wired) has no config surface yet — only the global
  limit from `LlmConfig.budget` is seeded.

## Lessons for the knowledge base

1. **Verify "already committed" against `git log`, not just `progress.json`.**
   This phase's tracking file was stale by 9 changes at the start of this
   reconciliation — several changes were committed in earlier turns but
   never counted, and some of those were themselves committed-but-unwired
   dead code (compiles, has its own unit tests, zero real call sites). A
   `grep -rn "record_skill_activation\|CostBudgetTracker::new\|useEntityList"`-style
   "is this actually called anywhere outside its own module/tests" check
   caught all of these — worth making a standard part of "verify this
   change is done," not just "does it compile."
2. **An uninitialized git submodule can silently gate an entire build.**
   `frontend/packages/prometheus-entity-management` had been empty this
   whole phase, breaking the embedded frontend build path specifically —
   `SKIP_FRONTEND_BUILD=1` masked this for backend-only verification passes,
   so it went uncaught until a frontend-touching change (CH-07/CH-10)
   actually needed to build the frontend for real. Worth a `git submodule
   status` check as a standing pre-flight for any phase expected to touch
   `frontend/`.
3. **A library's own `@deprecated` JSDoc is the fastest path to the right
   fix.** The `useEntityList`/`useEntity` drift looked at first like "the
   API just changed shape" (implying a mechanical options-object
   migration); reading the library's actual deprecation notice revealed
   the *intended* replacement was a different, simpler hook family
   (`useEntities`) — but that in turn required a transport-registry pattern
   (`registerEntityTransport`) this app's architecture (explicit
   fetcher-module hydration + SSE) was never built around. The actually-
   correct fix was neither "migrate to the new options object" nor
   "register transports" — it was recognizing the app already had a
   working, simpler pattern (`useGraphStore` selectors) established
   elsewhere and extending that. Don't take a deprecation notice's
   suggested replacement at face value without checking whether the
   surrounding architecture actually wants that replacement's assumptions.
4. **`tailwind-merge` doesn't resolve responsive-modifier conflicts you
   don't spell out.** Overriding a component's `sm:max-w-md` with a bare
   `max-w-3xl` silently loses at the `sm` breakpoint and above (nearly all
   real viewports) — `tailwind-merge` treats differently-prefixed classes
   for the same property as non-conflicting. Any override of a base
   component's responsive-prefixed class needs the same prefix.
5. **Implementation-first with batched compile checkpoints continues to
   work well for this codebase** (per this user's standing preference) —
   this pass used roughly 5 compile checkpoints across a much larger scope
   (8 backend changes + 2 frontend pages + a cross-cutting bug fix) than a
   typical single-checkpoint change, and all landed clean.

## Recommended next phase

**`uar-spec-v2-and-polish`** — carries forward G4 + G5 as a single phase
(they're sequentially related: G5's docs/perf work depends on G4's spec
work existing to document/profile). Seed goals:

- **G4 (primary, sequential chain):** CH-12 agent-spec-v2 (RFC + IR fields:
  `model_requirements`, `prompt_dialect`, `rag_configuration`,
  `context_strategy`, `api_harness`) → CH-13 compiler-v2-stages (PMPO
  stages handle v2 fields) → {CH-14 conformance-testing, CH-15
  agent-template-library} (parallel once CH-13 lands) → CH-17
  eval-targeted-suites (depends on CH-08 activation metrics + CH-09
  benchmark registry, both already done).
- **G5 (after G4, or in parallel where independent):** CH-19
  docs-overhaul-deploy-guide (architecture/router/dialect docs, production
  deployment guide, D-D pin rationale, D-B MemPalace status), CH-20
  perf-security-load (hot-path profiling, 1000-concurrent-agent load test,
  prompt-injection resistance review, `server.rs` split evaluation).

Carry-over items (not new goals, just don't let them drop again):

- Automate the artifact-refiner QA gate (4th consecutive phase carrying this).
- The 17 pre-existing `bun run typecheck` errors (Base UI `Select`
  nullability, `react-resizable-panels`, `recharts` types) — not blocking,
  but worth a dedicated cleanup pass since they're now the *only* remaining
  typecheck noise in this repo.
- CH-06 per-agent/per-task budget configuration surface.
- CH-08 activation-outcome correlation.
- Durable (non-session-scoped) cost/spend history for the CH-07 dashboard.

## Operator-only follow-ups (unchanged from prior phases, still open)

- Activate the eval gate: set `UAR_LLM__API_KEY` secret + `vars.UAR_EVAL_MODEL`,
  run `eval-nightly` via `workflow_dispatch update_baseline=true`, commit
  `evals/results/starter.baseline.json`.
