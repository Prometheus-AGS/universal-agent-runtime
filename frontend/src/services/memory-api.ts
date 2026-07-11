import type { MemoryItem, MemoryListQuery, MemoryStats } from "@/entities/memory-types";

export type { MemoryItem, MemoryListQuery, MemoryStats } from "@/entities/memory-types";

export interface MemoryListResponse {
  total: number;
  items: MemoryItem[];
}


export function buildMemoriesUrl(q: MemoryListQuery): string {
  const { userId, agentId, searchQ, searchMode } = q;
  const params = new URLSearchParams();
  if (userId) params.set("user_id", userId);
  if (agentId) params.set("agent_id", agentId);

  if (searchMode && searchQ) {
    return `/api/admin/memories/search?q=${encodeURIComponent(searchQ)}&limit=200${userId ? `&user_id=${userId}` : ""}${agentId ? `&agent_id=${agentId}` : ""}`;
  }
  return `/api/admin/memories?${params.toString()}`;
}

export async function fetchMemoryStats(): Promise<MemoryStats | null> {
  try {
    const r = await fetch("/api/admin/memories/stats");
    if (r.ok) return (await r.json()) as MemoryStats;
  } catch {
    /* non-fatal */
  }
  return null;
}

export async function fetchMemoriesList(q: MemoryListQuery): Promise<MemoryListResponse> {
  const url = buildMemoriesUrl(q);
  const r = await fetch(url);
  if (!r.ok) throw new Error(`${r.status}`);
  return (await r.json()) as MemoryListResponse;
}

export async function deleteMemoryApi(id: string): Promise<void> {
  const r = await fetch(`/api/admin/memories/${id}`, { method: "DELETE" });
  if (!r.ok) throw new Error(`${r.status}`);
}

export async function bulkDeleteMemoriesApi(userId: string, agentId: string): Promise<void> {
  const params = new URLSearchParams();
  if (userId) params.set("user_id", userId);
  if (agentId) params.set("agent_id", agentId);
  const r = await fetch(`/api/admin/memories?${params.toString()}`, { method: "DELETE" });
  if (!r.ok) throw new Error(`${r.status}`);
}
