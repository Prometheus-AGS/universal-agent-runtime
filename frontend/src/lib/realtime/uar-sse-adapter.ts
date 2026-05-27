/**
 * SSE-backed RealtimeAdapter for the UAR live-query bus.
 *
 * Subscribes to `GET /api/live/{topic}` on the UAR server (exposed by the
 * `surreal-live-query-bus` change) and translates each `event: create|update|delete`
 * frame into an `EntityChange` published through the entity-mgmt graph.
 *
 * One adapter instance per topic. Multiple views reading the same entity all
 * re-render from the single graph subscription — no per-view fetches.
 */
import type {
  AdapterStatus,
  ChangeSet,
  EntityChange,
  RealtimeAdapter,
  SubscriptionConfig,
  UnsubscribeFn,
} from "@prometheus-ags/prometheus-entity-management";

export interface UarSseAdapterOptions {
  /** Topic name from the bus: `knowledge_documents`, `agents`, `providers`, etc. */
  topic: string;
  /** Logical entity type the graph stores rows under (often equal to `topic`). */
  entityType: string;
  /**
   * Optional base URL. Defaults to same-origin (`/api/live/...`) which is what
   * the dev proxy and the production deploy both serve. Useful for cross-origin
   * dev rigs.
   */
  baseUrl?: string;
  /** Initial reconnect delay in ms. Doubles on each retry, capped at 30 s. */
  reconnectBaseDelay?: number;
  /** Max reconnect attempts before giving up. Defaults to Infinity. */
  maxReconnectAttempts?: number;
}

export function createUarSseAdapter(opts: UarSseAdapterOptions): RealtimeAdapter {
  const {
    topic,
    entityType,
    baseUrl = "",
    reconnectBaseDelay = 1_000,
    maxReconnectAttempts = Number.POSITIVE_INFINITY,
  } = opts;

  const statusCbs = new Set<(s: AdapterStatus) => void>();
  const handlers = new Set<(cs: ChangeSet) => void>();

  let es: EventSource | null = null;
  let attempts = 0;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let stopped = false;

  const emitStatus = (s: AdapterStatus) => {
    for (const cb of statusCbs) cb(s);
  };

  const parseLive = (raw: string, op: "insert" | "update" | "delete"): EntityChange[] => {
    try {
      const parsed = JSON.parse(raw) as { topic: string; id: string; data: unknown };
      return [
        {
          op,
          type: entityType,
          id: parsed.id,
          data: (parsed.data ?? {}) as Record<string, unknown>,
        },
      ];
    } catch {
      return [];
    }
  };

  function connect() {
    if (stopped) return;
    const url = `${baseUrl}/api/live/${encodeURIComponent(topic)}`;
    emitStatus("connecting");

    try {
      es = new EventSource(url, { withCredentials: true });
    } catch {
      emitStatus("error");
      scheduleReconnect();
      return;
    }

    es.onopen = () => {
      attempts = 0;
      emitStatus("connected");
    };

    const dispatch = (changes: EntityChange[]) => {
      if (!changes.length) return;
      const cs: ChangeSet = { changes, timestamp: new Date().toISOString() };
      for (const h of handlers) h(cs);
    };

    es.addEventListener("create", (e) => dispatch(parseLive((e as MessageEvent).data, "insert")));
    es.addEventListener("update", (e) => dispatch(parseLive((e as MessageEvent).data, "update")));
    es.addEventListener("delete", (e) => dispatch(parseLive((e as MessageEvent).data, "delete")));

    es.onerror = () => {
      emitStatus("error");
      es?.close();
      es = null;
      scheduleReconnect();
    };
  }

  function scheduleReconnect() {
    if (stopped) return;
    if (attempts >= maxReconnectAttempts) return;
    const delay = Math.min(reconnectBaseDelay * 2 ** Math.min(attempts++, 6), 30_000);
    reconnectTimer = setTimeout(connect, delay);
  }

  return {
    name: `uar-sse:${topic}`,
    subscribe(_config: SubscriptionConfig, handler): UnsubscribeFn {
      handlers.add(handler);
      if (!es) connect();
      return () => {
        handlers.delete(handler);
        if (handlers.size === 0) {
          stopped = true;
          if (reconnectTimer) clearTimeout(reconnectTimer);
          es?.close();
          es = null;
          emitStatus("disconnected");
        }
      };
    },
    onStatusChange(cb) {
      statusCbs.add(cb);
      return () => statusCbs.delete(cb);
    },
  };
}
