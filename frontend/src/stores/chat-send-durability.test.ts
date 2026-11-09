import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { registerChatTurnEntities, persistChatTurn, readChatTurn } from "@/platform/entities/chat-turns";
import { bindA2uiActionPersistence } from "@/platform/entities/a2ui-actions/durability";
import { useChatMessageStore } from "./chat-message-store";
import { useChatStreamStore } from "./chat-stream-store";
const io = vi.hoisted(() => ({ post: vi.fn(), recover: vi.fn(),
  db: { getMessages: vi.fn(async () => []), getRuns: vi.fn(async () => []), getRunEvents: vi.fn(async () => []),
    insertMessage: vi.fn(async () => {}), touchThread: vi.fn(async () => {}) } }));
vi.mock("@/services/chat-stream-api", () => ({ postChatCompletion: io.post, fetchChatTurnReceipt: io.recover,
  resumeRunStream: vi.fn(), cancelRun: vi.fn(), CHAT_COMPLETION_URL: "/api/chat/completion" }));
vi.mock("@/platform/pglite/client", () => ({ getDbInstance: () => io.db }));
vi.mock("@/features/settings/api", () => ({ fetchResilienceSettings: vi.fn(async () => ({})), onSettingsChanged: () => () => {} }));
vi.mock("@/entities/runtime-ingest", () => ({ ingestRuntimeEvent: vi.fn(), ingestAgUiEvent: vi.fn() }));
let serial = 0;
beforeEach(() => {
  registerChatTurnEntities(); bindA2uiActionPersistence(async () => {});
  io.post.mockReset(); io.recover.mockReset(); io.db.insertMessage.mockClear();
  useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
});
afterEach(() => bindA2uiActionPersistence(null));
describe("durable turn dispatch", () => {
  test("failed persistence retains the rich visible draft and sends no POST", async () => {
    const id = `unsaved-${++serial}`;
    bindA2uiActionPersistence(async () => { throw new Error("storage unavailable"); });
    const error = vi.fn();
    await useChatStreamStore.getState().startStream(id, { message: "inspect image", clientTurnId: id, persistedUserMessageId: `${id}-user`,
      attachments: [{ id: "image", filename: "image.png", url: "data:image/png;base64,eA==", content_type: "image/png" }] }, { onError: error });
    expect(io.post).not.toHaveBeenCalled();
    expect(useChatMessageStore.getState().messagesByThread[id][0]).toMatchObject({ id: `${id}-user`, content: [
      { type: "text", text: "inspect image" }, expect.objectContaining({ type: "image", url: "data:image/png;base64,eA==" })] });
    expect(error).toHaveBeenCalledWith(expect.objectContaining({ message: expect.stringContaining("no request was sent") }));
  });
  test("response loss never automatically POSTs the admitted intent again", async () => {
    const id = `uncertain-${++serial}`;
    io.post.mockRejectedValue(new TypeError("network lost before headers"));
    await useChatStreamStore.getState().startStream(id, { message: "one intent", clientTurnId: id, persistedUserMessageId: `${id}-user` });
    expect(io.post).toHaveBeenCalledTimes(1);
    expect(JSON.parse(io.post.mock.calls[0][0])).toMatchObject({ client_turn_id: id, user_message_id: `${id}-user` });
    expect(readChatTurn(id)).toMatchObject({ state: "uncertain", user_message_id: `${id}-user` });
    io.recover.mockResolvedValue({ client_turn_id: id, session_id: id, state: "unknown", retry_allowed: false });
    await useChatStreamStore.getState().startStream(id, { message: "one intent", clientTurnId: id, persistedUserMessageId: `${id}-user` });
    expect(io.post).toHaveBeenCalledTimes(1);
    expect(io.recover).toHaveBeenCalledWith(id, id);
  });
  test("restored pending receipt is only reconciled, never submitted again", async () => {
    const id = `restored-${++serial}`;
    await persistChatTurn({ id, session_id: id, user_message_id: `${id}-user`, state: "sending", run_id: null,
      message: { id: `${id}-user`, role: "user", content: [{ type: "text", text: "saved draft" }], createdAt: new Date(0) } });
    io.recover.mockRejectedValue(new Error("recovery unavailable"));
    await useChatStreamStore.getState().startStream(id, { message: "saved draft", clientTurnId: id, persistedUserMessageId: `${id}-user` });
    expect(io.post).not.toHaveBeenCalled();
  });
});
