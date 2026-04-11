import { create } from "zustand";

import { createProvider, deleteProvider, fetchCatalog, fetchConfiguredProviders, setDefaultProvider } from "@/services/providers-api";
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

export const useProvidersAdminStore = create<ProvidersAdminStore>((set, get) => ({
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
      set({
        catalog: catalogData.providers,
        configured: providersData.providers ?? [],
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
}));
