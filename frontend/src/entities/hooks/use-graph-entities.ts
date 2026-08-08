import { useCallback, useMemo } from "react";
import { useGraphStore } from "@/platform/entities";
import type { EntityType } from "@/platform/entities";

/**
 * Live list of every entity of `type` currently in the graph.
 *
 * This app hydrates entities explicitly (a fetcher module does a REST call
 * then `upsertEntity`s the results; SSE-driven mutations keep the graph
 * fresh thereafter) rather than through the library's transport-registry
 * hooks (`useEntities`/`useEntityQuery`, which require a
 * `registerEntityTransport` call this app never makes). This is a thin
 * `useGraphStore` selector — the read side of that pattern, matching what
 * `runtime-console-page.tsx` already does locally.
 */
export function useGraphEntities<T>(type: EntityType): T[] {
  const entityMap = useGraphStore((state) => state.entities[type]);
  return useMemo(() => Object.values(entityMap ?? {}) as T[], [entityMap]);
}

/** Live single entity of `type` by `id`, or `null` when absent/not yet hydrated. */
export function useGraphEntity<T>(type: EntityType, id: string | null | undefined): T | null {
  return useGraphStore(
    useCallback(
      (state) => (id ? ((state.entities[type]?.[id] as T | undefined) ?? null) : null),
      [type, id],
    ),
  );
}
