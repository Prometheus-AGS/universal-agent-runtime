import { beforeEach, describe, expect, test } from "bun:test";
import { useChatMessageStore } from "./chat-message-store";

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
    expect(messages[0]?.content).toHaveLength(1);

    const firstBlock = messages[0]?.content[0];
    expect(firstBlock?.type).toBe("tool-call");
    if (firstBlock?.type === "tool-call") {
      expect(firstBlock.toolName).toBe("__memory_recall__");
      expect(firstBlock.status).toBe("complete");
    }

    expect(useChatMessageStore.getState().streamingByThread[THREAD_ID]?.awaitingFirstToken).toBe(false);
  });
});
