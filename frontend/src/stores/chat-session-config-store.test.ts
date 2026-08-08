import { beforeEach, describe, expect, test, vi } from "vitest";

import * as agentFetcher from "@/features/agents/model";
import * as providersApi from "@/features/providers/api";
import * as sessionApi from "@/services/session-config-api";
import { useChatSessionConfigStore } from "@/stores/chat-session-config-store";

vi.mock("@/features/agents/model", () => ({ loadAgentsIntoGraph: vi.fn() }));
vi.mock("@/features/providers/api", () => ({ fetchConfiguredProviders: vi.fn() }));
vi.mock("@/services/session-config-api", () => ({ saveSessionAgentConfig: vi.fn() }));

beforeEach(() => {
  vi.resetAllMocks();
  useChatSessionConfigStore.setState({
    modelLabel: null,
    loadingAgents: false,
    saving: false,
    error: null,
  });
});

describe("chat session configuration store", () => {
  test("loads the configured default provider/model label", async () => {
    vi.mocked(providersApi.fetchConfiguredProviders).mockResolvedValue({
      default_id: "openai",
      providers: [{ id: "openai", models: [{ id: "gpt-5" }] }],
    } as never);

    await useChatSessionConfigStore.getState().loadDefaultModelLabel();
    expect(useChatSessionConfigStore.getState().modelLabel).toBe("openai/gpt-5");
  });

  test("hydrates agents and surfaces transport failures", async () => {
    vi.mocked(agentFetcher.loadAgentsIntoGraph).mockRejectedValue(new Error("agents unavailable"));
    await useChatSessionConfigStore.getState().loadAgents();
    expect(useChatSessionConfigStore.getState().error).toBe("agents unavailable");
    expect(useChatSessionConfigStore.getState().loadingAgents).toBe(false);
  });

  test("persists typed session intent and keeps the panel open on denial", async () => {
    vi.mocked(sessionApi.saveSessionAgentConfig).mockRejectedValue(new Error("403 denied"));
    await expect(
      useChatSessionConfigStore.getState().save("thread/1", { agent_id: "release" }),
    ).resolves.toBe(false);
    expect(sessionApi.saveSessionAgentConfig).toHaveBeenCalledWith("thread/1", {
      agent_id: "release",
    });
    expect(useChatSessionConfigStore.getState().error).toBe("403 denied");
  });
});
