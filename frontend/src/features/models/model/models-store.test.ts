import { beforeEach, describe, expect, test, vi } from "vitest";
import { useGraphStore } from "@/platform/entities";

import { fetchModelsCatalog } from "../api/models-api";
import { fetchConfiguredProviders, updateProvider } from "@/features/providers/api";
import { useModelsStore } from "./models-store";
import type { CatalogModelsResponse, UarProvider } from "@/types";

vi.mock("../api/models-api", () => ({ fetchModelsCatalog: vi.fn() }));
vi.mock("@/features/providers/api", () => ({
  fetchConfiguredProviders: vi.fn(),
  updateProvider: vi.fn(),
}));

const provider: UarProvider = {
  id: "stub",
  display_name: "Stub",
  base_url: "http://127.0.0.1:9999/v1",
  default_model: "first",
  models: [{ id: "first", enabled: true }],
};

const catalog = {
  stub: {
    display_name: "Stub",
    configured: true,
    models: {
      first: {
        name: "First",
        limit: { context: 8192 },
        tool_call: true,
        reasoning: false,
        modalities: { input: ["text"], output: ["text"] },
        cost: { input: 1, output: 2 },
        benchmarks: [],
      },
      sparse: {},
    },
  },
} as unknown as CatalogModelsResponse;

beforeEach(() => {
  vi.clearAllMocks();
  useGraphStore.setState({ entities: {} });
  useModelsStore.setState({
    configuredProviders: [],
    refreshing: false,
    busyModelKey: null,
    error: null,
  });
  vi.mocked(fetchModelsCatalog).mockResolvedValue(catalog);
  vi.mocked(fetchConfiguredProviders).mockResolvedValue({ providers: [provider] });
  vi.mocked(updateProvider).mockImplementation(async (_id, value) => value);
});

describe("models store", () => {
  test("hydrates catalog rows and supplies safe defaults for missing metadata", async () => {
    await useModelsStore.getState().load();

    expect(useGraphStore.getState().entities.Model?.["stub/first"]).toMatchObject({
      name: "First",
      context: 8192,
      tool_call: true,
    });
    expect(useGraphStore.getState().entities.Model?.["stub/sparse"]).toMatchObject({
      name: "sparse",
      context: 0,
      tool_call: false,
      vision: false,
      cost_input: 0,
      benchmarks: [],
    });
    expect(useModelsStore.getState().configuredProviders).toEqual([provider]);
  });

  test("surfaces catalog load failure", async () => {
    vi.mocked(fetchModelsCatalog).mockRejectedValue(new Error("models unavailable"));

    await expect(useModelsStore.getState().load()).rejects.toThrow("models unavailable");
    expect(useModelsStore.getState()).toMatchObject({
      refreshing: false,
      error: "models unavailable",
    });
  });

  test("adds a model through the provider resource", async () => {
    useModelsStore.setState({ configuredProviders: [provider] });

    await useModelsStore.getState().addModel("stub", { id: "second", enabled: true });

    expect(updateProvider).toHaveBeenCalledWith(
      "stub",
      expect.objectContaining({ models: [{ id: "first", enabled: true }, { id: "second", enabled: true }] }),
    );
    expect(useModelsStore.getState().configuredProviders[0]?.models).toHaveLength(2);
  });

  test("sets the default model", async () => {
    useModelsStore.setState({ configuredProviders: [provider] });

    await useModelsStore.getState().setDefaultModel("stub", "second");

    expect(updateProvider).toHaveBeenCalledWith(
      "stub",
      expect.objectContaining({ default_model: "second" }),
    );
    expect(useModelsStore.getState().configuredProviders[0]?.default_model).toBe("second");
  });

  test("removes a model and clears a matching default", async () => {
    useModelsStore.setState({ configuredProviders: [provider] });

    await useModelsStore.getState().removeModel("stub", "first");

    expect(updateProvider).toHaveBeenCalledWith(
      "stub",
      expect.objectContaining({ models: [], default_model: undefined }),
    );
  });

  test("rolls optimistic model mutations back on failure", async () => {
    useModelsStore.setState({ configuredProviders: [provider] });
    vi.mocked(updateProvider).mockRejectedValue(new Error("update rejected"));

    await expect(
      useModelsStore.getState().addModel("stub", { id: "second" }),
    ).rejects.toThrow("update rejected");
    expect(useModelsStore.getState().configuredProviders).toEqual([provider]);
    expect(useModelsStore.getState()).toMatchObject({
      busyModelKey: null,
      error: "update rejected",
    });
  });
});
