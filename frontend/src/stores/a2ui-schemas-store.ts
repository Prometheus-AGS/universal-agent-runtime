import { create } from "zustand";

import { fetchA2uiSchemas } from "@/services/a2ui-api";

interface A2uiSchemasState {
  schemas: unknown[];
  loading: boolean;
  error: string | null;
}

interface A2uiSchemasActions {
  load: () => Promise<void>;
}

export const useA2uiSchemasStore = create<A2uiSchemasState & A2uiSchemasActions>((set) => ({
  schemas: [],
  loading: true,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const data = (await fetchA2uiSchemas()) as unknown[] | { schemas?: unknown[] };
      const list = Array.isArray(data) ? data : (data?.schemas ?? []);
      set({ schemas: list, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },
}));
