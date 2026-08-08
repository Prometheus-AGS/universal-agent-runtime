import { useGraphStore } from "@/platform/entities";
import { beforeEach, describe, expect, test, vi } from "vitest";
import * as skillsApi from "../api/skills-api";
import { useSkillsAdminStore } from "./skills-admin-store";

vi.mock("../api/skills-api", () => ({
  createSkillApi: vi.fn(),
  deleteSkillApi: vi.fn(),
  fetchSkillsList: vi.fn(),
  importSkillFromDisk: vi.fn(),
  toggleSkillApi: vi.fn(),
  updateSkillApi: vi.fn(),
}));

const skill = { skill_id: "review", title: "Review", enabled: true };

beforeEach(() => {
  vi.resetAllMocks();
  useGraphStore.setState({ entities: {} } as never);
  useSkillsAdminStore.setState({ loading: false, error: null, saving: false, deleting: false, actionSkillId: null });
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
