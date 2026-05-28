# Assessment — `settings-store-retirement`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `direct-entity-migration-models` (70%, reflect_complete) — left settings reads on graph but writes still on `settings-store.ts`. This phase closes that out.

---

## 1. Phase goal

Retire `frontend/src/stores/settings-store.ts` (242 LOC) and migrate every write path to the direct pattern: `setSetting` becomes a local optimistic graph upsert (with a `dirty` marker), `saveAll` becomes a direct REST call wrapped in optimistic helpers, and the SSE-fed graph naturally reconciles. The change-bus stays in place as a thin facade that subscribes to graph mutations and emits the existing `uar:settings-changed` events for non-page consumers (`chat-stream-store`, `provider-models-store`).

End-state assertions:
- `frontend/src/stores/settings-store.ts` deleted
- `git grep "useSettingsStore" frontend/` empty
- `useSettings(namespace)` hook surface unchanged from the caller's POV; the implementation now owns `dirty`/`conflicts`/`saving` in component-scoped state plus a per-namespace module cache
- `settings-page.tsx` compiles unchanged (no call-site rewrites needed)
- 36/36 tests + clean build maintained

The `settings-types-meta-store.ts` is **out of scope** — it caches field-schema metadata (not per-row entities) and stays as a one-shot REST cache.

---

## 2. Current state inventory

### 2.1 Files in scope

| File | LOC | Role this phase |
|------|----:|-----------------|
| `frontend/src/stores/settings-store.ts` | 242 | retire entirely |
| `frontend/src/services/settings-change-bus.ts` | 89 | keep, but turn its source from `emitSettingsChanged` into a graph subscriber |
| `frontend/src/hooks/use-settings.ts` | ~95 | rewrite — owns `dirty`/`conflicts`/`saving` locally; writes via optimistic helpers |
| `frontend/src/admin/pages/settings-page.tsx` | 3334 | **no call-site rewrites**; hook contract unchanged |
| `frontend/src/entities/sync.ts` | (line 292) | currently calls `emitSettingsChanged` on incoming SSE — keep, this is the new emission point |
| `frontend/src/stores/chat-stream-store.ts` | (line 256) | non-page listener — keep `onSettingsChanged` |
| `frontend/src/stores/provider-models-store.ts` | (line 45) | non-page listener — keep `onSettingsChanged` |
| `frontend/src/main.tsx` | — | calls `initSettingsRealtimeBridge()` — remove |

### 2.2 Things `settings-store.ts` currently does

1. **Per-namespace caching** — keyed by `namespace` (e.g. `provider`, `agent_config`). Holds `settings`, `values`, `dirty`, `conflicts`, `loading`, `saving`, `error`.
2. **`load(namespace)`** — REST fetch + populate the slice.
3. **`setSetting(ns, key, val)`** — mark dirty.
4. **`saveAll(ns)`** — POST the dirty payload; on success update values + emit change-bus events.
5. **`applyRemoteSetting(row)`** — SSE-driven update; respects in-flight `dirty` to flag a `conflict`.
6. **`initSettingsRealtimeBridge()`** — bus listener that calls `applyRemoteSetting` on remote events.

After retirement (proposed):

| Responsibility | New home |
|---|---|
| Per-namespace read cache | entity graph (`Setting:<namespace>:<key>` rows) |
| `load(namespace)` | `loadSettingsIntoGraph(namespace)` (already exists in `entities/fetchers/settings.ts`) |
| `setSetting(ns, key, val)` | local `useReducer` inside the new `useSettings` hook; OR optimistic upsert on the graph with a `_dirty` marker the consumer reads |
| `saveAll(ns)` | direct `putSettingsNamespace(ns, payload)` wrapped in optimistic per-key upserts |
| `applyRemoteSetting` | already happens via the SSE adapter → graph; the change bus now emits on graph mutations |
| `initSettingsRealtimeBridge` | deleted; entity graph IS the realtime bridge |

### 2.3 28 call sites in `settings-page.tsx`

All go through `useSettings(namespace)`. The hook's return shape MUST stay identical:

```ts
interface UseSettingsReturn {
  values: Record<string, unknown>;
  settings: Record<string, SettingWithMeta>;
  dirty: Record<string, unknown>;
  conflicts: Record<string, unknown>;
  loading: boolean;
  saving: boolean;
  error: string | null;
  setSetting: (key: string, value: unknown) => void;
  saveAll: () => Promise<void>;
  reload: () => Promise<void>;
}
```

