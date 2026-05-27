# Plan — `use-optimistic-patch-helper-extraction`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec (detected at `openspec/`)
**Assessment input:** `.kbd-orchestrator/phases/use-optimistic-patch-helper-extraction/assessment.md`

---

## Decisions locked (defaults applied)

| Q | Answer |
|---|--------|
| Q1 — module name | **`frontend/src/lib/realtime/optimistic.ts`** — matches sibling realtime modules |
| Q2 — error contract | **Re-throw** on rejection; callers wrap. Matches the test's existing helper shape. |
| Q3 — singleton specialization | **Generic `optimisticUpsert`** — no dedicated `optimisticSetField`; the patch shape carries field info implicitly |
| Q4 — post-success refetch | **None** — rely on SSE reconciliation, same as today |

---

## Ordered change list (3 changes)

| # | Change ID | Title | Notes |
|---|-----------|-------|-------|
| 1 | `add-optimistic-helpers-module` | Author `frontend/src/lib/realtime/optimistic.ts` with `optimisticUpsert` + `optimisticRemove`; switch the existing contract test to import from it | additive — no call-site changes |
| 2 | `migrate-providers-page-to-helpers` | Replace inline `setDefault` + `removeProvider` patches in `providers-page.tsx` with helper calls; wrap with try/catch for local error state | 2 call sites |
| 3 | `migrate-agents-page-to-helpers` | Replace inline `patchAgentOptimistic` (delete the local helper) + `handleDelete` snapshot/rollback in `agents-page.tsx` | 2 call sites |

Each change runs `pnpm test` to confirm 36/36 stays green.

---

## Per-change synopsis

### 1. `add-optimistic-helpers-module`

New file `frontend/src/lib/realtime/optimistic.ts`:

```ts
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

/** Snapshot, optimistic shallow-merge upsert, rollback on rejection. */
export async function optimisticUpsert<T extends Record<string, unknown>>(
  type: string, id: string, patch: Partial<T>,
  serverCall: () => Promise<void>,
): Promise<void> {
  const graph = useGraphStore.getState();
  const snapshot = graph.entities[type]?.[id] as Record<string, unknown> | undefined;
  if (snapshot) {
    graph.upsertEntity(type, id, { ...snapshot, ...patch });
  }
  try {
    await serverCall();
  } catch (err) {
    if (snapshot) {
      useGraphStore.getState().upsertEntity(type, id, snapshot);
    }
    throw err;
  }
}

/** Snapshot, optimistic remove, re-upsert on rejection. */
export async function optimisticRemove(
  type: string, id: string,
  serverCall: () => Promise<void>,
): Promise<void> {
  const graph = useGraphStore.getState();
  const snapshot = graph.entities[type]?.[id] as Record<string, unknown> | undefined;
  graph.removeEntity(type, id);
  try {
    await serverCall();
  } catch (err) {
    if (snapshot) {
      useGraphStore.getState().upsertEntity(type, id, snapshot);
    }
    throw err;
  }
}
```

Update `frontend/src/lib/realtime/__tests__/optimistic-rollback.test.tsx` to:

- Remove the inline `optimisticUpsert` / `optimisticRemove` definitions.
- `import { optimisticUpsert, optimisticRemove } from "../optimistic";`

Acceptance: `pnpm --filter ./frontend test` → 36/36 green; bundle still builds clean.

### 2. `migrate-providers-page-to-helpers`

**`setDefault`** (lines 96–111). Today: 16 lines of snapshot/upsert/rollback for the ProviderMeta singleton. After:

```ts
const setDefault = async (id: string) => {
  try {
    await optimisticUpsert(
      "ProviderMeta", "current",
      { id: "current", default_id: id },
      () => setDefaultProviderApi(id),
    );
  } catch (e) {
    setError(`Failed to set default: ${(e as Error).message}`);
  }
};
```

**`removeProvider`** (lines 113–132). After:

```ts
const removeProvider = async (id: string) => {
  setRemoving(id);
  setError(null);
  try {
    await optimisticRemove("Provider", id, () => deleteProviderApi(id));
  } catch (e) {
    setError(`Failed to remove provider: ${(e as Error).message}`);
  } finally {
    setRemoving(null);
  }
};
```

Drop the now-unused `useGraphStore` + `ProviderEntity` imports from providers-page if nothing else references them.

Acceptance: page renders identically; tests stay green.

### 3. `migrate-agents-page-to-helpers`

**`patchAgentOptimistic`** (lines 79–105). Delete the local helper entirely; replace its callers with direct `optimisticUpsert` calls. Currently `AgentMemorySection.save` does:

```ts
await patchAgentOptimistic(agent.id, body);
```

After:

```ts
await optimisticUpsert("Agent", agent.id, body, () => patchAgentApi(agent.id, body));
```

**`handleDelete`** (lines 278–301). Today wraps the `deleteAgent` response check + optimistic remove + rollback inline. After:

```ts
const handleDelete = async () => {
  if (!deleteTarget) return;
  setDeleting(true);
  setDeleteError(null);
  try {
    await optimisticRemove("Agent", deleteTarget.id, async () => {
      const res = await deleteAgent(deleteTarget.id);
      if (!res.ok) throw new Error((await res.text()) || `${res.status}`);
    });
    if (selected?.id === deleteTarget.id) setSelected(null);
    setDeleteTarget(null);
  } catch (e) {
    setDeleteError((e as Error).message);
  } finally {
    setDeleting(false);
  }
};
```

The `Response.ok` check moves inside the `serverCall` closure so the helper sees a thrown error and rolls back as designed.

Drop the now-unused `useGraphStore` import if nothing else references it.

Acceptance: page renders identically; tests stay green; A1 + A2 smoke (when run) still pass.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Helper module's generic types don't compose with TS strict mode for `ProviderEntity` / `UarAgent` shapes | The helpers use `Record<string, unknown>` internally; call sites cast at the call boundary. The test already uses these signatures and compiles. |
| Optimistic upsert's "skip if no snapshot" branch hides creates | All 4 call sites have a guaranteed pre-existing entity (provider exists before set-default; agent exists before patch/delete; ProviderMeta:current exists from `loadProvidersIntoGraph` on mount). If a new call site needs create-optimism, add a separate helper. |
| Test losing the inline helpers means the test now imports the production module; if production has a bug, the test stays green if it mirrors the same bug | Acceptable — production = test source of truth is exactly the goal; bugs surface in the smoke scenario sweep. |
| Wrapping `deleteAgent` response check inside the helper's `serverCall` changes error semantics slightly (errors thrown from inside the closure travel through the helper's rollback path before reaching the caller) | This is desirable. The helper exists precisely to centralize that pattern. |

---

## Acceptance gate before phase reflect

1. `pnpm --filter ./frontend test` → 36/36 green.
2. `pnpm --filter ./frontend build` clean.
3. `git grep -nE "useGraphStore.getState\\(\\)" frontend/src/admin/pages` returns 0 results (or only references in legitimate non-optimistic paths).
4. Diff `git diff --stat` shows net LOC reduction.

---

## Progress signal

Completed kbd-plan — use-optimistic-patch-helper-extraction
