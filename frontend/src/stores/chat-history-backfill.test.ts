import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { RichMessage } from "@/types/chat-content";
import type { ServerSessionMessage } from "@/services/chat-sessions-api";
import { toChunks } from "@/features/chat/model/to-chunks";
import { useChatMessageStore } from "./chat-message-store";

const db = vi.hoisted(() => ({ insertMessage: vi.fn(async () => {}), getMessages: vi.fn(async (): Promise<RichMessage[]> => []), getRuns: vi.fn(async () => []) }));
const api = vi.hoisted(() => ({ fetchSessionMessages: vi.fn<() => Promise<ServerSessionMessage[] | null>>() }));
vi.mock("@/platform/pglite/client", () => ({ getDbInstance: () => db }));
vi.mock("@/services/chat-sessions-api", () => api);

const at = "2026-11-02T00:00:00Z";
const metadata = { usage_availability: "available", usage: { input_tokens: 2, output_tokens: 3, total_tokens: 5 },
  approvals_availability: "available", approvals: [{ approval_id: "approval", tool_call_id: "call", tool_name: "inspect",
    state: "approved", historical: true, resolvable: false, decision: { approved: true, decided_at: at } }] };
function history(toolOnly = false): ServerSessionMessage[] {
  return [
    { message_id: "user-0", role: "user", content: "inspect", status: "complete", created_at: at },
    { message_id: "run-r-assistant-0", run_id: "r", role: "assistant", content: [
      ...(toolOnly ? [] : [{ type: "text" as const, text: "Checking now." }]),
      { type: "toolUse", id: "call", name: "inspect", inputJson: "{}" },
      { type: "toolResult", toolUseId: "call", outputJson: "found", isError: false }], status: "complete", run_metadata: metadata },
    { message_id: "run-r-assistant-1", run_id: "r", role: "assistant", content: "Final answer.", status: "complete", run_metadata: metadata },
    { message_id: "user-1", role: "user", content: "next", status: "complete", created_at: "2026-11-02T00:01:00Z" },
  ];
}
function local(): RichMessage[] {
  return [
    { id: "user-0", role: "user", content: [{ type: "text", text: "inspect" }], createdAt: new Date(at), status: "complete" },
    { id: "run-r-assistant-0", role: "assistant", runId: "r", content: [
      { type: "text", text: "Checking now.Final answer." },
      { type: "toolUse", id: "call", name: "inspect", inputJson: "{}" },
      { type: "toolResult", toolUseId: "call", outputJson: "found", isError: false },
      { type: "artifact", id: "retained", kind: "text/plain", content: "tool evidence" },
      { type: "image", url: "data:image/png;base64,eA==", mime: "image/png", dataBase64: null }], createdAt: new Date(at), status: "complete" },
    { id: "user-1", role: "user", content: [{ type: "text", text: "next" }], createdAt: new Date("2026-11-02T00:01:00Z"), status: "complete" },
  ];
}
function rows() { return useChatMessageStore.getState().messagesByThread.thread; }
function chunks() { return rows().flatMap((row) => row.chunks ?? toChunks(row.content, { messageId: row.id, at, runId: row.runId, finalized: true })); }

