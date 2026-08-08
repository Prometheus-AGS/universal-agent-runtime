import { create } from "zustand";
import { useGraphStore } from "@/platform/entities";

import type { MemoryItem, MemoryListQuery } from "./memory-types";
import { bulkDeleteMemoriesApi, deleteMemoryApi, fetchMemoriesList, fetchMemoryStats } from "../api/memory-api";

interface MemoryAdminStore {
  loading: boolean;
  deleting: boolean;
  error: string | null;
  load: (query: MemoryListQuery) => Promise<void>;
  remove: (item: MemoryItem) => Promise<void>;
  removeVisible: (items: MemoryItem[], userId: string, agentId: string) => Promise<void>;
  clearError: () => void;
}

async function refreshStats() {
  const stats = await fetchMemoryStats();
  if (stats) useGraphStore.getState().upsertEntity("MemoryMeta", "current", { id: "current", ...stats });
}

export const useMemoryAdminStore = create<MemoryAdminStore>((set) => ({
  loading: false,
  deleting: false,
  error: null,
  clearError: () => set({ error: null }),
  load: async (query) => {
    set({ loading: true, error: null });
    try {
      const response = await fetchMemoriesList(query);
      const graph = useGraphStore.getState();
      const ids = new Set(response.items.map((item) => item.id));
      for (const id of Object.keys(graph.entities.Memory ?? {})) if (!ids.has(id)) graph.removeEntity("Memory", id);
      for (const item of response.items) graph.upsertEntity("Memory", item.id, item as unknown as Record<string, unknown>);
      await refreshStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ loading: false });
    }
  },
  remove: async (item) => {
    set({ deleting: true, error: null });
    try {
      await deleteMemoryApi(item.id);
      useGraphStore.getState().removeEntity("Memory", item.id);
      await refreshStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ deleting: false });
    }
  },
  removeVisible: async (items, userId, agentId) => {
    set({ deleting: true, error: null });
    try {
      await bulkDeleteMemoriesApi(userId, agentId);
      const graph = useGraphStore.getState();
      for (const item of items) graph.removeEntity("Memory", item.id);
      await refreshStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ deleting: false });
    }
  },
}));
