import { create } from "zustand";
import { useGraphStore } from "@/platform/entities";

import {
  fetchSettingsNamespace,
  putSettingsNamespace,
  type BulkSettingsUpdateResponse,
} from "../api/settings-api";

export interface SettingsNamespaceStatus {
  loading: boolean;
  saving: boolean;
  error: string | null;
}

const IDLE_STATUS: SettingsNamespaceStatus = {
  loading: false,
  saving: false,
  error: null,
};

interface SettingsState {
  statusByNamespace: Record<string, SettingsNamespaceStatus>;
}

interface SettingsActions {
  load: (namespace: string) => Promise<void>;
  save: (
    namespace: string,
    values: Record<string, unknown>,
  ) => Promise<BulkSettingsUpdateResponse>;
}

export type SettingsStore = SettingsState & SettingsActions;

function setNamespaceStatus(
  state: SettingsState,
  namespace: string,
  patch: Partial<SettingsNamespaceStatus>,
): SettingsState {
  return {
    statusByNamespace: {
      ...state.statusByNamespace,
      [namespace]: {
        ...(state.statusByNamespace[namespace] ?? IDLE_STATUS),
        ...patch,
      },
    },
  };
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  statusByNamespace: {},

  load: async (namespace) => {
    set((state) => setNamespaceStatus(state, namespace, { loading: true, error: null }));
    try {
      const records = await fetchSettingsNamespace(namespace);
      const graph = useGraphStore.getState();
      const ids: string[] = [];
      for (const setting of records) {
        const id = `${namespace}:${setting.key}`;
        ids.push(id);
        graph.upsertEntity("Setting", id, {
          id,
          namespace,
          ...(setting as unknown as Record<string, unknown>),
        });
      }
      graph.upsertEntity("SettingsNamespace", namespace, { id: namespace, namespace, ids });
    } catch (error) {
      set((state) =>
        setNamespaceStatus(state, namespace, { error: (error as Error).message }),
      );
      throw error;
    } finally {
      set((state) => setNamespaceStatus(state, namespace, { loading: false }));
    }
  },

  save: async (namespace, values) => {
    const graph = useGraphStore.getState();
    const snapshots: Record<string, Record<string, unknown> | undefined> = {};
    for (const [key, value] of Object.entries(values)) {
      const id = `${namespace}:${key}`;
      const existing = graph.entities["Setting"]?.[id];
      snapshots[key] = existing;
      graph.upsertEntity("Setting", id, {
        ...(existing ?? {}),
        id,
        namespace,
        key,
        data: value,
      });
    }

    set((state) => setNamespaceStatus(state, namespace, { saving: true, error: null }));
    try {
      const payload = Object.fromEntries(
        Object.entries(values).map(([key, value]) => [
          key.startsWith(`${namespace}.`) ? key.slice(namespace.length + 1) : key,
          value,
        ]),
      );
      const response = await putSettingsNamespace(namespace, payload);
      if (response.errors?.length) {
        throw new Error(response.errors.map(({ key, error }) => `${key}: ${error}`).join("; "));
      }
      for (const setting of response.updated ?? []) {
        const id = `${namespace}:${setting.key}`;
        useGraphStore.getState().upsertEntity("Setting", id, {
          id,
          namespace,
          ...(setting as unknown as Record<string, unknown>),
        });
      }
      return response;
    } catch (error) {
      const current = useGraphStore.getState();
      for (const [key, snapshot] of Object.entries(snapshots)) {
        const id = `${namespace}:${key}`;
        if (snapshot) current.upsertEntity("Setting", id, snapshot);
        else current.removeEntity("Setting", id);
      }
      set((state) =>
        setNamespaceStatus(state, namespace, { error: (error as Error).message }),
      );
      throw error;
    } finally {
      set((state) => setNamespaceStatus(state, namespace, { saving: false }));
    }
  },
}));

export function settingsStatusFor(
  state: SettingsStore,
  namespace: string,
): SettingsNamespaceStatus {
  return state.statusByNamespace[namespace] ?? IDLE_STATUS;
}
