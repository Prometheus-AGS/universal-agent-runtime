import { create } from "zustand";

import {
  createSkillApi,
  deleteSkillApi,
  fetchSkillsList,
  toggleSkillApi,
  updateSkillApi,
} from "@/services/skills-api";
import type { UarSkill } from "@/types";

interface SkillsAdminState {
  skills: UarSkill[];
  loading: boolean;
  error: string | null;
  actionSkillId: string | null;
  saving: boolean;
  deleting: boolean;
}

interface SkillsAdminActions {
  load: () => Promise<void>;
  setError: (msg: string | null) => void;
  toggle: (skill: UarSkill, enabled: boolean) => Promise<void>;
  remove: (skill: UarSkill) => Promise<void>;
  save: (editingId: string | null, createBody: unknown, updateBody: unknown) => Promise<void>;
}

export const useSkillsAdminStore = create<SkillsAdminState & SkillsAdminActions>((set, get) => ({
  skills: [],
  loading: true,
  error: null,
  actionSkillId: null,
  saving: false,
  deleting: false,

  setError: (msg) => set({ error: msg }),

  load: async () => {
    set({ loading: true, error: null });
    try {
      const skills = await fetchSkillsList();
      set({ skills, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },

  toggle: async (skill, enabled) => {
    set({ actionSkillId: skill.skill_id, error: null });
    try {
      await toggleSkillApi(skill.skill_id, enabled);
      set((s) => ({
        skills: s.skills.map((x) => (x.skill_id === skill.skill_id ? { ...x, enabled } : x)),
        actionSkillId: null,
      }));
    } catch (e) {
      set({ error: `Failed to toggle skill: ${(e as Error).message}`, actionSkillId: null });
    }
  },

  remove: async (skill) => {
    set({ deleting: true, error: null });
    try {
      await deleteSkillApi(skill.skill_id);
      set((s) => ({
        skills: s.skills.filter((x) => x.skill_id !== skill.skill_id),
        deleting: false,
      }));
    } catch (e) {
      set({ error: `Failed to delete skill: ${(e as Error).message}`, deleting: false });
      throw e;
    }
  },

  save: async (editingId, createBody, updateBody) => {
    set({ saving: true, error: null });
    try {
      if (editingId) await updateSkillApi(editingId, updateBody);
      else await createSkillApi(createBody);
      await get().load();
      set({ saving: false });
    } catch (e) {
      set({ error: `Failed to save skill: ${(e as Error).message}`, saving: false });
      throw e;
    }
  },
}));
