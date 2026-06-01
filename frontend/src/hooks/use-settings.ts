import { useCallback, useEffect, useMemo, useSyncExternalStore } from "react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

import { useSettingsEntity } from "@/entities/hooks/use-settings-entity";
import { loadSettingsIntoGraph } from "@/entities/fetchers/settings";
import { putSettingsNamespace } from "@/services/settings-api";
import {
  clearDirty,
  getDirty,
  setDirty,
  setSaving,
  subscribe,
} from "@/hooks/settings-form-cache";
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
 * - Mutates via direct `putSettingsNamespace` wrapped in an optimistic
 *   per-namespace upsert + rollback.
 *
 * Replaces the retired `settings-store.ts`. Hook contract unchanged so
 * the 3334 LOC `settings-page.tsx` requires zero call-site changes.
 */
export function useSettings(namespace: string): UseSettingsReturn {
  const graphView = useSettingsEntity(namespace);

  // Per-namespace dirty/saving/error from the module cache.
  const dirtyState = useSyncExternalStore(
    useCallback((cb) => subscribe(namespace, cb), [namespace]),
    useCallback(() => getDirty(namespace), [namespace]),
    useCallback(() => getDirty(namespace), [namespace]),
  );

  // Hydrate the graph on mount.
  useEffect(() => {
    void loadSettingsIntoGraph(namespace);
  }, [namespace]);

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
    () => loadSettingsIntoGraph(namespace),
    [namespace],
  );

  const saveAll = useCallback(async () => {
    const dirty = getDirty(namespace).values;
    if (Object.keys(dirty).length === 0) return;

    // Snapshot the current graph values for the dirty keys so we can roll back.
    const graph = useGraphStore.getState();
    const snapshots: Record<string, unknown> = {};
    for (const k of Object.keys(dirty)) {
      const id = `${namespace}:${k}`;
      snapshots[k] = graph.entities["Setting"]?.[id];
    }

    // Optimistically apply dirty to the graph.
    for (const [k, v] of Object.entries(dirty)) {
      const id = `${namespace}:${k}`;
      const existing = (graph.entities["Setting"]?.[id] ?? {}) as Record<string, unknown>;
      graph.upsertEntity("Setting", id, {
        ...existing,
        id,
        namespace,
        key: k,
        data: v,
      });
    }

    setSaving(namespace, true, null);
    try {
      const payload: Record<string, unknown> = {};
      for (const [k, v] of Object.entries(dirty)) {
        const stripped = k.startsWith(`${namespace}.`)
          ? k.slice(namespace.length + 1)
          : k;
        payload[stripped] = v;
      }
      const response = await putSettingsNamespace(namespace, payload);
      if (response.errors?.length) {
        throw new Error(
          response.errors.map((e) => `${e.key}: ${e.error}`).join("; "),
        );
      }
      // Apply server response to the graph; clear dirty.
      const after = useGraphStore.getState();
      for (const row of response.updated ?? []) {
        const id = `${namespace}:${row.key}`;
        after.upsertEntity("Setting", id, {
          id,
          namespace,
          ...(row as unknown as Record<string, unknown>),
        });
      }
      clearDirty(namespace);
      setSaving(namespace, false, null);
    } catch (e) {
      // Roll back graph to snapshots.
      const after = useGraphStore.getState();
      for (const [k, snap] of Object.entries(snapshots)) {
        const id = `${namespace}:${k}`;
        if (snap !== undefined) {
          after.upsertEntity("Setting", id, snap as Record<string, unknown>);
        }
      }
      setSaving(namespace, false, (e as Error).message);
      throw e;
    }
  }, [namespace]);

  const loading = graphView.records.length === 0;

  return {
    values,
    settings: graphView.settings,
    dirty: dirtyState.values,
    conflicts,
    loading,
    saving: dirtyState.saving,
    error: dirtyState.error,
    setSetting,
    saveAll,
    reload,
  };
}
