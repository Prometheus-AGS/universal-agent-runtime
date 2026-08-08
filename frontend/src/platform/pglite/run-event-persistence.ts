import type {
  AguiEventRow,
  RunPhaseTimings,
} from "@/platform/agui/agui-normalizer";
import type {
  AppendPersistedRunEventInput,
  FinishPersistedRunInput,
  PersistedRunEventKind,
  StartPersistedRunInput,
} from "@/platform/pglite/run-event-repository";

export interface RunEventPersistenceStore {
  startRun(input: StartPersistedRunInput): Promise<void>;
  finishRun(input: FinishPersistedRunInput): Promise<void>;
  appendRunEvent(input: AppendPersistedRunEventInput): Promise<number | null>;
}

export interface RunEventPersistenceContext {
  threadId: string;
  fallbackRunId: string;
  messageId?: string | null;
  model?: string | null;
}

interface BufferedContent {
  runId: string;
  messageId: string;
  kind: "message" | "reasoning";
  content: string;
  count: number;
  first: AguiEventRow;
  last: AguiEventRow;
}

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function kindOf(type: string): PersistedRunEventKind {
  if (type.startsWith("TEXT_MESSAGE_")) return "message";
  if (type.startsWith("REASONING_")) return "reasoning";
  if (type.startsWith("TOOL_CALL_")) return "tool";
  if (type.startsWith("STATE_") || type === "MESSAGES_SNAPSHOT") return "state";
  if (type === "CUSTOM") return "custom";
  if (type === "RAW") return "raw";
  return "lifecycle";
}

function messageIdOf(row: AguiEventRow): string {
  return typeof row.payload.messageId === "string" ? row.payload.messageId : "";
}

function terminalStatus(row: AguiEventRow): FinishPersistedRunInput["status"] | null {
  if (row.type === "RUN_FINISHED") return "finished";
  if (row.type === "RUN_ERROR") {
    return row.payload.code === "CANCELLED" ? "cancelled" : "error";
  }
  return null;
}

function usageOf(row: AguiEventRow): {
  usage?: Record<string, unknown>;
  costUsd?: number;
} {
  if (row.type !== "RUN_FINISHED") return {};
  const usage = record(record(row.payload.result).usage);
  const cost = usage.costUsdEstimate;
  return {
    usage,
    costUsd: typeof cost === "number" ? cost : undefined,
  };
}

export class RunEventPersistence {
  private readonly ensuredRuns = new Set<string>();
  private readonly buffers = new Map<string, BufferedContent>();
  private activeRunId: string;

  constructor(
    private readonly store: RunEventPersistenceStore,
    private readonly context: RunEventPersistenceContext,
  ) {
    this.activeRunId = context.fallbackRunId;
  }

  async ingest(row: AguiEventRow, phaseTimings?: RunPhaseTimings): Promise<void> {
    const runId = row.runId ?? this.activeRunId;
    this.activeRunId = runId;
    await this.ensureRun(runId, row.receivedAt);

    if (row.type === "TEXT_MESSAGE_CONTENT" || row.type === "REASONING_MESSAGE_CONTENT") {
      this.bufferContent(runId, row);
      return;
    }

    if (row.type === "TEXT_MESSAGE_END") {
      await this.flushBoundary(runId, "message", row);
      return;
    }
    if (row.type === "REASONING_MESSAGE_END") {
      await this.flushBoundary(runId, "reasoning", row);
      return;
    }
    if (row.type === "REASONING_END") {
      await this.flushBuffers(runId, "reasoning", row.receivedAt);
      await this.persistRow(runId, row);
      return;
    }

    const status = terminalStatus(row);
    if (status) {
      await this.flushBuffers(runId, "message", row.receivedAt);
      await this.flushBuffers(runId, "reasoning", row.receivedAt);
    }

    await this.persistRow(runId, row);

    if (status) {
      const usage = usageOf(row);
      await this.store.finishRun({
        id: runId,
        status,
        finishedAt: new Date(row.receivedAt).toISOString(),
        phaseTimings,
        usage: usage.usage,
        costUsd: usage.costUsd,
      });
    }
  }

