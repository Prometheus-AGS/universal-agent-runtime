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
  test.each([
    { name: "uar.artifact.available", event: "agui.artifact" },
    { name: "uar.artifact.input_required", event: "agui.artifact_input_request" },
  ])("bridges native $name fields without rewriting journal evidence", ({ name, event }) => {
    const value = { artifactId: "published-surface", artifactType: "a2ui/surface", title: "Published",
      content: "[]", ...(name === "uar.artifact.available" ? { language: "json" } : {}), sourceRunId: "run-1",
      metadata: { profile: "uar.a2ui/1", surface_generations: { "surface-7": 7 }, source_run_id: "run-1" } };
    const data = wire("CUSTOM", "2:8", 40, { name, value });
    const adapted = new UarAguiAdapter().ingest("CUSTOM", data, 2);
    expect(adapted).toMatchObject({ event, payload: { ...value,
      artifact_id: value.artifactId, artifact_type: value.artifactType, request_id: "run-1" } });
    expect(adapted?.eventRow?.payload).toEqual(JSON.parse(data));
    expect(adapted?.eventRow?.payload.value).toEqual(value);
    expect(adapted?.eventRow?.payload.value).not.toHaveProperty("artifact_id");
    expect(adapted?.runtimeChunk).toMatchObject(name === "uar.artifact.available"
      ? { kind: "a2ui-display", toolCallId: value.artifactId, component: value.artifactType,
        profile: value.metadata.profile, payload: value }
      : { kind: "a2ui-input", component: value.artifactType, payload: value });
    const legacyValue = { artifact_id: "legacy-surface", artifact_type: "a2ui/surface",
      title: "Legacy", content: "[]", metadata: value.metadata };
    expect(new UarAguiAdapter().ingest(event, JSON.stringify({ request_id: "run-1", ...legacyValue }), 2)?.payload)
      .toEqual({ request_id: "run-1", ...legacyValue });
    expect(new UarAguiAdapter().ingest("CUSTOM", wire("CUSTOM", "4:8", 72, {
      name, value: { ...legacyValue, ...value },
    }), 4)?.payload).toMatchObject({ artifact_id: value.artifactId, artifact_type: value.artifactType });
  });

  for (const type of ["RUN_FINISHED", "RUN_ERROR"] as const) {
    test.each([
      { label: "output only", usage: { inputTokens: null, outputTokens: 5, totalTokens: null, model: "reported-model", costUsdEstimate: 0.02 }, counts: undefined },
      { label: "input only", usage: { inputTokens: 5, outputTokens: null, totalTokens: null }, counts: undefined },
      { label: "unknown total", usage: { inputTokens: 2, outputTokens: 5, totalTokens: null }, counts: undefined },
      { label: "all null", usage: { inputTokens: null, outputTokens: null, totalTokens: null }, counts: undefined },
      { label: "absent", usage: undefined, counts: undefined },
      { label: "cache evidence independent of aggregate", usage: { inputTokens: null, outputTokens: 5, totalTokens: null,
        uncachedInputTokens: 2, cacheReadInputTokens: 3, cacheCreationInputTokens: 1, costUsdEstimate: 0.02, model: "reported-model" }, counts: undefined },
      { label: "known zero", usage: { inputTokens: 0, outputTokens: 0, totalTokens: 0 }, counts: { inputTokens: 0, outputTokens: 0, totalTokens: 0 } },
      { label: "complete with unavailable cache evidence", usage: { inputTokens: 2, outputTokens: 5, totalTokens: 7, model: "reported-model", costUsdEstimate: 0.02 },
        counts: { inputTokens: 2, outputTokens: 5, totalTokens: 7 } },
    ])(`${type} preserves $label usage availability`, ({ usage, counts }) => {
      const fields = { ...(type === "RUN_ERROR" ? { code: "CANCELLED", message: "Run cancelled" } : {}),
        ...(usage ? { result: { usage } } : {}) };
      const adapted = new UarAguiAdapter().ingest(type, wire(type, "3:0", 48, fields), 3);
      expect(adapted?.event).toBe(type === "RUN_FINISHED" ? "agui.done" : "agui.cancelled");
      if (usage) {
        expect(adapted?.eventRow?.payload.result).toEqual({ usage });
        expect(adapted?.payload.usage).toMatchObject({ input_tokens: usage.inputTokens,
          output_tokens: usage.outputTokens, total_tokens: usage.totalTokens });
      } else expect(adapted?.eventRow?.payload.result).toBeUndefined();
      if (counts) expect(adapted?.runtimeChunk).toMatchObject({ kind: "usage", ...counts });
      else {
        expect(adapted?.runtimeChunk).toMatchObject({ kind: "raw", type,
          payload: { type, ...fields } });
        expect(adapted?.runtimeChunk).not.toHaveProperty("inputTokens");
        expect(adapted?.runtimeChunk).not.toHaveProperty("outputTokens");
        expect(adapted?.runtimeChunk).not.toHaveProperty("totalTokens");
      }
    });
  }

  test("projects routing notices through both protocols without legacy sequence poisoning", () => {
    const value = { sourceAgentId: "orchestrator-agent", agent: { id: "registered", title: "Registered" },
      selectedThreadId: "child", selectionMethod: "intent", rosterRevision: "revision" };
    const adapter = new UarAguiAdapter();
    const legacy = adapter.ingest("agui.agent.selected", JSON.stringify({ request_id: "run-1", value }), 1);
    expect(legacy?.runtimeChunk).toMatchObject({ kind: "agent-selection", selectedThreadId: "child", agentId: "registered" });
    const official = adapter.ingest("CUSTOM", wire("CUSTOM", "2:8", 40, { name: "uar.agent.selected", value }), 2);
    expect(official?.runtimeChunk).toMatchObject({ kind: "agent-selection", id: legacy?.runtimeChunk?.id });
    expect(adapter.ingest("TEXT_MESSAGE_CONTENT", wire("TEXT_MESSAGE_CONTENT", "3:8", 56, { delta: "answer" }), 3)?.messageChunk)
      .toMatchObject({ kind: "text", delta: "answer" });
  });

  test("retains native cancellation usage without inventing unknown counts or accepting another protocol", () => {
    const cancelled = new UarAguiAdapter().ingest("RUN_ERROR", wire("RUN_ERROR", "9:8", 152, {
      code: "CANCELLED", message: "Run cancelled", result: { usage: {
        inputTokens: null, outputTokens: 3, totalTokens: null, costUsdEstimate: 0.02, model: "reported-model",
      } },
    }), 9);
    expect(cancelled).toMatchObject({ event: "agui.cancelled", eventId: "9:8", sequence: 152,
      payload: { kind: "cancelled", usage: { input_tokens: null, output_tokens: 3, total_tokens: null } },
      eventRow: { payload: { code: "CANCELLED", result: { usage: { outputTokens: 3, inputTokens: null } } } },
    });
    expect(new UarAguiAdapter().ingest("RUN_ERROR", wire("RUN_ERROR", "10:8", 168, {
      code: "CANCELLED", message: "Run cancelled",
    }), 10)?.payload.usage).toBeUndefined();
    expect(cancelled?.runtimeChunk).toMatchObject({ kind: "raw", payload: { code: "CANCELLED" } });
    const known = new UarAguiAdapter().ingest("RUN_ERROR", wire("RUN_ERROR", "11:8", 184, {
      code: "CANCELLED", message: "Run cancelled", result: { usage: {
        inputTokens: 0, outputTokens: 3, totalTokens: 3, costUsdEstimate: 0.02,
      } },
    }), 11);
    expect(known?.event).toBe("agui.cancelled");
    expect(known?.runtimeChunk).toMatchObject({ kind: "usage", inputTokens: 0, outputTokens: 3, totalTokens: 3, costUsd: 0.02 });
    // The standard /ag-ui binding stream is a separate unsupported protocol.
    expect(new UarAguiAdapter().ingest("message", JSON.stringify({
      type: "RUN_FINISHED", runId: "wire-run", threadId: "wire-thread", outcome: { type: "cancelled" },
    }), undefined)).toBeNull();
  });

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
