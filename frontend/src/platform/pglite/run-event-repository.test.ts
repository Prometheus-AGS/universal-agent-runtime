import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { PGlite } from "@electric-sql/pglite";
import { live, type PGliteWithLive } from "@electric-sql/pglite/live";

import {
  INITIAL_SCHEMA_SQL,
  MIGRATIONS,
  RUN_EVENT_MIGRATION_SQL,
} from "@/platform/pglite/migrations";
import {
  RunEventRepository,
  type PersistedRunSnapshot,
} from "@/platform/pglite/run-event-repository";
import { RunEventPersistence } from "@/platform/pglite/run-event-persistence";
import { UarAguiAdapter } from "@/platform/agui/agui-adapter";

describe("RunEventRepository", () => {
  let db: PGlite & PGliteWithLive;
  let repository: RunEventRepository;

  beforeEach(async () => {
    db = await PGlite.create({ extensions: { live } });
    await db.exec(INITIAL_SCHEMA_SQL);
    await db.exec(RUN_EVENT_MIGRATION_SQL);
    for (const migration of MIGRATIONS.filter((item) => item.version > 2)) {
      await db.exec(migration.up);
    }
    await db.query(
      `INSERT INTO threads (id, title, is_ephemeral)
       VALUES ('thread-1', 'Persisted thread', false)`,
    );
    repository = new RunEventRepository(db);
  });

  async function nativeTerminal(id: string, status: "finished" | "error" | "cancelled", eventId = "terminal") {
    const type = status === "finished" ? "RUN_FINISHED" : "RUN_ERROR";
    await repository.appendEvent({
      runId: id, eventId, wireSequence: 2, type, kind: "lifecycle",
      at: "2026-08-07T20:01:00.000Z", payload: {
        type, profile: "uar.agui/1", eventId, runId: id,
        ...(status === "cancelled" ? { code: "CANCELLED" } : {}),
      },
    });
    return { runId: id, eventId };
  }

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
      nativeTerminal: await nativeTerminal("run-1", "finished"),
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
      nativeTerminal: await nativeTerminal("run-1", "cancelled"),
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

  test.each([
    { type: "RUN_FINISHED", status: "finished", payload: {
      result: { usage: { totalTokens: 42, costUsdEstimate: 0.01 } },
    }, usage: { totalTokens: 42, costUsdEstimate: 0.01 }, costUsd: 0.01 },
    { type: "RUN_ERROR", status: "error", payload: {
      code: "PROVIDER_ERROR", message: "Provider failed",
    }, usage: null, costUsd: null },
    { type: "RUN_ERROR", status: "cancelled", payload: {
      code: "CANCELLED", message: "Run cancelled",
    }, usage: null, costUsd: null },
    { type: "RUN_ERROR", status: "cancelled", payload: {
      code: "CANCELLED", message: "Run cancelled", result: { usage: {
        inputTokens: null, outputTokens: 3, totalTokens: null, costUsdEstimate: 0.02, model: null,
      } },
    }, usage: { inputTokens: null, outputTokens: 3, totalTokens: null, costUsdEstimate: 0.02, model: null }, costUsd: 0.02 },
  ])("reconciles interrupted to $status through real stream persistence once", async (terminal) => {
    const nativeRunId = "run-1";
    let observedAt = Date.parse("2026-08-07T20:00:00.000Z");
    const createWriter = (repo: RunEventRepository) => new RunEventPersistence({
      startRun: (input) => repo.startRun(input),
      finishRun: (input) => repo.finishRun(input),
      appendRunEvent: (input) => repo.appendEvent(input),
    }, { threadId: "thread-1", fallbackRunId: nativeRunId, nativeRunId, model: "openai/gpt-5" });
    const frame = (type: string, eventId: string, sequence: number, payload = {}) => JSON.stringify({
      type, profile: "uar.agui/1", eventId, sequence, runId: nativeRunId, ...payload,
    });
    const firstAdapter = new UarAguiAdapter(() => observedAt);
    const firstWriter = createWriter(repository);
    const initial = firstAdapter.ingest("TEXT_MESSAGE_CONTENT", frame(
      "TEXT_MESSAGE_CONTENT", "initial", 1, { messageId: "answer", delta: "partial" },
    ), 1);
    if (!initial?.eventRow) throw new Error("Expected normalized initial delta");
    await firstWriter.ingest(initial.eventRow, initial.phaseTimings);
    await firstWriter.finish();
    const interrupted = (await repository.getRuns())[0];
    expect(interrupted).toMatchObject({ id: nativeRunId, status: "interrupted", usage: null });

    // New adapters represent reload/reattach; neither starting nor receiving a
    // replayed delta is evidence that the native run resumed or completed.
    const resumedRepository = new RunEventRepository(db);
    const resumedWriter = createWriter(resumedRepository);
    const resumedAdapter = new UarAguiAdapter(() => observedAt);
    const replay = resumedAdapter.ingest("TEXT_MESSAGE_CONTENT", frame(
      "TEXT_MESSAGE_CONTENT", "initial", 1, { messageId: "answer", delta: "partial" },
    ), 1);
    if (!replay?.eventRow) throw new Error("Expected normalized replay delta");
    await resumedWriter.ingest(replay.eventRow, replay.phaseTimings);
    expect((await resumedRepository.getRuns())[0]).toEqual(interrupted);
    await resumedWriter.finish();
    expect((await resumedRepository.getRuns())[0]).toEqual(interrupted);

    observedAt += 30_000;
    const completion = resumedAdapter.ingest(terminal.type, frame(
      terminal.type, "terminal", 2, terminal.payload,
    ), 2);
    if (!completion?.eventRow) throw new Error("Expected normalized native terminal");
    expect(completion.event).toBe(terminal.status === "cancelled" ? "agui.cancelled"
      : terminal.status === "error" ? "agui.error" : "agui.done");
    await resumedWriter.ingest(completion.eventRow, completion.phaseTimings);
    const readback = new RunEventRepository(db);
    const runs = await readback.getRuns();
    expect(runs).toHaveLength(1);
    expect(runs[0]).toEqual({
      id: nativeRunId, threadId: "thread-1", messageId: null, status: terminal.status,
      startedAt: "2026-08-07T20:00:00.000Z", finishedAt: new Date(observedAt).toISOString(),
      model: "openai/gpt-5", usage: terminal.usage, costUsd: terminal.costUsd,
      phaseTimings: completion.phaseTimings,
    });
    expect(completion.phaseTimings?.generate).toBe(30_000);
    expect((await readback.getRunEvents(nativeRunId)).filter((event) => event.eventId === "initial")).toHaveLength(1);
    expect((await readback.getRunEvents(nativeRunId)).filter((event) => event.eventId === "terminal")).toMatchObject([
      { payload: { ...terminal.payload, profile: "uar.agui/1", runId: nativeRunId, eventId: "terminal" } },
    ]);

    // Replay, conflicting late terminal, and a stale writer's local transport
    // failure must leave the first authoritative terminal and all metadata intact.
    await resumedWriter.ingest({ ...completion.eventRow, receivedAt: observedAt + 5_000 }, {
      context: 9, skill: 9, memory: 9, retrieval: 9, reasoning: 9, tool: 9, generate: 9,
    });
    await resumedWriter.ingest({
      ...completion.eventRow, id: "late-terminal", type: "RUN_ERROR", receivedAt: observedAt + 6_000,
      payload: { ...completion.eventRow.payload, eventId: "late-terminal", type: "RUN_ERROR", code: "LATE_ERROR" },
    });
    await firstWriter.finish();
    await resumedWriter.finish();
    expect(await readback.getRuns()).toEqual(runs);

    await expect(resumedWriter.ingest({
      ...completion.eventRow, id: "foreign", runId: "other-native-run",
      payload: { ...completion.eventRow.payload, eventId: "foreign", runId: "other-native-run" },
    })).rejects.toThrow("identity conflicts");
    await expect(resumedWriter.ingest({
      ...completion.eventRow, payload: { ...completion.eventRow.payload, runId: "other-native-run" },
    })).rejects.toThrow("identity conflicts");
    expect(await readback.getRuns()).toEqual(runs);
    expect(await readback.getRunEvents("other-native-run")).toEqual([]);
  });

  test.each(["finished", "error", "cancelled"] as const)("a weak local %s cannot freeze a running native run", async (status) => {
    await repository.startRun({ id: "run-1", threadId: "thread-1" });
    await repository.finishRun({ id: "run-1", status, finishedAt: "2026-08-07T20:00:01.000Z" });
    expect(await repository.getRuns()).toMatchObject([{ id: "run-1", status: "running", finishedAt: null }]);
    await repository.finishRun({ id: "run-1", status: "finished", finishedAt: "2026-08-07T20:00:02.000Z",
      nativeTerminal: await nativeTerminal("run-1", "finished") });
    expect(await repository.getRuns()).toMatchObject([{ id: "run-1", status: "finished" }]);
  });

  test("interrupted reconciliation requires persisted matching native terminal evidence", async () => {
    await repository.startRun({ id: "run-1", threadId: "thread-1" });
    await repository.finishRun({ id: "run-1", status: "interrupted", finishedAt: "2026-08-07T20:00:01.000Z" });
    const interrupted = await repository.getRuns();
    const finish = {
      id: "run-1", status: "finished" as const, finishedAt: "2026-08-07T20:01:00.000Z",
      nativeTerminal: { runId: "run-1", eventId: "terminal" },
    };
    await repository.finishRun(finish);
    expect(await repository.getRuns()).toEqual(interrupted);
    await repository.appendEvent({
      runId: "run-1", eventId: "terminal", wireSequence: 2, type: "RUN_ERROR", kind: "lifecycle",
      at: finish.finishedAt, payload: {
        type: "RUN_ERROR", profile: "uar.agui/1", eventId: "terminal", runId: "run-1", code: "CANCELLED",
      },
    });
    await repository.finishRun(finish);
    await repository.finishRun({ ...finish, status: "error" });
    await repository.finishRun({ ...finish, status: "cancelled", nativeTerminal: { runId: "other-run", eventId: "terminal" } });
    expect(await repository.getRuns()).toEqual(interrupted);
    await repository.finishRun({ ...finish, status: "cancelled" });
    expect(await repository.getRuns()).toMatchObject([{ id: "run-1", status: "cancelled", finishedAt: finish.finishedAt }]);
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
      nativeTerminal: await nativeTerminal("run-1", "finished"),
      finishedAt: "2026-08-07T20:00:02.000Z",
    });

    await vi.waitFor(() => {
      expect(snapshots.at(-1)).toMatchObject({
        run: { id: "run-1", status: "finished" },
        events: [
          { eventId: "initial", seq: 0 },
          { eventId: "next", seq: 1 },
          { eventId: "terminal", seq: 2 },
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
      expect(snapshots.at(-1)?.events).toHaveLength(3);
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
