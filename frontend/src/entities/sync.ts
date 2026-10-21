// frontend/src/entities/sync.ts
import { getRealtimeManager } from "@/platform/entities";
import type {
  AdapterStatus,
  ChangeSet,
  ChannelConfig,
  ChangeOperation,
  EntityChange,
  RealtimeAdapter,
  SubscriptionConfig,
} from "@/platform/entities";
import {
  emitSettingsChanged,
  emitSettingsRealtimeConnected,
} from "@/features/settings/api";
import {
  installSseRecoveryEvents,
  openSseStream,
  type SseFrame,
  type SseStream,
} from "@/lib/realtime/sse-stream";
import { UAR_TOPICS } from "@/lib/realtime/topics";
import { buildAuthoritativeSnapshotChangeSet } from "@/lib/realtime/uar-sse-adapter";

interface PersistenceInfo {
  provider: "surreal" | "postgres";
  mode: "embedded" | "remote";
  database_url?: string;
}

interface EmbeddedSsePayload {
  version?: number;
  topic?: string;
  table?: string;
  generation?: number;
  id?: string;
  action?: string;
  record?: Record<string, unknown>;
  ts?: string;
}

interface EmbeddedSseSnapshotPayload {
  version?: number;
  topic?: string;
  table?: string;
  generation?: number;
  complete?: boolean;
  records?: unknown[];
  ts?: string;
}

interface EmbeddedSseResetPayload {
  version?: number;
  topic?: string;
  table?: string;
  generation?: number;
  ready?: boolean;
  reason?: string;
  requires_snapshot?: boolean;
}

/** Retained for source compatibility. Reconnect durations are no longer used. */
interface EmbeddedSseAdapterOptions {
  reconnectBaseDelay?: number;
  maxReconnectDelay?: number;
}

const EMBEDDED_ENTITY_TYPES = new Map<string, string>(
  UAR_TOPICS.map(({ topic, entityType }) => [topic, entityType]),
);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function topicFrom(payload: { topic?: string; table?: string }): string | undefined {
  return payload.topic ?? payload.table;
}

function emitUarSettingChange(change: EntityChange) {
  const key = typeof change.data?.key === "string" ? change.data.key : change.id;
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
  const unregisters = adapters.map((adapter) => manager.register(adapter, channels));
  return () => {
    removeStatusListener?.();
    for (const unregister of unregisters) unregister();
  };
}

function parseFrame<T>(frame: SseFrame): T | null {
  try {
    const parsed = JSON.parse(frame.data) as unknown;
    return isRecord(parsed) ? (parsed as T) : null;
  } catch {
    return null;
  }
}

/**
 * Embedded stores publish an initial snapshot and subsequent entity changes.
 * A reset makes that snapshot authoritative only when it declares completion;
 * a partial or mismatched snapshot cannot delete local entity state.
 */
