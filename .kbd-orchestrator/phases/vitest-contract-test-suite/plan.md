# Plan — `vitest-contract-test-suite`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec (detected at `openspec/`)
**Assessment input:** `.kbd-orchestrator/phases/vitest-contract-test-suite/assessment.md`

---

## Decisions locked (defaults applied)

| Q | Answer |
|---|--------|
| Q1 — runner | **Vitest** — matches reflection guidance and aligns with the entity-mgmt submodule's setup |
| Q2 — DOM env | **happy-dom** — faster, sufficient for the React + EventSource surface we test |
| Q3 — existing tests | **Migrate the 6 existing `bun:test` files to Vitest in this phase** — clean slate; preserves running behaviour by switching the import line |
| Q4 — CI integration | **Punt** — keep this phase focused on a working local runner + the 4 canonical contracts; CI wiring lands in a separate phase |

---

## Ordered change list (6 changes)

| # | Change ID | Title | Depends on |
|---|-----------|-------|------------|
| 1 | `vitest-runner-stand-up` | Add `vitest.config.ts`, install testing deps, wire `pnpm test` script | — |
| 2 | `migrate-existing-bun-tests-to-vitest` | Swap `bun:test` imports → `vitest` across the 6 existing test files | 1 |
| 3 | `contract-graph-propagation` | First contract test: two `useEntity` consumers + synthetic graph update → both re-render | 1 |
| 4 | `contract-optimistic-rollback` | Inject service reject → assert graph reverts to snapshot for the Provider `setDefault` path | 1 |
| 5 | `contract-bridge-refetch` | Mock a store with a `load` spy; trigger graph mutation; assert exactly one refetch call | 1 |
| 6 | `contract-sse-adapter` | Mock `EventSource`; emit `create|update|delete`; assert `EntityChange` payload shape | 1 |

Changes 3-6 are independent and could land in any order once #1 ships. #2 is also independent of 3-6 but should land before merging so CI runs all tests.

---

## Per-change synopsis

### 1. `vitest-runner-stand-up`
- Author `frontend/vitest.config.ts` with:
  ```ts
  import { defineConfig } from "vitest/config";
  import react from "@vitejs/plugin-react";
  import path from "node:path";

  export default defineConfig({
    plugins: [react()],
    test: {
      environment: "happy-dom",
      globals: true,
      setupFiles: ["./src/test/setup.ts"],
      include: ["src/**/*.test.{ts,tsx}"],
      exclude: ["**/node_modules/**", "**/packages/**"],
    },
    resolve: {
      alias: { "@": path.resolve(__dirname, "./src") },
    },
  });
  ```
- Create `frontend/src/test/setup.ts`:
  ```ts
  import "@testing-library/jest-dom/vitest";
  import { afterEach, beforeEach } from "vitest";
  import { cleanup } from "@testing-library/react";
  import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

  beforeEach(() => {
    useGraphStore.setState({ entities: {} } as never, true);
  });
  afterEach(() => cleanup());
  ```
- Add devDependencies (via `pnpm add -D --filter ./frontend ...`):
  - `vitest` (pin to whatever the submodule uses — `4.1.7`)
  - `@vitest/ui`
  - `@vitejs/plugin-react`
  - `@testing-library/react`
  - `@testing-library/user-event`
  - `@testing-library/jest-dom`
  - `happy-dom`
- Add scripts in `frontend/package.json`:
  ```json
  "test": "vitest run",
  "test:watch": "vitest",
  "test:ui": "vitest --ui"
  ```
- Acceptance: `pnpm --filter ./frontend test` reports "0 tests" cleanly (no errors).

### 2. `migrate-existing-bun-tests-to-vitest`
Files to touch (one-line import swap each):
- `src/index.cursor-policy.test.ts`
- `src/stores/chat-message-store.test.ts`
- `src/entities/runtime-ingest.test.ts`
- `src/features/chat/use-message-stream.test.ts`
- `src/features/chat/use-thread-naming.test.ts`
- `src/admin/pages/skills-page.utils.test.ts`

Per file:
```diff
- import { beforeEach, describe, expect, test } from "bun:test";
+ import { beforeEach, describe, expect, test } from "vitest";
```

Audit each for `mock(...)` / `spyOn(...)` calls — `bun:test` and `vitest` differ slightly; remap to `vi.fn()` / `vi.spyOn(...)` if found.

