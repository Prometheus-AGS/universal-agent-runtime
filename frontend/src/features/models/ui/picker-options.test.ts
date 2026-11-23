import { describe, expect, it } from "vitest";

import type { CatalogModel, ConfiguredModel, Provider } from "@/platform/entities";
import {
  catalogModelOptions,
  configuredModelOptions,
  formatContext,
  providerModelOptions,
  providerOptions,
  sourceStatus,
} from "./picker-options";

function catalogModel(providerId: string, modelId: string, extra: Partial<CatalogModel> = {}): CatalogModel {
  return {
    id: `${providerId}/${modelId}`,
    provider_id: providerId,
    model_id: modelId,
    name: modelId,
    provider_name: providerId,
    provider_configured: false,
    context: 0,
    tool_call: false,
    reasoning: false,
    vision: false,
    open_weights: false,
    cost_input: 0,
    cost_output: 0,
    modalities_input: [],
    modalities_output: [],
    family: null,
    benchmarks: [],
    ...extra,
  };
}

describe("catalogModelOptions", () => {
  const models = [
    catalogModel("openai", "gpt-4o", { provider_configured: true }),
    catalogModel("openai", "gpt-3"),
    catalogModel("acme", "x1"),
  ];

  it("groups by provider and disables models that are not enabled on a configured provider", () => {
    const options = catalogModelOptions(models, new Set(["openai/gpt-4o"]));
    expect(options.map((o) => o.value)).toEqual(["acme/x1", "openai/gpt-3", "openai/gpt-4o"]);
    expect(options.map((o) => o.disabled)).toEqual([true, true, false]);
    expect(options[0]!.disabledReason).toBe("Provider not configured");
    expect(options[1]!.disabledReason).toBe("Not enabled");
  });
});

describe("providerModelOptions", () => {
  it("lists a provider's models in raw-id order and honors exclusions", () => {
    const models = [catalogModel("p", "b"), catalogModel("p", "GLM"), catalogModel("p", "a")];
    expect(providerModelOptions(models).map((o) => o.value)).toEqual(["GLM", "a", "b"]);
    expect(providerModelOptions(models, new Set(["a"])).map((o) => o.value)).toEqual(["GLM", "b"]);
  });
});

describe("providerOptions", () => {
  const providers = [{ id: "openai", display_name: "OpenAI" }] as Provider[];

  it("lists every catalog provider when unscoped", () => {
    expect(providerOptions(providers).map((o) => o.value)).toEqual(["openai"]);
  });

  it("keeps a scoped provider the catalog does not know", () => {
    const options = providerOptions(providers, [{ id: "openai" }, { id: "custom", label: "Custom" }]);
    expect(options.map((o) => [o.value, o.label])).toEqual([
      ["openai", "OpenAI"],
      ["custom", "Custom"],
    ]);
  });
});

describe("configuredModelOptions", () => {
  it("keeps the route as the value and shows the context window", () => {
    const [option] = configuredModelOptions([
      {
        id: "openai/gpt-4o",
        provider_id: "openai",
        provider_name: "OpenAI",
        model_id: "gpt-4o",
        display_name: "GPT-4o",
        enabled: true,
        context_window: 128000,
        supports_tools: null,
        supports_vision: null,
      } as ConfiguredModel,
    ]);
    expect(option).toMatchObject({ value: "openai/gpt-4o", badge: "128k", groupId: "openai" });
  });
});

describe("formatContext and sourceStatus", () => {
  it("formats context windows", () => {
    expect(formatContext(1_000_000)).toBe("1M");
    expect(formatContext(null)).toBeUndefined();
  });

  it("is ready with rows even after a failed refresh, loading with none", () => {
    expect(sourceStatus({ loaded: false, error: "boom", count: 3 })).toBe("ready");
    expect(sourceStatus({ loaded: false, error: "boom", count: 0 })).toBe("error");
    expect(sourceStatus({ loaded: false, error: null, count: 0 })).toBe("loading");
    expect(sourceStatus({ loaded: true, error: null, count: 0 })).toBe("ready");
  });
});
