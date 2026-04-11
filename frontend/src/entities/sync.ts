// frontend/src/entities/sync.ts
import {
  getRealtimeManager,
  createWebSocketAdapter,
} from "@prometheus-ags/prometheus-entity-management";
import type {
  RealtimeAdapter,
  ChangeSet,
  SubscriptionConfig,
  ChannelConfig,
  ChangeOperation,
} from "@prometheus-ags/prometheus-entity-management";

interface PersistenceInfo {
  provider: "surreal" | "postgres";
  mode: "embedded" | "remote";
  database_url?: string;
}

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

          handler({
            changes: [
              {
                op,
                type: raw.entity_type ?? "unknown",
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

/**
 * Detect backend persistence provider and initialize the appropriate
 * realtime sync transport. Returns a cleanup function.
 *
 * Transport matrix:
 * - Postgres           -> ElectricSQL (shape streams, bidirectional)
 * - SurrealDB remote   -> WebSocket (LIVE SELECT via WS)
 * - SurrealDB embedded -> SSE Bridge (Axum pushes via SSE)
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

  if (info.provider === "postgres") {
    // ElectricSQL sync -- will be implemented in Phase 6
    // For now, return no-op; REST polling handles data freshness
    return () => {};
  }

  // Default channel config: subscribe to all entity changes
  const channels: ChannelConfig[] = [{ type: "*" }];

  if (info.provider === "surreal" && info.mode === "remote" && info.database_url) {
    // Direct WebSocket to SurrealDB server
    const wsUrl = info.database_url.replace(/^https?/, "ws") + "/rpc";
    const adapter = createWebSocketAdapter({ url: wsUrl });
    const unregister = manager.register(adapter, channels);
    return unregister;
  }

  // surreal + embedded -> SSE bridge from Axum
  const adapter = createSSEAdapter("/api/uar/sync/stream");
  const unregister = manager.register(adapter, channels);
  return unregister;
}
