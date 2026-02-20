import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { useEffect } from "react";
import type { LocalThread } from "@/types";
import { getDbInstance } from "@/lib/db";

interface ThreadRegistryState {
  threads: Record<string, LocalThread>;
  activeThreadId: string | null;
}

interface ThreadRegistryActions {
  /** Load all threads from PGlite into Zustand (called once on mount). */
  hydrateFromDb(): Promise<void>;
  registerThread(id: string): void;
  markPersisted(id: string): void;
  setTitle(id: string, title: string): void;
  touch(id: string): void;
  setActive(id: string | null): void;
  removeThread(id: string): void;
  getLatestPersisted(): LocalThread | null;
}

type ThreadRegistryStore = ThreadRegistryState & ThreadRegistryActions;

export const useThreadRegistryStore = create<ThreadRegistryStore>()(
  immer((set, get) => ({
    threads: {},
    activeThreadId: null,

    hydrateFromDb: async () => {
      const db = getDbInstance();
      const threads = await db.getThreads();
      set((state) => {
        state.threads = {};
        for (const t of threads) {
          state.threads[t.id] = t;
        }
      });
    },

    registerThread: (id) => {
      const now = new Date().toISOString();
      const thread: LocalThread = {
        id,
        title: "New conversation",
        isEphemeral: true,
        createdAt: now,
        updatedAt: now,
      };
      set((state) => {
        if (state.threads[id]) return;
        state.threads[id] = thread;
      });
      void getDbInstance().upsertThread(thread);
    },

    markPersisted: (id) => {
      set((state) => {
        if (!state.threads[id]) return;
        state.threads[id].isEphemeral = false;
        state.threads[id].updatedAt = new Date().toISOString();
      });
      const updated = get().threads[id];
      if (updated) void getDbInstance().upsertThread(updated);
    },

    setTitle: (id, title) => {
      set((state) => {
        if (!state.threads[id]) return;
        state.threads[id].title = title;
        state.threads[id].updatedAt = new Date().toISOString();
      });
      const updated = get().threads[id];
      if (updated) void getDbInstance().upsertThread(updated);
    },

    touch: (id) => {
      set((state) => {
        if (!state.threads[id]) return;
        state.threads[id].updatedAt = new Date().toISOString();
      });
      void getDbInstance().touchThread(id);
    },

    setActive: (id) => set((state) => { state.activeThreadId = id; }),

    removeThread: (id) => {
      set((state) => {
        delete state.threads[id];
        if (state.activeThreadId === id) state.activeThreadId = null;
      });
      void getDbInstance().deleteThread(id);
    },

    getLatestPersisted: () => {
      const { threads } = get();
      const persisted = Object.values(threads)
        .filter((t) => !t.isEphemeral)
        .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime());
      return persisted[0] ?? null;
    },
  })),
);

// ---------------------------------------------------------------------------
// Convenience hook — hydrates threads from PGlite once on mount.
// Place this in the root layout so it runs once the DB is ready.
// ---------------------------------------------------------------------------

export function useThreadHydration(): void {
  useEffect(() => {
    void useThreadRegistryStore.getState().hydrateFromDb();
  }, []);
}
