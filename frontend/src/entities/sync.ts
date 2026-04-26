// frontend/src/entities/sync.ts
import {
  getRealtimeManager,
  createWebSocketAdapter,
  createElectricAdapter,
} from "@prometheus-ags/prometheus-entity-management";
import type {
  RealtimeAdapter,
  ChangeSet,
  SubscriptionConfig,
  ChannelConfig,
  ChangeOperation,
  ElectricTableConfig,
  EntityChange,
} from "@prometheus-ags/prometheus-entity-management";
import { PGlite } from "@electric-sql/pglite";
import { loadPgliteFsBundle } from "@/lib/pglite-assets";
import { emitSettingsChanged } from "@/services/settings-change-bus";

/**
 * Mirror of the library-internal ShapeMessage type.
 * Matches the ElectricSQL shape stream message format.
 */
interface ShapeMessage<T = Record<string, unknown>> {
  headers: { operation: "insert" | "update" | "delete" };
  offset: string;
  value: T;
  key: string;
}

/**
 * Mirror of the library-internal ShapeStream interface.
 * Used to construct ElectricTableConfig entries.
 */
interface ShapeStream<T = Record<string, unknown>> {
  subscribe(
    onMsg: (msgs: ShapeMessage<T>[]) => void,
    onErr?: (e: Error) => void,
  ): () => void;
  isUpToDate: boolean;
  lastOffset: string;
}

interface PersistenceInfo {
  provider: "surreal" | "postgres";
  mode: "embedded" | "remote";
  database_url?: string;
}

// ────────────────────────────────────────────────────────────────────
// ElectricSQL Shape Stream (HTTP long-poll)
// ────────────────────────────────────────────────────────────────────

/** Entity table descriptor used to build ElectricSQL shape streams. */
interface EntityTableDef {
  /** Postgres table name */
  table: string;
  /** Entity type key registered in the graph engine */
  entityType: string;
}

/** Default set of entity tables to sync via ElectricSQL. */
const ENTITY_TABLES: EntityTableDef[] = [
  { table: "providers", entityType: "Provider" },
  { table: "models", entityType: "Model" },
  { table: "agents", entityType: "Agent" },
  { table: "skills", entityType: "Skill" },
  { table: "knowledge_bases", entityType: "KnowledgeBase" },
  { table: "knowledge_documents", entityType: "Document" },
  { table: "runtime_runs", entityType: "RuntimeRun" },
  { table: "runtime_run_steps", entityType: "RuntimeRunStep" },
  { table: "runtime_tool_calls", entityType: "RuntimeToolCall" },
  { table: "runtime_approvals", entityType: "RuntimeApproval" },
  { table: "runtime_artifacts", entityType: "RuntimeArtifact" },
  { table: "runtime_memory_events", entityType: "RuntimeMemoryEvent" },
  { table: "runtime_ag_ui_events", entityType: "RuntimeAgUiEvent" },
  { table: "runtime_a2ui_surfaces", entityType: "RuntimeA2uiSurface" },
  {
    table: "runtime_model_route_decisions",
    entityType: "RuntimeModelRouteDecision",
  },
  { table: "runtime_provider_health", entityType: "RuntimeProviderHealth" },
  { table: "settings", entityType: "Setting" },
  { table: "settings_types", entityType: "SettingsType" },
];

/**
 * Lightweight ShapeStream implementation that connects to ElectricSQL's
 * HTTP Shape API. ElectricSQL exposes shapes at
 *   GET {electricUrl}/v1/shape/{table}?offset={offset}
 *
 * Each response is a batch of newline-delimited JSON messages.
 * The stream uses long-polling: once up-to-date, the server holds the
 * connection until new changes arrive.
 */
function createShapeStream<T extends Record<string, unknown>>(
  electricUrl: string,
  table: string,
): ShapeStream<T> {
  type Subscriber = (msgs: ShapeMessage<T>[]) => void;
  type ErrorHandler = (e: Error) => void;

  const subscribers: Array<{ onMsg: Subscriber; onErr?: ErrorHandler }> = [];
  let currentOffset = "-1";
  let shapeId: string | undefined;
  let upToDate = false;
  let abortController: AbortController | null = null;

  function notifyError(err: Error) {
    for (const sub of subscribers) {
      sub.onErr?.(err);
    }
  }

  async function poll() {
    while (subscribers.length > 0) {
      abortController = new AbortController();
      const params = new URLSearchParams({ offset: currentOffset });
      if (shapeId) params.set("shape_id", shapeId);

      let res: Response;
      try {
        res = await fetch(
          `${electricUrl}/v1/shape/${encodeURIComponent(table)}?${params.toString()}`,
          { signal: abortController.signal },
        );
      } catch (err) {
        if ((err as Error).name === "AbortError") return;
        // Back off on network errors
        notifyError(err as Error);
        await new Promise((r) => setTimeout(r, 2000));
        continue;
      }

      if (!res.ok) {
        notifyError(new Error(`ElectricSQL shape error: ${res.status}`));
        await new Promise((r) => setTimeout(r, 2000));
        continue;
      }

      // Capture shape_id from response headers (Electric convention)
      const hdrShapeId = res.headers.get("electric-shape-id");
      if (hdrShapeId) shapeId = hdrShapeId;

      const hdrUpToDate = res.headers.get("electric-up-to-date");
      if (hdrUpToDate === "true") upToDate = true;

      const text = await res.text();
      const messages: ShapeMessage<T>[] = [];

      for (const line of text.split("\n")) {
        if (!line.trim()) continue;
        try {
          const msg = JSON.parse(line) as ShapeMessage<T>;
          if (msg.offset) currentOffset = msg.offset;
          messages.push(msg);
        } catch {
          // Skip malformed lines
        }
      }

      if (messages.length > 0) {
        for (const sub of subscribers) {
          sub.onMsg(messages);
        }
      }
    }
  }

  return {
    subscribe(onMsg: Subscriber, onErr?: ErrorHandler): () => void {
      const entry = { onMsg, onErr };
      subscribers.push(entry);

      // Start polling on first subscriber
      if (subscribers.length === 1) {
        void poll();
      }

      return () => {
        const idx = subscribers.indexOf(entry);
        if (idx >= 0) subscribers.splice(idx, 1);
        if (subscribers.length === 0) {
          abortController?.abort();
        }
      };
    },
    get isUpToDate() {
      return upToDate;
    },
    get lastOffset() {
      return currentOffset;
    },
  };
}

