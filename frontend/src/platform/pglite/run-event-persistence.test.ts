import { beforeEach, describe, expect, test, vi } from "vitest";

import type { UarAguiEvent } from "@/platform/agui/agui-schema";
import type { AguiEventRow } from "@/platform/agui/agui-normalizer";
import {
  RunEventPersistence,
  type RunEventPersistenceStore,
} from "@/platform/pglite/run-event-persistence";

function row(
  type: string,
  id: string,
  sequence: number,
  payload: Record<string, unknown> = {},
): AguiEventRow {
  return {
    id,
    runId: "server-run-1",
    sequence,
    type,
    phase: type.startsWith("REASONING_") ? "reasoning" : "generate",
    receivedAt: sequence * 10,
    payload: {
      type,
      profile: "uar.agui/1",
      eventId: id,
      sequence,
      runId: "server-run-1",
      ...payload,
    } as unknown as UarAguiEvent,
  };
}

describe("RunEventPersistence", () => {
  let store: RunEventPersistenceStore;
  let writer: RunEventPersistence;

  beforeEach(() => {
    store = {
      startRun: vi.fn(async () => {}),
      finishRun: vi.fn(async () => {}),
      appendRunEvent: vi.fn(async () => 0),
    };
    writer = new RunEventPersistence(store, {
      threadId: "thread-1",
      fallbackRunId: "local-run-1",
      model: "openai/gpt-5",
    });
  });

  test("coalesces text content once at an explicit end", async () => {
    await writer.ingest(row("TEXT_MESSAGE_CONTENT", "2:0", 32, { delta: "hello " }));
    await writer.ingest(row("TEXT_MESSAGE_CONTENT", "3:0", 48, { delta: "world" }));
    expect(store.appendRunEvent).not.toHaveBeenCalled();

    await writer.ingest(row("TEXT_MESSAGE_END", "4:0", 64));

    expect(store.appendRunEvent).toHaveBeenCalledTimes(1);
    expect(store.appendRunEvent).toHaveBeenCalledWith(expect.objectContaining({
      runId: "server-run-1",
      eventId: "4:0",
      type: "TEXT_MESSAGE_END",
      kind: "message",
      payload: expect.objectContaining({
        content: "hello world",
        sourceEventCount: 2,
      }),
    }));
  });

  test("flushes both content kinds before the current terminal frame", async () => {
    await writer.ingest(row("TEXT_MESSAGE_CONTENT", "2:0", 32, { delta: "answer" }));
    await writer.ingest(row("REASONING_MESSAGE_CONTENT", "2:1", 32, { delta: "thought" }));
    const timings = {
      context: 1,
      skill: 2,
      memory: 3,
      retrieval: 4,
      reasoning: 5,
      tool: 6,
      generate: 7,
    };

    await writer.ingest(row("RUN_FINISHED", "3:0", 48, {
      result: { usage: { totalTokens: 42, costUsdEstimate: 0.01 } },
    }), timings);

    expect(vi.mocked(store.appendRunEvent).mock.calls.map(([event]) => event.type)).toEqual([
      "TEXT_MESSAGE_END",
      "REASONING_MESSAGE_END",
      "RUN_FINISHED",
    ]);
    expect(store.finishRun).toHaveBeenCalledWith(expect.objectContaining({
      id: "server-run-1",
      status: "finished",
      phaseTimings: timings,
      usage: { totalTokens: 42, costUsdEstimate: 0.01 },
      costUsd: 0.01,
    }));
  });

  test("retains distinct same-sequence events and opaque RAW payloads", async () => {
    await writer.ingest(row("TOOL_CALL_START", "7:0", 112, { toolCallId: "call-1" }));
    await writer.ingest(row("TOOL_CALL_ARGS", "7:1", 112, { delta: "{}" }));
    await writer.ingest(row("RAW", "8:0", 128, { event: { external: true } }));

    expect(vi.mocked(store.appendRunEvent).mock.calls.map(([event]) => ({
      eventId: event.eventId,
      wireSequence: event.wireSequence,
      kind: event.kind,
      payload: event.payload,
    }))).toEqual([
      expect.objectContaining({ eventId: "7:0", wireSequence: 112, kind: "tool" }),
      expect.objectContaining({ eventId: "7:1", wireSequence: 112, kind: "tool" }),
      expect.objectContaining({
        eventId: "8:0",
        wireSequence: 128,
        kind: "raw",
        payload: expect.objectContaining({ event: { external: true } }),
      }),
    ]);
  });

  test("keeps an official server run identity for transport-level finalization", async () => {
    await writer.ingest(row("RUN_STARTED", "1:0", 16));
    await writer.finish("finished");

    expect(store.finishRun).toHaveBeenCalledWith(expect.objectContaining({
      id: "server-run-1",
      status: "finished",
    }));
    expect(store.startRun).not.toHaveBeenCalledWith(expect.objectContaining({
      id: "local-run-1",
    }));
  });

  test("flushes separate message ids as separate terminal-fallback spans", async () => {
    await writer.ingest(row("TEXT_MESSAGE_CONTENT", "2:0", 32, {
      messageId: "message-1",
      delta: "first",
    }));
    await writer.ingest(row("TEXT_MESSAGE_CONTENT", "3:0", 48, {
      messageId: "message-2",
      delta: "second",
    }));

    await writer.ingest(row("RUN_FINISHED", "4:0", 64));

    expect(vi.mocked(store.appendRunEvent).mock.calls.map(([event]) => ({
      type: event.type,
      messageId: event.payload.messageId,
      content: event.payload.content,
    }))).toEqual([
      { type: "TEXT_MESSAGE_END", messageId: "message-1", content: "first" },
      { type: "TEXT_MESSAGE_END", messageId: "message-2", content: "second" },
      { type: "RUN_FINISHED", messageId: undefined, content: undefined },
    ]);
  });

  test("persists an explicit empty-span boundary", async () => {
    await writer.ingest(row("TEXT_MESSAGE_START", "2:0", 32, {
      messageId: "message-empty",
    }));
    await writer.ingest(row("TEXT_MESSAGE_END", "3:0", 48, {
      messageId: "message-empty",
    }));

    expect(vi.mocked(store.appendRunEvent).mock.calls.map(([event]) => event.type)).toEqual([
      "TEXT_MESSAGE_START",
      "TEXT_MESSAGE_END",
    ]);
  });
});
