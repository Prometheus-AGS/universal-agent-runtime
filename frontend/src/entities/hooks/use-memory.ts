import { useEntityList, useEntity } from "@prometheus-ags/prometheus-entity-management";
import type { MemoryItem, MemoryStats } from "@/services/memory-api";

/** Live list of all Memory entities currently hydrated in the graph. */
export function useMemory() {
  return useEntityList<MemoryItem>("Memory");
}

/** Singleton MemoryMeta row holding the latest stats. */
export function useMemoryStats(): MemoryStats | null {
  const row = useEntity<MemoryStats & { id: string }>("MemoryMeta", "current");
  if (!row) return null;
  const { id: _id, ...stats } = row;
  return stats as MemoryStats;
}
