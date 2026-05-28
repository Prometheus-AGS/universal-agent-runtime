import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import {
  fetchMemoriesList,
  fetchMemoryStats,
  type MemoryItem,
  type MemoryListQuery,
  type MemoryStats,
} from "@/services/memory-api";

/**
 * Fetch memory rows (with the given query) and upsert each as a `Memory`
 * entity. Stats are stored as a singleton `MemoryMeta:current` row so
 * components can read both with the same graph subscription.
 */
export async function loadMemoryIntoGraph(q: MemoryListQuery): Promise<void> {
  const data = await fetchMemoriesList(q);
  const { upsertEntity } = useGraphStore.getState();
  const items: MemoryItem[] = data.items ?? [];

  // Track the ids the current query brought in so consumers can derive a
  // query-scoped view; the page passes the same query down.
  const ids: string[] = [];
  for (const m of items) {
    upsertEntity("Memory", m.id, m as unknown as Record<string, unknown>);
    ids.push(m.id);
  }
  upsertEntity("MemoryQuery", "current", {
    id: "current",
    query: q,
    ids,
  });
}

/** Refresh the singleton MemoryMeta with the latest stats. */
export async function loadMemoryStatsIntoGraph(): Promise<void> {
  const stats: MemoryStats | null = await fetchMemoryStats();
  if (!stats) return;
  useGraphStore
    .getState()
    .upsertEntity("MemoryMeta", "current", {
      id: "current",
      ...(stats as unknown as Record<string, unknown>),
    });
}
