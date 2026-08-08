import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { PGlite } from "@electric-sql/pglite";
import { live, type PGliteWithLive } from "@electric-sql/pglite/live";

import {
  INITIAL_SCHEMA_SQL,
  RUN_EVENT_MIGRATION_SQL,
} from "@/platform/pglite/migrations";
import {
  RunEventRepository,
  type PersistedRunSnapshot,
} from "@/platform/pglite/run-event-repository";

describe("RunEventRepository", () => {
  let db: PGlite & PGliteWithLive;
  let repository: RunEventRepository;

  beforeEach(async () => {
    db = await PGlite.create({ extensions: { live } });
    await db.exec(INITIAL_SCHEMA_SQL);
    await db.exec(RUN_EVENT_MIGRATION_SQL);
    await db.query(
      `INSERT INTO threads (id, title, is_ephemeral)
       VALUES ('thread-1', 'Persisted thread', false)`,
    );
    repository = new RunEventRepository(db);
  });

  afterEach(async () => {
    await db.close();
  });

  test("migration is additive and idempotent", async () => {
    await repository.startRun({ id: "run-1", threadId: "thread-1" });

    await db.exec(RUN_EVENT_MIGRATION_SQL);

    expect(await repository.getRuns()).toMatchObject([
      { id: "run-1", threadId: "thread-1", status: "running" },
    ]);
    const { rows } = await db.query<{ title: string }>(
      "SELECT title FROM threads WHERE id = 'thread-1'",
    );
    expect(rows[0]?.title).toBe("Persisted thread");
  });

  test("keeps repeated wire sequences ordered and deduplicates event identity", async () => {
    await repository.startRun({ id: "run-1", threadId: "thread-1" });
    const base = {
      runId: "run-1",
      wireSequence: 112,
      kind: "tool" as const,
      at: "2026-08-07T20:00:00.000Z",
      payload: { toolCallId: "call-1" },
    };

    expect(await repository.appendEvent({
      ...base,
      eventId: "7:0",
      type: "TOOL_CALL_START",
    })).toBe(0);
    expect(await repository.appendEvent({
      ...base,
      eventId: "7:1",
      type: "TOOL_CALL_ARGS",
    })).toBe(1);
    expect(await repository.appendEvent({
      ...base,
      eventId: "7:1",
      type: "TOOL_CALL_ARGS",
    })).toBeNull();

    expect(await repository.getRunEvents("run-1")).toMatchObject([
      { seq: 0, eventId: "7:0", wireSequence: 112, type: "TOOL_CALL_START" },
      { seq: 1, eventId: "7:1", wireSequence: 112, type: "TOOL_CALL_ARGS" },
    ]);
  });

  test("persists terminal phase timings for offline reads", async () => {
    await repository.startRun({
      id: "run-1",
      threadId: "thread-1",
      model: "openai/gpt-5",
    });
    const phaseTimings = {
      context: 3,
      skill: 5,
      memory: 7,
      retrieval: 11,
      reasoning: 13,
      tool: 17,
      generate: 19,
    };

    await repository.finishRun({
      id: "run-1",
      status: "finished",
      finishedAt: "2026-08-07T20:01:00.000Z",
      phaseTimings,
      usage: { totalTokens: 42 },
      costUsd: 0.001,
    });

    expect(await repository.getRuns()).toMatchObject([
      {
        id: "run-1",
        status: "finished",
        model: "openai/gpt-5",
        phaseTimings,
        usage: { totalTokens: 42 },
        costUsd: 0.001,
      },
    ]);
  });

  test("does not overwrite the first terminal status", async () => {
    await repository.startRun({ id: "run-1", threadId: "thread-1" });
    await repository.finishRun({
      id: "run-1",
      status: "cancelled",
      finishedAt: "2026-08-07T20:01:00.000Z",
    });
    await repository.finishRun({
      id: "run-1",
      status: "finished",
      finishedAt: "2026-08-07T20:02:00.000Z",
    });

    expect(await repository.getRuns()).toMatchObject([
      {
        id: "run-1",
        status: "cancelled",
        finishedAt: "2026-08-07T20:01:00.000Z",
      },
    ]);
  });

  test("delivers selected-run snapshots and preserves one-shot reads", async () => {
    await repository.startRun({ id: "run-1", threadId: "thread-1" });
    await repository.startRun({ id: "run-2", threadId: "thread-1" });
    await repository.appendEvent({
      runId: "run-1",
      eventId: "initial",
      wireSequence: 1,
      type: "RUN_STARTED",
      kind: "lifecycle",
      at: "2026-08-07T20:00:00.000Z",
      payload: {},
    });

    const snapshots: PersistedRunSnapshot[] = [];
    const subscription = await repository.subscribeRunSnapshot(
      "run-1",
      (snapshot) => snapshots.push(snapshot),
    );

    await vi.waitFor(() => {
      expect(snapshots.at(-1)).toMatchObject({
        run: { id: "run-1", status: "running" },
        events: [{ eventId: "initial", seq: 0 }],
      });
    });
    expect(subscription.initialSnapshot).toMatchObject({
      run: { id: "run-1" },
      events: [{ eventId: "initial" }],
    });
    expect(await repository.getRunEvents("run-1")).toHaveLength(1);

    await repository.appendEvent({
      runId: "run-1",
      eventId: "next",
      wireSequence: 2,
      type: "TEXT_MESSAGE_START",
      kind: "message",
      at: "2026-08-07T20:00:01.000Z",
      payload: { messageId: "message-1" },
    });
    await repository.finishRun({
      id: "run-1",
      status: "finished",
      finishedAt: "2026-08-07T20:00:02.000Z",
    });

    await vi.waitFor(() => {
      expect(snapshots.at(-1)).toMatchObject({
        run: { id: "run-1", status: "finished" },
        events: [
          { eventId: "initial", seq: 0 },
          { eventId: "next", seq: 1 },
        ],
      });
    });

    const selectedSnapshotCount = snapshots.length;
    await repository.appendEvent({
      runId: "run-2",
      eventId: "other-run",
      wireSequence: 3,
      type: "RUN_STARTED",
      kind: "lifecycle",
      at: "2026-08-07T20:00:03.000Z",
      payload: {},
    });
    await vi.waitFor(() => {
      expect(snapshots.at(-1)?.events).toHaveLength(2);
    });
    expect(snapshots).toHaveLength(selectedSnapshotCount);

    await subscription.unsubscribe();
    await subscription.unsubscribe();
    await repository.appendEvent({
      runId: "run-1",
      eventId: "after-unsubscribe",
      wireSequence: 4,
      type: "RUN_FINISHED",
      kind: "lifecycle",
      at: "2026-08-07T20:00:04.000Z",
      payload: {},
    });
    await new Promise((resolve) => setTimeout(resolve, 25));
    expect(snapshots).toHaveLength(selectedSnapshotCount);
  });
});
