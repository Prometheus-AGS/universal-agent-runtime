import { create } from "zustand";
import { useGraphStore } from "@/platform/entities";

import {
  createSkillApi,
  deleteSkillApi,
  fetchSkillsList,
  importSkillFromDisk,
  toggleSkillApi,
  updateSkillApi,
  type ImportSkillResult,
} from "../api/skills-api";
import type { UarSkill } from "@/types";

interface SkillsAdminStore {
  loading: boolean;
  error: string | null;
  saving: boolean;
  deleting: boolean;
  actionSkillId: string | null;
  load: () => Promise<void>;
  toggle: (skill: UarSkill, enabled: boolean) => Promise<void>;
  remove: (skill: UarSkill) => Promise<void>;
  save: (editingId: string | null, createBody: unknown, updateBody: unknown) => Promise<void>;
  parseImport: (path: string) => Promise<ImportSkillResult>;
  importParsed: (result: ImportSkillResult) => Promise<void>;
  clearError: () => void;
}

function reconcileSkills(skills: UarSkill[]) {
  const graph = useGraphStore.getState();
  const ids = new Set(skills.map((skill) => skill.skill_id));
  for (const id of Object.keys(graph.entities.Skill ?? {})) {
    if (!ids.has(id)) graph.removeEntity("Skill", id);
  }
  for (const skill of skills) {
    graph.upsertEntity("Skill", skill.skill_id, skill as unknown as Record<string, unknown>);
  }
}

export const useSkillsAdminStore = create<SkillsAdminStore>((set, get) => ({
  loading: false,
  error: null,
  saving: false,
  deleting: false,
  actionSkillId: null,
  clearError: () => set({ error: null }),
  load: async () => {
    set({ loading: true, error: null });
    try {
      reconcileSkills(await fetchSkillsList());
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ loading: false });
    }
  },
  toggle: async (skill, enabled) => {
    set({ actionSkillId: skill.skill_id, error: null });
    const graph = useGraphStore.getState();
    graph.upsertEntity("Skill", skill.skill_id, { enabled });
    try {
      await toggleSkillApi(skill.skill_id, enabled);
    } catch (error) {
      graph.upsertEntity("Skill", skill.skill_id, { enabled: skill.enabled !== false });
      set({ error: `Failed to toggle skill: ${(error as Error).message}` });
      throw error;
    } finally {
      set({ actionSkillId: null });
    }
  },
  remove: async (skill) => {
    set({ deleting: true, error: null });
    try {
      await deleteSkillApi(skill.skill_id);
      useGraphStore.getState().removeEntity("Skill", skill.skill_id);
    } catch (error) {
      set({ error: `Failed to delete skill: ${(error as Error).message}` });
      throw error;
    } finally {
      set({ deleting: false });
    }
  },
  save: async (editingId, createBody, updateBody) => {
    set({ saving: true, error: null });
    try {
      if (editingId) await updateSkillApi(editingId, updateBody);
      else await createSkillApi(createBody);
      await get().load();
    } catch (error) {
      set({ error: `Failed to save skill: ${(error as Error).message}` });
      throw error;
    } finally {
      set({ saving: false });
    }
  },
  parseImport: importSkillFromDisk,
  importParsed: async ({ parsed }) => {
    await createSkillApi({
      name: parsed.name,
      version: parsed.version,
      description: parsed.description,
      triggers: parsed.triggers,
      prompt_overlay: parsed.prompt_overlay,
      preferred_tools: [],
      enabled: true,
    });
    await get().load();
  },
}));
