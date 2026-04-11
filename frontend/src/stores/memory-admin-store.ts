import { create } from "zustand";

import type { MemoryItem, MemoryListQuery, MemoryStats } from "@/services/memory-api";
import {
  bulkDeleteMemoriesApi,
  deleteMemoryApi,
  fetchMemoriesList,
  fetchMemoryStats,
} from "@/services/memory-api";

interface MemoryAdminState {
  items: MemoryItem[];
  stats: MemoryStats | null;
  loading: boolean;
  error: string | null;
  deleting: boolean;
}

interface MemoryAdminActions {
  load: (q: MemoryListQuery) => Promise<void>;
  loadStats: () => Promise<void>;
  deleteOne: (id: string) => Promise<void>;
  bulkDelete: (userId: string, agentId: string) => Promise<void>;
  removeItemLocal: (id: string) => void;
  clearItems: () => void;
}

export const useMemoryAdminStore = create<MemoryAdminState & MemoryAdminActions>((set, get) => ({
  items: [],
  stats: null,
  loading: false,
  error: null,
  deleting: false,

  loadStats: async () => {
    const stats = await fetchMemoryStats();
    if (stats) set({ stats });
  },

  load: async (q) => {
    set({ loading: true, error: null });
    try {
      const data = await fetchMemoriesList(q);
      set({ items: data.items ?? [], loading: false });
      void get().loadStats();
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },

  deleteOne: async (id) => {
    set({ deleting: true, error: null });
    try {
      await deleteMemoryApi(id);
      set((s) => ({
        items: s.items.filter((m) => m.id !== id),
        deleting: false,
      }));
      void get().loadStats();
    } catch (e) {
      set({ error: (e as Error).message, deleting: false });
      throw e;
    }
  },

  bulkDelete: async (userId, agentId) => {
    set({ deleting: true, error: null });
    try {
      await bulkDeleteMemoriesApi(userId, agentId);
      set({ items: [], deleting: false });
      void get().loadStats();
    } catch (e) {
      set({ error: (e as Error).message, deleting: false });
      throw e;
    }
  },

  removeItemLocal: (id) => {
    set((s) => ({ items: s.items.filter((m) => m.id !== id) }));
  },

  clearItems: () => set({ items: [] }),
}));