Keeping the contract means the 3334 LOC page needs **zero** changes — the entire diff lives in `use-settings.ts` + deleted `settings-store.ts` + restructured `settings-change-bus.ts`.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|---|---|
| C1 | `frontend/src/stores/settings-store.ts` deleted | `git rm`; `git grep useSettingsStore frontend/` empty |
| C2 | `useSettings()` contract unchanged | `settings-page.tsx` builds without modification |
| C3 | `dirty` / `conflicts` / `saving` / `error` work per-namespace; survive component remounts within a session | manual: edit, navigate away, come back — dirty values preserved |
| C4 | `saveAll` posts the dirty payload; on success values reflect server response | manual: save, refresh, value persists |
| C5 | SSE event arrives mid-edit → `conflicts` flagged on the dirty key, `values` unchanged for that key | manual: edit field A in tab1; remote update for field A from tab2; tab1 shows conflict marker |
| C6 | Non-page listeners (`chat-stream-store`, `provider-models-store`) still receive `uar:settings-changed` events | trace + manual smoke |
| C7 | `initSettingsRealtimeBridge` removed from `main.tsx`; no double-subscription | grep |
| C8 | `pnpm --filter ./frontend test` → ≥36/36 | output |
| C9 | `pnpm --filter ./frontend build` clean | output |
| C10 | Audit doc row for `Setting` flipped from `transitional-direct` → `direct` | file diff |
| C11 | Net LOC delta negative — ~242 (store) − ~80 (new hook code beyond change 10's version) ≈ −160 | `git diff --shortstat` |

---

## 4. Gap analysis

### 4.1 Dirty state placement

The dirty state is currently per-namespace global (Zustand). If the user opens namespace A, edits, navigates to namespace B, comes back to A — they expect dirty values preserved. Module-level cache in the hook works but is less safe. **Recommendation:** keep a small module-level `Map<namespace, DirtyState>` inside the new `use-settings.ts`, gated behind a `useSyncExternalStore`-style subscription. ~30 LOC.

### 4.2 Conflict tracking

The current store sets `conflicts[key] = remoteValue` when a remote arrives while local is dirty. After retirement the graph row holds the remote value (always authoritative); the hook compares `dirtyValue ≠ graphValue` and synthesises `conflicts` from that diff. No state needed — derived.

### 4.3 Change-bus consumers

`chat-stream-store` listens for `impact === "llm"` to update retry policies. `provider-models-store` listens for provider changes to refetch model lists. Both need keep getting events. Two paths:

- **A. Keep the bus, change its emission source.** Today `saveAll` emits; tomorrow a graph subscriber emits when `Setting:*` rows mutate. The bus interface (`onSettingsChanged(detail)`) stays identical. Cleanest.
- **B. Migrate the two consumers to subscribe to the graph directly.** More work; broader blast radius.

**Recommendation:** A.

### 4.4 `entities/sync.ts:292` already emits the bus

When the SSE adapter ingests a Setting upsert, it calls `emitSettingsChanged(...)`. That stays — it's exactly the right place. We just remove the `applyRemoteSetting` call inside the store (no store, no method).

### 4.5 Test coverage

No vitest covers the dirty/save flow today (the settings page is too big for contract tests at this resolution). The migration is structural; behaviour should be identical. **Recommendation:** add one small contract test in `frontend/src/hooks/__tests__/use-settings.test.tsx` that:
1. Sets a value → asserts dirty marker
2. Calls saveAll → asserts dirty cleared
3. Simulates a remote SSE upsert while dirty → asserts conflict surfaced

This guards future regressions on the most fragile part of the migration.

### 4.6 Risk areas

- The current store's `dirty[key]` includes raw values, but `saveAll` strips the `${namespace}.` prefix before posting. The new implementation must preserve that exact API contract.
- `conflicts` is currently a `Record<string, unknown>` of remote values — page UI may use that to render a "remote changed" indicator. Confirm no consumer treats it as a count or array.

---

## 5. Sequencing recommendation

4 changes, ordered:

1. **`add-settings-form-cache`** — new module-level dirty cache helper in `frontend/src/hooks/settings-form-cache.ts`. Pure utility; no consumer changes yet.
2. **`rewrite-use-settings-hook`** — `use-settings.ts` rewrites to read graph + use the form cache + post directly via `putSettingsNamespace`. Contract unchanged. Tests added.
3. **`retire-settings-store-and-bus-source`** — delete `settings-store.ts`. Restructure `settings-change-bus.ts` so its emit point is the SSE adapter (already true) and remove `initSettingsRealtimeBridge`. Update `main.tsx` to drop the init call.
4. **`audit-doc-and-grep-gate`** — flip the `Setting` audit row to `direct`. Confirm `useGraphStore.getState` grep in admin pages stays at the existing baseline (memory bulk-delete only).

Each change runs the test + build gate.

---

## 6. Open questions for the user

1. **Add a vitest contract test?** Author one test for the new `useSettings` covering dirty → save → SSE conflict (per §4.5)? Adds ~3 contract tests; protects the only PARTIAL → DIRECT transition in the project.
2. **Keep `settings-types-meta-store.ts`?** Confirmed in plan — it's a field-schema cache, not entity data. Should stay.
3. **Bulk save semantics** — current behaviour POSTs all dirty keys in one request; the new optimistic pattern would prefer per-key calls so each rolls back independently. Single-bulk-call is faster but rollback granularity is coarser. **Recommendation:** preserve bulk-call (existing UX) and on failure restore ALL dirty keys to graph-state. Simpler.
4. **Conflict UI signal** — keep the existing `conflicts` map shape, or switch to a per-key `{ remote, local }` tuple? Plan default: preserve the existing shape to keep the page-rendering code intact.

---

## 7. Progress signal

Assessment complete. Defaults sufficient unless you want to override §6. Next: `/kbd-plan settings-store-retirement`.