Acceptance: `pnpm --filter ./frontend test` runs all 6 tests; each passes (no behaviour change expected).

### 3. `contract-graph-propagation`
- New `src/lib/realtime/__tests__/graph-propagation.test.tsx`:
  ```tsx
  it("propagates a graph upsert to multiple consumers in one tick", async () => {
    function Reader({ id }: { id: string }) {
      const v = useGraphStore((s) => s.entities["Provider"]?.[id]);
      return <span data-testid={`p-${id}`}>{(v as { display_name?: string } | undefined)?.display_name ?? "—"}</span>;
    }
    render(<><Reader id="p1" /><Reader id="p1" /></>);
    act(() => {
      useGraphStore.getState().upsertEntity("Provider", "p1", { id: "p1", display_name: "Alpha" });
    });
    const matches = screen.getAllByTestId("p-p1");
    expect(matches).toHaveLength(2);
    for (const el of matches) expect(el.textContent).toBe("Alpha");
  });
  ```
- Acceptance: green; deliberately fails if `useGraphStore` subscription semantics regress.

### 4. `contract-optimistic-rollback`
- New `src/admin/pages/__tests__/providers-set-default-rollback.test.tsx`:
  - Mock `services/providers-api::setDefaultProvider` to throw.
  - Seed `ProviderMeta:current` with `default_id: "p1"`.
  - Render a tiny harness that calls the page's `setDefault("p2")` logic (extracted to a helper if needed for testability).
  - Assert `useGraphStore.getState().entities["ProviderMeta"]["current"].default_id === "p1"` after the rejection settles.
- Acceptance: green; deliberately fails if the rollback path is removed.

### 5. `contract-bridge-refetch`
- New `src/lib/realtime/__tests__/use-graph-bridge.test.tsx`:
  - Render a component that calls `useGraphBridge(["Provider"], load)` where `load` is a `vi.fn()`.
  - Call `useGraphStore.getState().upsertEntity("Provider", "p1", {...})`.
  - `await waitFor(() => expect(load).toHaveBeenCalledTimes(1));`
  - Also assert no spurious calls when an *unrelated* type is mutated.
- Acceptance: green; locks the bridge contract for the 8 still-bridged entities.

### 6. `contract-sse-adapter`
- New `src/lib/realtime/__tests__/uar-sse-adapter.test.ts`:
  - Replace global `EventSource` with a controllable mock (`vi.stubGlobal("EventSource", FakeES)`).
  - Build a `createUarSseAdapter({ topic: "providers", entityType: "Provider" })`.
  - `subscribe(handler)`; trigger `fakeES.dispatch("create", { topic: "providers", id: "p1", data: { id: "p1" } })`.
  - Assert handler received `{ changes: [{ op: "insert", type: "Provider", id: "p1", data: { id: "p1" } }], ... }`.
  - Repeat for `update` (→ `op: "update"`) and `delete` (→ `op: "delete"`).
- Acceptance: green; locks the SSE → graph payload contract for all 10 topics.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Vitest 4.x version mismatch with submodule's installed vitest | Pin to the exact submodule version (`4.1.7`); document in `frontend/package.json` |
| `bun:test` → `vitest` migration breaks an existing test due to API drift (e.g. `mock()` vs `vi.fn()`) | Audit each file individually in change 2; revert per-file if needed |
| `happy-dom` missing some DOM API used incidentally by a transitive dep | Switch to `jsdom` per-test via `// @vitest-environment jsdom` directive if it bites |
| `useGraphStore` reset in `beforeEach` interferes with module-level singletons elsewhere | Make the reset narrow: only `entities` slice, preserve other fields |
| `EventSource` mock diverges from real-browser semantics | Restrict the adapter test to the parsing path; integration validation stays manual |
| The page-level `setDefault` logic is currently inline in the component — hard to unit-test | Extract to a small testable helper as part of change 4 (or test through the rendered component with `userEvent`) |
| Tests slow down `cargo build` because `build.rs` triggers frontend build (which doesn't run tests) | n/a — tests are invoked separately via `pnpm test`, not by build.rs |

---

## Acceptance gate before phase reflect

1. `pnpm --filter ./frontend test` exits 0 with all migrated + new tests passing.
2. The 4 contract tests fail deliberately when their respective patterns are broken (verified by toggling a line and re-running).
3. README mentions `pnpm --filter ./frontend test` as the unit-test entry point.

---

## Progress signal

Completed kbd-plan — vitest-contract-test-suite
