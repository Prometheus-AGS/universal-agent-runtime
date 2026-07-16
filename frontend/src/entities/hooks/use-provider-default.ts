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

/**
 * True when the system-wide provider registry has both a default provider
 * id and that provider has a default model configured — i.e. an agent
 * with no per-agent override can actually resolve a model at chat time.
 */
export function useHasWorkingSystemDefault(): boolean {
  return useGraphStore((state) => {
    const meta = state.entities["ProviderMeta"]?.["current"] as
      | ProviderMetaEntity
      | undefined;
    return Boolean(meta?.default_id) && Boolean(meta?.default_model);
  });
}
