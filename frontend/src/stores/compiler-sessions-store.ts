import { create } from "zustand";

import { createCompilerSession, fetchCompilerSessions } from "@/services/compiler-api";
import type { UarCompilerSession } from "@/types";

interface CompilerSessionsState {
  sessions: UarCompilerSession[];
  loading: boolean;
  error: string | null;
  creating: boolean;
}

interface CompilerSessionsActions {
  load: () => Promise<void>;
  createSession: () => Promise<void>;
}

export const useCompilerSessionsStore = create<CompilerSessionsState & CompilerSessionsActions>((set, get) => ({
  sessions: [],
  loading: true,
  error: null,
  creating: false,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const data = await fetchCompilerSessions<
        | { sessions?: UarCompilerSession[]; data?: { sessions?: UarCompilerSession[] } }
        | UarCompilerSession[]
      >();
      const list = Array.isArray(data) ? data : (data.data?.sessions ?? (data as { sessions?: UarCompilerSession[] }).sessions ?? []);
      set({ sessions: list, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },

  createSession: async () => {
    set({ creating: true, error: null });
    try {
      await createCompilerSession<unknown>();
      await get().load();
    } catch (e) {
      set({ error: (e as Error).message });
    } finally {
      set({ creating: false });
    }
  },
}));
