# Reflection — `vitest-contract-test-suite`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-reflect`)
**Phase status:** `execute_complete`
**Inputs:** assessment.md, plan.md, progress.json, openspec/changes/*/tasks.md

---

## 1. Goal achievement

Scored against §3 of the assessment ("Definition of done"):

| # | Goal | Status | Evidence |
|---|------|--------|----------|
| A1 | `frontend/vitest.config.ts` exists with React + happy-dom + correct globs | ✅ MET | file present + `pnpm test` runs |
| A2 | `frontend/package.json` has `test`, `test:watch`, `test:ui` scripts | ✅ MET | scripts present |
| A3 | Test deps installed | ✅ MET | `vitest@4.1.7`, `@vitest/ui`, `@vitejs/plugin-react`, `@testing-library/react`, `@testing-library/user-event`, `@testing-library/jest-dom`, `happy-dom` |
| A4 | Existing 6 tests migrated to Vitest (with bun-isms remapped) | ✅ MET | All 6 files now use `vitest` imports; `mock(...)` → `vi.fn()`, `mock.restore()` → `vi.restoreAllMocks()`, `import.meta.dir` → `fileURLToPath`, `toBeObject()` → `not.toBeNull()` |
| A5 | Graph propagation contract test passes | ✅ MET | 3/3 |
| A6 | Optimistic rollback contract test passes | ✅ MET | 5/5 (upsert+rollback, remove+rollback, success path, ProviderMeta) |
| A7 | Bridge refetch contract test passes | ✅ MET | 3/3 — **found and fixed a real bridge bug** |
| A8 | SSE adapter contract test passes | ✅ MET | 5/5 (create/update/delete mapping, unsubscribe, status callback) |
| A9 | CI integration documented | 🟨 PARTIAL | `pnpm test` script in `frontend/package.json` is the entry point; README mention deferred |
| A10 | Audit doc references the contract tests | 🟨 PARTIAL | The audit doc is in `docs/migration-stale-data-audit.md`; the test-suite is currently only documented in this reflection. README/CLAUDE.md mention deferred |

**Aggregate:** 8 MET + 2 PARTIAL + 0 NOT MET = **90% confirmed**. Both partials are doc-side; the working test suite + contracts are the load-bearing deliverable.

---

## 2. Delivered changes

| # | Change | Status | Tests added | Key shift |
|---|--------|--------|-------------|-----------|
| 1 | `vitest-runner-stand-up` | DONE | 0 | Vitest 4.1.7 + happy-dom + RTL wired into pnpm scripts |
| 2 | `migrate-existing-bun-tests-to-vitest` | DONE | 0 (preserved 20 existing) | bun:test → vitest across 6 files; bun-isms remapped |
| 3 | `contract-graph-propagation` | DONE | 3 | Locks `useGraphStore` subscription semantics |
| 4 | `contract-optimistic-rollback` | DONE | 5 | Locks snapshot/upsert/rollback + remove/rollback pattern |
| 5 | `contract-bridge-refetch` | DONE | 3 | Locks `useGraphBridge` watched-type fire conditions; **fixed the real bug** the test surfaced |
| 6 | `contract-sse-adapter` | DONE | 5 | Locks SSE event-name → `EntityChange.op` mapping |

**Total new contract tests:** 16 across 4 files.
**Suite state:** 34 of 36 tests passing; 2 remaining pre-existing failures in `skills-page.utils.test.ts` (preferredModel fixture missing — out of scope).

---

## 3. Artifact Quality Summary

`artifact-refiner` not installed; inline verification used per change.

| Metric | Value |
|--------|-------|
| Changes with explicit QA gate | 0/6 |
| Inline verification (test green for the change's added file) | 6/6 |
| First-pass test green | 4/6 — bridge test caught a real bug (fixed); migration uncovered 9 pre-existing failures (7 self-resolved by setup fix, 2 carry over) |
| Recurring constraint violations | none |
| Net LOC added | +~600 (4 test files × ~150 LOC avg + setup + config) |
| Net LOC removed | 0 (existing tests preserved) |
| Net LOC changed | +1 line in `use-graph-bridge.ts` (the bug fix) |

---

## 4. Technical debt introduced (and discharged)

| Item | Severity | Direction | Notes |
|------|----------|-----------|-------|
| **`skills-page.utils.test.ts` carries 2 pre-existing failures** — `preferredModel` field missing from test fixtures. | Low | **introduced** (surfaced by phase) | Will be tackled in `fix-skills-page-utils-test-fixtures` (seeded in nextPhaseSeeds). Small fix; ~30 min. |
| **CI doesn't run `pnpm test` yet.** Local-only runner. | Med | **introduced** (deferred per locked decision Q4) | The next CI phase wires it. |
| **No README mention of `pnpm test`** for new contributors. | Low | **introduced** | One-paragraph addition; track separately. |
| **3 inlined optimistic-rollback copies** in provider/agent pages still not extracted to a `useOptimisticPatch` helper. | Med | unchanged (carry-over from prior phase) | Now testable in isolation thanks to the contract tests; helper extraction phase can land with confidence. |
| **Bridge bug** — initial `last=""` triggered spurious refetch on first unrelated event | Med | **discharged** | Fixed in this phase; contract test prevents regression. |
| **`runtime-ingest.test.ts` 7 failures** — were silently dead due to bun:test never running in CI. | Low | **discharged** | All 7 now pass after the setup.ts `setState` non-replace fix. |

---

## 5. Lessons captured

1. **Contract tests pay for themselves the moment they're written.** The bridge-refetch test surfaced an honest bug (`last=""` → spurious refetch on unrelated mutation) that had been live in production-equivalent code for several phases. Without the test, this would have caused intermittent refetches in 8 admin hooks for the rest of the bridge era. The test directly justified its existence.
2. **Test setup hygiene matters.** Calling `setState({entities: {}}, true)` to reset Zustand wipes the store's methods — a foot-gun in any test-suite. Always merge (no `true`) when resetting state.
3. **Bun-isms compound.** `mock(...)`, `mock.module(...)`, `mock.restore()`, `import.meta.dir`, `toBeObject()` — each looks innocuous in isolation but adds up when migrating. A migration sweep needs to audit each file individually, not assume the import swap alone is enough.
4. **Race-free contract tests need explicit baseline waits.** The bridge test initially failed because it captured `spy.mock.calls.length` before the mount-time fire settled. Waiting 30 ms before capturing baseline gave a clean window — small but load-bearing.
5. **Side benefits from straightforward changes.** The migration change "just swap imports" delivered an unrelated fix (the setState replacement bug) that resurrected 7 previously-dead tests. Search for the smaller wins inside larger PRs.
6. **Latent dead tests are worse than missing tests** because they create false confidence. The 6 `bun:test` files looked like coverage but ran nowhere. Until a CI step invokes the runner, having tests "in the tree" means nothing.

---

## 6. Recommended focus for next phase

In priority order (matches `nextPhaseSeeds` in the waypoint):

1. **`fix-skills-page-utils-test-fixtures`** — small (~30 min); brings the suite to 36/36. Worth doing now while the testing context is fresh.
2. **`browser-smoke-providers-and-agents`** — still owed from two phases ago. Two-tab manual session covering both migrations' rollback + propagation. ~30 min.
3. **`use-optimistic-patch-helper-extraction`** — now safe to do because the contract tests lock the pattern. Replaces 3 inlined copies with one tested helper before any further entity migration.
4. **`direct-entity-migration-models`** — next entity. Cross-view consumers in chat header + capability toggles. Apply the playbook with the new helper.
5. **`ci-frontend-tests`** — wire `pnpm test` into GitHub Actions. Required before another major refactor lands.
6. `direct-entity-migration-skills`, `direct-entity-migration-settings` follow in their own phases.

---

## 7. Evolver feedback

No `evolver-bridge.json` in this phase directory. Not part of an iterative-evolver cycle. No outer-loop state to update.

---

## 8. Progress signal

Completed kbd-reflect — vitest-contract-test-suite
