import { describe, expect, test } from "vitest";

import { explicitAgentModel, isPinnedAgent } from "./explicit-model";

describe("explicit agent model", () => {
  test("is the runtime's explicit_model", () => {
    expect(explicitAgentModel({ explicit_model: "openai/gpt-5.4-mini" })).toBe("openai/gpt-5.4-mini");
    expect(isPinnedAgent({ explicit_model: "bdd-provider-c/qwen-5-turbo" })).toBe(true);
  });

  test("an agent with no model, a blank model or no field follows the system default", () => {
    expect(isPinnedAgent({ explicit_model: null })).toBe(false);
    expect(isPinnedAgent({ explicit_model: "  " })).toBe(false);
    expect(isPinnedAgent({})).toBe(false);
    expect(isPinnedAgent(undefined)).toBe(false);
  });

  test("a federated agent is never counted", () => {
    expect(isPinnedAgent({ explicit_model: "a/b", _type: "federated" })).toBe(false);
  });
});
