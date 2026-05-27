import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import type { ProviderMetaEntity } from "@/entities/types";

/**
 * Reads the current default provider id from the `ProviderMeta` singleton
 * (id `"current"`). Returns `null` when nothing is configured yet.
 *
 * Hydrated by `loadProvidersIntoGraph()`; kept fresh by the realtime SSE
 * bus when the provider list mutates.
 */
export function useProviderDefault(): string | null {
  return useGraphStore((state) => {
    const meta = state.entities["ProviderMeta"]?.["current"] as
      | ProviderMetaEntity
      | undefined;
    return meta?.default_id ?? null;
  });
}
