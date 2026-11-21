import { create } from "zustand";
import {
  loadProviderHealth,
  loadProviderRegistry,
  setSystemDefaultProvider,
} from "@/platform/entities";

import { optimisticRemove } from "@/lib/realtime/optimistic";
import {
  createProvider,
  deleteProvider,
  fetchConfiguredProviders,
  updateProvider,
  type ProviderHealthEntry,
} from "../api/providers-api";
import type { CatalogProviderSummary } from "@/types";

export interface ConfigureProviderInput {
  provider: CatalogProviderSummary;
  apiKey: string;
  baseUrl: string;
}

interface ProvidersState {
  loaded: boolean;
  refreshing: boolean;
  saving: boolean;
  removingId: string | null;
  error: string | null;
  healthByProvider: Record<string, ProviderHealthEntry>;
  healthLoading: boolean;
  healthError: string | null;
}

interface ProvidersActions {
  load: () => Promise<void>;
  configure: (input: ConfigureProviderInput) => Promise<void>;
  setDefault: (providerId: string) => Promise<void>;
  remove: (providerId: string) => Promise<void>;
  loadHealth: () => Promise<void>;
  clearError: () => void;
}

export type ProvidersStore = ProvidersState & ProvidersActions;

export const useProvidersStore = create<ProvidersStore>((set) => ({
  loaded: false,
  refreshing: false,
  saving: false,
  removingId: null,
  error: null,
  healthByProvider: {},
  healthLoading: false,
  healthError: null,

  load: async () => {
    set({ refreshing: true, error: null });
    try {
      await loadProviderRegistry();
      set({ loaded: true });
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ refreshing: false });
    }
  },

  configure: async ({ provider, apiKey, baseUrl }) => {
    set({ saving: true, error: null });
    try {
      const configured = await fetchConfiguredProviders();
      const existing = configured.providers.find((candidate) => candidate.id === provider.id);
      const next = {
        ...(existing ?? {}),
        id: provider.id,
        display_name: provider.display_name ?? provider.id,
        base_url: baseUrl.trim() || provider.base_url || "",
        api_key: apiKey || undefined,
        protocol: existing?.protocol ?? "auto",
        enabled: true,
      };
      if (existing) {
        await updateProvider(provider.id, next);
      } else {
        const response = await createProvider(next);
        if (!response.ok) {
          const detail = await response.text().catch(() => "");
          throw new Error(detail || `Configure provider failed: ${response.status}`);
        }
      }
      await loadProviderRegistry();
      set({ loaded: true });
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ saving: false });
    }
  },

  setDefault: async (providerId) => {
    set({ error: null });
    try {
      // One write path: the same domain action the system default picker uses. It publishes the
      // default pair to the graph once, after the server confirmed it.
      await setSystemDefaultProvider(providerId);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  remove: async (providerId) => {
    set({ removingId: providerId, error: null });
    try {
      await optimisticRemove("Provider", providerId, () => deleteProvider(providerId));
    } catch (error) {
      set({ error: `Failed to remove provider: ${(error as Error).message}` });
      throw error;
    } finally {
      set({ removingId: null });
    }
  },

  loadHealth: async () => {
    set({ healthLoading: true, healthError: null });
    try {
      const response = await loadProviderHealth();
      set({ healthByProvider: response.providers });
    } catch (error) {
      set({ healthError: (error as Error).message });
      throw error;
    } finally {
      set({ healthLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