  async finish(status: FinishPersistedRunInput["status"]): Promise<void> {
    const runId = this.activeRunId;
    const receivedAt = Date.now();
    await this.ensureRun(runId, receivedAt);
    await this.flushBuffers(runId, "message", receivedAt);
    await this.flushBuffers(runId, "reasoning", receivedAt);
    await this.store.finishRun({
      id: runId,
      status,
      finishedAt: new Date(receivedAt).toISOString(),
    });
  }

  private async ensureRun(runId: string, receivedAt: number): Promise<void> {
    if (this.ensuredRuns.has(runId)) return;
    await this.store.startRun({
      id: runId,
      threadId: this.context.threadId,
      messageId: this.context.messageId,
      model: this.context.model,
      startedAt: new Date(receivedAt).toISOString(),
    });
    this.ensuredRuns.add(runId);
  }

  private bufferContent(runId: string, row: AguiEventRow): void {
    const kind = row.type === "TEXT_MESSAGE_CONTENT" ? "message" : "reasoning";
    const messageId = messageIdOf(row);
    const key = this.bufferKey(runId, kind, messageId);
    const delta = typeof row.payload.delta === "string" ? row.payload.delta : "";
    const current = this.buffers.get(key);
    this.buffers.set(key, current
      ? {
          ...current,
          content: current.content + delta,
          count: current.count + 1,
          last: row,
        }
      : {
          runId,
          messageId,
          kind,
          content: delta,
          count: 1,
          first: row,
          last: row,
        });
  }

  private async flushBoundary(
    runId: string,
    kind: BufferedContent["kind"],
    boundary: AguiEventRow,
  ): Promise<void> {
    const key = this.bufferKey(runId, kind, messageIdOf(boundary));
    const buffered = this.buffers.get(key);
    if (!buffered) {
      await this.persistRow(runId, boundary);
      return;
    }
    this.buffers.delete(key);
    await this.persistBuffer(buffered, boundary);
  }

  private async flushBuffers(
    runId: string,
    kind: BufferedContent["kind"],
    fallbackReceivedAt: number,
  ): Promise<void> {
    const pending = [...this.buffers.entries()].filter(
      ([, buffered]) => buffered.runId === runId && buffered.kind === kind,
    );
    for (const [key, buffered] of pending) {
      this.buffers.delete(key);
      await this.persistBuffer(buffered, undefined, fallbackReceivedAt);
    }
  }

  private async persistBuffer(
    buffered: BufferedContent,
    boundary?: AguiEventRow,
    fallbackReceivedAt?: number,
  ): Promise<void> {
    const { runId, kind } = buffered;

    const endType = kind === "message" ? "TEXT_MESSAGE_END" : "REASONING_MESSAGE_END";
    const eventId = boundary?.id ?? `${buffered.last.id}:coalesced`;
    const receivedAt = boundary?.receivedAt ?? fallbackReceivedAt ?? buffered.last.receivedAt;
    const payload = {
      ...buffered.last.payload,
      ...(boundary?.payload ?? {}),
      type: boundary?.type ?? endType,
      eventId,
      sequence: boundary?.sequence ?? buffered.last.sequence,
      content: buffered.content,
      coalesced: true,
      sourceEventCount: buffered.count,
    };

    await this.persistRow(runId, {
      id: eventId,
      runId,
      sequence: boundary?.sequence ?? buffered.last.sequence,
      type: boundary?.type ?? endType,
      phase: buffered.last.phase,
      receivedAt,
      payload: payload as AguiEventRow["payload"],
    });
  }

  private bufferKey(
    runId: string,
    kind: BufferedContent["kind"],
    messageId: string,
  ): string {
    return JSON.stringify([runId, kind, messageId]);
  }

  private async persistRow(runId: string, row: AguiEventRow): Promise<void> {
    await this.store.appendRunEvent({
      runId,
      eventId: row.id,
      wireSequence: row.sequence,
      type: row.type,
      kind: kindOf(row.type),
      at: new Date(row.receivedAt).toISOString(),
      payload: row.payload,
    });
  }
}
