import { describe, expect, test, vi } from "vitest";

import { summarizePinnedAgents } from "./pinned-agents";

vi.mock("@/platform/entities", () => ({
  graphStore: { getState: () => ({ replaceEntity: vi.fn() }) },
  registerSchema: vi.fn(),
  useGraphStore: vi.fn(),
}));
vi.mock("@/features/agents/api/agents-api", () => ({ fetchAgentsList: vi.fn() }));

describe("summarizePinnedAgents", () => {
  test("lists agents with their own model and counts the rest as followers", () => {
    const summary = summarizePinnedAgents([
      { id: "default", metadata: { title: "Default Assistant" }, explicit_model: null },
      { id: "dawn", metadata: { title: "BDD Agent Dawn" } },
      { id: "cirrus", metadata: { title: "BDD Agent Cirrus" }, explicit_model: "bdd-provider-c/qwen-5-turbo" },
      { id: "aurora", metadata: { title: "BDD Agent Aurora" }, explicit_model: "openai/gpt-5.4-mini" },
    ]);

    expect(summary.pinned.map((agent) => agent.title)).toEqual(["BDD Agent Aurora", "BDD Agent Cirrus"]);
    expect(summary.pinned[1]?.model).toBe("bdd-provider-c/qwen-5-turbo");
    expect(summary.follow_count).toBe(2);
  });

  test("does not count federated agents on either side", () => {
    const summary = summarizePinnedAgents([
      { id: "remote", _type: "federated", explicit_model: "x/y" },
      { id: "local", _type: "runtime" },
    ]);

    expect(summary.pinned).toEqual([]);
    expect(summary.follow_count).toBe(1);
  });
});