export function createEmbeddedSseAdapter(
  url: string,
  _options: EmbeddedSseAdapterOptions = {},
): RealtimeAdapter {
  const statusCallbacks = new Set<(status: AdapterStatus) => void>();
  const emitStatus = (status: AdapterStatus) => {
    for (const callback of statusCallbacks) callback(status);
  };

  return {
    name: "sse-surreal",
    subscribe(_config: SubscriptionConfig, handler: (changeset: ChangeSet) => void) {
      let stream: SseStream | null = null;
      let stopped = false;
      let hasConnected = false;
      const topicRecovery = new Map<string, {
        awaitingSnapshot: boolean;
        invalidatedDuringSnapshot: boolean;
        expectedGeneration?: number;
      }>();
      let removeRecoveryListeners = () => {};

      const markListsStale = () => handler({ changes: [], affectedListKeys: [""] });
      const stop = () => {
        stream?.abort();
        stream = null;
      };
      const recoveryFor = (topic: string) => {
        let recovery = topicRecovery.get(topic);
        if (!recovery) {
          recovery = { awaitingSnapshot: false, invalidatedDuringSnapshot: false };
          topicRecovery.set(topic, recovery);
        }
        return recovery;
      };

      const processChange = (frame: SseFrame) => {
        const raw = parseFrame<EmbeddedSsePayload>(frame);
        if (!raw) return;
        const opMap: Record<string, ChangeOperation> = {
          create: "insert",
          insert: "insert",
          update: "update",
          delete: "delete",
        };
        const op = opMap[raw.action ?? ""];
        const topic = topicFrom(raw);
        const type = EMBEDDED_ENTITY_TYPES.get(topic ?? "");
        if (!op || !topic || !type || typeof raw.id !== "string" || raw.id.length === 0) return;
        const recovery = recoveryFor(topic);
        if (recovery.awaitingSnapshot) {
          recovery.invalidatedDuringSnapshot = true;
          return;
        }
        if (op !== "delete" && !isRecord(raw.record)) return;

        const data = isRecord(raw.record) ? raw.record : undefined;
        const change: EntityChange = { op, type, id: raw.id, data };
        if (type === "Setting" && data) emitUarSettingChange(change);
        handler({ changes: [change], timestamp: raw.ts });
      };

      const processSnapshot = (frame: SseFrame) => {
        const raw = parseFrame<EmbeddedSseSnapshotPayload>(frame);
        const topic = raw ? topicFrom(raw) : undefined;
        const type = raw ? EMBEDDED_ENTITY_TYPES.get(topic ?? "") : undefined;
        if (!raw || !topic || !type || !Array.isArray(raw.records)) {
          emitStatus("error");
          markListsStale();
          return;
        }
        const recovery = recoveryFor(topic);
        if (
          recovery.expectedGeneration !== undefined &&
          raw.generation !== recovery.expectedGeneration
        ) {
          emitStatus("error");
          markListsStale();
          return;
        }
        if (recovery.invalidatedDuringSnapshot) {
          recovery.invalidatedDuringSnapshot = false;
          restart();
          return;
        }
        if (!raw.records.every(isRecord)) {
          emitStatus("error");
          markListsStale();
          return;
        }

        if (raw.complete !== true) {
          const partialChanges: EntityChange[] = [];
          for (const record of raw.records) {
            const id = record.id;
            if (typeof id !== "string" || id.length === 0) {
              emitStatus("error");
              markListsStale();
              return;
            }
            partialChanges.push({ op: "upsert", type, id, data: record });
          }
          handler({ changes: partialChanges, affectedListKeys: [""], timestamp: raw.ts });
          emitStatus("error");
          recovery.awaitingSnapshot = false;
          return;
        }

        const changeset = buildAuthoritativeSnapshotChangeSet(type, raw.records, raw.ts);
        if (!changeset) {
          emitStatus("error");
          markListsStale();
          return;
        }
        handler(changeset);
        recovery.awaitingSnapshot = false;
        recovery.expectedGeneration = raw.generation;
      };

      const processReset = (frame: SseFrame) => {
        const envelope = parseFrame<Record<string, unknown>>(frame);
        const raw = envelope && isRecord(envelope.data) ? envelope.data : envelope;
        if (!raw || raw.version !== 1) return;
        const reset = raw as EmbeddedSseResetPayload;
        const topic = topicFrom(reset);
        if (topic && !EMBEDDED_ENTITY_TYPES.has(topic)) return;
        const affectedTopics = topic ? [topic] : [...EMBEDDED_ENTITY_TYPES.keys()];
        for (const affectedTopic of affectedTopics) {
          const recovery = recoveryFor(affectedTopic);
          recovery.awaitingSnapshot = true;
          recovery.invalidatedDuringSnapshot = false;
          recovery.expectedGeneration =
            typeof reset.generation === "number" ? reset.generation : undefined;
        }
        markListsStale();
        if (reset.ready === false || reset.requires_snapshot === false) emitStatus("error");
      };

      const processFrame = (frame: SseFrame) => {
        if (frame.event === "entity.change") processChange(frame);
        if (frame.event === "entity.snapshot") processSnapshot(frame);
        if (frame.event === "reset") processReset(frame);
      };

      const connect = () => {
        if (stopped || stream) return;
        emitStatus("connecting");
        const candidate = openSseStream(url, {
          onOpen: () => {
            if (stopped || stream !== candidate) return;
            emitStatus("connected");
            if (hasConnected) emitSettingsRealtimeConnected();
            hasConnected = true;
          },
          onFrame: processFrame,
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
      const restart = () => {
        if (stopped) return;
        stop();
        connect();
      };

      removeRecoveryListeners = installSseRecoveryEvents({
        recover,
        reinitialize: restart,
      });
      connect();

      return () => {
        stopped = true;
        removeRecoveryListeners();
        stop();
        emitStatus("disconnected");
      };
    },
    onStatusChange(callback) {
      statusCallbacks.add(callback);
      return () => statusCallbacks.delete(callback);
    },
  };
}

/**
 * Detect backend persistence provider and initialize its event-driven
 * realtime transport. Stream termination leaves data stale until an explicit
 * reinitialize, host recovery, or browser online/visibility event occurs.
 */
export async function initSyncTransport(): Promise<() => void> {
  let info: PersistenceInfo;
  try {
    const response = await fetch("/api/config/persistence");
    if (!response.ok) return () => {};
    info = (await response.json()) as PersistenceInfo;
  } catch {
    return () => {};
  }

  const manager = getRealtimeManager({ flushInterval: 16 });
  const channels: ChannelConfig[] = [{ type: "*" }];
  if (info.provider === "postgres" || (info.provider === "surreal" && info.mode === "remote")) {
    return registerUarSseAdapters(manager, channels);
  }

  const unregister = manager.register(
    createEmbeddedSseAdapter("/api/uar/sync/stream"),
    channels,
  );
  return unregister;
}
