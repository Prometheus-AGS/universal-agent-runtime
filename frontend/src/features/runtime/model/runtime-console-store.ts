import { create } from "zustand";
import { useGraphStore } from "@/platform/entities";

import {
  fetchRuntimeA2uiSchemas,
  fetchRuntimeModelRoute,
  fetchRuntimeProviderHealth,
  resolveRuntimeApproval,
} from "../api/runtime-console-api";
import type { RuntimeApprovalEntity } from "@/entities/types";

const POLL_MS = 15_000;

interface RuntimeConsoleState {
  subscribers: number;
  timer: ReturnType<typeof setInterval> | null;
  start: () => void;
  stop: () => void;
  refresh: () => Promise<void>;
  resolveApproval: (approval: RuntimeApprovalEntity, approved: boolean) => Promise<boolean>;
}

function nowIso(): string {
  return new Date().toISOString();
}

async function hydrateProviderHealth(): Promise<void> {
  const body = await fetchRuntimeProviderHealth();
  const upsert = useGraphStore.getState().upsertEntity;
  for (const [providerId, health] of Object.entries(body.providers)) {
    const status = health.healthy
      ? "healthy"
      : health.cooldown_remaining_secs
        ? "degraded"
        : "offline";
    upsert("RuntimeProviderHealth", providerId, {
      id: providerId,
      provider_id: providerId,
      status,
      error: health.consecutive_errors
        ? `${health.consecutive_errors} consecutive error(s)`
        : undefined,
      updated_at: nowIso(),
    });
  }
}

async function hydrateA2uiSurfaces(): Promise<void> {
  const schemas = await fetchRuntimeA2uiSchemas();
  const upsert = useGraphStore.getState().upsertEntity;
  for (const schema of schemas) {
    const schemaId = String(schema.schema_id ?? schema.id ?? "");
    if (!schemaId) continue;
    upsert("RuntimeA2uiSurface", schemaId, {
      id: schemaId,
      schema_id: schemaId,
      title: String(schema.title ?? schema.name ?? schemaId),
      status: "rendered",
      payload: schema,
      updated_at: nowIso(),
    });
  }
}

async function hydrateModelRoute(): Promise<void> {
  const body = await fetchRuntimeModelRoute();
  if (!body.ok || !body.model_id) return;
  useGraphStore.getState().upsertEntity("RuntimeModelRouteDecision", "default", {
    id: "default",
    selected_model: body.model_id,
    selected_provider: body.provider_id,
    reason: "Default model resolution (Settings → Providers)",
    updated_at: nowIso(),
  });
}

export const useRuntimeConsoleStore = create<RuntimeConsoleState>((set, get) => ({
  subscribers: 0,
  timer: null,
  refresh: async () => {
    await Promise.allSettled([
      hydrateProviderHealth(),
      hydrateA2uiSurfaces(),
      hydrateModelRoute(),
    ]);
  },
  resolveApproval: async (approval, approved) => {
    if (approval.status !== "pending") return false;
    const graph = useGraphStore.getState();
    graph.upsertEntity("RuntimeApproval", approval.id, {
      ...approval,
      status: approved ? "approved" : "denied",
      updated_at: nowIso(),
    });
    try {
      await resolveRuntimeApproval(approval.run_id, approved);
      return true;
    } catch {
      graph.upsertEntity("RuntimeApproval", approval.id, approval);
      return false;
    }
  },
  start: () => {
    const subscribers = get().subscribers + 1;
    if (get().timer) {
      set({ subscribers });
      return;
    }
    void get().refresh();
    const timer = setInterval(() => void get().refresh(), POLL_MS);
    set({ subscribers, timer });
  },
  stop: () => {
    const subscribers = Math.max(0, get().subscribers - 1);
    if (subscribers === 0 && get().timer) {
      clearInterval(get().timer!);
      set({ subscribers, timer: null });
      return;
    }
    set({ subscribers });
  },
}));
