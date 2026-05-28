# Assessment — `vitest-contract-test-suite`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `direct-entity-migration-agents` (reflect_complete, 91% goal achievement)

---

## 1. Phase goal

Two direct-entity migrations (Providers, Agents) have now shipped with their rollback + cross-view propagation behaviours **unverified by automated tests**. The reflection on the Agent phase flagged this as the highest-priority follow-up before any further migrations (Models/Skills/Settings) land.

Goal: stand up a working unit-test runner with React + DOM support in `frontend/`, then author the **canonical regression tests** that lock the patterns shipped over the last two phases:

1. **"Two views, one SSE event, both re-render"** — proves the entity-graph propagates changes across mounted components.
2. **"Optimistic mutation rollback restores snapshot"** — proves the snapshot/upsert/rollback pattern in store-level (skill toggle, agent patch) and page-level (Provider set-default, Provider remove, Agent patch, Agent delete) mutations.
3. **"Bridge fires `load()` on graph change"** — proves the existing `useGraphBridge` helper, which the 8 still-bridged entities depend on.

Once these three contracts have tests, the next 3 entity migrations (Models, Skills, Settings) can each ship with their own targeted regressions.

---

## 2. Current state inventory

### 2.1 Test runner status

| Item | State |
|------|-------|
| `frontend/package.json` `scripts.test` | **absent** — no unit-test entry today |
| `frontend/vitest.config.ts` | **absent** — frontend root has no vitest config |
| `frontend/packages/prometheus-entity-management/vitest.config.ts` | present; the submodule has its own vitest setup |
| Vitest installed in `node_modules` | yes, transitively at `vitest@4.1.7` via the entity-mgmt submodule |
| Bun runtime | yes, kept available even after the pnpm migration |

### 2.2 Existing test files

| File | Runner |
|------|--------|
| `frontend/src/index.cursor-policy.test.ts` | `bun:test` (inferred from other test files; not opened) |
| `frontend/src/stores/chat-message-store.test.ts` | `bun:test` (verified: `import { beforeEach, describe, expect, test } from "bun:test"`) |
| `frontend/src/entities/runtime-ingest.test.ts` | likely `bun:test` |
| `frontend/src/features/chat/use-message-stream.test.ts` | likely `bun:test` |
| `frontend/src/features/chat/use-thread-naming.test.ts` | likely `bun:test` |
| `frontend/src/admin/pages/skills-page.utils.test.ts` | likely `bun:test` |

**Surprise:** 6 test files exist in the tree but **no npm script invokes them**. The existing `bun:test` suite has been sitting in the repo running only when a developer manually invokes `bun test`. There's no CI wiring, no documented developer workflow. This is its own debt.

### 2.3 What the contract tests need

| Capability | Provided by | Notes |
|------------|-------------|-------|
| Test runner | `vitest` or `bun:test` | Choice point — see §4 G1 |
| DOM emulation | `jsdom` or `happy-dom` | Neither currently installed at the frontend root |
| React render helpers | `@testing-library/react` | Not installed at the frontend root |
| User-event simulation | `@testing-library/user-event` | Same |
| `EventSource` mock | Hand-written (small) or `eventsource-polyfill` | Hand-written is ~30 LOC |
| Graph + bridge fixtures | Live entity-mgmt + bridge helper from this repo | No mocking needed — the library is already a workspace dep |
| `useGraphStore.getState()` reset between tests | Manual `useGraphStore.setState({ entities: {} })` in `beforeEach` | Pattern matches what `chat-message-store.test.ts` already does |

### 2.4 What the tests must prove

| Contract | Why it matters | Targets |
|----------|----------------|---------|
| Graph propagation | Two components reading the same entity must both re-render when an SSE update lands | regression for every `useEntity*` consumer |
| Optimistic rollback | Snapshot-based rollback must restore the prior state on server reject | regression for Provider set-default/remove, Agent patch/delete, Skill toggle |
| Bridge refetch | When the graph mutates, the bridge must call the underlying store's `load()` exactly once | regression for the 8 still-bridged entities (Knowledge/Memory/Compiler/Tools/MCP-Health/Models/Skills/Settings) |
| SSE adapter | `EventSource` `create|update|delete` events must produce graph operations of the right shape | regression for `uar-sse-adapter.ts` |

### 2.5 Existing patterns to honor

