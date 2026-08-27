import { useCallback, useEffect, useMemo, useSyncExternalStore } from "react";

import { useSettingsEntity } from "./use-settings-entity";
import {
  getDirty,
  reconcileSubmittedDirty,
  setDirty,
  subscribe,
} from "./settings-form-cache";
import { useSettingsStore } from "./settings-store";
import type { SettingWithMeta } from "@/types";
import type { BulkSettingsUpdateResponse } from "../api/settings-api";

export interface UseSettingsReturn {
  values: Record<string, unknown>;
  settings: Record<string, SettingWithMeta>;
  dirty: Record<string, unknown>;
  conflicts: Record<string, unknown>;
  loading: boolean;
  refreshing: boolean;
  saving: boolean;
  error: string | null;
  setSetting: (key: string, value: unknown) => void;
  saveAll: () => Promise<BulkSettingsUpdateResponse | null>;
  reload: () => Promise<void>;
}

export function successfulSubmittedKeys(
  submittedValues: Record<string, unknown>,
  response: BulkSettingsUpdateResponse,
): string[] {
  if (
    response.governance_outcome === "unknown" ||
    response.governance_outcome === "changed_elsewhere"
  ) {
    return [];
  }
  if (!response.results) return Object.keys(submittedValues);
  const authoritativeStatus =
    response.observed_governance_status ?? response.governance_status;
  return response.results
    .filter(
      (result) =>
        result.status === "updated" ||
        (result.key === "governance.enabled" &&
          result.status === "validation_rejected" &&
          authoritativeStatus?.effective_state === "required"),
    )
    .map((result) => result.key);
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
 * The live `settings-store.ts` remains the I/O owner; this hook combines its
 * actions and status with entity-graph values and presentation-only drafts.
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
    void loadNamespace(namespace).catch(() => {
      // The store retains the actionable namespace error for the panel.
    });
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
    const submitted = getDirty(namespace);
    if (Object.keys(submitted.values).length === 0) return null;

    const response = await saveNamespace(namespace, submitted.values);
    const successfulKeys = successfulSubmittedKeys(
      submitted.values,
      response,
    );
    reconcileSubmittedDirty(namespace, submitted, successfulKeys);
    if (namespace === "governance") {
      response.retained_draft_keys = Object.keys(getDirty(namespace).values);
    }
    return response;
  }, [namespace, saveNamespace]);

  const loading = (status?.loading ?? false) && graphView.records.length === 0;
  const refreshing = status?.loading ?? false;

  return {
    values,
    settings: graphView.settings,
    dirty: dirtyState.values,
    conflicts,
    loading,
    refreshing,
    saving: status?.saving ?? false,
    error: status?.error ?? null,
    setSetting,
    saveAll,
    reload,
  };
}
