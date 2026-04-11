import { useMemo } from "react";
import { useEntityView, useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import type { FilterClause, ViewDescriptor } from "@prometheus-ags/prometheus-entity-management";
import type { ToolEntity } from "@/entities/types";

const EMPTY_TOOLS: ToolEntity[] = [];

/**
 * Live, filterable view of all Tool entities in the graph.
 *
 * Sorted alphabetically by name.
 * Supports optional free-text search across name, description, and namespace,
 * and optional filtering by namespace.
 */
export function useTools(searchTerm?: string, namespaceFilter?: string) {
  const filter = useMemo<FilterClause[] | undefined>(() => {
    if (!namespaceFilter || namespaceFilter === "all") return undefined;
    return [{ field: "namespace", op: "eq" as const, value: namespaceFilter }];
  }, [namespaceFilter]);

  const view = useMemo<ViewDescriptor>(() => {
    const baseFilter: FilterClause[] = filter ? [...filter] : [];

    return {
      filter: baseFilter.length > 0 ? baseFilter : undefined,
      sort: [
        { field: "namespace", direction: "asc" as const },
        { field: "name", direction: "asc" as const },
      ],
      search:
        searchTerm && searchTerm.length > 0
          ? {
              query: searchTerm,
              fields: ["name", "description", "namespace"],
              minChars: 1,
            }
          : undefined,
    };
  }, [searchTerm, filter]);

  return useEntityView<ToolEntity>({
    type: "Tool",
    baseQueryKey: ["tools", searchTerm ?? "", namespaceFilter ?? ""],
    view,
    mode: "local",
  });
}

/**
 * Returns a single Tool entity by id, read directly from the graph.
 *
 * Uses `useGraphStore` selector so the component re-renders only when
 * that tool's data changes. Returns null when the tool is not loaded.
 */
export function useTool(id: string | undefined): ToolEntity | null {
  return useGraphStore((state) => {
    if (!id) return null;

    const toolMap = state.entities["Tool"];
    if (!toolMap) return null;

    const entity = toolMap[id];
    if (!entity) return null;

    return entity as unknown as ToolEntity;
  });
}

/**
 * Returns all Tool entities for a given namespace, read directly from the graph.
 *
 * Returns a stable empty array when no tools match (avoids the Zustand
 * infinite-render bug).
 */
export function useToolsByNamespace(namespace: string | undefined): ToolEntity[] {
  return useGraphStore((state) => {
    if (!namespace) return EMPTY_TOOLS;

    const toolMap = state.entities["Tool"];
    if (!toolMap) return EMPTY_TOOLS;

    const results: ToolEntity[] = [];
    for (const id of Object.keys(toolMap)) {
      const entity = toolMap[id];
      if (
        entity &&
        (entity as Record<string, unknown>)["namespace"] === namespace
      ) {
        results.push(entity as unknown as ToolEntity);
      }
    }

    return results.length > 0 ? results : EMPTY_TOOLS;
  });
}
