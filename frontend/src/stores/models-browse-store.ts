import { create } from "zustand";

import { fetchModelsCatalog } from "@/services/models-api";
import type { CatalogModelsResponse } from "@/types";

interface ModelsBrowseState {
  response: CatalogModelsResponse;
  loading: boolean;
  error: string | null;
}

interface ModelsBrowseActions {
  load: () => Promise<void>;
}

export const useModelsBrowseStore = create<ModelsBrowseState & ModelsBrowseActions>((set) => ({
  response: {},
  loading: true,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const data = await fetchModelsCatalog();
      set({ response: data, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },
}));
