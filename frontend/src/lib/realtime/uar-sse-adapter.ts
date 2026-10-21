/**
 * Fetch-stream-backed adapter for the UAR multiplexed live bus.
 *
 * A reset is an invalidation, never a row update. The adapter keeps the live
 * connection open before asking the authenticated snapshot endpoint for an
 * authoritative replacement. Events that overlap the fetch only request a
 * newer snapshot; their payloads cannot overwrite that snapshot out of order.
 */
import { useGraphStore } from "@/platform/entities";
import type {
  AdapterStatus,
  ChangeSet,
  EntityChange,
  RealtimeAdapter,
  SubscriptionConfig,
  UnsubscribeFn,
} from "@/platform/entities";
import {
  installSseRecoveryEvents,
  openSseStream,
  type SseFrame,
  type SseStream,
} from "./sse-stream";

export interface UarRealtimeReset {
  version: 1;
  topic: string;
  generation?: number;
  ready?: boolean;
  reason?: string;
  requires_snapshot: boolean;
}

export interface UarTopicSnapshot {
  version: 1;
  topic: string;
  generation?: number;
  complete: boolean;
  records: Array<Record<string, unknown>>;
}

export type UarSnapshotFetcher = (
  topic: string,
  signal: AbortSignal,
) => Promise<UarTopicSnapshot>;

export interface UarSseAdapterOptions {
  /** Topic name from the bus: `knowledge_documents`, `agents`, `providers`, etc. */
  topic: string;
  /** Logical entity type the graph stores rows under (often equal to `topic`). */
  entityType: string;
  /** Optional same-origin or cross-origin development base URL. */
  baseUrl?: string;
  /** Retained only for source compatibility. Reconnect delays are not used. */
  reconnectBaseDelay?: number;
  /** Retained only for source compatibility. Retry counts are not used. */
  maxReconnectAttempts?: number;
  /** Observe a validated change before it is forwarded to the entity graph. */
  onChange?: (change: EntityChange) => void;
  /** Observe a lifecycle invalidation before its snapshot is requested. */
  onReset?: (reset: UarRealtimeReset) => void;
  /** Injected by tests or a host-specific authenticated snapshot bridge. */
  fetchSnapshot?: UarSnapshotFetcher;
}

type LiveOp = "insert" | "update" | "delete";
type TopicEvent =
  | { kind: "change"; op: LiveOp; id: string; data: Record<string, unknown> }
  | { kind: "reset"; reset: UarRealtimeReset };
type TopicListener = (event: TopicEvent) => void;

interface SharedConnection {
  addListener(topic: string, listener: TopicListener): void;
  removeListener(topic: string, listener: TopicListener): void;
  onStatus(callback: (status: AdapterStatus) => void): () => void;
  dispose(): void;
}

const sharedConnections = new Map<string, SharedConnection>();

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function asGeneration(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function parseReset(raw: string): UarRealtimeReset | null {
  let envelope: unknown;
  try {
    envelope = JSON.parse(raw) as unknown;
  } catch {
    return null;
  }
  if (!isRecord(envelope)) return null;

  const data = isRecord(envelope.data) ? envelope.data : envelope;
  const topic =
    typeof envelope.topic === "string"
      ? envelope.topic
      : typeof data.topic === "string"
        ? data.topic
        : undefined;
  if (!topic || data.version !== 1) return null;

  return {
    version: 1,
    topic,
    generation: asGeneration(data.generation),
    ready: typeof data.ready === "boolean" ? data.ready : undefined,
    reason: typeof data.reason === "string" ? data.reason : undefined,
    requires_snapshot: data.requires_snapshot !== false,
  };
}

function parseChange(raw: string, op: LiveOp): { event: TopicEvent; topic: string } | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw) as unknown;
  } catch {
    return null;
  }
  if (!isRecord(parsed) || typeof parsed.topic !== "string" || typeof parsed.id !== "string") {
    return null;
  }
  return {
    topic: parsed.topic,
    event: {
      kind: "change",
      op,
      id: parsed.id,
      data: isRecord(parsed.data) ? parsed.data : {},
    },
  };
}

