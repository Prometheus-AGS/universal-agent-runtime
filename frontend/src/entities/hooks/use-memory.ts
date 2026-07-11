import { useGraphEntities, useGraphEntity } from "@/entities/hooks/use-graph-entities";
import type { MemoryItem, MemoryStats } from "@/services/memory-api";

/** Live list of all Memory entities currently hydrated in the graph. */
export function useMemory(): { items: MemoryItem[] } {
  return { items: useGraphEntities<MemoryItem>("Memory") };
}

/** Singleton MemoryMeta row holding the latest stats. */
export function useMemoryStats(): MemoryStats | null {
  const row = useGraphEntity<MemoryStats & { id: string }>("MemoryMeta", "current");
  if (!row) return null;
  const { id, ...stats } = row;
  void id;
  return stats as MemoryStats;
}
