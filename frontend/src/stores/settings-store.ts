import { create } from "zustand";

import { fetchSettingsNamespace, namespaceToSlug, putSettingValue } from "@/services/settings-api";
import type { SettingWithMeta } from "@/types";

export interface NamespaceSettingsSlice {
  settings: Record<string, SettingWithMeta>;
  values: Record<string, unknown>;
  dirty: Record<string, unknown>;
  loading: boolean;
  saving: boolean;
  error: string | null;
}

function emptySlice(): NamespaceSettingsSlice {
  return {
    settings: {},
    values: {},
    dirty: {},
    loading: true,
    saving: false,
    error: null,
  };
}

interface SettingsStoreState {
  namespaces: Record<string, NamespaceSettingsSlice>;
}

interface SettingsStoreActions {
  load: (namespace: string) => Promise<void>;
  setSetting: (namespace: string, key: string, value: unknown) => void;
  saveAll: (namespace: string) => Promise<void>;
}

type SettingsStore = SettingsStoreState & SettingsStoreActions;

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  namespaces: {},

  load: async (namespace: string) => {
    set((s) => ({
      namespaces: {
        ...s.namespaces,
        [namespace]: {
          ...(s.namespaces[namespace] ?? emptySlice()),
          loading: true,
          error: null,
        },
      },
    }));
    try {
      const slug = namespaceToSlug(namespace);
      const data = await fetchSettingsNamespace(slug);
      const byKey: Record<string, SettingWithMeta> = {};
      const vals: Record<string, unknown> = {};
      for (const row of data) {
        byKey[row.key] = row;
        vals[row.key] = row.data;
      }
      set((s) => ({
        namespaces: {
          ...s.namespaces,
          [namespace]: {
            settings: byKey,
            values: vals,
            dirty: {},
            loading: false,
            saving: false,
            error: null,
          },
        },
      }));
    } catch (e) {
      set((s) => ({
        namespaces: {
          ...s.namespaces,
          [namespace]: {
            ...(s.namespaces[namespace] ?? emptySlice()),
            loading: false,
            error: (e as Error).message,
          },
        },
      }));
    }
  },

  setSetting: (namespace: string, key: string, value: unknown) => {
    set((s) => {
      const cur = s.namespaces[namespace] ?? emptySlice();
      return {
        namespaces: {
          ...s.namespaces,
          [namespace]: {
            ...cur,
            values: { ...cur.values, [key]: value },
            dirty: { ...cur.dirty, [key]: value },
          },
        },
      };
    });
  },

  saveAll: async (namespace: string) => {
    const slice = get().namespaces[namespace];
    if (!slice || Object.keys(slice.dirty).length === 0) return;
    const dirty = slice.dirty;
    set((s) => {
      const cur = s.namespaces[namespace];
      if (!cur) return s;
      return {
        namespaces: {
          ...s.namespaces,
          [namespace]: { ...cur, saving: true, error: null },
        },
      };
    });
    try {
      await Promise.all(
        Object.entries(dirty).map(([key, value]) => putSettingValue(key, value)),
      );
      set((s) => {
        const cur = s.namespaces[namespace];
        if (!cur) return s;
        return {
          namespaces: {
            ...s.namespaces,
            [namespace]: { ...cur, dirty: {}, saving: false, error: null },
          },
        };
      });
    } catch (e) {
      set((s) => {
        const cur = s.namespaces[namespace];
        if (!cur) return s;
        return {
          namespaces: {
            ...s.namespaces,
            [namespace]: { ...cur, saving: false, error: (e as Error).message },
          },
        };
      });
    }
  },
}));
