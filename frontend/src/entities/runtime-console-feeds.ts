import { useEffect } from "react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

// Runtime Console REST feeds.
//
// Three console panels — Provider Health, A2UI Surfaces, and Model Routing —
// are backed by REST snapshot/registry endpoints rather than the live
// `runtime.*` SSE stream (there is no per-event source for them). This hook
// polls those endpoints while the console is mounted and upserts the results
// into the same runtime entity graph the SSE-backed panels read, so every
// panel is fed from one normalized source (per the runtime-console spec's
// "normalize into runtime entity types" requirement).

const POLL_MS = 15_000;

function nowIso(): string {
  return new Date().toISOString();
}

async function fetchProviderHealth(): Promise<void> {
  const res = await fetch("/api/uar/providers/health");
  if (!res.ok) return;
  const body = (await res.json()) as {
    providers?: Record<
      string,
      {
        healthy?: boolean;
        consecutive_errors?: number;
        cooldown_remaining_secs?: number | null;
      }
    >;
  };
  const upsert = useGraphStore.getState().upsertEntity;
  for (const [providerId, h] of Object.entries(body.providers ?? {})) {
    const status = h.healthy
      ? "healthy"
      : h.cooldown_remaining_secs
        ? "degraded"
        : "offline";
    upsert("RuntimeProviderHealth", providerId, {
      id: providerId,
      provider_id: providerId,
      status,
      error: h.consecutive_errors
        ? `${h.consecutive_errors} consecutive error(s)`
        : undefined,
      updated_at: nowIso(),
    });
  }
}

async function fetchA2uiSurfaces(): Promise<void> {
  const res = await fetch("/api/uar/a2ui/schemas");
  if (!res.ok) return;
  const schemas = (await res.json()) as Array<Record<string, unknown>>;
  if (!Array.isArray(schemas)) return;
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

async function fetchModelRoute(): Promise<void> {
  const res = await fetch("/api/uar/resolve-model");
  if (!res.ok) return;
  const body = (await res.json()) as {
    ok?: boolean;
    provider_id?: string;
    model_id?: string;
  };
  if (!body.ok || !body.model_id) return;
  useGraphStore.getState().upsertEntity("RuntimeModelRouteDecision", "default", {
    id: "default",
    selected_model: body.model_id,
    selected_provider: body.provider_id,
    reason: "Default model resolution (Settings → Providers)",
    updated_at: nowIso(),
  });
}

/**
 * Poll the Runtime Console REST feeds while mounted. Safe to call once at the
 * admin shell level; failures are swallowed so a missing provider/registry
 * never breaks the console.
 */
export function useRuntimeConsoleFeeds(): void {
  useEffect(() => {
    let active = true;
    const run = () => {
      if (!active) return;
      void fetchProviderHealth().catch(() => {});
      void fetchA2uiSurfaces().catch(() => {});
      void fetchModelRoute().catch(() => {});
    };
    run();
    const timer = setInterval(run, POLL_MS);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, []);
}
