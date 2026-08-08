import { describe, expect, test } from "vitest";

import { UarAguiAdapter } from "@/platform/agui/agui-adapter";

function wire(
  type: string,
  eventId: string,
  sequence: number,
  fields: Record<string, unknown> = {},
): string {
  return JSON.stringify({
    type,
    profile: "uar.agui/1",
    eventId,
    sequence,
    runId: "run-1",
    ...fields,
  });
}

describe("UarAguiAdapter", () => {
  test("projects official frames into the canonical chat event shape", () => {
    const adapter = new UarAguiAdapter();

    expect(adapter.ingest(
      "TEXT_MESSAGE_CONTENT",
      wire("TEXT_MESSAGE_CONTENT", "2:8", 40, { messageId: "message-1", delta: "hello" }),
      2,
    )).toMatchObject({
      event: "agui.message.delta",
      eventId: "2:8",
      sequence: 40,
      payload: {
        kind: "message",
        phase: "delta",
        request_id: "run-1",
        delta: { text: "hello" },
      },
      messageChunk: { kind: "text", messageId: "message-1", delta: "hello" },
      eventRow: { id: "2:8", sequence: 40, type: "TEXT_MESSAGE_CONTENT", phase: "generate" },
    });
  });

  test("is idempotent by profile eventId even when an SSE cursor is replayed", () => {
    const adapter = new UarAguiAdapter();
    const data = wire("RUN_STARTED", "1:8", 24, { input: { agentId: "agent-1" } });

    expect(adapter.ingest("RUN_STARTED", data, 1)).not.toBeNull();
    expect(adapter.ingest("RUN_STARTED", data, 1)).toBeNull();
  });

  test("keeps distinct lifecycle frames that share one SSE cursor", () => {
    const adapter = new UarAguiAdapter();

    expect(adapter.ingest(
      "TEXT_MESSAGE_START",
      wire("TEXT_MESSAGE_START", "2:0", 32, { messageId: "m-1" }),
      2,
    )?.eventId).toBe("2:0");
    expect(adapter.ingest(
      "TEXT_MESSAGE_CONTENT",
      wire("TEXT_MESSAGE_CONTENT", "2:8", 40, { delta: "hello" }),
      2,
    )?.eventId).toBe("2:8");
  });

  test("rejects malformed official frames instead of partially reducing them", () => {
    const adapter = new UarAguiAdapter();

    expect(adapter.ingest(
      "TEXT_MESSAGE_CONTENT",
      JSON.stringify({ type: "TEXT_MESSAGE_CONTENT", delta: "unsafe" }),
      2,
    )).toBeNull();
  });

  test("drops late frames so replay cannot regress reduced state", () => {
    const adapter = new UarAguiAdapter();

    expect(adapter.ingest("RUN_STARTED", wire("RUN_STARTED", "2:8", 40), 2)).not.toBeNull();
    expect(adapter.ingest("RUN_ERROR", wire("RUN_ERROR", "1:8", 24, { message: "late" }), 1))
      .toBeNull();
  });

  test("applies RFC 6902 deltas and recovers from divergence on a fresh snapshot", () => {
    const adapter = new UarAguiAdapter();
    const snapshot = adapter.ingest(
      "STATE_SNAPSHOT",
      wire("STATE_SNAPSHOT", "1:8", 24, { snapshot: { count: 1 } }),
      1,
    );
    expect(snapshot?.payload).toMatchObject({ snapshot: { count: 1 }, synchronized: true });

    const delta = adapter.ingest(
      "STATE_DELTA",
      wire("STATE_DELTA", "2:8", 40, {
        delta: [{ op: "replace", path: "/count", value: 2 }],
      }),
      2,
    );
    expect(delta?.payload).toMatchObject({ snapshot: { count: 2 }, synchronized: true });

    const divergent = adapter.ingest(
      "STATE_DELTA",
      wire("STATE_DELTA", "3:8", 56, {
        delta: [{ op: "replace", path: "/missing", value: true }],
      }),
      3,
    );
    expect(divergent?.payload.synchronized).toBe(false);

    const recovered = adapter.ingest(
      "STATE_SNAPSHOT",
      wire("STATE_SNAPSHOT", "4:8", 72, { snapshot: { count: 4 } }),
      4,
    );
    expect(recovered?.payload).toMatchObject({ snapshot: { count: 4 }, synchronized: true });
  });

  test("keeps approval interruptions and terminal outcomes visible", () => {
    const approvalAdapter = new UarAguiAdapter();
    const approval = approvalAdapter.ingest(
      "CUSTOM",
      wire("CUSTOM", "1:8", 24, {
        name: "uar.tool.approval_required",
        value: { toolCallId: "call-1", name: "shell", riskReason: "writes files" },
      }),
      1,
    );
    expect(approval).toMatchObject({
      event: "agui.tool_call.approval_required",
      payload: { toolCallId: "call-1", riskReason: "writes files" },
    });

    const cancelled = new UarAguiAdapter().ingest(
      "RUN_ERROR",
      wire("RUN_ERROR", "2:8", 40, { code: "CANCELLED", message: "Run cancelled" }),
      2,
    );
    expect(cancelled?.event).toBe("agui.cancelled");

    const failed = new UarAguiAdapter().ingest(
      "RUN_ERROR",
      wire("RUN_ERROR", "3:8", 56, { code: "PROVIDER_ERROR", message: "provider failed" }),
      3,
    );
    expect(failed).toMatchObject({
      event: "agui.error",
      payload: { code: "PROVIDER_ERROR", message: "provider failed" },
    });
  });

  test("passes RAW through as an opaque event row", () => {
    const adapter = new UarAguiAdapter(() => 10);
    const result = adapter.ingest(
      "RAW",
      wire("RAW", "5:8", 88, { event: { external: true } }),
      5,
    );

    expect(result).toMatchObject({
      event: "agui.raw",
      payload: { type: "RAW", event: { external: true } },
      eventRow: { id: "5:8", type: "RAW", phase: null, receivedAt: 10 },
    });
    expect(result?.messageChunk).toBeUndefined();
  });
});
