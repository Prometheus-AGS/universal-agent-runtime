import { create } from "zustand";
import { loadModelCatalog, setSystemDefaultModel } from "@/platform/entities";

import { fetchConfiguredProviders, updateProvider } from "@/features/providers/api";
import type { UarModel, UarProvider } from "@/types";

interface ModelsState {
  configuredProviders: UarProvider[];
  refreshing: boolean;
  busyModelKey: string | null;
  error: string | null;
}

interface ModelsActions {
  load: () => Promise<void>;
  addModel: (providerId: string, model: UarModel) => Promise<void>;
  setDefaultModel: (providerId: string, modelId: string) => Promise<void>;
  removeModel: (providerId: string, modelId: string) => Promise<void>;
  clearError: () => void;
}

export type ModelsStore = ModelsState & ModelsActions;

export const useModelsStore = create<ModelsStore>((set, get) => {
  const saveProvider = async (next: UarProvider): Promise<void> => {
    const snapshot = get().configuredProviders;
    set({
      configuredProviders: snapshot.map((provider) =>
        provider.id === next.id ? next : provider,
      ),
      error: null,
    });
    try {
      const saved = await updateProvider(next.id, next);
      set((state) => ({
        configuredProviders: state.configuredProviders.map((provider) =>
          provider.id === saved.id ? saved : provider,
        ),
      }));
    } catch (error) {
      set({ configuredProviders: snapshot, error: (error as Error).message });
      throw error;
    }
  };

  return {
    configuredProviders: [],
    refreshing: false,
    busyModelKey: null,
    error: null,

    load: async () => {
      set({ refreshing: true, error: null });
      try {
        // The catalog is ingested by one domain action; a failed fetch leaves the graph unchanged.
        const [configured] = await Promise.all([fetchConfiguredProviders(), loadModelCatalog()]);
        set({ configuredProviders: configured.providers });
      } catch (error) {
        set({ error: (error as Error).message });
        throw error;
      } finally {
        set({ refreshing: false });
      }
    },

    addModel: async (providerId, model) => {
      const provider = get().configuredProviders.find((item) => item.id === providerId);
      if (!provider) throw new Error(`Configured provider not found: ${providerId}`);
      const key = `${providerId}/${model.id}`;
      set({ busyModelKey: key });
      try {
        await saveProvider({ ...provider, models: [...(provider.models ?? []), model] });
      } finally {
        set({ busyModelKey: null });
      }
    },

    setDefaultModel: async (providerId, modelId) => {
      const key = `${providerId}/${modelId}`;
      set({ busyModelKey: key, error: null });
      try {
        // The star changes the system default: the same domain action as the Providers page and
        // the system default picker. It saves the provider's default model, then the default
        // provider, and publishes the pair to the graph once.
        const result = await setSystemDefaultModel(providerId, modelId);
        set((state) => ({
          configuredProviders: state.configuredProviders.map((provider) =>
            provider.id === result.provider.id ? result.provider : provider,
          ),
        }));
      } catch (error) {
        set({ error: (error as Error).message });
        throw error;
      } finally {
        set({ busyModelKey: null });
      }
    },

    removeModel: async (providerId, modelId) => {
      const provider = get().configuredProviders.find((item) => item.id === providerId);
      if (!provider) throw new Error(`Configured provider not found: ${providerId}`);
      const key = `${providerId}/${modelId}`;
      set({ busyModelKey: key });
      try {
        await saveProvider({
          ...provider,
          models: (provider.models ?? []).filter((model) => model.id !== modelId),
          default_model:
            provider.default_model === modelId ? undefined : provider.default_model,
        });
      } finally {
        set({ busyModelKey: null });
      }
    },

    clearError: () => set({ error: null }),
  };
});
