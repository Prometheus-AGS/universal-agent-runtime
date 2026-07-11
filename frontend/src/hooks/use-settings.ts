import { useCallback, useEffect, useMemo, useSyncExternalStore } from "react";

import { useSettingsEntity } from "@/entities/hooks/use-settings-entity";
import {
  clearDirty,
  getDirty,
  setDirty,
  subscribe,
} from "@/hooks/settings-form-cache";
import { useSettingsStore } from "@/stores/settings-store";
import type { SettingWithMeta } from "@/types";

export interface UseSettingsReturn {
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

/**
 * Direct settings hook.
 *
 * - Reads `values` + `settings` from the entity graph (live, SSE-fed).
 * - Owns `dirty` / `saving` / `error` in a module-level per-namespace cache
 *   so edits survive component re-mount within a session.
 * - Exposes the settings store's load/save actions while retaining only
 *   unsaved form drafts in the presentation cache.
 *
 * Replaces the retired `settings-store.ts`. Hook contract unchanged so
 * the 3334 LOC `settings-page.tsx` requires zero call-site changes.
 */
export function useSettings(namespace: string): UseSettingsReturn {
  const graphView = useSettingsEntity(namespace);
  const status = useSettingsStore((state) => state.statusByNamespace[namespace]);
  const loadNamespace = useSettingsStore((state) => state.load);
  const saveNamespace = useSettingsStore((state) => state.save);

  // Per-namespace dirty/saving/error from the module cache.
  const dirtyState = useSyncExternalStore(
    useCallback((cb) => subscribe(namespace, cb), [namespace]),
    useCallback(() => getDirty(namespace), [namespace]),
    useCallback(() => getDirty(namespace), [namespace]),
  );

  // Hydrate the graph on mount.
  useEffect(() => {
    void loadNamespace(namespace);
  }, [namespace, loadNamespace]);

  // Conflicts: where dirty diverges from remote-known.
  const conflicts = useMemo(() => {
    const out: Record<string, unknown> = {};
    for (const [k, dv] of Object.entries(dirtyState.values)) {
      const remote = graphView.values[k];
      if (remote !== undefined && !Object.is(remote, dv)) {
        out[k] = remote;
      }
    }
    return out;
  }, [dirtyState.values, graphView.values]);

  // Display values: dirty wins.
  const values = useMemo(
    () => ({ ...graphView.values, ...dirtyState.values }),
    [graphView.values, dirtyState.values],
  );

  const setSetting = useCallback(
    (key: string, value: unknown) => setDirty(namespace, key, value),
    [namespace],
  );

  const reload = useCallback(
    () => loadNamespace(namespace),
    [namespace, loadNamespace],
  );

  const saveAll = useCallback(async () => {
    const dirty = getDirty(namespace).values;
    if (Object.keys(dirty).length === 0) return;

    await saveNamespace(namespace, dirty);
    clearDirty(namespace);
  }, [namespace, saveNamespace]);

  const loading = (status?.loading ?? false) && graphView.records.length === 0;

  return {
    values,
    settings: graphView.settings,
    dirty: dirtyState.values,
    conflicts,
    loading,
    saving: status?.saving ?? false,
    error: status?.error ?? null,
    setSetting,
    saveAll,
    reload,
  };
}
