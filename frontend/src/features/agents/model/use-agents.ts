import { useMemo } from "react";
import { useGraphStore } from "@/platform/entities";
import type { AgentEntity } from "@/entities/types";
import {
  AGENT_LOAD_STATE_ENTITY,
  AGENT_LOAD_STATE_ID,
} from "./agents-graph";
import type { AgentLoadState } from "./agents-graph";

const EMPTY_AGENTS: AgentEntity[] = [];

export function useAgentLoadState(): AgentLoadState | null {
  return useGraphStore((state) =>
    (state.entities[AGENT_LOAD_STATE_ENTITY]?.[AGENT_LOAD_STATE_ID] as
      | AgentLoadState
      | undefined) ?? null,
  );
}

/**
 * Live, filterable view of all Agent entities in the graph.
 *
 * Sorted alphabetically by title.
 * Supports optional free-text search across title and description,
 * and optional status filtering (active/draft/disabled).
 *
 * Previously used `useEntityView` (the library's deprecated local-list
 * hook), which derives its visible ids from `graph.lists[baseKey]` — a
 * list index only ever written by `setListResult`/`appendListResult`.
 * `loadAgentsIntoGraph()` only calls `upsertEntity` (entity *data*, no list
 * index), so that list key was NEVER populated: the agent selector's
 * popover showed "Loading agents..." forever, for every agent, not just
 * newly-created ones. `useGraphEntities` (already the working pattern for
 * `useModels`/`useAgentsByStatus` above) reads entities directly and has
 * no such list-index dependency.
 */
export function useAgents(searchTerm?: string, statusFilter?: string): { items: AgentEntity[] } {
  const agentMap = useGraphStore((state) => state.entities["Agent"]);

  const items = useMemo(() => {
    const all = Object.values(agentMap ?? {}) as unknown as AgentEntity[];
    const needle = searchTerm?.trim().toLowerCase();

    const filtered = all.filter((a) => {
      if (statusFilter && statusFilter !== "all" && (a as unknown as Record<string, unknown>).status !== statusFilter) {
        return false;
      }
      if (needle) {
        const title = a.metadata?.title?.toLowerCase() ?? "";
        const description = a.metadata?.description?.toLowerCase() ?? "";
        if (!title.includes(needle) && !description.includes(needle)) return false;
      }
      return true;
    });

    return filtered.sort((a, b) => (a.metadata?.title ?? "").localeCompare(b.metadata?.title ?? ""));
  }, [agentMap, searchTerm, statusFilter]);

  return { items: items.length > 0 ? items : EMPTY_AGENTS };
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
