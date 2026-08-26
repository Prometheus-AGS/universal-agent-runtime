import { create } from "zustand";

import {
  fetchUserSettings,
  putUserSettings,
  type UarUserSettings,
  type UarUserSettingsUpdate,
} from "../api/user-settings-api";

function jwtHeaders(): {
  Authorization: string;
  "Content-Type": string;
} | null {
  const apiKey =
    (import.meta as unknown as { env: Record<string, string> }).env
      .VITE_UAR_API_KEY ?? "";
  if (!apiKey.startsWith("ey")) return null;
  return {
    "Content-Type": "application/json",
    Authorization: `Bearer ${apiKey}`,
  };
}

interface UserJwtSettingsState {
  settings: UarUserSettings | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
}

interface UserJwtSettingsActions {
  load: () => Promise<void>;
  save: (partial: UarUserSettingsUpdate) => Promise<void>;
}

export const useUserJwtSettingsStore = create<
  UserJwtSettingsState & UserJwtSettingsActions
>((set, get) => ({
  settings: null,
  loading: false,
  saving: false,
  error: null,

  load: async () => {
    const h = jwtHeaders();
    if (!h) return;
    set({ loading: true, error: null });
    try {
      const data = await fetchUserSettings(h);
      set({ settings: data, loading: false });
    } catch (e) {
      set({
        error: e instanceof Error ? e.message : "Failed to load user settings",
        loading: false,
      });
    }
  },

  save: async (partial) => {
    const h = jwtHeaders();
    const cur = get().settings;
    if (!h || !cur) return;
    set({ saving: true, error: null });
    try {
      const data = await putUserSettings(h, partial);
      set({ settings: data, saving: false });
    } catch (e) {
      const msg =
        e instanceof Error ? e.message : "Failed to save user settings";
      set({ error: msg, saving: false });
      throw e;
    }
  },
}));
