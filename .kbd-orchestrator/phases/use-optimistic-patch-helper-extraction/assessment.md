# Assessment — `use-optimistic-patch-helper-extraction`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `browser-smoke-providers-and-agents` (reflect_partial; manual walkthrough still owed but tracked separately)

---

## 1. Phase goal

Extract the four inlined snapshot/upsert/rollback copies in providers-page.tsx and agents-page.tsx into a single reusable module at `frontend/src/lib/realtime/optimistic.ts`. Replace each inline copy with a one-line call to the helper. Wire the existing contract test (`optimistic-rollback.test.tsx`) to import from the extracted module so the test becomes the canonical regression for the helper, not a parallel inline duplicate.

This phase was queued as a follow-up to the contract-test suite — those tests verified the pattern works, and three near-identical copies have accumulated. Three repetitions = right time to extract (per the lesson captured in the agents-phase reflection).

---

## 2. Current state inventory

### 2.1 Inline call sites (4 total)

| File | Function | Lines | Kind |
|------|----------|-------|------|
| `admin/pages/providers-page.tsx` | `setDefault(id)` | 96–111 | singleton patch on `ProviderMeta:current` (`default_id` field) |
| `admin/pages/providers-page.tsx` | `removeProvider(id)` | 113–132 | snapshot + remove + rollback on `Provider:<id>` |
| `admin/pages/agents-page.tsx` | `patchAgentOptimistic` (already extracted within the file) | 81–104 | shallow merge on `Agent:<id>` |
| `admin/pages/agents-page.tsx` | `handleDelete` body | 278–301 | snapshot + remove + rollback on `Agent:<id>` |

### 2.2 Existing test scaffolding

`frontend/src/lib/realtime/__tests__/optimistic-rollback.test.tsx` **already defines** the exact helpers we want to extract — they live inline in the test as two local functions:

- `optimisticUpsert<T>(type, id, patch, serverCall)` — snapshot → upsert merged → rollback on throw.
- `optimisticRemove(type, id, serverCall)` — snapshot → remove → rollback on throw.

Both **re-throw** on rejection (caller decides to swallow/display).

The test file's helpers are byte-for-byte the contract we want to lock. Extracting to `optimistic.ts` is therefore "move the test's helpers to production, and update the test to import them" — a clean swap that preserves the test signature and behavioural guarantees.

### 2.3 Error-handling inconsistency across the 4 call sites

| Call site | Error handling today |
|-----------|---------------------|
| `setDefault` | **Swallows** — sets local `error` state, no rethrow |
| `removeProvider` | **Swallows** — sets local `error` state, no rethrow |
| `patchAgentOptimistic` | **Re-throws** — caller's `try/catch` sets error state |
| `handleDelete` (agent) | **Catches** — sets `setDeleteError` |

Two patterns. The extracted helper's contract (re-throw) matches the test and aligns with cleaner separation: helper handles rollback, caller handles UI. The two swallowing call sites need a tiny `try { await optimisticX(...) } catch (e) { setError(...) }` wrapper added — net same LOC.

### 2.4 Naming + module shape

