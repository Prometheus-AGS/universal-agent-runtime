# Plan — `settings-store-retirement`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec
**Decisions:** defaults locked from assessment §6 (vitest test ✅; settings-types-meta untouched; bulk POST preserved; conflict shape preserved)

---

## Ordered change list (4)

| # | Change ID | Title | Effort |
|---|---|---|---|
| 1 | `add-settings-form-cache` | Author module-level `Map<namespace, DirtyState>` helper in `frontend/src/hooks/settings-form-cache.ts` with `useSyncExternalStore` subscription | S |
| 2 | `rewrite-use-settings-hook` | Rewrite `use-settings.ts` to read from graph, mutate via form cache + direct `putSettingsNamespace`, preserve contract. Author contract test. | M |
| 3 | `retire-settings-store-and-bus-source` | `git rm stores/settings-store.ts`; restructure change-bus (emit point = SSE adapter, already true); remove `initSettingsRealtimeBridge` from `main.tsx` | S |
| 4 | `audit-doc-flip-settings-direct` | Flip `Setting` row to `direct`; document the form-cache pattern in the audit's playbook section | XS |

Each change runs `pnpm --filter ./frontend test` (≥36/36) and `pnpm --filter ./frontend build` clean.

---

## Per-change synopsis

### 1. `add-settings-form-cache`

```ts
// frontend/src/hooks/settings-form-cache.ts
type DirtyState = { values: Record<string, unknown>; saving: boolean; error: string | null };
const cache = new Map<string, DirtyState>();
const listeners = new Map<string, Set<() => void>>();

export function getDirty(ns: string): DirtyState { … }
export function setDirty(ns: string, key: string, value: unknown): void { … }
export function clearDirty(ns: string): void { … }
export function setSaving(ns: string, saving: boolean, error?: string | null): void { … }
export function subscribe(ns: string, cb: () => void): () => void { … }
```

### 2. `rewrite-use-settings-hook`

```ts
export function useSettings(namespace: string): UseSettingsReturn {
  // Reads from the entity graph
  const graphView = useSettingsEntity(namespace);

  // Dirty/saving/error from the module cache (per-namespace, survives re-mount)
  const dirty = useSyncExternalStore(
    (cb) => subscribe(namespace, cb),
    () => getDirty(namespace),
  );

  useEffect(() => { void loadSettingsIntoGraph(namespace); }, [namespace]);

  const conflicts = useMemo(() => {
    const out: Record<string, unknown> = {};
    for (const [k, dv] of Object.entries(dirty.values)) {
      const remote = graphView.values[k];
      if (remote !== undefined && !Object.is(remote, dv)) out[k] = remote;
    }
    return out;
  }, [dirty.values, graphView.values]);

  const setSetting = useCallback((key: string, val: unknown) => setDirty(namespace, key, val), [namespace]);

  const saveAll = useCallback(async () => {
    if (Object.keys(dirty.values).length === 0) return;
    setSaving(namespace, true, null);
    const snapshot = { ...graphView.values };
    // Optimistic: apply dirty to graph
    const graph = useGraphStore.getState();
    for (const [k, v] of Object.entries(dirty.values)) {
      const id = `${namespace}:${k}`;
      graph.upsertEntity("Setting", id, { id, namespace, key: k, data: v, ...graphView.settings[k] });
    }
    try {
      const payload = Object.fromEntries(
        Object.entries(dirty.values).map(([k, v]) => [k.startsWith(`${namespace}.`) ? k.slice(namespace.length + 1) : k, v]),
      );
      await putSettingsNamespace(namespace, payload);
      clearDirty(namespace);
      setSaving(namespace, false, null);
      await loadSettingsIntoGraph(namespace);
    } catch (e) {
      // Rollback: restore snapshot
      for (const [k, v] of Object.entries(snapshot)) {
        const id = `${namespace}:${k}`;
        useGraphStore.getState().upsertEntity("Setting", id, { id, namespace, key: k, data: v });
      }
      setSaving(namespace, false, (e as Error).message);
      throw e;
    }
  }, [namespace, dirty.values, graphView]);

  return {
    values: { ...graphView.values, ...dirty.values }, // dirty wins for display
    settings: graphView.settings,
    dirty: dirty.values,
    conflicts,
    loading: graphView.records.length === 0,
    saving: dirty.saving,
    error: dirty.error,
    setSetting,
    saveAll,
    reload: () => loadSettingsIntoGraph(namespace),
  };
}
```

Contract test at `frontend/src/hooks/__tests__/use-settings.test.tsx`:
- setSetting → dirty marker present
- saveAll → dirty cleared
- SSE upsert while dirty → conflict synthesised

### 3. `retire-settings-store-and-bus-source`

- `git rm frontend/src/stores/settings-store.ts`
- `frontend/src/services/settings-change-bus.ts`: keep `emitSettingsChanged` + `onSettingsChanged` + `impactForSettingsNamespace` exports. Drop nothing — `entities/sync.ts:292` already emits.
- `frontend/src/main.tsx`: remove `initSettingsRealtimeBridge()` call + import.
- `git grep useSettingsStore frontend/` empty.

### 4. `audit-doc-flip-settings-direct`

- Flip `Setting` row in `docs/migration-stale-data-audit.md` to `direct`.
- Append a "Form-cache pattern" sub-section to the Direct Migration Playbook explaining that pages with dirty/save semantics get a module-level `Map<namespace, DirtyState>` instead of growing graph entities with `_dirty` markers.

---

## Verification matrix

| Gate | Where | When |
|---|---|---|
| `pnpm --filter ./frontend test` ≥ 37/37 (new test added) | every change after #2 | always |
| `pnpm --filter ./frontend build` clean | every change | always |
| `git grep useSettingsStore frontend/` empty | change-3 | end of phase |
| `git grep initSettingsRealtimeBridge frontend/` empty | change-3 | end of phase |
| Manual smoke: edit setting + save + refresh | change-2 | change-2 |
| Manual smoke: two-tab conflict surfaces | change-2 | change-2 |
| Audit doc row | change-4 | end of phase |

---

## Next step

`/kbd-execute settings-store-retirement` — proceeds straight through per established "jet through" cadence.
