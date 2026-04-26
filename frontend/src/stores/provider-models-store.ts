import { create } from "zustand";

import {
  fetchModelsCatalog,
  modelsRowsForProvider,
  type ProviderModelRow,
} from "@/services/models-api";
import { onSettingsChanged } from "@/services/settings-change-bus";

interface ProviderModelsState {
  /** Models shown for the last completed load for this provider id */
  rowsByProvider: Record<string, ProviderModelRow[]>;
  loadingProviderId: string | null;
}

interface ProviderModelsActions {
  loadForProvider: (providerId: string) => Promise<void>;
}

type ProviderModelsStore = ProviderModelsState & ProviderModelsActions;

export const useProviderModelsStore = create<ProviderModelsStore>((set) => ({
  rowsByProvider: {},
  loadingProviderId: null,

  loadForProvider: async (providerId: string) => {
    set({ loadingProviderId: providerId });
    try {
      const data = await fetchModelsCatalog();
      const rows = modelsRowsForProvider(data, providerId);
      set((s) => ({
        rowsByProvider: { ...s.rowsByProvider, [providerId]: rows },
        loadingProviderId: null,
      }));
    } catch {
      set((s) => ({
        rowsByProvider: { ...s.rowsByProvider, [providerId]: [] },
        loadingProviderId: null,
      }));
    }
  },
}));

if (typeof window !== "undefined") {
  onSettingsChanged((detail) => {
    if (detail.impact !== "providers" && detail.impact !== "llm") return;
    for (const providerId of Object.keys(
      useProviderModelsStore.getState().rowsByProvider,
    )) {
      void useProviderModelsStore.getState().loadForProvider(providerId);
    }
  });
}
