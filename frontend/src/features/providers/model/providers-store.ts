import { create } from "zustand";
import {
  serializeKey,
  useGraphStore,
} from "@/platform/entities";

import type { ProviderEntity } from "@/entities/types";
import { optimisticRemove, optimisticUpsert } from "@/lib/realtime/optimistic";
import {
  createProvider,
  deleteProvider,
  fetchCatalog,
  fetchConfiguredProviders,
  fetchProviderHealth,
  setDefaultProvider,
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

async function hydrateProviders(): Promise<void> {
  const [catalog, configured] = await Promise.all([
    fetchCatalog(),
    fetchConfiguredProviders(),
  ]);
  const graph = useGraphStore.getState();
  const configuredIds = new Set(configured.providers.map((provider) => provider.id));
  const providerIds = new Set<string>();

  for (const provider of catalog.providers) {
    providerIds.add(provider.id);
    const entity: ProviderEntity = {
      id: provider.id,
      display_name: provider.display_name,
      base_url: provider.base_url,
      configured: configuredIds.has(provider.id) || provider.configured,
      auth_env_var: provider.auth_env_var,
      endpoints: provider.endpoints,
      model_count: provider.model_count,
      status: provider.status,
      status_detail: provider.status_detail,
    };
    graph.upsertEntity("Provider", provider.id, entity);
  }

  for (const provider of configured.providers) {
    providerIds.add(provider.id);
    if (catalog.providers.some((candidate) => candidate.id === provider.id)) continue;
    graph.upsertEntity("Provider", provider.id, {
      id: provider.id,
      display_name: provider.display_name ?? provider.id,
      base_url: provider.base_url,
      configured: true,
      endpoints: [],
      model_count: provider.models?.length ?? 0,
    });
  }

  const defaultProvider = configured.providers.find(
    (provider) => provider.id === configured.default_id,
  );
  graph.upsertEntity("ProviderMeta", "current", {
    id: "current",
    default_id: configured.default_id ?? null,
    default_model: defaultProvider?.default_model ?? null,
  });
  graph.setListResult(serializeKey(["providers", "", "[]"]), [...providerIds], {
    total: providerIds.size,
  });
}

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
      await hydrateProviders();
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
      await hydrateProviders();
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
      await optimisticUpsert(
        "ProviderMeta",
        "current",
        { id: "current", default_id: providerId, default_model: null },
        () => setDefaultProvider(providerId),
      );
    } catch (error) {
      set({ error: `Failed to set default: ${(error as Error).message}` });
      throw error;
    }
    try {
      await hydrateProviders();
      set({ loaded: true });
    } catch (error) {
      set({
        error: `Default changed, but provider metadata refresh failed: ${(error as Error).message}`,
      });
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
      const response = await fetchProviderHealth();
      set({ healthByProvider: response.providers });
      const graph = useGraphStore.getState();
      const updatedAt = new Date().toISOString();
      for (const [providerId, health] of Object.entries(response.providers)) {
        graph.upsertEntity("RuntimeProviderHealth", providerId, {
          id: providerId,
          provider_id: providerId,
          status: health.healthy
            ? "healthy"
            : health.cooldown_remaining_secs
              ? "degraded"
              : "offline",
          error: health.consecutive_errors
            ? `${health.consecutive_errors} consecutive error(s)`
            : undefined,
          updated_at: updatedAt,
        });
      }
    } catch (error) {
      set({ healthError: (error as Error).message });
      throw error;
    } finally {
      set({ healthLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
