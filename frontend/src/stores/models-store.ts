import { create } from "zustand";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

import type { ModelEntity } from "@/entities/types";
import { fetchModelsCatalog } from "@/services/models-api";
import { fetchConfiguredProviders, updateProvider } from "@/services/providers-api";
import type { CatalogModelsResponse, UarModel, UarProvider } from "@/types";

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

function hydrateCatalog(data: CatalogModelsResponse): void {
  const graph = useGraphStore.getState();
  for (const [providerId, providerData] of Object.entries(data)) {
    for (const [modelId, model] of Object.entries(providerData.models ?? {})) {
      const id = `${providerId}/${modelId}`;
      const entity: ModelEntity & Record<string, unknown> = {
        id,
        name: model.name ?? modelId,
        provider_id: providerId,
        context: model.limit?.context ?? 0,
        tool_call: model.tool_call === true,
        reasoning: model.reasoning === true,
        vision: model.modalities?.input?.includes("image") === true,
        model_id: modelId,
        provider_name: providerData.display_name ?? providerId,
        provider_configured: providerData.configured === true,
        cost_input: model.cost?.input ?? 0,
        cost_output: model.cost?.output ?? 0,
        modalities_input: model.modalities?.input ?? [],
        modalities_output: model.modalities?.output ?? [],
        family: model.family,
        benchmarks: model.benchmarks ?? [],
      };
      graph.upsertEntity("Model", id, entity);
    }
  }
}

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
        const [catalog, configured] = await Promise.all([
          fetchModelsCatalog(),
          fetchConfiguredProviders(),
        ]);
        hydrateCatalog(catalog);
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
      const provider = get().configuredProviders.find((item) => item.id === providerId);
      if (!provider) throw new Error(`Configured provider not found: ${providerId}`);
      const key = `${providerId}/${modelId}`;
      set({ busyModelKey: key });
      try {
        await saveProvider({ ...provider, default_model: modelId });
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
