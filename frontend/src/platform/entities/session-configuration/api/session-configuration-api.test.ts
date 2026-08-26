import { afterEach, describe, expect, test, vi } from "vitest";

import {
  decodeAgentSessionConfig,
  decodeSessionPromptCaching,
  fetchAgentSessionConfig,
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
