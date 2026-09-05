import { useGraphStore } from "@/platform/entities";
import { beforeEach, describe, expect, test, vi } from "vitest";

import * as agentsApi from "../api/agents-api";
import { fetchKnowledgeBases } from "@/features/knowledge/api";
import { fetchSkillsList } from "@/features/skills/api";
import { fetchToolsDiscovery } from "@/features/tools/api";
import { useAgentsAdminStore } from "./agents-admin-store";

vi.mock("../api/agents-api", () => ({
  createAgent: vi.fn(),
  deleteAgent: vi.fn(),
  fetchAgentsList: vi.fn(),
  generateAgentDefinition: vi.fn(),
  patchAgent: vi.fn(),
  updateAgentFull: vi.fn(),
}));
vi.mock("@/features/knowledge/api", () => ({ fetchKnowledgeBases: vi.fn() }));
vi.mock("@/features/skills/api", () => ({ fetchSkillsList: vi.fn() }));
vi.mock("@/features/tools/api", () => ({ fetchToolsDiscovery: vi.fn() }));

const agent = {
  id: "agent-1",
  version: "1.0",
  kind: "agent",
  metadata: { title: "Release Agent", description: "Certifies releases", tags: [] },
};

describe("agents admin store", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    useGraphStore.setState({ entities: {} } as never);
    useAgentsAdminStore.setState({
      loading: false,
      error: null,
      availableSkills: [],
      availableTools: [],
      availableKnowledgeBases: [],
      capabilitiesLoading: false,
      capabilitiesError: null,
    });
    vi.mocked(agentsApi.fetchAgentsList).mockResolvedValue([]);
  });

  test("reconciles authoritative agents and removes stale graph rows", async () => {
    useGraphStore.getState().upsertEntity("Agent", "stale", { id: "stale" });
    vi.mocked(agentsApi.fetchAgentsList).mockResolvedValue([agent as never]);

    await useAgentsAdminStore.getState().load();

    expect(useGraphStore.getState().entities.Agent?.stale).toBeUndefined();
    expect(useGraphStore.getState().entities.Agent?.[agent.id]).toMatchObject(agent);
    expect(useAgentsAdminStore.getState()).toMatchObject({ loading: false, error: null });
  });

  test("surfaces load and authorization failures", async () => {
    vi.mocked(agentsApi.fetchAgentsList).mockRejectedValue(new Error("403 agents denied"));

    await expect(useAgentsAdminStore.getState().load()).rejects.toThrow("403 agents denied");
    expect(useAgentsAdminStore.getState()).toMatchObject({ loading: false, error: "403 agents denied" });
  });

  test("creates, updates, patches, and deletes only after service success", async () => {
    vi.mocked(agentsApi.fetchAgentsList).mockResolvedValue([agent as never]);
    await useAgentsAdminStore.getState().save(undefined, agent);
    await useAgentsAdminStore.getState().save(agent.id, agent);
    await useAgentsAdminStore.getState().patch(agent.id, { status: "active" });
    await useAgentsAdminStore.getState().remove(agent.id);

    expect(agentsApi.createAgent).toHaveBeenCalledWith(agent);
    expect(agentsApi.updateAgentFull).not.toHaveBeenCalled();
    expect(agentsApi.patchAgent).toHaveBeenCalledWith(agent.id, agent);
    expect(agentsApi.patchAgent).toHaveBeenCalledWith(agent.id, { status: "active" });
    expect(agentsApi.deleteAgent).toHaveBeenCalledWith(agent.id);
    expect(useGraphStore.getState().entities.Agent?.[agent.id]).toBeUndefined();
  });

  test("retains the agent when deletion fails", async () => {
    useGraphStore.getState().upsertEntity("Agent", agent.id, agent);
    vi.mocked(agentsApi.deleteAgent).mockRejectedValue(new Error("delete denied"));

    await expect(useAgentsAdminStore.getState().remove(agent.id)).rejects.toThrow("delete denied");
    expect(useGraphStore.getState().entities.Agent?.[agent.id]).toMatchObject(agent);
  });

  test("loads editor capabilities in parallel and reports failure", async () => {
    vi.mocked(fetchSkillsList).mockResolvedValue([{ skill_id: "review" }] as never);
    vi.mocked(fetchToolsDiscovery).mockResolvedValue({ data: { tools: [{ name: "search" }], built_in_tools: [] } } as never);
    vi.mocked(fetchKnowledgeBases).mockResolvedValue([{ id: "kb-1", name: "Release" }] as never);

    await useAgentsAdminStore.getState().loadCapabilities();
    expect(useAgentsAdminStore.getState()).toMatchObject({
      capabilitiesLoading: false,
      capabilitiesError: null,
      availableSkills: [{ skill_id: "review" }],
      availableTools: [{ name: "search" }],
      availableKnowledgeBases: [{ id: "kb-1", name: "Release" }],
    });

    vi.mocked(fetchSkillsList).mockRejectedValue(new Error("capabilities unavailable"));
    await expect(useAgentsAdminStore.getState().loadCapabilities()).rejects.toThrow("capabilities unavailable");
    expect(useAgentsAdminStore.getState().capabilitiesError).toBe("capabilities unavailable");
  });
});
