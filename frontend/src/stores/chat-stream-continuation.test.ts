import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { PGlite } from "@electric-sql/pglite";
import { live, type PGliteWithLive } from "@electric-sql/pglite/live";
import { INITIAL_SCHEMA_SQL, MIGRATIONS, RUN_EVENT_MIGRATION_SQL } from "@/platform/pglite/migrations";
import { RunEventRepository } from "@/platform/pglite/run-event-repository";
import { acquireChatStream } from "@/platform/entities/a2ui-actions/stream-ownership";
import { useChatMessageStore } from "./chat-message-store";
import { useChatStreamStore } from "./chat-stream-store";
import { ingestAgUiEvent, ingestRuntimeEvent } from "@/entities/runtime-ingest";

const io = vi.hoisted(() => ({
  resume: vi.fn(), post: vi.fn(),
  db: { getMessages: vi.fn(), getRuns: vi.fn<RunEventRepository["getRuns"]>(async () => []), getRunEvents: vi.fn<RunEventRepository["getRunEvents"]>(async () => []), insertMessage: vi.fn(async () => {}), touchThread: vi.fn(async () => {}),
    startRun: vi.fn<RunEventRepository["startRun"]>(async () => {}), appendRunEvent: vi.fn<RunEventRepository["appendEvent"]>(async () => 1), finishRun: vi.fn<RunEventRepository["finishRun"]>(async () => {}) },
}));
vi.mock("@/services/chat-stream-api", () => ({ resumeRunStream: io.resume, postChatCompletion: io.post,
  cancelRun: vi.fn(async () => {}), CHAT_COMPLETION_URL: "/api/chat/completion" }));
