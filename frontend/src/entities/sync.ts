// frontend/src/entities/sync.ts
import { getRealtimeManager } from "@/platform/entities";
import type {
  RealtimeAdapter,
  ChangeSet,
  SubscriptionConfig,
  ChannelConfig,
  ChangeOperation,
} from "@/platform/entities";
import { emitSettingsChanged } from "@/features/settings/api";

interface PersistenceInfo {
  provider: "surreal" | "postgres";
  mode: "embedded" | "remote";
  database_url?: string;
}

// ────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────
// SSE Adapter (embedded SurrealDB)
// ────────────────────────────────────────────────────────────────────

/**
 * Custom SSE adapter for embedded SurrealDB.
 * The Axum backend subscribes to LIVE SELECT internally and pushes
 * ChangeSet events via Server-Sent Events.
 */
function createSSEAdapter(url: string): RealtimeAdapter {
  return {
    name: "sse-surreal",

    subscribe(
      _config: SubscriptionConfig,
      handler: (changeset: ChangeSet) => void,
    ) {
      const eventSource = new EventSource(url);

      eventSource.onmessage = (event) => {
        try {
          const raw = JSON.parse(event.data) as {
            entity_type?: string;
            id?: string;
            action?: string;
            data?: Record<string, unknown>;
          };

          // Map raw SSE event action to ChangeOperation
          const opMap: Record<string, ChangeOperation> = {
            create: "insert",
            insert: "insert",
            update: "update",
            delete: "delete",
          };
          const op = opMap[raw.action ?? ""] ?? "upsert";
          const type = normalizeEntityType(raw.entity_type ?? "unknown");
          if (type === "Setting") {
            const key =
              typeof raw.data?.key === "string" ? raw.data.key : (raw.id ?? "");
            const namespace = key.split(".")[0] ?? "";
            emitSettingsChanged({
              namespace,
              key,
              value: raw.data?.data,
              source: "remote",
              updated_at:
                typeof raw.data?.updated_at === "string"
                  ? raw.data.updated_at
                  : undefined,
            });
          }

          handler({
            changes: [
              {
                op,
                type,
                id: raw.id ?? "",
                data: raw.data,
              },
            ],
          });
        } catch {
          // Skip malformed events
        }
      };

      return () => eventSource.close();
    },
  };
}

function normalizeEntityType(rawType: string): string {
  const map: Record<string, string> = {
    settings: "Setting",
    setting: "Setting",
    settings_types: "SettingsType",
    settings_type: "SettingsType",
  };
  return map[rawType] ?? rawType;
}

// ────────────────────────────────────────────────────────────────────
// Transport initializer
// ────────────────────────────────────────────────────────────────────

/**
 * Detect backend persistence provider and initialize the appropriate
 * realtime sync transport. Returns a cleanup function.
 *
 * Transport matrix:
 * - Postgres           -> UAR SSE (/api/live/{topic}) via the LISTEN/NOTIFY bus
 * - SurrealDB remote   -> UAR SSE (/api/live/{topic}) via the live-query bus
 * - SurrealDB embedded -> SSE Bridge (Axum pushes via /api/uar/sync/stream)
 *
 * NOTE: the Postgres realtime source is the server-side `PostgresNotifyBus`
 * (Postgres LISTEN/NOTIFY → the SAME `/api/live/{topic}` SSE the SurrealDB path
 * uses). This keeps the client backend-agnostic and needs NO ElectricSQL server.
 */
export async function initSyncTransport(): Promise<() => void> {
  let info: PersistenceInfo;
  try {
    const res = await fetch("/api/config/persistence");
    if (!res.ok) return () => {};
    info = (await res.json()) as PersistenceInfo;
  } catch {
    // Fallback: no realtime sync, rely on REST polling + staleTime
    return () => {};
  }

  const manager = getRealtimeManager({ flushInterval: 16 });

  // Default channel config: subscribe to all entity changes
  const channels: ChannelConfig[] = [{ type: "*" }];

  // ── Postgres -> UAR SSE (/api/live) via the LISTEN/NOTIFY bus ─────
  //
  // The UAR backend's PostgresNotifyBus turns Postgres LISTEN/NOTIFY into the
  // same per-topic SSE feed (`/api/live/{topic}`) the SurrealDB path serves, so
  // we reuse the identical UAR SSE adapters here — backend-agnostic, no Electric.
  if (info.provider === "postgres") {
    try {
      const { createAllUarAdapters } = await import("@/lib/realtime/topics");
      const unregisters = createAllUarAdapters().map((adapter) =>
        manager.register(adapter, channels),
      );
      return () => {
        for (const u of unregisters) u();
      };
    } catch (err) {
      console.warn(
        "[sync] UAR SSE adapters failed, falling back to REST polling",
        err,
      );
      return () => {};
    }
  }

  // ── SurrealDB remote -> UAR LiveQueryBus over SSE ─────────────────
  //
  // UAR fronts the SurrealDB live-query feed with per-topic SSE endpoints
  // at /api/live/{topic}. This keeps the JWT auth gate intact (vs. a direct
  // WebSocket to Surreal, which would need its own auth) and gives us one
  // adapter per entity topic so subscribers don't share a single channel.
  if (
    info.provider === "surreal" &&
    info.mode === "remote"
  ) {
    try {
      const { createAllUarAdapters } = await import("@/lib/realtime/topics");
      const unregisters = createAllUarAdapters().map((adapter) =>
        manager.register(adapter, channels),
      );
      return () => {
        for (const u of unregisters) u();
      };
    } catch (err) {
      console.warn(
        "[sync] UAR SSE adapters failed, falling back to REST polling",
        err,
      );
      return () => {};
    }
  }

  // ── SurrealDB embedded -> SSE bridge from Axum ───────────────────
  const adapter = createSSEAdapter("/api/uar/sync/stream");
  const unregister = manager.register(adapter, channels);
  return unregister;
}
