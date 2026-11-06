import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { PersistedRun, PersistedRunEvent } from "@/platform/pglite/run-event-repository";
import type { RichMessage } from "@/types/chat-content";
import { useChatMessageStore } from "./chat-message-store";

const db = vi.hoisted(() => ({ getMessages: vi.fn(), getRuns: vi.fn<() => Promise<PersistedRun[]>>(async () => []), getRunEvents: vi.fn<() => Promise<PersistedRunEvent[]>>(async () => []), insertMessage: vi.fn(async () => {}) }));
vi.mock("@/platform/pglite/client", () => ({ getDbInstance: () => db }));

describe("chat hydration ownership before stream dispatch", () => {
  beforeEach(() => {
    db.getMessages.mockReset();
    db.getRuns.mockReset().mockResolvedValue([]);
    db.getRunEvents.mockReset().mockResolvedValue([]);
    db.insertMessage.mockClear();
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("hydration must not dispatch network requests"));
    useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
  });

  afterEach(() => {
    try {
      expect(globalThis.fetch).not.toHaveBeenCalled();
    } finally {
      vi.restoreAllMocks();
    }
  });

  it("shares pending hydration and preserves an intervening live message", async () => {
    let resolve!: (messages: RichMessage[]) => void;
    db.getMessages.mockReturnValue(new Promise<RichMessage[]>((done) => { resolve = done; }));
    const store = useChatMessageStore.getState();
    const first = store.loadMessagesFromDb("delayed-history");
    expect(store.loadMessagesFromDb("delayed-history")).toBe(first);
    expect(db.getMessages).toHaveBeenCalledTimes(1);
    store.beginStream("delayed-history", "live-run");
    store.appendTextDelta("delayed-history", "live-run", "live answer");
    resolve([{ id: "old", role: "assistant", content: [], createdAt: new Date(), status: "complete" }]);
    await first;
    expect(useChatMessageStore.getState().messagesByThread["delayed-history"]).toHaveLength(1);
    expect(useChatMessageStore.getState().messagesByThread["delayed-history"]?.[0]?.id).not.toBe("old");
  });

  it("propagates failed hydration and releases only its own pending operation", async () => {
    db.getMessages.mockRejectedValueOnce(new Error("owned database unavailable"));
    await expect(useChatMessageStore.getState().loadMessagesFromDb("failed-history")).rejects.toThrow("owned database unavailable");
    db.getMessages.mockResolvedValueOnce([]);
    await useChatMessageStore.getState().loadMessagesFromDb("failed-history");
    expect(db.getMessages).toHaveBeenCalledTimes(2);
  });
  // Mocked repository rows shaped like the native producer and original-delta
  // persistence path; this fixture does not exercise real PGlite durability.
  it("recovers a durable partial delta after abrupt page loss without claiming completion", async () => {
    const thread = "partial-history";
    db.getMessages.mockResolvedValueOnce([{ id: "original-user", role: "user", content: [{ type: "text", text: "question" }], createdAt: new Date(0), status: "complete" }]);
    db.getRuns.mockResolvedValueOnce([{ id: "admitted", threadId: thread, messageId: null, status: "running", startedAt: "2026-11-02T00:00:00Z",
      finishedAt: null, model: null, usage: null, costUsd: null, phaseTimings: {} }]);
    db.getRunEvents.mockResolvedValueOnce([
      { runId: "admitted", seq: 0, eventId: "1:0", wireSequence: 16,
        type: "RUN_STARTED", kind: "lifecycle", at: "2026-11-02T00:00:00Z",
        payload: { type: "RUN_STARTED", profile: "uar.agui/1", threadId: "admitted", runId: "admitted",
          input: { agentId: "native-agent" }, eventId: "1:0", sequence: 16 } },
      { runId: "admitted", seq: 1, eventId: "2:0", wireSequence: 32,
        type: "TEXT_MESSAGE_CONTENT", kind: "message", at: "2026-11-02T00:00:01Z",
        payload: { type: "TEXT_MESSAGE_CONTENT", profile: "uar.agui/1", threadId: "admitted", runId: "admitted",
          messageId: "admitted:assistant", delta: "durable partial", eventId: "2:0", sequence: 32 } },
    ]);
    await useChatMessageStore.getState().loadMessagesFromDb(thread);
    const messages = useChatMessageStore.getState().messagesByThread[thread];
    expect(messages).toContainEqual(expect.objectContaining({
      id: "run-admitted-assistant-0", runId: "admitted", status: "interrupted", content: [{ type: "text", text: "durable partial" }] }));
    expect(messages?.filter((message) => message.role === "assistant")).toHaveLength(1);
    expect(messages?.some((message) => message.role === "assistant" && message.status === "complete")).toBe(false);
    expect(db.getRunEvents).toHaveBeenCalledWith("admitted");
    expect(db.insertMessage).toHaveBeenCalledWith(thread, expect.objectContaining({ status: "interrupted" }));
    expect(db.insertMessage).toHaveBeenCalledTimes(1);
    expect(globalThis.fetch).not.toHaveBeenCalled();
  });

  it("refuses the original delta-only journal without inventing an assistant or redispatching", async () => {
    const thread = "malformed-partial-history";
    const user: RichMessage = { id: "original-user", role: "user", content: [{ type: "text", text: "question" }], createdAt: new Date(0), status: "complete" };
    db.getMessages.mockResolvedValueOnce([user]);
    db.getRuns.mockResolvedValueOnce([{ id: "admitted", threadId: thread, messageId: null, status: "running", startedAt: "2026-11-02T00:00:00Z",
      finishedAt: null, model: null, usage: null, costUsd: null, phaseTimings: {} }]);
    db.getRunEvents.mockResolvedValueOnce([{ runId: "admitted", seq: 1, eventId: "delta-1", wireSequence: 1,
      type: "TEXT_MESSAGE_CONTENT", kind: "message", at: "2026-11-02T00:00:01Z", payload: { delta: "durable partial" } }]);
    await useChatMessageStore.getState().loadMessagesFromDb(thread);
    expect(useChatMessageStore.getState().messagesByThread[thread]).toEqual([user]);
    expect(db.insertMessage).not.toHaveBeenCalled();
    expect(globalThis.fetch).not.toHaveBeenCalled();
  });
});