- `chat-message-store.test.ts` uses `useChatMessageStore.setState(...)` in `beforeEach` to reset. Repeat for `useGraphStore`.
- The entity-mgmt submodule's own vitest config uses `environment: "jsdom"` (based on its react peer dep). Adopt the same.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|-----------|--------------|
| A1 | `frontend/vitest.config.ts` exists with `environment: "jsdom"` (or `happy-dom`), React-aware setup, and globs covering `src/**/*.test.ts(x)`. | file present + `pnpm --filter ./frontend test` runs at least one test |
| A2 | `frontend/package.json` has a `test` script invoking vitest, plus `test:watch` and `test:ui` variants. | scripts present |
| A3 | `@testing-library/react`, `@testing-library/user-event`, `jsdom` (or `happy-dom`), and `@vitest/ui` added to `devDependencies`. | `pnpm install` succeeds |
| A4 | Existing `bun:test` files **still run** when invoked via `bun test`, OR are migrated to Vitest in a single sweep within this phase. | choice + verification |
| A5 | Contract test #1 — **graph propagation** — passes. Two `useEntity("Provider", "p1")` consumers, synthetic `upsertEntity` update, both rerender within one tick. | green |
| A6 | Contract test #2 — **optimistic rollback** — passes. Inject a reject into `setDefaultProvider`, verify graph reverts to snapshot. | green |
| A7 | Contract test #3 — **bridge refetch** — passes. Mock store with `load` spy, trigger `upsertEntity` on a watched type, assert `load` called exactly once. | green |
| A8 | Contract test #4 — **SSE adapter** — passes. Mock `EventSource`; emit `create`/`update`/`delete` events; assert `EntityChange` payloads of correct shape land on the adapter's handler. | green |
| A9 | CI integration documented (even if just a README snippet pointing at the new `pnpm test` command). | doc snippet |
| A10 | `docs/migration-stale-data-audit.md` notes that contract tests now back the patterns; subsequent direct-entity migrations should add per-entity tests alongside the migration. | doc updated |

---

## 4. Gap analysis

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| G1 | **Test runner choice: Vitest vs. `bun:test`.** The 6 existing tests use `bun:test`. Switching to Vitest means migrating them (or running both). Sticking with `bun:test` is simpler but conflicts with the reflection's call-out and with the submodule's vitest setup. | **High** | Decision point. Default per the reflection: **Vitest**, migrating the 6 existing files. |
| G2 | DOM environment not installed (`jsdom` vs `happy-dom`). | High | `happy-dom` is significantly faster; React 19 has known compatibility with both. Default: **happy-dom**. |
| G3 | `@testing-library/react` not installed. | High | Required for React component renders. |
| G4 | `EventSource` is browser-only; jsdom provides it, happy-dom does too. Adapter tests can use either env or a hand-written mock. | Low | Use a small in-file mock for the SSE adapter test; avoid env-specific behaviour. |
| G5 | Existing test files reference `bun:test` imports. Migration is mechanical: `import { … } from "bun:test"` → `import { … } from "vitest"`. | Med | Same `describe/test/expect/beforeEach` API surface; usually zero behaviour change. Audit each file for `mock()` vs `vi.fn()` differences. |
| G6 | `useGraphStore` is a Zustand store; tests must reset it between tests to avoid leakage. | Low | `beforeEach(() => useGraphStore.setState({ entities: {} }, true))` |
| G7 | No CI workflow currently runs frontend unit tests. | Med | Add a CI step in a follow-up phase; this phase just stands up the local runner. |
| G8 | Coverage tooling (`@vitest/coverage-v8`) — nice-to-have, not blocking. | Low | Skip for now; revisit when test count exceeds ~20. |
| G9 | The bridge helper's effect-driven dispatch is async; tests need `await act(...)` or `await waitFor(...)` to flush. | Med | Standard testing-library pattern; document in the first test. |
| G10 | Tests that mount React components need `<Suspense>` boundaries and the entity-mgmt provider tree. | Low | Submodule already exports a `<GraphProvider>`-equivalent; if not, manual `<></>` is fine because the graph is a module-singleton. |

---

## 5. Sequencing recommendation

1. **G1+G2+G3 — install runner + deps.** Add Vitest config + the 3 testing deps. Migrate the 6 existing files from `bun:test` → `vitest` in a single sweep.
2. **A5 — graph propagation test.** The simplest one; nails the core "no stale data anywhere" contract end-to-end.
3. **A6 — optimistic rollback test.** Forces explicit reasoning about snapshot/rollback semantics — easy to get wrong, useful to lock.
4. **A7 — bridge refetch test.** Regression for the 8 still-bridged entities.
5. **A8 — SSE adapter test.** EventSource mock + payload shape assertions.
6. **A9 — CI doc snippet** (no CI work in this phase; just point at the command).
7. **A10 — audit doc + reflection.**

Each test is independently runnable; ship them as one PR per contract.

---

## 6. Open questions for the user before planning

1. **Test runner: Vitest** (recommended; matches reflection + submodule) **or `bun:test`** (preserves existing files, simpler)?
2. **DOM environment: `happy-dom`** (faster) **or `jsdom`** (more battle-tested)?
3. **Migrate the 6 existing `bun:test` files in this phase** (clean slate) **or leave them under `bun test` and add Vitest alongside** (lower risk)?
4. **CI integration scope:** include CI workflow in this phase, or punt to a separate one? (Recommendation: punt — keep this phase focused on local-runnable contracts.)

---

## 7. Progress signal

Completed kbd-assess — vitest-contract-test-suite
