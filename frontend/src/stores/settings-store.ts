import { create } from "zustand";

import {
  fetchSettingsNamespace,
  namespaceToSlug,
  putSettingsNamespace,
} from "@/services/settings-api";
import {
  emitSettingsChanged,
  onSettingsChanged,
} from "@/services/settings-change-bus";
import type { SettingWithMeta } from "@/types";

export interface NamespaceSettingsSlice {
  settings: Record<string, SettingWithMeta>;
  values: Record<string, unknown>;
  dirty: Record<string, unknown>;
  conflicts: Record<string, unknown>;
  loading: boolean;
  saving: boolean;
  error: string | null;
}

function emptySlice(): NamespaceSettingsSlice {
  return {
    settings: {},
    values: {},
    dirty: {},
    conflicts: {},
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
  applyRemoteSetting: (setting: SettingWithMeta) => void;
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
            conflicts: {},
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
    const payload: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(dirty)) {
      payload[
        key.startsWith(`${namespace}.`) ? key.slice(namespace.length + 1) : key
      ] = value;
    }
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
      const response = await putSettingsNamespace(namespace, payload);
      if (response.errors?.length) {
        throw new Error(
          response.errors.map((e) => `${e.key}: ${e.error}`).join("; "),
        );
      }
      set((s) => {
        const cur = s.namespaces[namespace];
        if (!cur) return s;
        const settings = { ...cur.settings };
        const values = { ...cur.values };
        for (const row of response.updated ?? []) {
          settings[row.key] = row;
          values[row.key] = row.data;
        }
        return {
          namespaces: {
            ...s.namespaces,
            [namespace]: {
              ...cur,
              settings,
              values,
              dirty: {},
              conflicts: {},
              saving: false,
              error: null,
            },
          },
        };
      });
      for (const row of response.updated ?? []) {
        emitSettingsChanged({
          namespace,
          key: row.key,
          value: row.data,
          source: "local",
          updated_at: row.updated_at,
        });
      }
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
      throw e;
    }
  },

  applyRemoteSetting: (setting: SettingWithMeta) => {
    const namespace = setting.key.split(".")[0] ?? "";
    if (!namespace) return;
    set((s) => {
      const cur = s.namespaces[namespace];
      if (!cur) return s;
      const isDirty = Object.prototype.hasOwnProperty.call(
        cur.dirty,
        setting.key,
      );
      return {
        namespaces: {
          ...s.namespaces,
          [namespace]: {
            ...cur,
            settings: { ...cur.settings, [setting.key]: setting },
            values: isDirty
              ? cur.values
              : { ...cur.values, [setting.key]: setting.data },
            conflicts: isDirty
              ? { ...cur.conflicts, [setting.key]: setting.data }
              : cur.conflicts,
          },
        },
      };
    });
  },
}));

let realtimeBridgeStarted = false;

export function initSettingsRealtimeBridge() {
  if (realtimeBridgeStarted) return;
  realtimeBridgeStarted = true;
  onSettingsChanged((detail) => {
    if (detail.source !== "remote") return;
    const row = {
      id: detail.key,
      settings_type_id: detail.namespace,
      key: detail.key,
      name: detail.key,
      data: detail.value,
      created_at: detail.updated_at ?? new Date().toISOString(),
      updated_at: detail.updated_at,
      meta: {
        source: "Api",
        is_drift: true,
      },
    } satisfies SettingWithMeta;
    useSettingsStore.getState().applyRemoteSetting(row);
  });
}
