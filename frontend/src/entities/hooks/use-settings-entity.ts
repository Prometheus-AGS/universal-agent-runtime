import { useMemo } from "react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { useGraphEntity } from "@/entities/hooks/use-graph-entities";
import type { SettingWithMeta } from "@/types";

interface SettingsNamespaceRow {
  id: string;
  namespace: string;
  ids: string[];
}

/**
 * Live view of a single settings namespace. Reads from the graph; consumers
 * call `loadSettingsIntoGraph(namespace)` on mount to hydrate.
 *
 * Returns the records keyed by setting key (e.g. `auto_persist_memory`),
 * along with derived `values` and `settings` lookup maps that match the
 * shape the legacy `useSettings()` hook returned (eases page migration).
 */
export function useSettingsEntity(namespace: string): {
  records: SettingWithMeta[];
  values: Record<string, unknown>;
  settings: Record<string, SettingWithMeta>;
} {
  const nsRow = useGraphEntity<SettingsNamespaceRow>("SettingsNamespace", namespace);
  // Subscribe to the Setting slice so re-renders track per-row updates.
  const settingsMap = useGraphStore((s) => s.entities["Setting"]);

  return useMemo(() => {
    const ids = nsRow?.ids ?? [];
    const records: SettingWithMeta[] = [];
    for (const id of ids) {
      const row = settingsMap?.[id];
      if (row) records.push(row as unknown as SettingWithMeta);
    }
    const values: Record<string, unknown> = {};
    const settings: Record<string, SettingWithMeta> = {};
    for (const r of records) {
      values[r.key] = r.data;
      settings[r.key] = r;
    }
    return { records, values, settings };
  }, [nsRow, settingsMap]);
}
