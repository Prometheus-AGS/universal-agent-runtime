import { create } from "zustand";

import { fetchSettingTypes } from "../api/settings-api";
import type { SettingsType } from "@/types";

interface State {
  types: SettingsType[];
}

interface Actions {
  load: () => Promise<void>;
}

export const useSettingsTypesMetaStore = create<State & Actions>((set) => ({
  types: [],

  load: async () => {
    try {
      const d = await fetchSettingTypes();
      set({ types: Array.isArray(d) ? (d as SettingsType[]) : [] });
    } catch {
      set({ types: [] });
    }
  },
}));
