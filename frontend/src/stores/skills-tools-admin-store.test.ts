import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { beforeEach, describe, expect, test, vi } from "vitest";

import * as skillsApi from "@/services/skills-api";
import * as toolsApi from "@/services/tools-api";
import { useSkillsAdminStore } from "@/stores/skills-admin-store";
import { useToolsAdminStore } from "@/stores/tools-admin-store";

vi.mock("@/services/skills-api", () => ({
  createSkillApi: vi.fn(),
  deleteSkillApi: vi.fn(),
  fetchSkillsList: vi.fn(),
  importSkillFromDisk: vi.fn(),
  toggleSkillApi: vi.fn(),
  updateSkillApi: vi.fn(),
}));
vi.mock("@/services/tools-api", () => ({
  executeTool: vi.fn(),
  fetchToolsDiscovery: vi.fn(),
}));

const skill = { skill_id: "review", title: "Review", enabled: true };

beforeEach(() => {
  vi.resetAllMocks();
  useGraphStore.setState({ entities: {} } as never);
  useSkillsAdminStore.setState({ loading: false, error: null, saving: false, deleting: false, actionSkillId: null });
  useToolsAdminStore.setState({ loading: false, error: null, executing: false, executionError: null });
  vi.mocked(skillsApi.fetchSkillsList).mockResolvedValue([]);
});

describe("skills admin store", () => {
  test("reconciles skills and rolls back a failed toggle", async () => {
    vi.mocked(skillsApi.fetchSkillsList).mockResolvedValue([skill as never]);
    await useSkillsAdminStore.getState().load();
    vi.mocked(skillsApi.toggleSkillApi).mockRejectedValue(new Error("toggle denied"));

    await expect(useSkillsAdminStore.getState().toggle(skill as never, false)).rejects.toThrow("toggle denied");
    expect(useGraphStore.getState().entities.Skill?.review).toMatchObject({ enabled: true });
    expect(useSkillsAdminStore.getState().error).toContain("toggle denied");
  });

  test("retains a skill when deletion fails", async () => {
    useGraphStore.getState().upsertEntity("Skill", "review", skill);
    vi.mocked(skillsApi.deleteSkillApi).mockRejectedValue(new Error("delete denied"));
    await expect(useSkillsAdminStore.getState().remove(skill as never)).rejects.toThrow("delete denied");
    expect(useGraphStore.getState().entities.Skill?.review).toMatchObject(skill);
  });
});

describe("tools admin store", () => {
  test("loads discovery and executes through the governed service", async () => {
    vi.mocked(toolsApi.fetchToolsDiscovery).mockResolvedValue({
      tools: [{ name: "search", namespaced_name: "web::search" }],
      built_in_tools: [],
    } as never);
    vi.mocked(toolsApi.executeTool).mockResolvedValue({ result: { ok: true }, duration_ms: 5, success: true });

    await useToolsAdminStore.getState().load();
    await expect(useToolsAdminStore.getState().execute("web::search", { q: "release" })).resolves.toMatchObject({ success: true });
    expect(useGraphStore.getState().entities.Tool?.["web::search"]).toMatchObject({ _ns: "web" });
    expect(toolsApi.executeTool).toHaveBeenCalledWith("web::search", { q: "release" });
  });

  test("surfaces transport execution failures", async () => {
    vi.mocked(toolsApi.executeTool).mockRejectedValue(new Error("MCP transport closed"));
    await expect(useToolsAdminStore.getState().execute("web::search", {})).rejects.toThrow("MCP transport closed");
    expect(useToolsAdminStore.getState()).toMatchObject({ executing: false, executionError: "MCP transport closed" });
  });
});
