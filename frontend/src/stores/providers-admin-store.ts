import { create } from "zustand";

import {
  createProvider,
  deleteProvider,
  fetchCatalog,
  fetchConfiguredProviders,
  setDefaultProvider,
} from "@/services/providers-api";
import { onSettingsChanged } from "@/services/settings-change-bus";
import type { CatalogProviderSummary, UarProvider } from "@/types";

interface ProvidersAdminState {
  catalog: CatalogProviderSummary[];
  configured: UarProvider[];
  defaultId: string | undefined;
  loading: boolean;
  error: string | null;
  saving: boolean;
  removing: string | null;
}

interface ProvidersAdminActions {
  load: () => Promise<void>;
  configureProvider: (args: {
    addTarget: CatalogProviderSummary;
    apiKey: string;
    baseUrl: string;
  }) => Promise<void>;
  setDefault: (id: string) => Promise<void>;
  removeProvider: (id: string) => Promise<void>;
  clearError: () => void;
}

export type ProvidersAdminStore = ProvidersAdminState & ProvidersAdminActions;

export const useProvidersAdminStore = create<ProvidersAdminStore>(
  (set, get) => ({
    catalog: [],
    configured: [],
    defaultId: undefined,
    loading: true,
    error: null,
    saving: false,
    removing: null,

    clearError: () => set({ error: null }),

    load: async () => {
      set({ loading: true, error: null });
      try {
        const [catalogData, providersData] = await Promise.all([
          fetchCatalog(),
          fetchConfiguredProviders(),
        ]);

        const configuredProviders = providersData.providers ?? [];
        const configuredIds = new Set(configuredProviders.map((p) => p.id));

        // Merge configured status into catalog entries
        const mergedCatalog = catalogData.providers.map((entry) =>
          configuredIds.has(entry.id) ? { ...entry, configured: true } : entry,
        );

        // Add any configured providers not present in the catalog
        for (const cp of configuredProviders) {
          if (!mergedCatalog.some((c) => c.id === cp.id)) {
            mergedCatalog.push({
              id: cp.id,
              display_name: cp.display_name ?? cp.id,
              base_url: cp.base_url,
              model_count: cp.models?.length ?? 0,
              configured: true,
              endpoints: [],
            });
          }
        }

        set({
          catalog: mergedCatalog,
          configured: configuredProviders,
          defaultId: providersData.default_id,
          loading: false,
        });
      } catch (e) {
        set({ error: (e as Error).message, loading: false });
      }
    },

    configureProvider: async ({ addTarget, apiKey, baseUrl }) => {
      set({ saving: true, error: null });
      try {
        const resolvedBase = (baseUrl.trim() || addTarget.base_url) ?? "";
        const body = {
          id: addTarget.id,
          display_name: addTarget.display_name ?? "",
          base_url: resolvedBase,
          api_key: apiKey || undefined,
          protocol: "auto",
          enabled: true,
        };
        const res = await createProvider(body);
        if (!res.ok && res.status !== 409) throw new Error(`${res.status}`);
        await get().load();
      } catch (e) {
        set({ error: (e as Error).message });
        throw e;
      } finally {
        set({ saving: false });
      }
    },

    setDefault: async (id: string) => {
      try {
        await setDefaultProvider(id);
        set({ defaultId: id });
      } catch {
        /* ignore */
      }
    },

    removeProvider: async (id: string) => {
      set({ removing: id, error: null });
      try {
        await deleteProvider(id);
        await get().load();
      } catch {
        /* ignore */
      } finally {
        set({ removing: null });
      }
    },
  }),
);

if (typeof window !== "undefined") {
  onSettingsChanged((detail) => {
    if (detail.impact === "providers" || detail.impact === "llm") {
      void useProvidersAdminStore.getState().load();
    }
  });
}
