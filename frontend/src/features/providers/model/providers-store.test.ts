import { beforeEach, describe, expect, test, vi } from "vitest";
import { useGraphStore } from "@/platform/entities";

import {
  createProvider,
  deleteProvider,
  fetchCatalog,
  fetchConfiguredProviders,
  fetchProviderHealth,
  setDefaultProvider,
  updateProvider,
} from "../api/providers-api";
import { useProvidersStore } from "./providers-store";

vi.mock("../api/providers-api", () => ({
  createProvider: vi.fn(),
  deleteProvider: vi.fn(),
  fetchCatalog: vi.fn(),
  fetchConfiguredProviders: vi.fn(),
  fetchProviderHealth: vi.fn(),
  setDefaultProvider: vi.fn(),
  updateProvider: vi.fn(),
}));

const catalogProvider = {
  id: "stub",
  display_name: "Stub Provider",
  base_url: "http://127.0.0.1:9999/v1",
  model_count: 1,
  configured: false,
  endpoints: ["openai"],
  status: "available" as const,
};

const configuredProvider = {
  id: "stub",
  display_name: "Stub Provider",
  base_url: "http://127.0.0.1:9999/v1",
  default_model: "stub-model",
  enabled: true,
  models: [{ id: "stub-model", enabled: true }],
};

function primeLoads(configured = false): void {
  vi.mocked(fetchCatalog).mockResolvedValue({
    provider_count: 1,
    model_count: 1,
    providers: [catalogProvider],
  });
  vi.mocked(fetchConfiguredProviders).mockResolvedValue({
    providers: configured ? [configuredProvider] : [],
    default_id: configured ? "stub" : undefined,
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  useGraphStore.setState({ entities: {} });
  useProvidersStore.setState({
    loaded: false,
    refreshing: false,
    saving: false,
    removingId: null,
    error: null,
    healthByProvider: {},
    healthLoading: false,
    healthError: null,
  });
});

describe("providers store", () => {
  test("loads catalog, configured state, default, and diagnostic status", async () => {
    primeLoads(true);

    await useProvidersStore.getState().load();

    const graph = useGraphStore.getState().entities;
    expect(graph.Provider?.stub).toMatchObject({
      configured: true,
      status: "available",
      model_count: 1,
    });
    expect(graph.ProviderMeta?.current).toMatchObject({
      default_id: "stub",
      default_model: "stub-model",
    });
    expect(useProvidersStore.getState()).toMatchObject({
      loaded: true,
      refreshing: false,
      error: null,
    });
  });

  test("surfaces load failure without leaving loading active", async () => {
    vi.mocked(fetchCatalog).mockRejectedValue(new Error("catalog unavailable"));
    vi.mocked(fetchConfiguredProviders).mockResolvedValue({ providers: [] });

    await expect(useProvidersStore.getState().load()).rejects.toThrow("catalog unavailable");
    expect(useProvidersStore.getState()).toMatchObject({
      refreshing: false,
      error: "catalog unavailable",
    });
  });

  test("configures a provider and reconciles the authoritative graph", async () => {
    primeLoads(false);
    vi.mocked(createProvider).mockResolvedValue(new Response(null, { status: 201 }));

    await useProvidersStore.getState().configure({
      provider: catalogProvider,
      apiKey: "secret-value",
      baseUrl: configuredProvider.base_url,
    });

    expect(createProvider).toHaveBeenCalledWith(
      expect.objectContaining({ id: "stub", api_key: "secret-value", enabled: true }),
    );
    expect(useGraphStore.getState().entities.Provider?.stub).toMatchObject({ configured: true });
    expect(useProvidersStore.getState()).toMatchObject({ saving: false, error: null });
  });

  test("surfaces configure failure and never writes the submitted secret to state", async () => {
    primeLoads(false);
    vi.mocked(createProvider).mockResolvedValue(
      new Response("credential rejected", { status: 422 }),
    );

    await expect(
      useProvidersStore.getState().configure({
        provider: catalogProvider,
        apiKey: "do-not-retain",
        baseUrl: configuredProvider.base_url,
      }),
    ).rejects.toThrow("credential rejected");
    expect(useProvidersStore.getState()).toMatchObject({
      saving: false,
      error: "credential rejected",
    });
    expect(JSON.stringify(useProvidersStore.getState())).not.toContain("do-not-retain");
  });

  test("updates an existing provider before reconciling the authoritative graph", async () => {
    primeLoads(true);
    vi.mocked(updateProvider).mockResolvedValue(configuredProvider);

    await useProvidersStore.getState().configure({
      provider: catalogProvider,
      apiKey: "replacement-secret",
      baseUrl: configuredProvider.base_url,
    });

    expect(updateProvider).toHaveBeenCalledWith(
      "stub",
      expect.objectContaining({
        id: "stub",
        api_key: "replacement-secret",
        enabled: true,
      }),
    );
    expect(useGraphStore.getState().entities.Provider?.stub).toMatchObject({ configured: true });
  });

  test("sets default optimistically and retains it on success", async () => {
    primeLoads(true);
    useGraphStore.getState().upsertEntity("ProviderMeta", "current", {
      id: "current",
      default_id: null,
      default_model: null,
    });
    vi.mocked(setDefaultProvider).mockResolvedValue();

    await useProvidersStore.getState().setDefault("stub");

    expect(setDefaultProvider).toHaveBeenCalledWith("stub");
    expect(useGraphStore.getState().entities.ProviderMeta?.current).toMatchObject({
      default_id: "stub",
      default_model: "stub-model",
    });
  });

  test("rolls default back when persistence fails", async () => {
    useGraphStore.getState().upsertEntity("ProviderMeta", "current", {
      id: "current",
      default_id: "old",
    });
    vi.mocked(setDefaultProvider).mockRejectedValue(new Error("default rejected"));

    await expect(useProvidersStore.getState().setDefault("stub")).rejects.toThrow(
      "default rejected",
    );
    expect(useGraphStore.getState().entities.ProviderMeta?.current).toMatchObject({
      default_id: "old",
    });
  });

  test("removes a provider on success and restores it on failure", async () => {
    const graph = useGraphStore.getState();
    graph.upsertEntity("Provider", "stub", catalogProvider);
    vi.mocked(deleteProvider).mockResolvedValue();

    await useProvidersStore.getState().remove("stub");
    expect(useGraphStore.getState().entities.Provider?.stub).toBeUndefined();

    graph.upsertEntity("Provider", "stub", catalogProvider);
    vi.mocked(deleteProvider).mockRejectedValue(new Error("delete rejected"));
    await expect(useProvidersStore.getState().remove("stub")).rejects.toThrow(
      "delete rejected",
    );
    expect(useGraphStore.getState().entities.Provider?.stub).toMatchObject({ id: "stub" });
  });

  test("normalizes health success and surfaces health failure", async () => {
    vi.mocked(fetchProviderHealth).mockResolvedValue({
      providers: {
        stub: { healthy: false, consecutive_errors: 2, cooldown_remaining_secs: 15 },
      },
    });

    await useProvidersStore.getState().loadHealth();
    expect(useGraphStore.getState().entities.RuntimeProviderHealth?.stub).toMatchObject({
      provider_id: "stub",
      status: "degraded",
      error: "2 consecutive error(s)",
    });

    vi.mocked(fetchProviderHealth).mockRejectedValue(new Error("health unavailable"));
    await expect(useProvidersStore.getState().loadHealth()).rejects.toThrow(
      "health unavailable",
    );
    expect(useProvidersStore.getState()).toMatchObject({
      healthLoading: false,
      healthError: "health unavailable",
    });
  });
});
