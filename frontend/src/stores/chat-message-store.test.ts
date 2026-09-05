import { beforeEach, describe, expect, test } from "vitest";
import { decodePersistedChatContent } from "@/platform/pglite/legacy-chat-content";
import {
  selectRetryAttempt,
  selectRetryDelayMs,
  selectRetryMaxAttempts,
  useChatMessageStore,
} from "./chat-message-store";

const THREAD_ID = "thread-test";

describe("chat-message-store streaming", () => {
  beforeEach(() => {
    useChatMessageStore.setState({
      messagesByThread: {},
      streamingByThread: {},
    });
  });

  test("beginStream marks thread as awaiting first token", () => {
    useChatMessageStore.getState().initThread(THREAD_ID, []);
    useChatMessageStore.getState().beginStream(THREAD_ID, "run-1");

    const streaming = useChatMessageStore.getState().streamingByThread[THREAD_ID];
    expect(streaming).toBeDefined();
    expect(streaming?.isStreaming).toBe(true);
    expect(streaming?.runId).toBe("run-1");
    expect(streaming?.awaitingFirstToken).toBe(true);
  });

  test("markStreamStarted clears waiting only for matching run", () => {
    useChatMessageStore.getState().initThread(THREAD_ID, []);
    useChatMessageStore.getState().beginStream(THREAD_ID, "run-1");

    useChatMessageStore.getState().markStreamStarted(THREAD_ID, "run-2");
    expect(useChatMessageStore.getState().streamingByThread[THREAD_ID]?.awaitingFirstToken).toBe(true);

    useChatMessageStore.getState().markStreamStarted(THREAD_ID, "run-1");
    expect(useChatMessageStore.getState().streamingByThread[THREAD_ID]?.awaitingFirstToken).toBe(false);
  });

  test("addToolCall creates assistant message even before text deltas", () => {
    useChatMessageStore.getState().initThread(THREAD_ID, []);
    useChatMessageStore.getState().beginStream(THREAD_ID, "run-1");

    useChatMessageStore.getState().addToolCall(THREAD_ID, "run-1", {
      type: "tool-call",
      toolCallId: "memory-recall-1",
      toolName: "__memory_recall__",
      args: { count: 1, items: [] },
      status: "complete",
    });

    const messages = useChatMessageStore.getState().messagesByThread[THREAD_ID] ?? [];
    expect(messages).toHaveLength(1);
    expect(messages[0]?.role).toBe("assistant");
    expect(messages[0]?.status).toBe("in_progress");
    expect(messages[0]?.content).toEqual([]);
    expect(messages[0]?.chunks).toEqual([expect.objectContaining({
      kind: "memory-recall",
      items: [],
    })]);

    expect(useChatMessageStore.getState().streamingByThread[THREAD_ID]?.awaitingFirstToken).toBe(false);
  });

  test("addRuntimeChunk keeps approval interruptions visible", () => {
    useChatMessageStore.getState().beginStream(THREAD_ID, "run-approval");
    useChatMessageStore.getState().addRuntimeChunk(THREAD_ID, "run-approval", {
      id: "approval-1",
      kind: "tool-approval",
      at: "2026-08-19T00:00:00.000Z",
      runId: "run-approval",
      seq: 1,
      toolCallId: "call-1",
      toolName: "native_echo",
      args: { text: "hello" },
      reason: "operator decision required",
    });

    expect(useChatMessageStore.getState().messagesByThread[THREAD_ID]?.[0]?.chunks).toEqual([
      expect.objectContaining({ kind: "tool-approval", toolCallId: "call-1" }),
    ]);
  });

  test("updates A2UI input and display chunks when tool results arrive", () => {
    const store = useChatMessageStore.getState();
    store.initThread(THREAD_ID, []);
    store.beginStream(THREAD_ID, "run-1");
    store.addToolCall(THREAD_ID, "run-1", {
      type: "tool-call",
      toolCallId: "input-1",
      toolName: "__a2ui_input__",
      args: { runId: "request-1", artifactType: "confirm", title: "Confirm" },
      status: "running",
    });
    store.updateToolCall(THREAD_ID, "input-1", { result: "accepted", status: "complete" });
    store.addToolCall(THREAD_ID, "run-1", {
      type: "tool-call",
      toolCallId: "display-1",
      toolName: "__a2ui_display__",
      args: {
        artifactType: "Card",
        title: "Result",
        metadata: { profile: "uar.a2ui/1", protocol_version: "v0.9.1" },
      },
      status: "running",
    });
    store.updateToolCall(THREAD_ID, "display-1", { result: "Rendered content", status: "complete" });

    const message = useChatMessageStore.getState().messagesByThread[THREAD_ID]?.[0];
    expect(message?.chunks?.find(({ kind }) => kind === "a2ui-input")).toMatchObject({
      toolCallId: "input-1",
      status: "submitted",
      response: "accepted",
    });
    expect(message?.chunks?.find(({ kind }) => kind === "a2ui-display")).toMatchObject({
      toolCallId: "display-1",
      profile: "uar.a2ui/1",
      version: "v0.9.1",
      payload: expect.objectContaining({ content: "Rendered content" }),
    });
    expect(message?.content.find((block) => block.type === "artifact" && block.id === "display-1")).toMatchObject({
      kind: "application/vnd.uar.a2ui+json",
    });
    expect(JSON.parse(String(message?.content.find((block) => block.type === "artifact" && block.id === "input-1")?.content))).toMatchObject({
      component: "confirm",
      status: "submitted",
      response: "accepted",
    });
    expect(JSON.parse(String(message?.content.find((block) => block.type === "artifact" && block.id === "display-1")?.content))).toMatchObject({
      component: "Card",
      payload: expect.objectContaining({ content: "Rendered content" }),
      validation: "valid",
    });
  });

  test("keeps thinking and reasoning as distinct runtime chunks", () => {
    const store = useChatMessageStore.getState();
    store.initThread(THREAD_ID, []);
    store.beginStream(THREAD_ID, "run-1");
    store.appendThinkingDelta(THREAD_ID, "run-1", "deliberation", "thinking");
    store.appendThinkingDelta(THREAD_ID, "run-1", "analysis", "reasoning");

    expect(useChatMessageStore.getState().messagesByThread[THREAD_ID]?.[0]?.chunks).toEqual([
      expect.objectContaining({ kind: "thinking", text: "deliberation" }),
      expect.objectContaining({ kind: "reasoning", text: "analysis" }),
    ]);
  });

  test.each([
    ["json", "json", "application/json", '{"presentation":{"requested_mode":"text"}}'],
    ["note", "text", "text/plain", "A readable answer, not an interactive surface"],
  ])("ordinary %s artifacts keep their content and title through canonical reload", (artifactType, language, mime, content) => {
    const store = useChatMessageStore.getState();
    store.initThread(THREAD_ID, []);
    store.beginStream(THREAD_ID, "run-artifact");
    store.addToolCall(THREAD_ID, "run-artifact", {
      type: "tool-call", toolCallId: "ordinary", toolName: "__a2ui_display__",
      args: { artifactType, language, title: "Run policy", metadata: {} },
      result: content, status: "complete",
    });
    const message = useChatMessageStore.getState().messagesByThread[THREAD_ID]?.[0];
    expect(message?.content).toEqual([{ type: "artifact", id: "ordinary", kind: mime, title: "Run policy", content }]);
    expect(message?.chunks).toEqual([expect.objectContaining({ kind: "artifact", mime, title: "Run policy", content })]);
    // Rebuild exclusively from persisted canonical content, not derived chunks.
    const restored = decodePersistedChatContent(JSON.parse(JSON.stringify(message?.content)), {
      messageId: "reloaded", at: "2026-09-05T00:00:00Z", finalized: true,
    });
    expect(restored.content).toEqual(message?.content);
    expect(restored.chunks).toEqual([expect.objectContaining({ kind: "artifact", mime, title: "Run policy", content })]);
  });

  test.each([
    { artifactType: "a2ui", language: "json", metadata: {} },
    { artifactType: "display", language: "json", metadata: {} },
    { artifactType: "a2ui/template", language: "json", metadata: {} },
    { artifactType: "json", language: "application/a2ui+json", metadata: {} },
    { artifactType: "json", language: "application/vnd.uar.a2ui+json", metadata: {} },
    { artifactType: "json", language: "a2ui", metadata: {} },
    { artifactType: "json", language: "json", metadata: { profile: "uar.a2ui/unknown" } },
  ])("explicit A2UI declaration stays on the validation path: %j", (declaration) => {
    const store = useChatMessageStore.getState();
    store.initThread(THREAD_ID, []);
    store.beginStream(THREAD_ID, "run-artifact");
    store.addToolCall(THREAD_ID, "run-artifact", {
      type: "tool-call", toolCallId: "declared-ui", toolName: "__a2ui_display__",
      args: { ...declaration, validation: "invalid", validationError: "Unsupported profile" },
      result: "{}", status: "complete",
    });
    const message = useChatMessageStore.getState().messagesByThread[THREAD_ID]?.[0];
    expect(message?.chunks).toEqual([expect.objectContaining({ kind: "a2ui-display", validation: "invalid" })]);
    expect(message?.content[0]).toMatchObject({ type: "artifact", kind: "application/vnd.uar.a2ui+json" });
  });

  test("persists terminal tool status even when the tool has no output", () => {
    const store = useChatMessageStore.getState();
    store.initThread(THREAD_ID, []);
    store.beginStream(THREAD_ID, "run-1");
    store.addToolCall(THREAD_ID, "run-1", {
      type: "tool-call",
      toolCallId: "no-output",
      toolName: "notify",
      args: {},
      status: "running",
    });
    store.updateToolCall(THREAD_ID, "no-output", { status: "complete" });

    expect(useChatMessageStore.getState().messagesByThread[THREAD_ID]?.[0]?.content).toContainEqual({
      type: "toolResult",
      toolUseId: "no-output",
      outputJson: "",
      isError: false,
    });
  });

  test("keeps usage chunk identity stable across metadata updates", () => {
    const store = useChatMessageStore.getState();
    store.initThread(THREAD_ID, []);
    store.beginStream(THREAD_ID, "run-1");
    store.setMessageMeta(THREAD_ID, "run-1", { usage: { inputTokens: 1, outputTokens: 2, totalTokens: 3 } });
    const first = useChatMessageStore.getState().messagesByThread[THREAD_ID]?.[0]?.chunks?.find(({ kind }) => kind === "usage");
    store.setMessageMeta(THREAD_ID, "run-1", { model: "openai/gpt-5", usage: { inputTokens: 4, outputTokens: 5, totalTokens: 9 } });
    const updated = useChatMessageStore.getState().messagesByThread[THREAD_ID]?.[0]?.chunks?.find(({ kind }) => kind === "usage");

    expect(updated).toMatchObject({ id: first?.id, seq: first?.seq, inputTokens: 4, outputTokens: 5, totalTokens: 9, model: "openai/gpt-5" });
  });

  test("retry selectors expose primitive values", () => {
    useChatMessageStore.getState().initThread(THREAD_ID, []);
    useChatMessageStore.getState().beginStream(THREAD_ID, "run-1");
    useChatMessageStore.getState().setAwaitingRetry(THREAD_ID, "run-1", 2, 5, 1200);

    const state = useChatMessageStore.getState();
    expect(selectRetryAttempt(THREAD_ID)(state)).toBe(2);
    expect(selectRetryMaxAttempts(THREAD_ID)(state)).toBe(5);
    expect(selectRetryDelayMs(THREAD_ID)(state)).toBe(1200);
  });
});
