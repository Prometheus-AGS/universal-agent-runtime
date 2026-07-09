import { create } from "zustand";

import { createAuthKey, deleteAuthKey, fetchAuthKeys } from "@/services/auth-api";
import type { UarApiKey } from "@/types";

interface AuthKeysState {
  keys: UarApiKey[];
  loading: boolean;
  error: string | null;
  saving: boolean;
  revoking: boolean;
}

interface AuthKeysActions {
  load: () => Promise<void>;
  createKey: (name: string) => Promise<{ key?: string; raw_key?: string } | null>;
  revokeKey: (id: string) => Promise<void>;
}

export const useAuthKeysStore = create<AuthKeysState & AuthKeysActions>((set, get) => ({
  keys: [],
  loading: true,
  error: null,
  saving: false,
  revoking: false,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const keys = await fetchAuthKeys();
      set({ keys, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },

  createKey: async (name: string) => {
    set({ saving: true, error: null });
    try {
      const data = await createAuthKey(name);
      await get().load();
      return data;
    } catch (e) {
      set({ error: (e as Error).message });
      return null;
    } finally {
      set({ saving: false });
    }
  },

  revokeKey: async (id: string) => {
    set({ revoking: true, error: null });
    try {
      await deleteAuthKey(id);
      set((s) => ({ keys: s.keys.filter((k) => k.id !== id), revoking: false }));
    } catch (e) {
      set({ error: (e as Error).message, revoking: false });
    }
  },
}));