describe("canonical tool-loop backfill", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("backfill fixture must not dispatch network requests"));
    useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
  });

  afterEach(() => {
    try {
      expect(globalThis.fetch).not.toHaveBeenCalled();
    } finally {
      vi.restoreAllMocks();
    }
  });

  for (const warm of [false, true]) for (const toolOnly of [false, true]) {
    it(`restores every canonical segment once (warm=${warm}, toolOnly=${toolOnly}) through reload`, async () => {
      api.fetchSessionMessages.mockResolvedValue(history(toolOnly));
      useChatMessageStore.getState().initThread("thread", warm ? local() : []);
      await useChatMessageStore.getState().backfillFromServer("thread");
      const restored = rows();
      expect(restored.map((row) => row.id)).toEqual(history().map((row) => row.message_id));
      expect(restored.filter((row) => row.role === "assistant").map((row) => row.content.filter((block) => block.type === "text").map((block) => block.text).join("")))
        .toEqual([toolOnly ? "" : "Checking now.", "Final answer."]);
      expect(chunks().filter((chunk) => chunk.kind === "tool-call")).toEqual([expect.objectContaining({ toolCallId: "call", result: "found", status: "complete" })]);
      expect(chunks().filter((chunk) => chunk.kind === "tool-approval")).toEqual([expect.objectContaining({ approvalId: "approval", historical: true, historicalState: "approved", decision: "approved" })]);
      expect(chunks().filter((chunk) => chunk.kind === "usage")).toHaveLength(1);
      if (warm) {
        expect(restored[1].content).toContainEqual(expect.objectContaining({ type: "artifact", id: "retained" }));
        expect(restored[1].content).toContainEqual(expect.objectContaining({ type: "image", url: "data:image/png;base64,eA==" }));
      }
      await useChatMessageStore.getState().backfillFromServer("thread");
      expect(rows()).toEqual(restored);
      db.getMessages.mockResolvedValueOnce(restored);
      await useChatMessageStore.getState().loadMessagesFromDb("thread");
      await useChatMessageStore.getState().backfillFromServer("thread");
      expect(rows()).toEqual(restored);
      expect(db.insertMessage).toHaveBeenCalledWith("thread", expect.objectContaining({ id: "run-r-assistant-1", status: "complete" }));
    });
  }

  it("refuses a server missing later user turns and preserves partial interruption truth", async () => {
    const before = local();
    const server = history().slice(0, 3);
    server[2] = { ...server[2], content: "Partial final", status: "interrupted" };
    api.fetchSessionMessages.mockResolvedValue(server);
    // Local users ahead of the server keep the existing mismatch safeguard.
    useChatMessageStore.getState().initThread("thread", before);
    await useChatMessageStore.getState().backfillFromServer("thread");
    expect(rows()).toEqual(before);
    expect(db.insertMessage).not.toHaveBeenCalled();
    useChatMessageStore.getState().initThread("thread", before.slice(0, 2));
    await useChatMessageStore.getState().backfillFromServer("thread");
    expect(rows()[2]).toMatchObject({ id: "run-r-assistant-1", status: "interrupted", createdAtKnown: false });
  });

  for (const variant of ["same", "prefix", "changed", "changed-final", "unanchored"] as const) it(`reconciles actual server text-only segments while retaining local tool evidence (${variant})`, async () => {
    const before = local();
    before[1].content[0] = { type: "text", text: "Checking now." };
    before[1].content.push({ type: "text", text: "Evidence checked." });
    before[1].chunks = toChunks(before[1].content, { messageId: before[1].id, at, runId: "r", finalized: true });
    before[1].chunks.splice(before[1].chunks.length - 1, 0, { kind: "tool-approval", id: "original-approval", at, seq: 5,
      runId: "r", approvalId: "approval", toolCallId: "call", toolName: "inspect", args: {}, historical: true,
      historicalState: "approved", decision: "approved", decidedAt: at });
    const intro = variant === "changed" ? "Rechecked now." : variant === "unanchored" ? "Revised explanation." : "Checking now.";
    const conclusion = variant === "changed-final" ? "Updated evidence." : variant === "same" || variant === "changed" ? "Evidence checked." : "";
    const server = history();
    server[1] = { ...server[1], content: [{ type: "text", text: intro + conclusion }] };
    api.fetchSessionMessages.mockResolvedValue(server);
    useChatMessageStore.getState().initThread("thread", before);
    await useChatMessageStore.getState().backfillFromServer("thread");
    expect(rows().map((row) => row.id)).toEqual(server.map((row) => row.message_id));
    expect(rows()[2].content).toContainEqual({ type: "text", text: "Final answer." });
    expect(chunks().filter((chunk) => chunk.kind === "tool-call")).toEqual([expect.objectContaining({ toolCallId: "call", result: "found", status: "complete" })]);
    expect(chunks().filter((chunk) => chunk.kind === "tool-approval")).toHaveLength(1);
    expect(rows()[1].content.filter(block => block.type !== "artifact" || block.id !== "run-r-assistant-0:history")
      .map(block => block.type === "text" ? block.text : block.type)).toEqual([
      intro, "toolUse", "toolResult", "artifact", "image", ...(conclusion ? [conclusion] : []),
    ]);
    const rich = rows()[1].chunks!.filter(chunk => chunk.kind !== "usage"
      && !(chunk.kind === "artifact" && chunk.mime === "application/vnd.uar.history-provenance+json"));
    expect(rich.map(chunk => chunk.kind === "markdown" ? chunk.source : chunk.kind))
      .toEqual([intro, "tool-call", "artifact", "image", "tool-approval", ...(conclusion ? [conclusion] : [])]);
    expect(rich.filter(chunk => chunk.kind !== "markdown").map(chunk => chunk.id))
      .toEqual(before[1].chunks.filter(chunk => chunk.kind !== "markdown").map(chunk => chunk.id));
    expect(rich.find(chunk => chunk.kind === "tool-approval"))
      .toMatchObject({ id: "original-approval", decision: "approved", historicalState: "approved", decidedAt: at });
    const restored = rows();
    await useChatMessageStore.getState().backfillFromServer("thread");
    expect(rows()).toEqual(restored);
    db.getMessages.mockResolvedValueOnce(restored);
    await useChatMessageStore.getState().loadMessagesFromDb("thread");
    await useChatMessageStore.getState().backfillFromServer("thread");
    expect(rows()).toEqual(restored);
  });

  for (const canonicalTime of [at, null]) {
    it(`uses canonical timestamp availability consistently (created_at=${canonicalTime})`, async () => {
      const before = local();
      before[1].createdAt = canonicalTime ? new Date(0) : new Date(at);
      before[1].createdAtKnown = !canonicalTime;
      const server = history();
      server[1] = { ...server[1], created_at: canonicalTime };
      api.fetchSessionMessages.mockResolvedValue(server);
      useChatMessageStore.getState().initThread("thread", before);
      await useChatMessageStore.getState().backfillFromServer("thread");
      const restored = rows()[1];
      const expectedTime = new Date(canonicalTime ?? 0).toISOString();
      expect(restored.id).toBe("run-r-assistant-0");
      expect(restored.historyPosition).toBe(1);
      expect(restored.createdAt.toISOString()).toBe(expectedTime);
      expect(restored.createdAtKnown).toBe(Boolean(canonicalTime));
      expect(restored.content).toContainEqual({ type: "text", text: "Checking now." });
      expect(restored.chunks?.find(chunk => chunk.kind === "markdown")).toMatchObject({ at: expectedTime, source: "Checking now." });
      const provenance = restored.content.find(block => block.type === "artifact" && block.kind === "application/vnd.uar.history-provenance+json");
      expect(provenance?.type === "artifact" && JSON.parse(provenance.content).created_at_availability).toBe(canonicalTime ? "available" : "unavailable");
      expect(db.insertMessage).toHaveBeenCalledWith("thread", expect.objectContaining({ id: restored.id,
        createdAt: new Date(canonicalTime ?? 0), createdAtKnown: Boolean(canonicalTime) }));
      await useChatMessageStore.getState().backfillFromServer("thread");
      expect(rows()[1]).toEqual(restored);
    });
  }

  it("preserves original legacy user IDs while inserting segments before the next user", async () => {
    const before = local();
    before[0].id = "original-local-user";
    before[2].id = "original-next-user";
    const server = history();
    // The API marks identity-free history with legacy aliases. Distinct
    // modern server IDs are no longer evidence of the same cached user turn.
    server[0].message_id = "legacy:thread:0:user";
    server[3].message_id = "legacy:thread:3:user";
    api.fetchSessionMessages.mockResolvedValue(server);
    useChatMessageStore.getState().initThread("thread", before);
    await useChatMessageStore.getState().backfillFromServer("thread");
    expect(rows().map((row) => row.id)).toEqual(["original-local-user", "run-r-assistant-0", "run-r-assistant-1", "original-next-user"]);
    await useChatMessageStore.getState().backfillFromServer("thread");
    expect(rows().map((row) => row.id)).toEqual(["original-local-user", "run-r-assistant-0", "run-r-assistant-1", "original-next-user"]);
  });

  it("repairs prior appended-segment order without duplicating canonical rows", async () => {
    const before = local();
    before.push({ id: "run-r-assistant-1", runId: "r", role: "assistant", content: [{ type: "text", text: "Final answer." }], createdAt: new Date(0), status: "complete" });
    api.fetchSessionMessages.mockResolvedValue(history());
    useChatMessageStore.getState().initThread("thread", before);
    await useChatMessageStore.getState().backfillFromServer("thread");
    expect(rows().map((row) => row.id)).toEqual(history().map((row) => row.message_id));
    expect(chunks().filter((chunk) => chunk.kind === "tool-approval")).toHaveLength(1);
    expect(chunks().filter((chunk) => chunk.kind === "usage")).toHaveLength(1);
  });

  it("never overwrites a live stream acquired during the history request", async () => {
    let resolve!: (value: ServerSessionMessage[]) => void;
    api.fetchSessionMessages.mockReturnValueOnce(new Promise((done) => { resolve = done; }));
    useChatMessageStore.getState().initThread("thread", local());
    const pending = useChatMessageStore.getState().backfillFromServer("thread");
    useChatMessageStore.getState().beginStream("thread", "live");
    useChatMessageStore.getState().appendTextDelta("thread", "live", "live reply");
    const live = rows();
    resolve(history());
    await pending;
    expect(rows()).toBe(live);
    expect(db.insertMessage).not.toHaveBeenCalled();
  });

  for (const variant of ["caption", "image-only", "reordered", "mixed", "identical-legacy", "modern-text", "client-turn"] as const) {
    it(`leaves all cached history untouched when user identity is ambiguous (${variant})`, async () => {
      const before = local();
      const first = { type: "image" as const, url: "data:image/png;base64,eA==", mime: "image/png", dataBase64: null };
      const second = { ...first, url: "data:image/png;base64,eQ==" };
      const caption = { type: "text" as const, text: "inspect" };
      before[0].content = variant === "modern-text" || variant === "client-turn" ? [caption]
        : variant === "image-only" ? [first] : variant === "reordered" ? [caption, first, second] : [caption, first];
      before[0].chunks = toChunks(before[0].content, { messageId: before[0].id, at, finalized: true });
      if (variant === "client-turn") before[0].clientTurnId = "original-turn";
      before[1].chunks = toChunks(before[1].content, { messageId: before[1].id, at, runId: "r", finalized: true });
      before[1].chunks.push({ kind: "tool-approval", id: "original-approval", at, seq: 8, runId: "r",
        approvalId: "approval", toolCallId: "call", toolName: "inspect", args: {}, historical: true,
        historicalState: "approved", decision: "approved", decidedAt: at });
      const server = history();
      server[0] = { ...server[0], message_id: variant === "modern-text" ? "different-user" : "legacy:thread:0:user",
        content: variant === "modern-text" || variant === "client-turn" ? [caption]
          : variant === "image-only" ? [second] : variant === "reordered" ? [caption, second, first]
            : variant === "mixed" ? [first, caption] : variant === "identical-legacy" ? [caption, first] : [caption, second],
        ...(variant === "client-turn" ? { client_turn_id: "different-turn" } : {}) };
      api.fetchSessionMessages.mockResolvedValue(server);
      const original = structuredClone(before);
      useChatMessageStore.getState().initThread("thread", before);
      const owned = rows();
      await useChatMessageStore.getState().backfillFromServer("thread");
      expect(rows()).toBe(owned);
      expect(rows()).toEqual(original);
      expect(db.insertMessage).not.toHaveBeenCalled();
      db.getMessages.mockResolvedValueOnce(original);
      await useChatMessageStore.getState().loadMessagesFromDb("thread");
      await useChatMessageStore.getState().backfillFromServer("thread");
      expect(rows()).toEqual(original);
      expect(rows()[0].content).toEqual(original[0].content);
      expect(rows()[1].chunks).toEqual(original[1].chunks);
      expect(db.insertMessage).not.toHaveBeenCalled();
      expect(api.fetchSessionMessages).toHaveBeenCalledTimes(2);
    });
  }

  it("accepts an exact stable image identity and safe legacy text alias", async () => {
    for (const image of [true, false]) {
      const before = local().slice(0, 2);
      if (image) before[0].content.push({ type: "image", url: "data:image/png;base64,eA==", mime: "image/png", dataBase64: null });
      const server = history().slice(0, 3);
      server[0] = { ...server[0], message_id: image ? before[0].id : "legacy:thread:0:user", content: before[0].content };
      api.fetchSessionMessages.mockResolvedValue(server);
      useChatMessageStore.getState().initThread("thread", before);
      await useChatMessageStore.getState().backfillFromServer("thread");
      expect(rows().map(row => row.id)).toEqual([before[0].id, before[1].id, "run-r-assistant-1"]);
      if (image) expect(rows()[0].content).toContainEqual(before[0].content[1]);
      expect(rows()[2].content).toContainEqual({ type: "text", text: "Final answer." });
    }
  });
});