/**
 * Build ElectricSQL table configs with shape streams for each entity table.
 */
function buildElectricTableConfigs(
  electricUrl: string,
): ElectricTableConfig<Record<string, unknown>>[] {
  return ENTITY_TABLES.map((def) => ({
    type: def.entityType,
    table: def.table,
    shapeStream: createShapeStream(electricUrl, def.table),
  }));
}

// ────────────────────────────────────────────────────────────────────
// SurrealDB WebSocket message parser
// ────────────────────────────────────────────────────────────────────

/**
 * Parse a SurrealDB LIVE SELECT notification into EntityChange[].
 * SurrealDB sends JSON-RPC notifications with structure:
 *   { "result": { "action": "CREATE"|"UPDATE"|"DELETE", "result": {...} } }
 */
function parseSurrealMessage(data: unknown): EntityChange[] | null {
  if (typeof data !== "object" || data === null) return null;
  const msg = data as Record<string, unknown>;

  // SurrealDB LIVE SELECT notification format
  const result = msg.result as Record<string, unknown> | undefined;
  if (!result || typeof result !== "object") return null;

  const action = result.action as string | undefined;
  const record = result.result as Record<string, unknown> | undefined;
  if (!action || !record) return null;

  const opMap: Record<string, ChangeOperation> = {
    CREATE: "insert",
    UPDATE: "update",
    DELETE: "delete",
  };
  const op = opMap[action] ?? "upsert";

  // Derive entity type from the SurrealDB record id (e.g. "providers:abc")
  const rawId = (record.id as string) ?? "";
  const entityType = rawId.includes(":") ? rawId.split(":")[0] : "unknown";

  return [
    {
      op,
      type: entityType,
      id: rawId,
      data: record,
    },
  ];
}

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

  // Default channel config: subscribe to all entity changes
  const channels: ChannelConfig[] = [{ type: "*" }];

  // ── Postgres -> ElectricSQL ──────────────────────────────────────
  if (info.provider === "postgres") {
    try {
      // Derive ElectricSQL URL: same host, default port 3000
      const electricUrl = `${window.location.protocol}//${window.location.hostname}:3000`;

      // PGlite runs in-browser as the local replica for Electric sync
      const fsBundle = await loadPgliteFsBundle();
      const pg = new PGlite({ fsBundle });
      const pglite = pg as unknown as Parameters<
        typeof createElectricAdapter
      >[0]["pglite"];

      const tables = buildElectricTableConfigs(electricUrl);

      const adapter = createElectricAdapter({
        pglite,
        tables,
        onSynced: () => {
          console.info("[sync] ElectricSQL initial sync complete");
        },
      });

      // SyncAdapter extends RealtimeAdapter, so register works
      const unregister = manager.register(adapter, channels);

      return () => {
        unregister();
        // TODO: PGlite does not currently expose a close() method;
        // if it does in a future version, call pg.close() here.
      };
    } catch (err) {
      console.warn(
        "[sync] ElectricSQL adapter unavailable, falling back to REST polling",
        err,
      );
      return () => {};
    }
  }

  // ── SurrealDB remote -> WebSocket ────────────────────────────────
  if (
    info.provider === "surreal" &&
    info.mode === "remote" &&
    info.database_url
  ) {
    try {
      // Convert http(s) URL to ws(s) for SurrealDB RPC endpoint
      const wsUrl = info.database_url.replace(/^http/, "ws") + "/rpc";

      const adapter = createWebSocketAdapter({
        url: wsUrl,
        parseMessage: parseSurrealMessage,
        // TODO: If SurrealDB requires authentication, add a protocols or
        // custom header-based auth mechanism here once the backend exposes
        // a sync token endpoint.
        reconnectBaseDelay: 1000,
        maxReconnectAttempts: 10,
        pingInterval: 30_000,
        pingMessage: JSON.stringify({ method: "ping" }),
      });

      const unregister = manager.register(adapter, channels);
      return unregister;
    } catch (err) {
      console.warn(
        "[sync] WebSocket adapter failed, falling back to REST polling",
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
