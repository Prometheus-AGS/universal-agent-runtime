import type { PGlitePersistenceClient } from "@/platform/entities";
import type { RunPhaseTimings } from "@/platform/agui/agui-normalizer";
import type { LiveNamespace } from "@electric-sql/pglite/live";

export type PersistedRunStatus = "running" | "finished" | "error" | "cancelled";

export type PersistedRunEventKind =
  | "lifecycle"
  | "message"
  | "reasoning"
  | "tool"
  | "state"
  | "custom"
  | "raw";

export interface PersistedRun {
  id: string;
  threadId: string;
  messageId: string | null;
  status: PersistedRunStatus;
  startedAt: string;
  finishedAt: string | null;
  model: string | null;
  usage: Record<string, unknown> | null;
  costUsd: number | null;
  phaseTimings: RunPhaseTimings | Record<string, never>;
}

export interface PersistedRunEvent {
  runId: string;
  seq: number;
  eventId: string;
  wireSequence: number;
  type: string;
  kind: PersistedRunEventKind;
  at: string;
  payload: Record<string, unknown>;
}

export interface PersistedRunSnapshot {
  run: PersistedRun | null;
  events: PersistedRunEvent[];
}

export interface PersistedRunSnapshotSubscription {
  initialSnapshot: PersistedRunSnapshot;
  unsubscribe: () => Promise<void>;
}

interface RunEventLiveClient extends PGlitePersistenceClient {
  readonly live: LiveNamespace;
}

export interface StartPersistedRunInput {
  id: string;
  threadId: string;
  messageId?: string | null;
  model?: string | null;
  startedAt?: string;
}

export interface FinishPersistedRunInput {
  id: string;
  status: Exclude<PersistedRunStatus, "running">;
  finishedAt: string;
  phaseTimings?: RunPhaseTimings;
  usage?: Record<string, unknown> | null;
  costUsd?: number | null;
}

export interface AppendPersistedRunEventInput {
  runId: string;
  eventId: string;
  wireSequence: number;
  type: string;
  kind: PersistedRunEventKind;
  at: string;
  payload: Record<string, unknown>;
}

interface RunRow {
  id: string;
  thread_id: string;
  message_id: string | null;
  status: PersistedRunStatus;
  started_at: string | Date;
  finished_at: string | Date | null;
  model: string | null;
  usage: Record<string, unknown> | string | null;
  cost_usd: number | string | null;
  phase_timings: RunPhaseTimings | string;
}

interface RunEventRow {
  run_id: string;
  seq: number | string;
  event_id: string;
  wire_sequence: number | string;
  type: string;
  kind: PersistedRunEventKind;
  at: string | Date;
  payload: Record<string, unknown> | string;
}

function jsonRecord(value: Record<string, unknown> | string | null): Record<string, unknown> | null {
  if (value === null) return null;
  if (typeof value !== "string") return value;
  try {
    const parsed = JSON.parse(value) as unknown;
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? parsed as Record<string, unknown>
      : {};
  } catch {
    return {};
  }
}

function timestamp(value: string | Date): string {
  return value instanceof Date ? value.toISOString() : value;
}

function toRun(row: RunRow): PersistedRun {
  return {
    id: row.id,
    threadId: row.thread_id,
    messageId: row.message_id,
    status: row.status,
    startedAt: timestamp(row.started_at),
    finishedAt: row.finished_at === null ? null : timestamp(row.finished_at),
    model: row.model,
    usage: jsonRecord(row.usage),
    costUsd: row.cost_usd === null ? null : Number(row.cost_usd),
    phaseTimings: (jsonRecord(row.phase_timings) ?? {}) as RunPhaseTimings | Record<string, never>,
  };
}

function toRunEvent(row: RunEventRow): PersistedRunEvent {
  return {
    runId: row.run_id,
    seq: Number(row.seq),
    eventId: row.event_id,
    wireSequence: Number(row.wire_sequence),
    type: row.type,
    kind: row.kind,
    at: timestamp(row.at),
    payload: jsonRecord(row.payload) ?? {},
  };
}

export class RunEventRepository {
  constructor(private readonly db: RunEventLiveClient) {}

