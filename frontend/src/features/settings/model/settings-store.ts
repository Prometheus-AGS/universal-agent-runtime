import { create } from "zustand";
import { useGraphStore } from "@/platform/entities";

import {
  fetchGovernanceStatus,
  fetchSettingsNamespace,
  putSettingsNamespace,
  type BulkSettingsUpdateResponse,
} from "../api/settings-api";
import {
  ingestGovernanceStatus,
  governanceStatusSnapshot,
  invalidateGovernanceStatus,
  nextGovernanceRequestSequence,
} from "./governance-status";

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
    const governanceRequestSequence =
      namespace === "governance" ? nextGovernanceRequestSequence() : null;
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
      if (namespace === "governance" && response.results) {
        const current = useGraphStore.getState();
        for (const result of response.results) {
          if (result.status === "updated") continue;
          const snapshot = snapshots[result.key];
          const id = `${namespace}:${result.key}`;
          if (snapshot) current.upsertEntity("Setting", id, snapshot);
          else current.removeEntity("Setting", id);
        }
        if (
          response.governance_status &&
          response.applied_status &&
          governanceRequestSequence !== null
        ) {
          const ingested = ingestGovernanceStatus(
            response.governance_status,
            governanceRequestSequence,
          );
          let confirmationFailed = false;
          let authoritative = governanceStatusSnapshot();
          if (
            ingested.restarted ||
            (authoritative &&
              authoritative.boot_instance_id !==
                response.applied_status.boot_instance_id)
          ) {
            const confirmationSequence = nextGovernanceRequestSequence();
            try {
              const expectedBootInstance = authoritative?.boot_instance_id;
              const confirmation = await fetchGovernanceStatus();
              const confirmationResult = ingestGovernanceStatus(
                confirmation,
                confirmationSequence,
              );
              if (
                !confirmationResult.accepted ||
                confirmationResult.restarted ||
                confirmation.boot_instance_id !== expectedBootInstance
              ) {
                confirmationFailed = invalidateGovernanceStatus(
                  confirmationSequence,
                );
                authoritative = confirmationFailed
                  ? null
                  : governanceStatusSnapshot();
              } else {
                authoritative = governanceStatusSnapshot();
              }
            } catch {
              confirmationFailed = invalidateGovernanceStatus(
                confirmationSequence,
              );
              authoritative = confirmationFailed
                ? null
                : governanceStatusSnapshot();
            }
          }

          const updatedCount = response.results.filter(
            (result) => result.status === "updated",
          ).length;
          const matchingBoot =
            authoritative?.boot_instance_id ===
            response.applied_status.boot_instance_id;
          const matchingRevision =
            matchingBoot &&
            authoritative?.revision === response.applied_status.revision;
          const newerRevision =
            matchingBoot &&
            typeof authoritative?.revision === "number" &&
            authoritative.revision > response.applied_status.revision;
          response.governance_outcome =
            confirmationFailed || !authoritative
              ? "unknown"
              : newerRevision || !matchingBoot
                ? "changed_elsewhere"
                : matchingRevision
                  ? response.status === "updated"
                    ? "confirmed"
                    : updatedCount > 0
                      ? "partial"
                      : "rejected"
                  : "unknown";
          response.observed_governance_status = authoritative ?? undefined;

          if (response.governance_outcome === "unknown") {
            for (const [key, snapshot] of Object.entries(snapshots)) {
              const id = `${namespace}:${key}`;
              if (snapshot) current.upsertEntity("Setting", id, snapshot);
              else current.removeEntity("Setting", id);
            }
          }
        }
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
      if (namespace === "governance" && governanceRequestSequence !== null) {
        invalidateGovernanceStatus(governanceRequestSequence);
      }
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
