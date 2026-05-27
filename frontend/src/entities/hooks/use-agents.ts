import { useMemo } from "react";
import { useEntityView, useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import type { FilterClause, ViewDescriptor } from "@prometheus-ags/prometheus-entity-management";
import type { AgentEntity } from "@/entities/types";

const EMPTY_AGENTS: AgentEntity[] = [];

/**
 * Live, filterable view of all Agent entities in the graph.
 *
 * Sorted alphabetically by name.
 * Supports optional free-text search across name and description,
 * and optional status filtering (active/draft/disabled).
 */
export function useAgents(searchTerm?: string, statusFilter?: string) {
  const filter = useMemo<FilterClause[] | undefined>(() => {
    if (!statusFilter || statusFilter === "all") return undefined;
    return [{ field: "status", op: "eq" as const, value: statusFilter }];
  }, [statusFilter]);

  const view = useMemo<ViewDescriptor>(() => {
    const baseFilter: FilterClause[] = filter ? [...filter] : [];

    return {
      filter: baseFilter.length > 0 ? baseFilter : undefined,
      sort: [{ field: "name", direction: "asc" as const }],
      search:
        searchTerm && searchTerm.length > 0
          ? { query: searchTerm, fields: ["name", "description"], minChars: 1 }
          : undefined,
    };
  }, [searchTerm, filter]);

  return useEntityView<AgentEntity>({
    type: "Agent",
    baseQueryKey: ["agents", searchTerm ?? "", statusFilter ?? ""],
    view,
    mode: "local",
  });
}

/**
 * Returns a single Agent entity by id, read directly from the graph.
 *
 * Uses `useGraphStore` selector so the component re-renders only when
 * that agent's data changes. Returns null when the agent is not loaded.
 */
export function useAgent(id: string | undefined): AgentEntity | null {
  return useGraphStore((state) => {
    if (!id) return null;

    const agentMap = state.entities["Agent"];
    if (!agentMap) return null;

    const entity = agentMap[id];
    if (!entity) return null;

    return entity as unknown as AgentEntity;
  });
}

/**
 * Returns all Agent entities filtered by status, read directly from the graph.
 *
 * Returns a stable empty array when no agents match (avoids the Zustand
 * infinite-render bug).
 */
export function useAgentsByStatus(
  status: string,
): AgentEntity[] {
  return useGraphStore((state) => {
    const agentMap = state.entities["Agent"];
    if (!agentMap) return EMPTY_AGENTS;

    const results: AgentEntity[] = [];
    for (const id of Object.keys(agentMap)) {
      const entity = agentMap[id];
      if (entity && (entity as Record<string, unknown>)["status"] === status) {
        results.push(entity as unknown as AgentEntity);
      }
    }

    return results.length > 0 ? results : EMPTY_AGENTS;
  });
}
