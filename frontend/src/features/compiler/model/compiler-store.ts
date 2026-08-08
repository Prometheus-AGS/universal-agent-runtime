import { create } from "zustand";
import { useGraphStore } from "@/platform/entities";
import { createCompilerSession, fetchCompilerSessions } from "../api/compiler-api";
import type { UarCompilerSession } from "@/types";

interface CompilerStore {
  loading: boolean;
  creating: boolean;
  error: string | null;
  load: () => Promise<void>;
  createSession: () => Promise<void>;
}

export const useCompilerStore = create<CompilerStore>((set, get) => ({
  loading: false,
  creating: false,
  error: null,
  load: async () => {
    set({ loading: true, error: null });
    try {
      const response = await fetchCompilerSessions<{ sessions?: UarCompilerSession[]; data?: { sessions?: UarCompilerSession[] } } | UarCompilerSession[]>();
      const sessions = Array.isArray(response) ? response : response.data?.sessions ?? response.sessions ?? [];
      const graph = useGraphStore.getState();
      const ids = new Set(sessions.map((session) => session.id));
      for (const id of Object.keys(graph.entities.CompilerSession ?? {})) if (!ids.has(id)) graph.removeEntity("CompilerSession", id);
      for (const session of sessions) graph.upsertEntity("CompilerSession", session.id, session as unknown as Record<string, unknown>);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ loading: false });
    }
  },
  createSession: async () => {
    set({ creating: true, error: null });
    try {
      await createCompilerSession<unknown>();
      await get().load();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ creating: false });
    }
  },
}));
