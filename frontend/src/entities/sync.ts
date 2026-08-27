// frontend/src/entities/sync.ts
import { getRealtimeManager, useGraphStore } from "@/platform/entities";
import type {
  AdapterStatus,
  RealtimeAdapter,
  ChangeSet,
  SubscriptionConfig,
  ChannelConfig,
  ChangeOperation,
  EntityChange,
} from "@/platform/entities";
import {
  emitSettingsChanged,
  emitSettingsRealtimeConnected,
} from "@/features/settings/api";
import { UAR_TOPICS } from "@/lib/realtime/topics";

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
interface EmbeddedSsePayload {
  table?: string;
  id?: string;
  action?: string;
  record?: Record<string, unknown>;
  ts?: string;
}

interface EmbeddedSseSnapshotPayload {
  table?: string;
  records?: unknown[];
  ts?: string;
}

interface EmbeddedSseAdapterOptions {
  reconnectBaseDelay?: number;
  maxReconnectDelay?: number;
}

const EMBEDDED_ENTITY_TYPES = new Map<string, string>(
  UAR_TOPICS.map(({ topic, entityType }) => [topic, entityType]),
);

function emitUarSettingChange(change: EntityChange) {
  const key =
    typeof change.data?.key === "string" ? change.data.key : change.id;
  emitSettingsChanged({
    namespace: key.includes(".") ? (key.split(".")[0] ?? "") : "*",
    key,
    value: change.data?.data,
    source: "remote",
    updated_at:
      typeof change.data?.updated_at === "string"
        ? change.data.updated_at
        : undefined,
  });
}

async function registerUarSseAdapters(
  manager: ReturnType<typeof getRealtimeManager>,
  channels: ChannelConfig[],
) {
  const { createAllUarAdapters } = await import("@/lib/realtime/topics");
  const adapters = createAllUarAdapters("", emitUarSettingChange);
  let hasConnected = false;
  const removeStatusListener = adapters[0]?.onStatusChange?.((status) => {
    if (status !== "connected") return;
    if (hasConnected) emitSettingsRealtimeConnected();
    hasConnected = true;
  });
  const unregisters = adapters.map((adapter) =>
    manager.register(adapter, channels),
  );
  return () => {
    removeStatusListener?.();
    for (const unregister of unregisters) unregister();
  };
}

