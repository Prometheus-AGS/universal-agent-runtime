import { describe, expect, test } from "vitest";

import type {
  PersistedRun,
  PersistedRunEvent,
} from "@/platform/pglite/run-event-repository";
import {
  buildRunTraceSegments,
  DEFAULT_RUN_TRACE_FILTERS,
  projectA2uiReplay,
  projectRunTrace,
} from "@/features/chat/model/run-trace-projection";

const run: PersistedRun = {
  id: "run-1",
  threadId: "thread-1",
  messageId: "message-1",
  status: "finished",
  startedAt: "2026-08-07T20:00:00.000Z",
  finishedAt: "2026-08-07T20:00:01.000Z",
  model: "openai/gpt-5",
  usage: null,
  costUsd: null,
  phaseTimings: {
    context: 1,
    skill: 0,
    memory: 0,
    retrieval: 0,
    reasoning: 9,
    tool: 0,
    generate: 90,
  },
};

function event(
  seq: number,
  type: string,
  kind: PersistedRunEvent["kind"],
  payload: Record<string, unknown> = {},
): PersistedRunEvent {
  return {
    runId: "run-1",
    seq,
    eventId: `event-${seq}`,
    wireSequence: seq,
    type,
    kind,
    at: new Date(Date.parse(run.startedAt) + seq * 10).toISOString(),
    payload,
  };
}

function project(
  events: PersistedRunEvent[],
  overrides: Partial<Parameters<typeof projectRunTrace>[0]> = {},
) {
  return projectRunTrace({
    run,
    events,
    filters: { ...DEFAULT_RUN_TRACE_FILTERS },
    expandedNodeIds: new Set(["run:run-1", "phase:lifecycle", "phase:tool", "phase:generate"]),
    selectedNodeId: null,
    ...overrides,
  });
}