vi.mock("@/features/settings/api", () => ({ fetchResilienceSettings: vi.fn(async () => ({})), onSettingsChanged: () => () => {} }));
vi.mock("@/platform/pglite/client", () => ({ getDbInstance: () => io.db }));
vi.mock("@/entities/runtime-ingest", () => ({ ingestAgUiEvent: vi.fn(), ingestRuntimeEvent: vi.fn() }));
let serial = 0;
function stream(runId: string, terminal: "RUN_FINISHED" | "RUN_ERROR" | null, terminalFields: Record<string, unknown> = {}, diagnostic = false) {
  const rows = [{ type: "RUN_STARTED", runId, threadId: runId },
    { type: "TEXT_MESSAGE_CONTENT", runId, messageId: "answer", delta: "actual continuation" },
    ...(diagnostic ? [{ type: "CUSTOM", runId, threadId: runId, name: "uar.presentation.diagnostic", value: {
      code: "session_persist_failed", message: "Conversation terminal could not be saved",
    } }] : []),
    ...(terminal ? [{ type: terminal, runId, message: "interrupted", ...terminalFields }] : [])];
  return new Response(rows.map((row, index) => `id: ${index + 1}\ndata: ${JSON.stringify({ ...row,
    profile: "uar.agui/1", eventId: `${index + 1}:0`, sequence: (index + 1) * 16 })}\n\n`).join(""),
  { headers: { "Content-Type": "text/event-stream" } });
}
beforeEach(() => {
  io.resume.mockReset(); io.post.mockReset();
  io.db.getMessages.mockResolvedValue([{ id: "earlier", role: "user", content: [{ type: "text", text: "prior fact" }], createdAt: new Date(), status: "complete" }]);
  io.db.appendRunEvent.mockReset().mockResolvedValue(1);
  io.db.startRun.mockReset().mockResolvedValue(undefined);
  io.db.finishRun.mockReset().mockResolvedValue(undefined);
  io.db.getRuns.mockReset().mockResolvedValue([]);
  io.db.getRunEvents.mockReset().mockResolvedValue([]);
  io.db.insertMessage.mockReset().mockResolvedValue(undefined);
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


// Actual shipped stream loop → adapter → persistence → PGlite/repository.
// Only HTTP and the unrelated message-storage facade are substituted.
describe("local observer abort and authoritative native reattachment", () => {
  let db: PGlite & PGliteWithLive;
  let repository: RunEventRepository;
  beforeEach(async () => {
    db = await PGlite.create({ extensions: { live } });
    await db.exec(INITIAL_SCHEMA_SQL);
    await db.exec(RUN_EVENT_MIGRATION_SQL);
    for (const migration of MIGRATIONS.filter((item) => item.version > 2)) await db.exec(migration.up);
    repository = new RunEventRepository(db);
    io.db.startRun.mockImplementation((...args: Parameters<RunEventRepository["startRun"]>) => repository.startRun(...args));
    io.db.finishRun.mockImplementation((...args: Parameters<RunEventRepository["finishRun"]>) => repository.finishRun(...args));
    io.db.appendRunEvent.mockImplementation((...args: Parameters<RunEventRepository["appendEvent"]>) => repository.appendEvent(...args));
    io.db.getRuns.mockImplementation(() => repository.getRuns());
    io.db.getRunEvents.mockImplementation((...args: Parameters<RunEventRepository["getRunEvents"]>) => repository.getRunEvents(...args));
  });
  afterEach(async () => { await db.close(); });

  test.each([
    { label: "partial", usage: { inputTokens: null, outputTokens: 5, totalTokens: null, model: "reported-model", costUsdEstimate: 0.02 }, counts: undefined },
    { label: "absent", usage: undefined, counts: undefined },
    { label: "cache-independent", usage: { inputTokens: null, outputTokens: 5, totalTokens: null,
      uncachedInputTokens: 2, cacheReadInputTokens: 3, cacheCreationInputTokens: 1 }, counts: undefined },
    { label: "zero", usage: { inputTokens: 0, outputTokens: 0, totalTokens: 0 }, counts: { inputTokens: 0, outputTokens: 0, totalTokens: 0 } },
    { label: "complete", usage: { inputTokens: 2, outputTokens: 5, totalTokens: 7 }, counts: { inputTokens: 2, outputTokens: 5, totalTokens: 7 } },
  ])("successful continuation retains $label native usage availability", async ({ usage, counts }) => {
    const runId = `usage-continuation-${++serial}`;
    const thread = `thread-${serial}`;
    await db.query("INSERT INTO threads (id, title, is_ephemeral) VALUES ($1, 'Usage continuation', false)", [thread]);
    io.resume.mockResolvedValueOnce(stream(runId, "RUN_FINISHED", usage ? { result: { usage } } : {}));
    const lease = acquireChatStream();
    try { await expect(useChatStreamStore.getState().attachContinuation(thread, runId, lease)).resolves.toBeUndefined(); }
    finally { lease.release(); }
    const assistant = useChatMessageStore.getState().messagesByThread[thread].filter((row) => row.role === "assistant").at(-1);
    expect(assistant).toMatchObject({ status: "complete", runId });
    expect(assistant?.usage).toEqual(counts);
    if (!counts) expect(assistant?.chunks?.some((chunk) => chunk.kind === "usage")).toBe(false);
    expect(io.db.insertMessage).toHaveBeenCalledWith(thread, expect.objectContaining({ id: assistant?.id, status: "complete" }));
    const run = (await repository.getRuns()).find((row) => row.id === runId);
    expect(run).toMatchObject({ status: "finished", usage: usage ?? null });
    expect(run?.costUsd).toBe(usage && "costUsdEstimate" in usage ? usage.costUsdEstimate : null);
    const terminal = (await repository.getRunEvents(runId)).filter((row) => row.eventId === "3:0");
    expect(terminal).toHaveLength(1);
    expect(terminal[0]?.payload.result).toEqual(usage ? { usage } : undefined);
    expect(io.post).not.toHaveBeenCalled();
    await useChatMessageStore.getState().finishStream(thread, "interrupted");
    expect((await repository.getRuns()).find((row) => row.id === runId)).toEqual(run);
  });

  test.each([
    { type: "RUN_FINISHED" as const, fields: { result: { usage: { totalTokens: 4 } } }, status: "finished", message: "complete", resolves: true },
    { type: "RUN_ERROR" as const, fields: { code: "PROVIDER_ERROR", message: "provider failed" }, status: "error", message: "failed", resolves: false },
    { type: "RUN_ERROR" as const, fields: { code: "CANCELLED", message: "Run cancelled" }, status: "cancelled", message: "cancelled", resolves: false },
    { type: "RUN_ERROR" as const, fields: { code: "CANCELLED", message: "Run cancelled", result: { usage: { inputTokens: null, outputTokens: 3, totalTokens: null, costUsdEstimate: 0.02 } } }, status: "cancelled", message: "cancelled", resolves: false },
    { type: "RUN_ERROR" as const, fields: { code: "CANCELLED", message: "Run cancelled", result: { usage: { inputTokens: 0, outputTokens: 3, totalTokens: 3, costUsdEstimate: 0.02 } } }, status: "cancelled", message: "cancelled", resolves: false },
  ])("abort remains provisional before later $type/$status", async (terminal) => {
    const runId = `reattach-${++serial}`;
    const thread = `thread-${serial}`;
    await db.query("INSERT INTO threads (id, title, is_ephemeral) VALUES ($1, 'Recovery', false)", [thread]);
    io.resume.mockImplementationOnce(async (_runId: string, _cursor: number, signal: AbortSignal) => new Response(
      new ReadableStream<Uint8Array>({ start(controller) {
        signal.addEventListener("abort", () => controller.error(Object.assign(new Error("observer stopped"), { name: "AbortError" })), { once: true });
        controller.enqueue(new TextEncoder().encode(`id: 2\ndata: ${JSON.stringify({ type: "TEXT_MESSAGE_CONTENT", profile: "uar.agui/1", eventId: "2:0", sequence: 32, runId, messageId: "answer", delta: "actual continuation" })}\n\n`));
      } }), { headers: { "Content-Type": "text/event-stream", "x-uar-run-id": runId } },
    ));
    let lease = acquireChatStream();
    try {
      const pending = useChatStreamStore.getState().attachContinuation(thread, runId, lease);
      const rejected = expect(pending).rejects.toThrow("interrupted");
      await vi.waitFor(() => expect(io.db.appendRunEvent).toHaveBeenCalled());
      useChatStreamStore.getState().cancelStream();
      await rejected;
    } finally { lease.release(); }
    expect(await repository.getRuns()).toMatchObject([{ id: runId, status: "interrupted", usage: null }]);
    expect(useChatMessageStore.getState().messagesByThread[thread].some((row) => row.role === "assistant" && row.status === "interrupted")).toBe(true);

    const cancelled = terminal.type === "RUN_ERROR" && terminal.fields.code === "CANCELLED";
    io.resume.mockResolvedValueOnce(stream(runId, terminal.type, terminal.fields, cancelled));
    lease = acquireChatStream();
    try {
      const reattachment = useChatStreamStore.getState().attachContinuation(thread, runId, lease);
      if (terminal.resolves) await expect(reattachment).resolves.toBeUndefined();
      else await expect(reattachment).rejects.toThrow();
    } finally { lease.release(); }
    expect(await repository.getRuns()).toMatchObject([{ id: runId, status: terminal.status }]);
    const messages = useChatMessageStore.getState().messagesByThread[thread];
    expect(messages.some((row) => row.role === "assistant" && row.status === terminal.message)).toBe(true);
    if (terminal.type === "RUN_FINISHED") {
      // A reported total alone does not establish either constituent counter.
      expect(messages.filter((row) => row.role === "assistant").at(-1)?.usage).toBeUndefined();
    }
    if (terminal.type === "RUN_ERROR" && terminal.fields.code === "CANCELLED") {
      const usage = terminal.fields.result?.usage ?? null;
      expect((await repository.getRuns())[0]?.usage).toEqual(usage);
      if (usage) {
        expect((await repository.getRuns())[0]?.costUsd).toBe(0.02);
        const messageUsage = messages.filter((row) => row.role === "assistant").at(-1)?.usage;
        if (usage.inputTokens === null) expect(messageUsage).toBeUndefined();
        else expect(messageUsage).toEqual({ inputTokens: 0, outputTokens: 3, totalTokens: 3 });
      }
    }
    expect(io.post).not.toHaveBeenCalled();
    const persistedEvents = await repository.getRunEvents(runId);
    expect(persistedEvents.filter((row) => row.eventId === (cancelled ? "4:0" : "3:0"))).toHaveLength(1);
    if (cancelled) {
      expect(persistedEvents.filter((row) => row.type === "CUSTOM")).toMatchObject([{
        eventId: "3:0", wireSequence: 48, payload: {
          type: "CUSTOM", name: "uar.presentation.diagnostic", runId, value: {
            code: "session_persist_failed", message: "Conversation terminal could not be saved",
          },
        },
      }]);
      expect(persistedEvents.filter((row) => row.type === "RUN_ERROR")).toMatchObject([{
        eventId: "4:0", wireSequence: 64, payload: { code: "CANCELLED", runId },
      }]);
      expect(persistedEvents.findIndex((row) => row.eventId === "3:0"))
        .toBeLessThan(persistedEvents.findIndex((row) => row.eventId === "4:0"));
      expect(persistedEvents.some((row) => row.type === "RUN_FINISHED")).toBe(false);
    }
    expect(io.db.finishRun.mock.calls.some(([input]) => input.status === "cancelled" && !input.nativeTerminal)).toBe(false);
  });

  test.each([
    { type: "RUN_FINISHED", result: { usage: { inputTokens: 70, outputTokens: 80, totalTokens: 150 } } },
    { type: "RUN_ERROR", code: "PROVIDER_ERROR", message: "foreign provider detail" },
    { type: "RUN_ERROR", code: "CANCELLED", message: "foreign cancellation", result: { usage: { inputTokens: 70, outputTokens: 80, totalTokens: 150 } } },
    { type: "TEXT_MESSAGE_CONTENT", messageId: "foreign-answer", delta: "foreign content" },
    { type: "STATE_SNAPSHOT", snapshot: { foreign: true } },
    { type: "CUSTOM", name: "uar.memory.recall", value: { items: [{ id: "foreign-memory", content: "foreign content" }] } },
  ])("continuation rejects foreign $type before runtime/chat reduction", async (foreign) => {
    const runId = `bound-continuation-${++serial}`;
    const foreignRunId = `foreign-continuation-${serial}`;
    const thread = `bound-thread-${serial}`;
    await db.query("INSERT INTO threads (id, title, is_ephemeral) VALUES ($1, 'Boundary', false)", [thread]);
    const nativeFrame = (nativeRunId: string, sequence: number, payload: Record<string, unknown>) =>
      `id: ${sequence}\ndata: ${JSON.stringify({ ...payload, runId: nativeRunId, profile: "uar.agui/1", eventId: `${sequence}:0`, sequence: sequence * 16 })}\n\n`;
    io.resume.mockResolvedValueOnce(new Response(
      nativeFrame(runId, 1, { type: "RUN_STARTED", threadId: runId })
      + nativeFrame(runId, 2, { type: "TEXT_MESSAGE_CONTENT", messageId: `${runId}:assistant`, delta: "original continuation partial" })
      + nativeFrame(foreignRunId, 3, foreign),
      { headers: { "Content-Type": "text/event-stream" } },
    ));
    vi.mocked(ingestAgUiEvent).mockClear();
    vi.mocked(ingestRuntimeEvent).mockClear();
    let lease = acquireChatStream();
    try { await expect(useChatStreamStore.getState().attachContinuation(thread, runId, lease)).rejects.toThrow("identity"); }
    finally { lease.release(); }
    const partial = useChatMessageStore.getState().messagesByThread[thread];
    const assistant = partial.filter((message) => message.role === "assistant");
    expect(assistant).toHaveLength(1);
    expect(assistant[0]).toMatchObject({ id: `run-${runId}-assistant-0`, runId, status: "interrupted",
      content: [{ type: "text", text: "original continuation partial" }] });
    expect(assistant[0]?.usage).toBeUndefined();
    expect(assistant[0]?.chunks?.some((chunk) => chunk.kind === "error" || chunk.kind === "usage" || chunk.kind === "memory-recall" || chunk.kind === "state-snapshot")).toBe(false);
    expect(await repository.getRuns()).toMatchObject([{ id: runId, status: "interrupted", usage: null, phaseTimings: {} }]);
    expect(await repository.getRunEvents(foreignRunId)).toEqual([]);
    expect((await repository.getRunEvents(runId)).some((event) => event.eventId === "3:0")).toBe(false);
    expect((await repository.getRunEvents(runId)).find((event) => event.eventId === "2:0")?.payload)
      .toMatchObject({ type: "TEXT_MESSAGE_CONTENT", runId, messageId: `${runId}:assistant`, delta: "original continuation partial" });
    expect(vi.mocked(ingestAgUiEvent).mock.calls.map(([, event]) => event.id)).toEqual(["1:0", "2:0"]);
    expect(vi.mocked(ingestRuntimeEvent).mock.calls).toEqual([]);
    // Reattach the exact original ID; no replacement chat dispatch is allowed.
    io.resume.mockResolvedValueOnce(new Response(nativeFrame(runId, 4, { type: "RUN_FINISHED" }),
      { headers: { "Content-Type": "text/event-stream", "x-uar-run-id": runId } }));
    lease = acquireChatStream();
    try { await expect(useChatStreamStore.getState().attachContinuation(thread, runId, lease)).resolves.toBeUndefined(); }
    finally { lease.release(); }
    expect(useChatMessageStore.getState().messagesByThread[thread].filter((message) => message.role === "assistant"))
      .toMatchObject([{ id: assistant[0]?.id, runId, status: "complete", content: [{ type: "text", text: "original continuation partial" }] }]);
    expect(await repository.getRuns()).toMatchObject([{ id: runId, status: "finished" }]);
    expect(io.post).not.toHaveBeenCalled();
    expect(io.resume).toHaveBeenCalledTimes(2);
  });

  test.each([
    { label: "bodyless", body: null },
    { label: "forged stream", body: "id: 1\ndata: invalid-forged-stream\n\n" },
  ])("continuation receipt refuses a conflicting response header before identity adoption ($label)", async ({ body }) => {
    const runId = `header-continuation-${++serial}`;
    const thread = `header-thread-${serial}`;
    const foreignRunId = `header-foreign-${serial}`;
    const response = new Response(body, { headers: { "Content-Type": "text/event-stream", "x-uar-run-id": foreignRunId } });
    io.resume.mockResolvedValueOnce(response);
    const lease = acquireChatStream();
    try { await expect(useChatStreamStore.getState().attachContinuation(thread, runId, lease)).rejects.toThrow("identity"); }
    finally { lease.release(); }
    expect(await repository.getRuns()).toEqual([]);
    expect(useChatMessageStore.getState().messagesByThread[thread].filter((message) => message.role === "assistant")).toEqual([]);
    expect(response.bodyUsed).toBe(false);
    expect(io.db.startRun).not.toHaveBeenCalled();
    expect(io.db.appendRunEvent).not.toHaveBeenCalled();
    expect(io.db.finishRun).not.toHaveBeenCalled();
    expect(io.post).not.toHaveBeenCalled();
    expect(io.resume).toHaveBeenCalledTimes(1);
  });

  test.each([undefined, "", 42])("terminal with invalid native identity %s cannot settle the attached continuation", async (terminalRunId) => {
    const runId = `weak-continuation-${++serial}`;
    const thread = `weak-thread-${serial}`;
    await db.query("INSERT INTO threads (id, title, is_ephemeral) VALUES ($1, 'Weak proof', false)", [thread]);
    const nativeFrame = (sequence: number, payload: Record<string, unknown>) =>
      `id: ${sequence}\ndata: ${JSON.stringify({ ...payload, profile: "uar.agui/1", eventId: `${sequence}:0`, sequence: sequence * 16 })}\n\n`;
    io.resume.mockResolvedValueOnce(new Response(
      nativeFrame(1, { type: "RUN_STARTED", runId, threadId: runId })
      + nativeFrame(2, { type: "TEXT_MESSAGE_CONTENT", runId, messageId: "answer", delta: "weak proof partial" })
      + nativeFrame(3, { type: "RUN_FINISHED", runId: terminalRunId, result: { usage: { inputTokens: 9, outputTokens: 9, totalTokens: 18 } } }),
      { headers: { "Content-Type": "text/event-stream", "x-uar-run-id": runId } },
    ));
    vi.mocked(ingestAgUiEvent).mockClear();
    vi.mocked(ingestRuntimeEvent).mockClear();
    const lease = acquireChatStream();
    try { await expect(useChatStreamStore.getState().attachContinuation(thread, runId, lease)).rejects.toThrow("identity"); }
    finally { lease.release(); }
    const assistant = useChatMessageStore.getState().messagesByThread[thread].filter((message) => message.role === "assistant");
    expect(assistant).toHaveLength(1);
    expect(assistant[0]).toMatchObject({ id: `run-${runId}-assistant-0`, runId, status: "interrupted",
      content: [{ type: "text", text: "weak proof partial" }] });
    expect(assistant[0]?.usage).toBeUndefined();
    expect(await repository.getRuns()).toMatchObject([{ id: runId, status: "interrupted", usage: null, phaseTimings: {} }]);
    expect((await repository.getRunEvents(runId)).some((event) => event.eventId === "3:0")).toBe(false);
    expect(vi.mocked(ingestAgUiEvent).mock.calls.map(([, event]) => event.id)).toEqual(["1:0", "2:0"]);
    expect(vi.mocked(ingestRuntimeEvent).mock.calls).toEqual([]);
    expect(io.post).not.toHaveBeenCalled();
    expect(io.resume).toHaveBeenCalledTimes(1);
  });

});
