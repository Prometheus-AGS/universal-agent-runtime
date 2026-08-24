import { useEffect } from "react";
import { useShallow } from "zustand/react/shallow";

import { useGraphStore } from "@/platform/entities";
import type { ProviderMetaEntity } from "@/entities/types";
import { useProvidersStore } from "./providers-store";

export type SystemDefaultModelStatus =
  | {
      status: "loading" | "unavailable" | "error";
      providerId: string | null;
      modelId: string | null;
    }
  | {
      status: "available";
      providerId: string;
      modelId: string;
    };

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
  return useSystemDefaultModelStatus().status === "available";
}

/**
 * Loads and classifies the system-wide model route for status consumers.
 * Provider routing remains graph-owned; callers receive only the resolved
 * primitives needed to render the current state.
 */
export function useSystemDefaultModelStatus(): SystemDefaultModelStatus {
  const meta = useGraphStore(useShallow((state) => {
    const meta = state.entities["ProviderMeta"]?.["current"] as
      | ProviderMetaEntity
      | undefined;
    return {
      providerId: meta?.default_id ?? null,
      modelId: meta?.default_model ?? null,
    };
  }));
  const loaded = useProvidersStore((state) => state.loaded);
  const refreshing = useProvidersStore((state) => state.refreshing);
  const error = useProvidersStore((state) => state.error);
  const load = useProvidersStore((state) => state.load);

  useEffect(() => {
    if (!loaded && !refreshing && !error) {
      void load().catch(() => undefined);
    }
  }, [error, load, loaded, refreshing]);

  if (!loaded) {
    return {
      status: error ? "error" : "loading",
      providerId: null,
      modelId: null,
    };
  }

  if (meta.providerId && meta.modelId) {
    return {
      status: "available",
      providerId: meta.providerId,
      modelId: meta.modelId,
    };
  }

  return {
    status: "unavailable",
    providerId: meta.providerId,
    modelId: meta.modelId,
  };
}