describe("run trace projection", () => {
  test("groups every event exactly once by first-observed phase order", () => {
    const events = [
      event(0, "RUN_STARTED", "lifecycle"),
      event(1, "TOOL_CALL_START", "tool", { toolCallId: "call-1" }),
      event(2, "TEXT_MESSAGE_START", "message", { messageId: "message-1" }),
      event(3, "RAW", "raw", { untouched: "<raw>" }),
    ];
    const result = project(events, {
      expandedNodeIds: new Set([
        "run:run-1",
        "phase:lifecycle",
        "phase:tool",
        "phase:generate",
      ]),
    });

    expect(result.nodesById.get("run:run-1")).toMatchObject({
      childIds: ["phase:lifecycle", "phase:tool", "phase:generate"],
    });
    const eventNodes = [...result.nodesById.values()].filter((node) => node.type === "event");
    expect(eventNodes).toHaveLength(events.length);
    expect(new Set(eventNodes.map((node) => node.type === "event" && node.event.eventId)).size).toBe(events.length);
    expect(result.eventsById.get("event-3")?.payload).toEqual({ untouched: "<raw>" });
    expect(result.messageNodeIds.get("message-1")).toEqual(["event:event-2"]);
  });

  test("filters leaves, retains ancestors, and reports structural metadata", () => {
    const events = [
      event(0, "RUN_STARTED", "lifecycle"),
      event(1, "TOOL_CALL_START", "tool", { toolCallId: "call-1" }),
      event(2, "TOOL_CALL_END", "tool", { toolCallId: "call-1" }),
    ];
    const result = project(events, {
      filters: { ...DEFAULT_RUN_TRACE_FILTERS, lifecycle: false },
      expandedNodeIds: new Set(["run:run-1", "phase:tool"]),
    });

    expect(result.visibleRows.map((row) => row.id)).toEqual([
      "run:run-1",
      "phase:tool",
      "event:event-1",
      "event:event-2",
    ]);
    expect(result.visibleRows[2]).toMatchObject({
      depth: 3,
      positionInSet: 1,
      setSize: 2,
      parentId: "phase:tool",
    });
    expect(result.countsByKind).toMatchObject({ lifecycle: 1, tool: 2 });
  });

  test("preserves visible selection and falls back to the first visible event", () => {
    const events = [
      event(0, "RUN_STARTED", "lifecycle"),
      event(1, "TEXT_MESSAGE_START", "message", { messageId: "message-1" }),
    ];
    expect(project(events, { selectedNodeId: "event:event-1" }).selectedNodeId).toBe("event:event-1");
    expect(project(events, { selectedNodeId: "event:missing" }).selectedNodeId).toBe("event:event-0");
  });

  test("derives explicit and correlated durations without inventing unmatched spans", () => {
    const events = [
      event(0, "TOOL_CALL_START", "tool", { toolCallId: "call-1" }),
      event(1, "CUSTOM", "custom", { name: "uar.skill.activated", value: {}, duration_ms: 4 }),
      event(2, "TOOL_CALL_END", "tool", { toolCallId: "call-1" }),
      event(3, "TEXT_MESSAGE_START", "message", { messageId: "unmatched" }),
    ];
    const timings = project(events).timingsByEventId;

    expect(timings.get("event-0")).toMatchObject({
      gapMs: 0,
      durationMs: 20,
      durationSource: "correlated",
    });
    expect(timings.get("event-1")).toMatchObject({
      gapMs: 10,
      durationMs: 4,
      durationSource: "explicit",
    });
    expect(timings.get("event-3")).toMatchObject({
      durationMs: null,
      durationSource: "instant",
    });
  });

  test("keeps exact percentages while applying a three-percent visual floor", () => {
    expect(buildRunTraceSegments(run.phaseTimings)).toEqual([
      { phase: "context", durationMs: 1, exactPercentage: 1, visualWeight: 3 },
      { phase: "reasoning", durationMs: 9, exactPercentage: 9, visualWeight: 9 },
      { phase: "generate", durationMs: 90, exactPercentage: 90, visualWeight: 90 },
    ]);
  });

  test("replays valid patches in order through the A2UI trust boundary", () => {
    const result = projectA2uiReplay([
      {
        op: "add",
        path: "/a2ui/surfaces/surface-1",
        value: { catalogId: "urn:uar:a2ui:catalog:1" },
      },
      {
        op: "replace",
        path: "/a2ui/surfaces/surface-1/components",
        value: { components: [{ id: "root", component: "Text", text: "Ready" }] },
      },
      {
        op: "replace",
        path: "/a2ui/surfaces/surface-1/dataModel",
        value: { path: "/status", value: "done" },
      },
    ]);

    expect(result.appliedOperations).toBe(3);
    expect(result.state.surfaces["surface-1"]).toMatchObject({
      ready: true,
      data: { status: "done" },
    });
  });

  test("rejects invalid replay paths, operations, and executable content", () => {
    expect(() => projectA2uiReplay([
      { op: "move", path: "/a2ui/surfaces/surface-1", value: {} },
    ])).toThrow("Unsupported A2UI replay operation");
    expect(() => projectA2uiReplay([
      { op: "add", path: "/other/surface-1", value: {} },
    ])).toThrow("Invalid A2UI replay path");
    expect(() => projectA2uiReplay([
      {
        op: "add",
        path: "/a2ui/surfaces/surface-1",
        value: { catalogId: "urn:uar:a2ui:catalog:1" },
      },
      {
        op: "replace",
        path: "/a2ui/surfaces/surface-1/components",
        value: { components: [{ id: "root", component: "Text", text: "<script>bad()</script>" }] },
      },
    ])).toThrow(/executable/i);
  });

  test("projects a deterministic 500-event trace within twenty milliseconds", () => {
    const events = Array.from({ length: 500 }, (_, index) => event(
      index,
      index % 2 === 0 ? "TOOL_CALL_ARGS" : "TEXT_MESSAGE_END",
      index % 2 === 0 ? "tool" : "message",
      index % 2 === 0
        ? { toolCallId: `call-${index}` }
        : { messageId: `message-${index}` },
    ));
    const startedAt = performance.now();
    const result = project(events);
    const duration = performance.now() - startedAt;

    expect(result.eventsById.size).toBe(500);
    expect(duration).toBeLessThan(20);
  });
});