function dispatchFrame(
  frame: SseFrame,
  listeners: Map<string, Set<TopicListener>>,
): void {
  if (frame.event === "reset") {
    const reset = parseReset(frame.data);
    if (!reset) return;
    for (const listener of listeners.get(reset.topic) ?? []) {
      listener({ kind: "reset", reset });
    }
    return;
  }

  const op =
    frame.event === "create"
      ? "insert"
      : frame.event === "update"
        ? "update"
        : frame.event === "delete"
          ? "delete"
          : undefined;
  if (!op) return;
  const parsed = parseChange(frame.data, op);
  if (!parsed) return;
  for (const listener of listeners.get(parsed.topic) ?? []) listener(parsed.event);
}

function getSharedConnection(baseUrl: string): SharedConnection {
  const existing = sharedConnections.get(baseUrl);
  if (existing) return existing;

  const listeners = new Map<string, Set<TopicListener>>();
  const statusCallbacks = new Set<(status: AdapterStatus) => void>();
  let stream: SseStream | null = null;
  let stopped = false;
  let removeRecoveryListeners = () => {};

  const emitStatus = (status: AdapterStatus) => {
    for (const callback of statusCallbacks) callback(status);
  };
  const hasListeners = () => [...listeners.values()].some((set) => set.size > 0);
  const stop = () => {
    stream?.abort();
    stream = null;
  };
  const connect = () => {
    if (stopped || stream || !hasListeners()) return;
    emitStatus("connecting");
    const candidate = openSseStream(`${baseUrl}/api/live`, {
      onOpen: () => {
        if (!stopped && stream === candidate) emitStatus("connected");
      },
      onFrame: (frame) => dispatchFrame(frame, listeners),
    });
    stream = candidate;
    void candidate.done.then(
      () => {
        if (!stopped && stream === candidate) {
          stream = null;
          emitStatus("disconnected");
        }
      },
      () => {
        if (!stopped && stream === candidate) {
          stream = null;
          emitStatus("error");
        }
      },
    );
  };
  const recover = () => connect();
  const reinitialize = () => {
    if (stopped || !hasListeners()) return;
    stop();
    connect();
  };

  const connection: SharedConnection = {
    addListener(topic, listener) {
      const wasEmpty = !hasListeners();
      let topicListeners = listeners.get(topic);
      if (!topicListeners) {
        topicListeners = new Set();
        listeners.set(topic, topicListeners);
      }
      topicListeners.add(listener);
      if (wasEmpty) {
        removeRecoveryListeners = installSseRecoveryEvents({ recover, reinitialize });
        connect();
      }
    },
    removeListener(topic, listener) {
      const topicListeners = listeners.get(topic);
      topicListeners?.delete(listener);
      if (topicListeners?.size === 0) listeners.delete(topic);
      if (!hasListeners()) {
        removeRecoveryListeners();
        removeRecoveryListeners = () => {};
        stop();
        emitStatus("disconnected");
      }
    },
    onStatus(callback) {
      statusCallbacks.add(callback);
      return () => statusCallbacks.delete(callback);
    },
    dispose() {
      stopped = true;
      removeRecoveryListeners();
      removeRecoveryListeners = () => {};
      stop();
      listeners.clear();
      emitStatus("disconnected");
    },
  };

  sharedConnections.set(baseUrl, connection);
  return connection;
}

export function buildAuthoritativeSnapshotChangeSet(
  entityType: string,
  records: Array<Record<string, unknown>>,
  timestamp?: string,
): ChangeSet | null {
  const snapshotIds = new Set<string>();
  const upserts: EntityChange[] = [];
  for (const record of records) {
    const id = record.id;
    if (typeof id !== "string" || id.length === 0 || snapshotIds.has(id)) return null;
    snapshotIds.add(id);
    upserts.push({ op: "upsert", type: entityType, id, data: record });
  }

  const currentIds = Object.keys(useGraphStore.getState().entities[entityType] ?? {});
  return {
    changes: [
      ...upserts,
      ...currentIds
        .filter((id) => !snapshotIds.has(id))
        .map((id) => ({ op: "delete" as const, type: entityType, id })),
    ],
    affectedListKeys: [""],
    timestamp,
  };
}

