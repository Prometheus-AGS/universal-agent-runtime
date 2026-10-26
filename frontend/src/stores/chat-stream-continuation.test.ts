import { beforeEach, describe, expect, test, vi } from "vitest";
import { acquireChatStream } from "@/platform/entities/a2ui-actions/stream-ownership";
import { useChatMessageStore } from "./chat-message-store";
import { useChatStreamStore } from "./chat-stream-store";

const io = vi.hoisted(() => ({
  resume: vi.fn(), post: vi.fn(),
  db: { getMessages: vi.fn(), insertMessage: vi.fn(async () => {}), touchThread: vi.fn(async () => {}),
    startRun: vi.fn(async () => {}), appendRunEvent: vi.fn(async () => 1), finishRun: vi.fn(async () => {}) },
}));
vi.mock("@/services/chat-stream-api", () => ({ resumeRunStream: io.resume, postChatCompletion: io.post,
  cancelRun: vi.fn(async () => {}), CHAT_COMPLETION_URL: "/api/chat/completion" }));
vi.mock("@/features/settings/api", () => ({ fetchResilienceSettings: vi.fn(async () => ({})), onSettingsChanged: () => () => {} }));
vi.mock("@/platform/pglite/client", () => ({ getDbInstance: () => io.db }));
vi.mock("@/entities/runtime-ingest", () => ({ ingestAgUiEvent: vi.fn(), ingestRuntimeEvent: vi.fn() }));
let serial = 0;
function stream(runId: string, terminal: "RUN_FINISHED" | "RUN_ERROR" | null) {
  const rows = [{ type: "RUN_STARTED", runId, threadId: runId },
    { type: "TEXT_MESSAGE_CONTENT", runId, messageId: "answer", delta: "actual continuation" },
    ...(terminal ? [{ type: terminal, runId, message: "interrupted" }] : [])];
  return new Response(rows.map((row, index) => `id: ${index + 1}\ndata: ${JSON.stringify({ ...row,
    profile: "uar.agui/1", eventId: `${runId}:${index + 1}`, sequence: index + 1 })}\n\n`).join(""),
  { headers: { "Content-Type": "text/event-stream" } });
}
beforeEach(() => {
  io.resume.mockReset(); io.post.mockReset();
  io.db.getMessages.mockResolvedValue([{ id: "earlier", role: "user", content: [{ type: "text", text: "prior fact" }], createdAt: new Date(), status: "complete" }]);
  io.db.appendRunEvent.mockReset().mockResolvedValue(1);
});

describe("attach an admitted continuation through the existing stream loop", () => {
  test("GETs the admitted run, preserves history and never appends a fake user message or POSTs chat", async () => {
    const runId = `continuation-${++serial}`;
    const thread = `thread-${serial}`;
    io.resume.mockResolvedValue(stream(runId, "RUN_FINISHED"));
    const lease = acquireChatStream();
    try { await useChatStreamStore.getState().attachContinuation(thread, runId, lease); }
    finally { lease.release(); }
    expect(io.post).not.toHaveBeenCalled();
    expect(io.resume).toHaveBeenCalledWith(runId, 0, expect.any(AbortSignal));
    const messages = useChatMessageStore.getState().messagesByThread[thread];
    expect(messages.filter((row) => row.role === "user")).toHaveLength(1);
    expect(messages[0].content).toEqual([{ type: "text", text: "prior fact" }]);
    expect(messages.some((row) => row.role === "assistant" && row.status === "complete")).toBe(true);
  });
  test.each(["RUN_ERROR", null] as const)("does not certify completion for terminal %s", async (terminal) => {
    const runId = `continuation-${++serial}`;
    io.resume.mockResolvedValue(stream(runId, terminal));
    const lease = acquireChatStream();
    try { await expect(useChatStreamStore.getState().attachContinuation(`thread-${serial}`, runId, lease)).rejects.toThrow(); }
    finally { lease.release(); }
    expect(io.post).not.toHaveBeenCalled();
  });
  test("a native terminal whose event persistence failed remains interrupted", async () => {
    const runId = `continuation-${++serial}`;
    io.resume.mockResolvedValue(stream(runId, "RUN_FINISHED"));
    io.db.appendRunEvent.mockRejectedValue(new Error("local persistence failed"));
    const lease = acquireChatStream();
    try { await expect(useChatStreamStore.getState().attachContinuation(`thread-${serial}`, runId, lease)).rejects.toThrow("saved"); }
    finally { lease.release(); }
  });
});
