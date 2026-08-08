import { describe, expect, test } from "vitest";

import {
  UarAguiNormalizer,
  phaseOfAguiEvent,
} from "@/platform/agui/agui-normalizer";
import type { UarAguiEvent } from "@/platform/agui/agui-schema";

function wire(type: string, fields: Record<string, unknown> = {}): UarAguiEvent {
  return {
    type,
    profile: "uar.agui/1",
    eventId: `${type}:1`,
    sequence: 1,
    runId: "run-1",
    ...fields,
  };
}

describe("UarAguiNormalizer", () => {
  test("maps official and custom events to target run phases", () => {
    expect(phaseOfAguiEvent(wire("STATE_DELTA"))).toBe("context");
    expect(phaseOfAguiEvent(wire("CUSTOM", { name: "uar.skill.activated" }))).toBe("skill");
    expect(phaseOfAguiEvent(wire("CUSTOM", { name: "uar.memory.recall" }))).toBe("memory");
    expect(phaseOfAguiEvent(wire("CUSTOM", { name: "uar.rag_citations" }))).toBe("retrieval");
    expect(phaseOfAguiEvent(wire("REASONING_MESSAGE_CONTENT"))).toBe("reasoning");
    expect(phaseOfAguiEvent(wire("TOOL_CALL_START"))).toBe("tool");
    expect(phaseOfAguiEvent(wire("TEXT_MESSAGE_CONTENT"))).toBe("generate");
    expect(phaseOfAguiEvent(wire("RAW"))).toBeNull();
  });

  test("emits typed message chunks and one official event row", () => {
    const normalizer = new UarAguiNormalizer(() => 12);
    const projection = normalizer.normalize(wire("TEXT_MESSAGE_CONTENT", {
      eventId: "2:8",
      sequence: 40,
      messageId: "message-1",
      delta: "hello",
    }));

    expect(projection.messageChunk).toEqual({
      kind: "text",
      messageId: "message-1",
      delta: "hello",
    });
    expect(projection.eventRow).toMatchObject({
      id: "2:8",
      sequence: 40,
      type: "TEXT_MESSAGE_CONTENT",
      runId: "run-1",
      phase: "generate",
      receivedAt: 12,
    });
  });

  test("emits clamped phase timings once and assigns gaps to generation", () => {
    const times = [100, 110, 130, 150, 180, 200];
    const normalizer = new UarAguiNormalizer(() => times.shift() ?? 200);

    normalizer.normalize(wire("RUN_STARTED"));
    normalizer.normalize(wire("CUSTOM", { name: "uar.memory.recall" }));
    normalizer.normalize(wire("CUSTOM", { name: "uar.memory.mutation" }));
    normalizer.normalize(wire("TEXT_MESSAGE_CONTENT", { delta: "a" }));
    normalizer.normalize(wire("TEXT_MESSAGE_CONTENT", { delta: "b" }));
    const terminal = normalizer.normalize(wire("RUN_FINISHED"));

    expect(terminal.phaseTimings).toEqual({
      context: 0,
      skill: 0,
      memory: 20,
      retrieval: 0,
      reasoning: 0,
      tool: 0,
      generate: 80,
    });
    expect(normalizer.normalize(wire("RUN_ERROR")).phaseTimings).toBeUndefined();
  });

  test("treats a run error as a terminal timing boundary", () => {
    const times = [5, 15, 25];
    const normalizer = new UarAguiNormalizer(() => times.shift() ?? 25);

    normalizer.normalize(wire("RUN_STARTED"));
    normalizer.normalize(wire("REASONING_MESSAGE_CONTENT", { delta: "checking" }));

    expect(normalizer.normalize(wire("RUN_ERROR")).phaseTimings).toEqual({
      context: 0,
      skill: 0,
      memory: 0,
      retrieval: 0,
      reasoning: 0,
      tool: 0,
      generate: 20,
    });
  });

  test("keeps RAW payloads opaque", () => {
    const normalizer = new UarAguiNormalizer(() => 1);
    const raw = wire("RAW", { event: { provider: "external", nested: [1, 2] } });
    const projection = normalizer.normalize(raw);

    expect(projection.eventRow.payload).toBe(raw);
    expect(projection.eventRow.phase).toBeNull();
    expect(projection.messageChunk).toBeUndefined();
    expect(projection.runtimeChunk).toMatchObject({
      kind: "raw",
      type: "RAW",
      payload: raw,
    });
  });

  test("preserves unknown CUSTOM payloads as hidden raw chunks", () => {
    const normalizer = new UarAguiNormalizer(() => 1);
    const custom = wire("CUSTOM", { name: "provider.experimental", value: { nested: [1, 2] } });

    expect(normalizer.normalize(custom).runtimeChunk).toMatchObject({
      kind: "raw",
      type: "provider.experimental",
      payload: { nested: [1, 2] },
    });
  });
});