Plain functions (not hooks — they're called from event handlers, not render):

```ts
// frontend/src/lib/realtime/optimistic.ts
export async function optimisticUpsert<T extends Record<string, unknown>>(
  type: string, id: string, patch: Partial<T>,
  serverCall: () => Promise<void>,
): Promise<void> { /* ... */ }

export async function optimisticRemove(
  type: string, id: string,
  serverCall: () => Promise<void>,
): Promise<void> { /* ... */ }
```

The `ProviderMeta:current` singleton case works with `optimisticUpsert` because the singleton row exists before the patch (`loadProvidersIntoGraph` upserts it on page mount); the patch shallow-merges `default_id`.

### 2.5 Special-cases worth noting

- **Singleton patch (`setDefault`)** — the inline version captures `previousDefault` (the value of `default_id`) and only rolls back that field. The helper-based version captures the **entire entity** snapshot and rolls back the entire entity. Behaviour is equivalent because the singleton only has `id` + `default_id` fields. No regression.
- **`removeProvider`** — captures snapshot **before** the optimistic delete. The extracted helper does the same. Direct match.
- **`patchAgentOptimistic`** — already structured exactly like the helper; replacing it removes the local function definition entirely.
- **`handleDelete` (agent)** — same pattern as `removeProvider`. Direct match.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|-----------|--------------|
| A1 | `frontend/src/lib/realtime/optimistic.ts` exists exporting `optimisticUpsert` + `optimisticRemove` with the exact signatures from the contract test | file present |
| A2 | `providers-page.tsx::setDefault` calls `optimisticUpsert("ProviderMeta", "current", { default_id: id }, () => setDefaultProviderApi(id))` and handles the throw locally | grep + manual review |
| A3 | `providers-page.tsx::removeProvider` calls `optimisticRemove("Provider", id, () => deleteProviderApi(id))` | grep |
| A4 | `agents-page.tsx::patchAgentOptimistic` removed; call site (`AgentMemorySection.save`) calls `optimisticUpsert("Agent", agent.id, body, () => patchAgentApi(...))` directly | file diff |
| A5 | `agents-page.tsx::handleDelete` calls `optimisticRemove("Agent", id, () => deleteAgent(id))` (with `deleteAgent`'s `Response.ok` check still wrapped) | file diff |
| A6 | `optimistic-rollback.test.tsx` imports from `@/lib/realtime/optimistic` instead of inline-defined helpers; **5/5 tests still green** | `pnpm test` |
| A7 | `pnpm --filter ./frontend build` clean | output |
| A8 | `pnpm --filter ./frontend test` reports **36/36 passing** (no regression) | output |
| A9 | Net LOC delta is **negative** (~30 LOC deduplicated minus the new module's ~40 LOC = roughly neutral to mildly positive; the cleanup is structural, not size-driven) | `git diff --stat` |

---

## 4. Gap analysis

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| G1 | Helper module doesn't exist yet | High | Trivial author from the test's existing copy. |
| G2 | `setDefault` and `removeProvider` swallow errors; helper rethrows | Low | Wrap each with `try/catch` at the call site; ~3 lines added per site. |
| G3 | The `Response.ok` check in `handleDelete` (lines 282–286) sits between optimistic remove and the rollback. If we wrap the whole `deleteAgent`+`!res.ok throw` in the helper's `serverCall`, the helper sees the throw and rolls back as designed. | Low | Clean wrap. |
| G4 | The singleton-patch rollback in `setDefault` only restores `default_id`, not the full singleton (because `previousDefault` is a string). After extraction, the rollback restores the full snapshot — same result, slightly more state copied. | Trivial | No regression. |
| G5 | If we later add a third kind of helper (e.g. `optimisticInsert` for full-create cases), the module needs to grow. Out of scope today. | n/a | Future. |
| G6 | Test imports inline helpers; updating to import from `@/lib/realtime/optimistic` is a one-line change but the test should also delete the inline copies. | Low | One file edit. |

---

## 5. Sequencing recommendation

Single PR-equivalent change, but with internal ordering for safety:

1. **Author the module** `frontend/src/lib/realtime/optimistic.ts` — copy the test's helpers verbatim, add brief docstrings.
2. **Switch the test** to import from the new module; delete the inline definitions. Run `pnpm test` — confirm 5/5 still green.
3. **Replace inline call sites one at a time**, running `pnpm test` between each to catch any contract drift:
   1. `patchAgentOptimistic` → simplest (already a helper-shaped local function).
   2. `agents-page handleDelete` → snapshot+remove pattern, direct match.
   3. `providers-page removeProvider` → direct match.
   4. `providers-page setDefault` → singleton patch, slightly different shape but equivalent.
4. **Final sweep** — `git grep "useGraphStore.getState().*upsertEntity"` outside the helper module should drop significantly (any remaining occurrences are legitimate non-optimistic upserts).

---

## 6. Open questions for the user before planning

1. **Module name** — `optimistic.ts` (recommended; matches existing realtime-module siblings) or `use-optimistic-patch.ts` (matches the original phase name despite not being a hook)?
2. **Error contract** — helper re-throws (matches test, callers wrap) — confirm OK?
3. **Should the singleton case use a dedicated helper** (e.g. `optimisticSetField(type, id, field, value, serverCall)`) so the call site reads `optimisticSetField("ProviderMeta", "current", "default_id", id, ...)` instead of the more generic `optimisticUpsert`? Cosmetic; not necessary.
4. **Should `loadProvidersIntoGraph()` be called after `setDefault` succeeds** (as a belt-and-suspenders consistency check against SSE), or rely on the SSE bus to reconcile? Today neither is done explicitly — the optimistic patch sticks unless server rejects. Recommend leaving as-is.

---

## 7. Progress signal

Completed kbd-assess — use-optimistic-patch-helper-extraction