export async function fetchUarTopicSnapshot(
  baseUrl: string,
  topic: string,
  signal: AbortSignal,
): Promise<UarTopicSnapshot> {
  const response = await fetch(
    `${baseUrl}/api/live/${encodeURIComponent(topic)}/snapshot`,
    { credentials: "include", headers: { Accept: "application/json" }, signal },
  );
  if (!response.ok) throw new Error(`Snapshot request failed: ${response.status}`);
  const payload = (await response.json()) as unknown;
  if (
    !isRecord(payload) ||
    payload.version !== 1 ||
    payload.topic !== topic ||
    typeof payload.complete !== "boolean" ||
    !Array.isArray(payload.records) ||
    !payload.records.every(isRecord)
  ) {
    throw new Error("Snapshot response violated the UAR lifecycle contract");
  }
  return {
    version: 1,
    topic,
    generation: asGeneration(payload.generation),
    complete: payload.complete,
    records: payload.records,
  };
}

/** @internal Test reset also closes active streams to prevent leaked readers. */
export function __resetUarSseConnections(): void {
  for (const connection of sharedConnections.values()) connection.dispose();
  sharedConnections.clear();
}

export function createUarSseAdapter(opts: UarSseAdapterOptions): RealtimeAdapter {
  const {
    topic,
    entityType,
    baseUrl = "",
    onChange,
    onReset,
    fetchSnapshot = (snapshotTopic, signal) =>
      fetchUarTopicSnapshot(baseUrl, snapshotTopic, signal),
  } = opts;
  const connection = getSharedConnection(baseUrl);

  return {
    name: `uar-sse:${topic}`,
    subscribe(_config: SubscriptionConfig, handler): UnsubscribeFn {
      let stopped = false;
      let recoveryVersion = 0;
      let recovering = false;
      let invalidatedDuringSnapshot = false;
      let snapshotController: AbortController | null = null;
      const markListsStale = () => handler({ changes: [], affectedListKeys: [""] });
      const recover = (reset: UarRealtimeReset) => {
        recoveryVersion += 1;
        const expectedVersion = recoveryVersion;
        invalidatedDuringSnapshot = false;
        snapshotController?.abort();
        snapshotController = new AbortController();
        recovering = true;
        onReset?.(reset);
        markListsStale();
        if (reset.ready === false || !reset.requires_snapshot) {
          recovering = false;
          return;
        }

        void fetchSnapshot(topic, snapshotController.signal).then(
          (snapshot) => {
            if (stopped || expectedVersion !== recoveryVersion) return;
            if (
              !snapshot.complete ||
              snapshot.topic !== topic ||
              (reset.generation !== undefined && snapshot.generation !== reset.generation)
            ) {
              recovering = false;
              snapshotController = null;
              markListsStale();
              return;
            }
            if (invalidatedDuringSnapshot) {
              recover({ ...reset, reason: "change_during_snapshot" });
              return;
            }
            const changeset = buildAuthoritativeSnapshotChangeSet(entityType, snapshot.records);
            if (!changeset) {
              recovering = false;
              snapshotController = null;
              markListsStale();
              return;
            }
            handler(changeset);
            recovering = false;
            snapshotController = null;
          },
          () => {
            if (stopped || expectedVersion !== recoveryVersion) return;
            recovering = false;
            snapshotController = null;
            markListsStale();
          },
        );
      };
      const listener: TopicListener = (event) => {
        if (event.kind === "reset") {
          recover(event.reset);
          return;
        }
        if (recovering) {
          invalidatedDuringSnapshot = true;
          return;
        }
        const change: EntityChange = {
          op: event.op,
          type: entityType,
          id: event.id,
          data: event.data,
        };
        onChange?.(change);
        handler({ changes: [change], timestamp: new Date().toISOString() });
      };

      connection.addListener(topic, listener);
      return () => {
        stopped = true;
        snapshotController?.abort();
        connection.removeListener(topic, listener);
      };
    },
    onStatusChange(callback) {
      return connection.onStatus(callback);
    },
  };
}
