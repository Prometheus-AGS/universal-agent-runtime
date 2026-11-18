import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import type { GraphState } from "@prometheus-ags/entity-graph-core";
import { useMemo } from "react";
import { TEAMS_RUNTIME_ID, TEAMS_SESSION_ID, TEAM_RUNTIME_ENTITY, lastIdSegment, type TeamRuntimeState } from "./entities";

/** Shared selector helpers. Every hook resolves the admitted owner here, so a persisted graph shows nothing until this session has admitted one. */

export const EMPTY_IDS: string[] = [];

export function ownerOf(state: GraphState): string | null {
  const runtime = state.entities[TEAM_RUNTIME_ENTITY]?.[TEAMS_RUNTIME_ID] as unknown as TeamRuntimeState | undefined;
  return runtime?.session === TEAMS_SESSION_ID ? runtime.ownerId : null;
}

export function runtimeOf(state: GraphState): TeamRuntimeState | undefined {
  const runtime = state.entities[TEAM_RUNTIME_ENTITY]?.[TEAMS_RUNTIME_ID] as unknown as TeamRuntimeState | undefined;
  return runtime?.session === TEAMS_SESSION_ID ? runtime : undefined;
}

export function entityOf<T>(state: GraphState, type: string, id: string): T | undefined {
  return state.entities[type]?.[id] as unknown as T | undefined;
}

export function idsOf(state: GraphState, key: string): string[] {
  return state.lists[key]?.ids ?? EMPTY_IDS;
}

/** One field of one owner-scoped record: a primitive or a stable graph reference, never a fresh object. */
export function useScopedField<E extends object, K extends keyof E>(
  type: string, graphId: (owner: string) => string, field: K,
): E[K] | undefined {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    return owner ? entityOf<E>(state, type, graphId(owner))?.[field] : undefined;
  });
}

/** An id list mapped to its domain ids (`teamId`, `memberId`, ...). Stable while the list is. */
export function useScopedIds(listKey: (owner: string) => string): string[] {
  const ids = useGraphStore((state) => {
    const owner = ownerOf(state);
    return owner ? idsOf(state, listKey(owner)) : EMPTY_IDS;
  });
  return useMemo(() => (ids.length === 0 ? EMPTY_IDS : ids.map(lastIdSegment)), [ids]);
}