  async startRun(input: StartPersistedRunInput): Promise<void> {
    await this.db.query(
      `INSERT INTO run (id, thread_id, message_id, status, started_at, model)
       VALUES ($1, $2, $3, 'running', $4, $5)
       ON CONFLICT (id) DO UPDATE
         SET thread_id = EXCLUDED.thread_id,
             message_id = COALESCE(run.message_id, EXCLUDED.message_id),
             model = COALESCE(run.model, EXCLUDED.model)`,
      [
        input.id,
        input.threadId,
        input.messageId ?? null,
        input.startedAt ?? new Date().toISOString(),
        input.model ?? null,
      ],
    );
  }

  async finishRun(input: FinishPersistedRunInput): Promise<void> {
    await this.db.query(
      `UPDATE run
          SET status = $2,
              finished_at = $3,
              phase_timings = COALESCE($4::jsonb, phase_timings),
              usage = COALESCE($5::jsonb, usage),
              cost_usd = COALESCE($6, cost_usd)
        WHERE id = $1
          AND status = 'running'`,
      [
        input.id,
        input.status,
        input.finishedAt,
        input.phaseTimings ? JSON.stringify(input.phaseTimings) : null,
        input.usage ? JSON.stringify(input.usage) : null,
        input.costUsd ?? null,
      ],
    );
  }

  async appendEvent(input: AppendPersistedRunEventInput): Promise<number | null> {
    const { rows } = await this.db.query<{ seq: number | string }>(
      `WITH next_event AS (
         SELECT COALESCE(MAX(seq), -1) + 1 AS seq
           FROM run_event
          WHERE run_id = $1
       )
       INSERT INTO run_event
         (run_id, seq, event_id, wire_sequence, type, kind, at, payload)
       SELECT $1, next_event.seq, $2, $3, $4, $5, $6, $7::jsonb
         FROM next_event
       ON CONFLICT (run_id, event_id) DO NOTHING
       RETURNING seq`,
      [
        input.runId,
        input.eventId,
        input.wireSequence,
        input.type,
        input.kind,
        input.at,
        JSON.stringify(input.payload),
      ],
    );
    return rows[0] ? Number(rows[0].seq) : null;
  }

  async getRuns(): Promise<PersistedRun[]> {
    const { rows } = await this.db.query<RunRow>(
      `SELECT id, thread_id, message_id, status, started_at, finished_at,
              model, usage, cost_usd, phase_timings
         FROM run
        ORDER BY started_at DESC`,
    );
    return rows.map(toRun);
  }

  async getRunEvents(runId: string): Promise<PersistedRunEvent[]> {
    const { rows } = await this.db.query<RunEventRow>(
      `SELECT run_id, seq, event_id, wire_sequence, type, kind, at, payload
         FROM run_event
        WHERE run_id = $1
        ORDER BY seq ASC`,
      [runId],
    );
    return rows.map(toRunEvent);
  }

  async subscribeRunSnapshot(
    runId: string,
    onSnapshot: (snapshot: PersistedRunSnapshot) => void,
  ): Promise<PersistedRunSnapshotSubscription> {
    let active = true;
    let runRows: RunRow[] | null = null;
    let eventRows: RunEventRow[] | null = null;

    const emitSnapshot = (): void => {
      if (!active || runRows === null || eventRows === null) return;
      onSnapshot({
        run: runRows[0] ? toRun(runRows[0]) : null,
        events: eventRows.map(toRunEvent),
      });
    };

    const [runQuery, eventQuery] = await Promise.all([
      this.db.live.query<RunRow>(
        `SELECT id, thread_id, message_id, status, started_at, finished_at,
                model, usage, cost_usd, phase_timings
           FROM run
          WHERE id = $1`,
        [runId],
        (results) => {
          runRows = results.rows;
          emitSnapshot();
        },
      ),
      this.db.live.query<RunEventRow>(
        `SELECT run_id, seq, event_id, wire_sequence, type, kind, at, payload
           FROM run_event
          WHERE run_id = $1
          ORDER BY seq ASC`,
        [runId],
        (results) => {
          eventRows = results.rows;
          emitSnapshot();
        },
      ),
    ]);

    const initialSnapshot = {
      run: runQuery.initialResults.rows[0]
        ? toRun(runQuery.initialResults.rows[0])
        : null,
      events: eventQuery.initialResults.rows.map(toRunEvent),
    };
    let unsubscribePromise: Promise<void> | null = null;

    return {
      initialSnapshot,
      unsubscribe: () => {
        unsubscribePromise ??= (async () => {
          active = false;
          await Promise.all([
            runQuery.unsubscribe(),
            eventQuery.unsubscribe(),
          ]);
        })();
        return unsubscribePromise;
      },
    };
  }
}