export function createEmbeddedSseAdapter(
  url: string,
  options: EmbeddedSseAdapterOptions = {},
): RealtimeAdapter {
  const statusCallbacks = new Set<(status: AdapterStatus) => void>();
  const reconnectBaseDelay = options.reconnectBaseDelay ?? 1_000;
  const maxReconnectDelay = options.maxReconnectDelay ?? 30_000;
  const emitStatus = (status: AdapterStatus) => {
    for (const callback of statusCallbacks) callback(status);
  };

  return {
    name: "sse-surreal",

    subscribe(
      _config: SubscriptionConfig,
      handler: (changeset: ChangeSet) => void,
    ) {
      let eventSource: EventSource | null = null;
      let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
      let reconnectAttempts = 0;
      let hasConnected = false;
      let stopped = false;

      const handleEntityChange = (event: Event) => {
        try {
          const raw = JSON.parse((event as MessageEvent).data) as EmbeddedSsePayload;

          const opMap: Record<string, ChangeOperation> = {
            create: "insert",
            insert: "insert",
            update: "update",
            delete: "delete",
          };
          const op = opMap[raw.action ?? ""];
          const type = EMBEDDED_ENTITY_TYPES.get(raw.table ?? "");
          if (
            !op ||
            !type ||
            typeof raw.id !== "string" ||
            raw.id.length === 0 ||
            typeof raw.record !== "object" ||
            raw.record === null ||
            Array.isArray(raw.record)
          ) {
            return;
          }

          if (type === "Setting") {
            const key =
              typeof raw.record.key === "string" ? raw.record.key : raw.id;
            const namespace = key.split(".")[0] ?? "";
            emitSettingsChanged({
              namespace,
              key,
              value: raw.record.data,
              source: "remote",
              updated_at:
                typeof raw.record.updated_at === "string"
                  ? raw.record.updated_at
                  : undefined,
            });
          }

          handler({
            changes: [
              {
                op,
                type,
                id: raw.id,
                data: raw.record,
              },
            ],
            timestamp: raw.ts,
          });
        } catch {
          // Skip malformed events
        }
      };

      const handleEntitySnapshot = (event: Event) => {
        try {
          const raw = JSON.parse(
            (event as MessageEvent).data,
          ) as EmbeddedSseSnapshotPayload;
          const type = EMBEDDED_ENTITY_TYPES.get(raw.table ?? "");
          if (!type || !Array.isArray(raw.records)) return;

          const records = raw.records.map((record) => {
            if (
              typeof record !== "object" ||
              record === null ||
              Array.isArray(record) ||
              typeof (record as Record<string, unknown>).id !== "string" ||
              (record as Record<string, unknown>).id === ""
            ) {
              throw new Error("invalid embedded SSE snapshot record");
            }
            return record as Record<string, unknown> & { id: string };
          });
          const snapshotIds = new Set(records.map(({ id }) => id));
          const currentIds = Object.keys(
            useGraphStore.getState().entities[type] ?? {},
          );

          handler({
            changes: [
              ...records.map(({ id, ...data }) => ({
                op: "upsert" as const,
                type,
                id,
                data: { id, ...data },
              })),
              ...currentIds
                .filter((id) => !snapshotIds.has(id))
                .map((id) => ({ op: "delete" as const, type, id })),
            ],
            timestamp: raw.ts,
          });
        } catch {
          // Skip malformed snapshots rather than deleting local entities.
        }
      };

      const scheduleReconnect = () => {
        if (stopped || reconnectTimer) return;
        const delay = Math.min(
          reconnectBaseDelay * 2 ** Math.min(reconnectAttempts++, 6),
          maxReconnectDelay,
        );
        reconnectTimer = setTimeout(connect, delay);
      };

      const closeSource = (source: EventSource) => {
        source.removeEventListener("entity.change", handleEntityChange);
        source.removeEventListener("entity.snapshot", handleEntitySnapshot);
        source.onopen = null;
        source.onerror = null;
        source.close();
      };

      function connect() {
        reconnectTimer = null;
        if (stopped) return;
        emitStatus("connecting");

        let source: EventSource;
        try {
          source = new EventSource(url);
        } catch {
          emitStatus("error");
          scheduleReconnect();
          return;
        }

        eventSource = source;
        source.onopen = () => {
          if (stopped || eventSource !== source) return;
          reconnectAttempts = 0;
          emitStatus("connected");
          if (hasConnected) emitSettingsRealtimeConnected();
          hasConnected = true;
        };
        source.addEventListener("entity.change", handleEntityChange);
        source.addEventListener("entity.snapshot", handleEntitySnapshot);
        source.onerror = () => {
          if (stopped || eventSource !== source) return;
          emitStatus("error");
          closeSource(source);
          eventSource = null;
          scheduleReconnect();
        };
      }

      connect();

      return () => {
        stopped = true;
        if (reconnectTimer) {
          clearTimeout(reconnectTimer);
          reconnectTimer = null;
        }
        if (eventSource) closeSource(eventSource);
        eventSource = null;
        emitStatus("disconnected");
      };
    },
    onStatusChange(callback) {
      statusCallbacks.add(callback);
      return () => statusCallbacks.delete(callback);
    },
  };
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
      return await registerUarSseAdapters(manager, channels);
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
      return await registerUarSseAdapters(manager, channels);
    } catch (err) {
      console.warn(
        "[sync] UAR SSE adapters failed, falling back to REST polling",
        err,
      );
      return () => {};
    }
  }

  // ── SurrealDB embedded -> SSE bridge from Axum ───────────────────
  const adapter = createEmbeddedSseAdapter("/api/uar/sync/stream");
  const unregister = manager.register(adapter, channels);
  return unregister;
}
