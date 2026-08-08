import { useMemo } from "react";
import { useEntityView, useGraphStore } from "@/platform/entities";
import type { FilterClause, ViewDescriptor } from "@/platform/entities";
import type { ProviderEntity, ModelEntity } from "@/entities/types";

// Stable empty array to avoid Zustand infinite-render loops when no models exist.
const EMPTY_MODELS: ModelEntity[] = [];

/**
 * Live, filterable view of all Provider entities in the graph.
 *
 * Sorted configured-first, then alphabetical by display_name.
 * Supports optional free-text search across id and display_name.
 */
export function useProviders(searchTerm?: string, filter?: FilterClause[]) {
  const view = useMemo<ViewDescriptor>(() => {
    const baseFilter: FilterClause[] = filter ? [...filter] : [];

    return {
      filter: baseFilter.length > 0 ? baseFilter : undefined,
      sort: [
        {
          field: "configured",
          direction: "desc" as const,
          comparator: (a: unknown, b: unknown) =>
            (a === true ? 1 : 0) - (b === true ? 1 : 0),
        },
        { field: "display_name", direction: "asc" as const },
      ],
      search:
        searchTerm && searchTerm.length > 0
          ? { query: searchTerm, fields: ["id", "display_name"], minChars: 1 }
          : undefined,
    };
  }, [searchTerm, filter]);

  return useEntityView<ProviderEntity>({
    type: "Provider",
    baseQueryKey: ["providers", searchTerm ?? "", JSON.stringify(filter ?? [])],
    view,
    mode: "local",
  });
}

/**
 * Returns all Model entities for a given provider, read directly from the graph.
 *
 * Uses `useGraphStore` selector so the component re-renders only when that
 * provider's model slice changes.  Returns a stable empty array when no
 * models are loaded yet (avoids the Zustand infinite-render bug).
 */
export function useProviderModelsEntity(providerId: string | undefined): ModelEntity[] {
  return useGraphStore((state) => {
    if (!providerId) return EMPTY_MODELS;

    const modelMap = state.entities["Model"];
    if (!modelMap) return EMPTY_MODELS;

    const results: ModelEntity[] = [];
    for (const id of Object.keys(modelMap)) {
      const entity = modelMap[id];
      if (entity && (entity as Record<string, unknown>)["provider_id"] === providerId) {
        results.push(entity as unknown as ModelEntity);
      }
    }

    return results.length > 0 ? results : EMPTY_MODELS;
  });
}
