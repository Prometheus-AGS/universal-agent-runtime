/**
 * SSE-backed RealtimeAdapter for the UAR live-query bus.
 *
 * Every topic shares a SINGLE `GET /api/live` EventSource. Browsers cap HTTP/1.1
 * connections at 6 per origin; opening one stream per topic (10 of them)
 * exhausted that budget and starved every other request — including the PGlite
 * WASM the SPA needs to boot — hanging startup. The multiplexed endpoint fans
 * all topics over one connection, tagging each event with its `topic`; this
 * module demultiplexes back to per-topic, per-`entityType` `EntityChange`s.
 *
 * `createUarSseAdapter` still returns one adapter per topic (so the entity graph
 * keeps routing changes by `entityType`), but all adapters reuse the one shared
 * connection underneath.
 */
import type {
  AdapterStatus,
  ChangeSet,
  EntityChange,
  RealtimeAdapter,
  SubscriptionConfig,
  UnsubscribeFn,
} from "@/platform/entities";

export interface UarSseAdapterOptions {
  /** Topic name from the bus: `knowledge_documents`, `agents`, `providers`, etc. */
  topic: string;
  /** Logical entity type the graph stores rows under (often equal to `topic`). */
  entityType: string;
  /**
   * Optional base URL. Defaults to same-origin (`/api/live`) which is what the
   * dev proxy and the production deploy both serve. Useful for cross-origin
   * dev rigs.
   */
  baseUrl?: string;
  /** Initial reconnect delay in ms. Doubles on each retry, capped at 30 s. */
  reconnectBaseDelay?: number;
  /** Max reconnect attempts before giving up. Defaults to Infinity. */
  maxReconnectAttempts?: number;
}

type LiveOp = "insert" | "update" | "delete";
type TopicListener = (op: LiveOp, id: string, data: Record<string, unknown>) => void;

// ── Shared multiplexed connection ──────────────────────────────────────────
// One EventSource to `/api/live` per origin, demultiplexed to per-topic
// listeners. Created lazily on first subscriber and disconnected when the last
// listener (across all topics) goes away. The registry object persists so
// status subscribers and later re-subscribes reuse the same instance.

interface SharedConnection {
  addListener(topic: string, fn: TopicListener): void;
  removeListener(topic: string, fn: TopicListener): void;
  onStatus(cb: (s: AdapterStatus) => void): () => void;
}

const sharedConnections = new Map<string, SharedConnection>();

/**
 * Test-only: drop all shared connections so each test starts from a clean
 * slate. The shared connection is a module-level singleton (one socket for the
 * whole app); without this, state would leak between test cases. Never called
 * in production.
 *
 * @internal
 */
export function __resetUarSseConnections(): void {
  sharedConnections.clear();
}

function getSharedConnection(
  baseUrl: string,
  reconnectBaseDelay: number,
  maxReconnectAttempts: number,
): SharedConnection {
  const existing = sharedConnections.get(baseUrl);
  if (existing) return existing;

  const listeners = new Map<string, Set<TopicListener>>();
  const statusCbs = new Set<(s: AdapterStatus) => void>();

  let es: EventSource | null = null;
  let attempts = 0;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  const emitStatus = (s: AdapterStatus) => {
    for (const cb of statusCbs) cb(s);
  };

  const totalListeners = (): number => {
    let n = 0;
    for (const set of listeners.values()) n += set.size;
    return n;
  };

  const dispatch = (raw: string, op: LiveOp) => {
    let parsed: { topic: string; id: string; data: unknown };
    try {
      parsed = JSON.parse(raw) as { topic: string; id: string; data: unknown };
    } catch {
      return;
    }
    const set = listeners.get(parsed.topic);
    if (!set || set.size === 0) return;
    const data = (parsed.data ?? {}) as Record<string, unknown>;
    for (const fn of set) fn(op, parsed.id, data);
  };

  function connect() {
    reconnectTimer = null;
    const url = `${baseUrl}/api/live`;
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

    es.addEventListener("create", (e) => dispatch((e as MessageEvent).data, "insert"));
    es.addEventListener("update", (e) => dispatch((e as MessageEvent).data, "update"));
    es.addEventListener("delete", (e) => dispatch((e as MessageEvent).data, "delete"));

    es.onerror = () => {
      emitStatus("error");
      es?.close();
      es = null;
      scheduleReconnect();
    };
  }

  function scheduleReconnect() {
    if (totalListeners() === 0) return;
    if (attempts >= maxReconnectAttempts) return;
    const delay = Math.min(reconnectBaseDelay * 2 ** Math.min(attempts++, 6), 30_000);
    reconnectTimer = setTimeout(connect, delay);
  }

  const conn: SharedConnection = {
    addListener(topic, fn) {
      let set = listeners.get(topic);
      if (!set) {
        set = new Set();
        listeners.set(topic, set);
      }
      set.add(fn);
      // First listener of the whole connection — open the stream.
      if (!es && reconnectTimer === null) {
        attempts = 0;
        connect();
      }
    },
    removeListener(topic, fn) {
      const set = listeners.get(topic);
      if (set) {
        set.delete(fn);
        if (set.size === 0) listeners.delete(topic);
      }
      // Last listener across all topics — tear the stream down.
      if (totalListeners() === 0) {
        if (reconnectTimer) {
          clearTimeout(reconnectTimer);
          reconnectTimer = null;
        }
        es?.close();
        es = null;
        attempts = 0;
        emitStatus("disconnected");
      }
    },
    onStatus(cb) {
      statusCbs.add(cb);
      return () => statusCbs.delete(cb);
    },
  };

  sharedConnections.set(baseUrl, conn);
  return conn;
}

export function createUarSseAdapter(opts: UarSseAdapterOptions): RealtimeAdapter {
  const {
    topic,
    entityType,
    baseUrl = "",
    reconnectBaseDelay = 1_000,
    maxReconnectAttempts = Number.POSITIVE_INFINITY,
  } = opts;

  const conn = getSharedConnection(baseUrl, reconnectBaseDelay, maxReconnectAttempts);

  return {
    name: `uar-sse:${topic}`,
    subscribe(_config: SubscriptionConfig, handler): UnsubscribeFn {
      const listener: TopicListener = (op, id, data) => {
        const change: EntityChange = { op, type: entityType, id, data };
        const cs: ChangeSet = { changes: [change], timestamp: new Date().toISOString() };
        handler(cs);
      };
      conn.addListener(topic, listener);
      return () => {
        conn.removeListener(topic, listener);
      };
    },
    onStatusChange(cb) {
      return conn.onStatus(cb);
    },
  };
}
