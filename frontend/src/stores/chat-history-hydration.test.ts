import { beforeEach, describe, expect, it, vi } from "vitest";
import type { RichMessage } from "@/types/chat-content";
import { useChatMessageStore } from "./chat-message-store";

const db = vi.hoisted(() => ({ getMessages: vi.fn() }));
vi.mock("@/platform/pglite/client", () => ({ getDbInstance: () => db }));

describe("chat hydration ownership before stream dispatch", () => {
  beforeEach(() => {
    db.getMessages.mockReset();
    useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
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
});
