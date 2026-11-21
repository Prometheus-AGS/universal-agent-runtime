import { afterEach, describe, expect, test, vi } from "vitest";

import {
  SessionConfigurationSaveError,
  SessionModelNotOfferedError,
  decodeAgentSessionConfig,
  decodeSessionPromptCaching,
  fetchAgentSessionConfig,
  saveAgentSessionConfig,
} from "./session-configuration-api";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("session configuration API", () => {
  test.each([204, 404])(
    "treats %s as an absent owner-scoped configuration",
    async (status) => {
      vi.stubGlobal(
        "fetch",
        vi.fn().mockResolvedValue(new Response(null, { status })),
      );

      await expect(fetchAgentSessionConfig("session/one")).resolves.toBeNull();
    },
  );

  test("decodes missing legacy prompt-caching fields as Inherit", () => {
    expect(
      decodeAgentSessionConfig({
        agent_id: "agent-one",
        model: null,
        tools: null,
        skills: null,
        knowledge_bases: null,
        mcp_servers: null,
        tool_approval: null,
      }).prompt_caching_enabled,
    ).toBeNull();
  });

  test("decodes the authoritative effective value and source", () => {
    expect(
      decodeSessionPromptCaching("session-one", {
        enabled: true,
        source: "user",
        session_override: null,
        user_override: true,
        global_default: false,
      }),
    ).toEqual({
      id: "session-one",
      session_id: "session-one",
      enabled: true,
      source: "user",
      session_override: null,
      user_override: true,
      global_default: false,
    });
  });
});

describe("saveAgentSessionConfig refusal", () => {
  const config = {
    agent_id: "agent-one",
    model: "bdd-provider-b/claude-5-retired",
    tools: null,
    skills: null,
    knowledge_bases: null,
    mcp_servers: null,
    tool_approval: null,
    prompt_caching_enabled: null,
  };

  test("surfaces the runtime's model_not_offered reason as a typed, certain failure", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          JSON.stringify({
            error: "Provider 'bdd-provider-b' does not offer model 'claude-5-retired'",
            code: "model_not_offered",
            provider_id: "bdd-provider-b",
            model_id: "claude-5-retired",
          }),
          { status: 422 },
        ),
      ),
    );

    const failure = await saveAgentSessionConfig("s1", config).catch((error: unknown) => error);

    expect(failure).toBeInstanceOf(SessionModelNotOfferedError);
    const refused = failure as SessionModelNotOfferedError;
    expect(refused.message).toContain("not offered");
    expect(refused.message).toContain("does not offer model 'claude-5-retired'");
    expect(refused.uncertain).toBe(false);
    expect(refused.modelId).toBe("claude-5-retired");
  });

  test("any other 422 stays a plain save failure", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(new Response(JSON.stringify({ error: "bad" }), { status: 422 })),
    );

    const failure = await saveAgentSessionConfig("s1", config).catch((error: unknown) => error);

    expect(failure).toBeInstanceOf(SessionConfigurationSaveError);
    expect(failure).not.toBeInstanceOf(SessionModelNotOfferedError);
  });
});
