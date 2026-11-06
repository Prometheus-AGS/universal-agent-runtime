import { act, render, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { PGlite } from "@electric-sql/pglite";
import { live, type PGliteWithLive } from "@electric-sql/pglite/live";
import { UarDb, setDbInstance } from "@/platform/pglite/client";
import { MIGRATIONS } from "@/platform/pglite/migrations";
import { registerChatTurnEntities, readChatTurn, pendingChatTurns } from "@/platform/entities/chat-turns";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { registerA2uiActionEntities } from "@/platform/entities/a2ui-actions";
import { A2UI_ACTION_ENTITY, a2uiActionId } from "@/platform/entities/a2ui-actions/contracts";
import { latestActionReceipt, reconcileA2uiAction } from "@/platform/entities/a2ui-actions/domain";
import { acquireChatStream } from "@/platform/entities/a2ui-actions/stream-ownership";
import { bindA2uiActionPersistence } from "@/platform/entities/a2ui-actions/durability";
import { useChatMessageStore } from "@/stores/chat-message-store";
import { useChatStreamStore } from "@/stores/chat-stream-store";
import { useThreadRegistryStore } from "@/stores/thread-registry-store";
import { useChatRuntime } from "./use-chat-runtime";
import { ingestAgUiEvent, ingestRuntimeEvent } from "@/entities/runtime-ingest";
import { isUarAguiEvent } from "@/platform/agui/agui-schema";
import { toRuntimeChunk } from "@/platform/agui/runtime-chunk-projection";
import { toChunks } from "@/features/chat/model/to-chunks";
import { ChunkRenderer } from "./ui/chunks/chunk-renderer";
import { render as renderMath } from "katex";

const runtimeAdapter = vi.hoisted(() => ({ onCancel: null as null | (() => Promise<void>) }));
const artifactPolicyProps = vi.hoisted(() => ({ display: [] as Record<string, unknown>[], input: [] as Record<string, unknown>[] }));
// Observe the shipped renderer's policy boundary; no action is submitted.
vi.mock("@/features/chat/components/a2ui-artifact-block", () => ({
  A2uiDisplayBlock: (props: Record<string, unknown>) => { artifactPolicyProps.display.push(props); return null; },
  A2uiInputBlock: (props: Record<string, unknown>) => { artifactPolicyProps.input.push(props); return null; },
}));
// Capture the callbacks passed to assistant-ui, not a substitute Stop function.
// The actual React hook, message/stream stores, HTTP service, native adapter,
// message persistence and run-event repository execute below.
vi.mock("@assistant-ui/react", () => ({ useExternalStoreRuntime: (options: { onCancel: () => Promise<void> }) => {
  runtimeAdapter.onCancel = options.onCancel;
  return {};
} }));
vi.mock("@/features/settings/api", () => ({ fetchResilienceSettings: vi.fn(async () => ({})), onSettingsChanged: () => () => {} }));
vi.mock("@/entities/runtime-ingest", () => ({ ingestAgUiEvent: vi.fn(), ingestRuntimeEvent: vi.fn() }));

let serial = 0;
let engine: PGlite & PGliteWithLive;
let database: UarDb;
const encoder = new TextEncoder();
function frame(runId: string, sequence: number, payload: Record<string, unknown>) {
  return `id: ${sequence}\ndata: ${JSON.stringify({ ...payload, runId, profile: "uar.agui/1", eventId: `${sequence}:0`, sequence: sequence * 16 })}\n\n`;
}
function response(body: BodyInit | ReadableStream<Uint8Array>, runId: string) {
  return new Response(body, { headers: { "Content-Type": "text/event-stream", "x-uar-run-id": runId } });
}
async function stop() {
  const callback = runtimeAdapter.onCancel;
  if (!callback) throw new Error("Actual useChatRuntime Stop callback was not registered");
  await act(callback);
}
beforeEach(async () => {
  engine = await PGlite.create({ extensions: { live } });
  for (const migration of MIGRATIONS) await engine.exec(migration.up);
  // Same production-adapter construction seam as chat-history-timestamp.test.ts.
  database = Reflect.construct(UarDb, [engine]) as UarDb;
  setDbInstance(database);
  registerChatTurnEntities();
  bindA2uiActionPersistence(async () => {});
  useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
  useThreadRegistryStore.setState({ threads: {}, activeThreadId: null, hydrated: false });
  runtimeAdapter.onCancel = null;
  artifactPolicyProps.display.length = 0;
  artifactPolicyProps.input.length = 0;
});
afterEach(async () => {
  bindA2uiActionPersistence(null);
  vi.unstubAllGlobals();
  await engine.close();
});

const terminals = [
  { type: "RUN_FINISHED", fields: { result: { usage: { inputTokens: 1, outputTokens: 2, totalTokens: 3 } } }, runStatus: "finished", status: "complete" },
  { type: "RUN_ERROR", fields: { code: "PROVIDER_ERROR", message: "provider failed" }, runStatus: "error", status: "failed" },
  { type: "RUN_ERROR", fields: { code: "CANCELLED", message: "Run cancelled" }, runStatus: "cancelled", status: "cancelled" },
] as const;

test("the standards-mode fixture supports actual KaTeX DOM rendering", () => {
  expect(document.doctype?.name).toBe("html");
  expect(document.doctype?.publicId).toBe("");
  expect(document.doctype?.systemId).toBe("");
  expect(document.compatMode).toBe("CSS1Compat");
  const element = document.createElement("div");
  renderMath("x^2", element, { throwOnError: true });
  expect(element.querySelector(".katex")).not.toBeNull();
  expect(element.querySelector(".katex-mathml math")).not.toBeNull();
  expect(element.querySelector("annotation")?.textContent).toBe("x^2");
});

test.each(["native", "legacy"] as const)("%s artifacts retain publication and input identity through the actual stream and local database", async (protocol) => {
  const thread = `artifact-${++serial}`;
  const runId = `artifact-native-${serial}`;
  const turnId = `artifact-intent-${serial}`;
  const userId = `artifact-user-${serial}`;
  const now = new Date().toISOString();
  const localThread = { id: thread, title: "Native artifacts", isEphemeral: false, createdAt: now, updatedAt: now };
  await database.upsertThread(localThread);
  useThreadRegistryStore.setState({ threads: { [thread]: localThread } });
  await database.insertMessage(thread, { id: userId, role: "user", clientTurnId: turnId, status: "complete", createdAt: new Date(),
    content: [{ type: "text", text: "publish artifacts" }] });
  const metadata = { profile: "uar.a2ui/1", protocol_version: "v0.9.1", source_run_id: runId,
    surface_generations: { "surface-7": 7 }, publication_id: "publication-7" };
  // Same field names as ArtifactDisplay/ArtifactInputRequest in adapters.rs.
  const display = { artifactId: "display-7", artifactType: "a2ui/surface", title: "Published surface",
    content: "[{\"surfaceId\":\"surface-7\"}]", language: "json", metadata, sourceRunId: runId };
  const input = { artifactId: "input-7", artifactType: "confirm", title: "Confirm publication",
    content: "{\"question\":\"Continue?\"}", metadata, sourceRunId: runId };
  const plain = { artifactId: "report-7", artifactType: "application/json", title: "Report",
    content: "{\"retained\":true}", language: "json", metadata: { source_run_id: runId }, sourceRunId: runId };
  function artifactFrame(sequence: number, name: string, value: typeof input | typeof display | typeof plain) {
    if (protocol === "native") return frame(runId, sequence, { type: "CUSTOM", name, value });
    const { artifactId, artifactType, ...rest } = value;
    const event = name === "uar.artifact.input_required" ? "agui.artifact_input_request" : "agui.artifact";
    return `id: ${sequence}\nevent: ${event}\ndata: ${JSON.stringify({ ...rest, artifact_id: artifactId,
      artifact_type: artifactType, request_id: runId })}\n\n`;
  }
  const posts = vi.fn();
  const history = vi.fn();
  vi.stubGlobal("fetch", vi.fn(async (request: RequestInfo | URL): Promise<Response> => {
    const url = String(request);
    if (url === "/api/chat/completion") {
      posts();
      return response(frame(runId, 1, { type: "RUN_STARTED", threadId: runId })
        + artifactFrame(2, "uar.artifact.available", display)
        + artifactFrame(3, "uar.artifact.input_required", input)
        + artifactFrame(4, "uar.artifact.available", plain)
        + frame(runId, 5, { type: "RUN_FINISHED" }), runId);
    }
    if (url === `/api/uar/sessions/${thread}`) { history(); return new Response(null, { status: 404 }); }
    throw new Error(`Unexpected HTTP request: ${url}`);
  }));
  const hook = renderHook(() => useChatRuntime(thread));
  try {
    await waitFor(() => expect(history).toHaveBeenCalledTimes(1));
    await act(async () => { await useChatStreamStore.getState().startStream(thread, {
      message: "publish artifacts", clientTurnId: turnId, persistedUserMessageId: userId,
    }); });
    const saved = await database.getMessages(thread);
    const chunks = saved.flatMap((message) => message.chunks ?? toChunks(message.content, {
      messageId: message.id, runId: message.runId, at: message.createdAt.toISOString(), finalized: true,
    }));
    expect(chunks.filter((chunk) => chunk.kind === "a2ui-display")).toMatchObject([{
      toolCallId: display.artifactId, component: display.artifactType, profile: metadata.profile,
      payload: { title: display.title, language: display.language, content: display.content, metadata },
    }]);
    expect(chunks.filter((chunk) => chunk.kind === "a2ui-input")).toMatchObject([{
      toolCallId: input.artifactId, component: input.artifactType, requestId: runId, status: "awaiting",
      payload: { artifactId: input.artifactId, artifactType: input.artifactType, title: input.title, content: input.content, metadata },
    }]);
    expect(chunks.filter((chunk) => chunk.kind === "artifact")).toMatchObject([{
      artifactId: plain.artifactId, mime: plain.artifactType, title: plain.title, content: plain.content,
    }]);
    const rendered = render(<>{chunks.filter((chunk) => chunk.kind === "a2ui-display" || chunk.kind === "a2ui-input")
      .map((chunk) => <ChunkRenderer key={chunk.id} chunk={chunk} />)}</>);
    expect(artifactPolicyProps.display).toMatchObject([{
      runId, surfaceGenerations: metadata.surface_generations, artifactType: display.artifactType,
      title: display.title, content: display.content, language: display.language, profile: metadata.profile,
    }]);
    expect(artifactPolicyProps.input).toMatchObject([{
      runId, artifactId: input.artifactId, artifactType: input.artifactType, title: input.title,
      content: input.content, metadata, status: "running",
    }]);
    rendered.unmount();
    const blocks = saved.flatMap((message) => message.content);
    expect(blocks.filter((block) => block.type === "artifact").map((block) => block.id))
      .toEqual([display.artifactId, input.artifactId, plain.artifactId]);
    if (protocol === "native") {
      const events = await database.getRunEvents(runId);
      for (const [eventId, value] of [["2:0", display], ["3:0", input], ["4:0", plain]] as const) {
        const row = events.find((event) => event.eventId === eventId);
        expect(row?.payload.value).toEqual(value);
        expect(row?.payload.value).not.toHaveProperty("artifact_id");
        const projected = row && isUarAguiEvent(row.payload) ? toRuntimeChunk(row.payload, Date.now()) : undefined;
        if (value === display) {
          expect(projected).toMatchObject({
            kind: "a2ui-display", id: eventId, runId, seq: row?.payload.sequence,
            toolCallId: display.artifactId, profile: metadata.profile, version: metadata.protocol_version,
            component: display.artifactType, validation: "valid", payload: display,
          });
          expect(projected).toHaveProperty("payload", display);
          expect(projected?.kind).not.toBe("artifact");
        } else {
          expect(projected).toMatchObject(value === input
            ? { kind: "a2ui-input", component: input.artifactType, payload: input }
            : { kind: "artifact", artifactId: value.artifactId, mime: value.artifactType, content: value.content });
        }
      }
    }
    expect(await database.getMessages(thread)).toEqual(saved);
    expect(posts).toHaveBeenCalledTimes(1);
  } finally {
    hook.unmount();
  }
});

describe("actual chat Stop callback and original native turn recovery", () => {
  for (const cancellation of ["failed", "delayed"] as const) {
    test.each(terminals)(`${cancellation} cancellation stays provisional until exact $runStatus terminal`, async (terminal) => {
      const thread = `hook-stop-${++serial}`;
      const runId = `native-${serial}`;
      const turnId = `intent-${serial}`;
      const userId = `user-${serial}`;
      const originalId = `run-${runId}-assistant-0`;
      const olderId = `older-${serial}`;
      const now = new Date().toISOString();
      const localThread = { id: thread, title: "Stop recovery", isEphemeral: false, createdAt: now, updatedAt: now };
      await database.upsertThread(localThread);
      useThreadRegistryStore.setState({ threads: { [thread]: localThread } });
      await database.insertMessage(thread, { id: olderId, role: "assistant", runId: "older-run", status: "complete", createdAt: new Date(0),
        content: [{ type: "text", text: "earlier segment" }] });
      await database.insertMessage(thread, { id: userId, role: "user", clientTurnId: turnId, status: "complete", createdAt: new Date(),
        content: [{ type: "text", text: "one original intent" }] });
      let releaseCancellation: (() => void) | undefined;
      const posts = vi.fn();
      const cancels = vi.fn();
      const recoveries = vi.fn();
      const resumes = vi.fn();
      const history = vi.fn();
      const fetch = vi.fn(async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
        const url = String(input);
        if (url === "/api/chat/completion") {
          posts(JSON.parse(String(init?.body)));
          const signal = init?.signal;
          return response(new ReadableStream<Uint8Array>({ start(controller) {
            signal?.addEventListener("abort", () => controller.error(Object.assign(new Error("observer stopped"), { name: "AbortError" })), { once: true });
            controller.enqueue(encoder.encode(frame(runId, 1, { type: "RUN_STARTED", threadId: runId })
              + frame(runId, 2, { type: "TEXT_MESSAGE_CONTENT", messageId: "answer", delta: "original partial" })));
          } }), runId);
        }
        if (url.endsWith("/cancel")) {
          cancels(url);
          if (cancellation === "failed") throw new TypeError("cancellation network failure");
          return new Promise<Response>((resolve) => { releaseCancellation = () => resolve(new Response(null, { status: 202 })); });
        }
        if (url.startsWith("/api/uar/chat-turns/")) {
          recoveries(url);
          return Response.json({ client_turn_id: turnId, session_id: thread, user_message_id: userId, run_id: runId,
            state: terminal.status === "failed" ? "error" : terminal.status, retry_allowed: false });
        }
        if (url.includes(`/runs/${runId}/stream`)) {
          resumes(url);
          // No replayed text or synthetic start is needed to settle the original partial.
          return response(frame(runId, 3, { type: terminal.type, ...terminal.fields }), runId);
        }
        if (url === `/api/uar/sessions/${thread}`) { history(); return new Response(null, { status: 404 }); }
        throw new Error(`Unexpected HTTP request: ${url}`);
      });
      vi.stubGlobal("fetch", fetch);
      const hook = renderHook(() => useChatRuntime(thread));
      await waitFor(() => expect(history).toHaveBeenCalledTimes(1));
      const payload = { message: "one original intent", clientTurnId: turnId, persistedUserMessageId: userId };
      let pending: Promise<void> = Promise.resolve();
      act(() => { pending = useChatStreamStore.getState().startStream(thread, payload); });
      await waitFor(() => expect(useChatMessageStore.getState().messagesByThread[thread]?.find((message) => message.id === originalId)?.content)
        .toEqual([{ type: "text", text: "original partial" }]));
      await stop();
      await act(async () => { await pending; });
      expect(cancels).toHaveBeenCalledTimes(1);
      expect(readChatTurn(turnId)).toMatchObject({ id: turnId, run_id: runId, user_message_id: userId, state: "uncertain" });
      expect(await database.getRuns()).toMatchObject([{ id: runId, status: "interrupted" }]);
      expect((await database.getMessages(thread)).find((message) => message.id === originalId)).toMatchObject({ status: "interrupted", runId,
        content: [{ type: "text", text: "original partial" }] });
      await act(async () => { await useChatStreamStore.getState().startStream(thread, payload); });
      releaseCancellation?.();
      const saved = await database.getMessages(thread);
      expect(saved.map((message) => message.id)).toEqual([olderId, userId, originalId]);
      expect(saved.find((message) => message.id === originalId)).toMatchObject({ status: terminal.status, runId,
        content: expect.arrayContaining([{ type: "text", text: "original partial" }]) });
      expect(useChatMessageStore.getState().messagesByThread[thread].map((message) => message.id)).toEqual([olderId, userId, originalId]);
      expect(await database.getRuns()).toMatchObject([{ id: runId, status: terminal.runStatus }]);
      const events = await database.getRunEvents(runId);
      expect(events.filter((event) => event.eventId === "3:0")).toHaveLength(1);
      expect(readChatTurn(turnId)).toMatchObject({ id: turnId, user_message_id: userId, run_id: runId, state: "settled" });
      expect(posts).toHaveBeenCalledTimes(1);
      expect(posts).toHaveBeenCalledWith(expect.objectContaining({ client_turn_id: turnId, user_message_id: userId }));
      expect(recoveries).toHaveBeenCalledTimes(1);
      expect(resumes).toHaveBeenCalledTimes(1);
      // A late local Stop/abort cannot rewrite the accepted native outcome or add a bubble.
      await stop();
      await useChatMessageStore.getState().finishStream(thread, "interrupted");
      expect(await database.getMessages(thread)).toEqual(saved);
      expect(await database.getRuns()).toMatchObject([{ id: runId, status: terminal.runStatus }]);
      hook.unmount();
    });
  }
});

// Control: native success reaches durable persistence before Stop wins the
// observer race. Stop must not turn that accepted success into interruption.
test("native success winning the Stop race remains complete under the actual callback", async () => {
  const thread = `stop-race-${++serial}`;
  const runId = `race-native-${serial}`;
  const turnId = `race-intent-${serial}`;
  const now = new Date().toISOString();
  const localThread = { id: thread, title: "Native success race", isEphemeral: false, createdAt: now, updatedAt: now };
  await database.upsertThread(localThread);
  useThreadRegistryStore.setState({ threads: { [thread]: localThread } });
  await database.insertMessage(thread, { id: `user-${serial}`, role: "user", status: "complete", createdAt: new Date(),
    clientTurnId: turnId, content: [{ type: "text", text: "race control" }] });
  const posts = vi.fn();
  const history = vi.fn();
  vi.stubGlobal("fetch", vi.fn(async (input: RequestInfo | URL): Promise<Response> => {
    const url = String(input);
    if (url === "/api/chat/completion") {
      posts();
      return response(frame(runId, 1, { type: "RUN_STARTED", threadId: runId })
        + frame(runId, 2, { type: "TEXT_MESSAGE_CONTENT", messageId: "answer", delta: "completed before Stop" })
        + frame(runId, 3, { type: "RUN_FINISHED" }), runId);
    }
    if (url.endsWith("/cancel")) return new Response(null, { status: 202 });
    if (url === `/api/uar/sessions/${thread}`) { history(); return new Response(null, { status: 404 }); }
    throw new Error(`Unexpected HTTP request: ${url}`);
  }));
  const hook = renderHook(() => useChatRuntime(thread));
  await waitFor(() => expect(history).toHaveBeenCalledTimes(1));
  await act(async () => { await useChatStreamStore.getState().startStream(thread, {
    message: "race control", clientTurnId: turnId, persistedUserMessageId: `user-${serial}`,
  }); });
  const saved = await database.getMessages(thread);
  expect(saved.filter((message) => message.role === "assistant")).toMatchObject([
    { id: `run-${runId}-assistant-0`, runId, status: "complete", content: [{ type: "text", text: "completed before Stop" }] },
  ]);
  await stop();
  await useChatMessageStore.getState().finishStream(thread, "interrupted");
  expect(await database.getMessages(thread)).toEqual(saved);
  expect(await database.getRuns()).toMatchObject([{ id: runId, status: "finished" }]);
  expect(posts).toHaveBeenCalledTimes(1);
  hook.unmount();
});

test("terminal-only reconciliation preserves every exact-run rich partial segment and foreign settled rows", async () => {
  const thread = `segments-${++serial}`;
  const runId = `segments-native-${serial}`;
  await database.upsertThread({ id: thread, title: "Partial segments", isEphemeral: false,
    createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() });
  const messages = [
    { id: "foreign-settled", role: "assistant" as const, status: "cancelled" as const, runId: "foreign-native", createdAt: new Date(0),
      content: [{ type: "text" as const, text: "other turn" }] },
    { id: `run-${runId}-assistant-0`, role: "assistant" as const, status: "interrupted" as const, runId, createdAt: new Date(1),
      content: [{ type: "text" as const, text: "preamble" }, { type: "artifact" as const, id: "rich-partial", kind: "application/json", content: "{\"retained\":true}" }] },
    { id: `run-${runId}-assistant-1`, role: "assistant" as const, status: "interrupted" as const, runId, createdAt: new Date(2),
      content: [{ type: "text" as const, text: "later partial" }] },
  ];
  for (const message of messages) await database.insertMessage(thread, message);
  await useChatMessageStore.getState().loadMessagesFromDb(thread);
  useChatMessageStore.getState().beginStream(thread, runId);
  // Metadata can accompany a terminal-only response; it must not erase partials.
  useChatMessageStore.getState().setMessageMeta(thread, runId, { usage: { inputTokens: 1, outputTokens: 2, totalTokens: 3 } });
  await useChatMessageStore.getState().finishStream(thread, "complete");
  const saved = await database.getMessages(thread);
  expect(saved.map((message) => message.id)).toEqual(messages.map((message) => message.id));
  for (const original of messages) {
    expect(saved.find((message) => message.id === original.id)).toMatchObject({ runId: original.runId, content: original.content,
      status: original.runId === runId ? "complete" : "cancelled" });
  }
  await useChatMessageStore.getState().finishStream(thread, "interrupted");
  expect(await database.getMessages(thread)).toEqual(saved);
});

test.each(terminals)("Stop during accepted $runStatus persistence return still settles the original bubble", async (terminal) => {
  const thread = `persistence-race-${++serial}`;
  const runId = `persistence-native-${serial}`;
  const turnId = `persistence-intent-${serial}`;
  const userId = `persistence-user-${serial}`;
  const originalId = `run-${runId}-assistant-0`;
  const now = new Date().toISOString();
  const localThread = { id: thread, title: "Accepted terminal race", isEphemeral: false, createdAt: now, updatedAt: now };
  await database.upsertThread(localThread);
  useThreadRegistryStore.setState({ threads: { [thread]: localThread } });
  await database.insertMessage(thread, { id: userId, role: "user", status: "complete", clientTurnId: turnId,
    createdAt: new Date(), content: [{ type: "text", text: "one raced intent" }] });
  let releaseTerminal = () => {};
  const held = new Promise<void>((resolve) => { releaseTerminal = resolve; });
  let terminalAccepted = () => {};
  const accepted = new Promise<void>((resolve) => { terminalAccepted = resolve; });
  const finishRun = database.finishRun.bind(database);
  const finish = vi.spyOn(database, "finishRun").mockImplementation(async (input) => {
    await finishRun(input);
    if (input.nativeTerminal) { terminalAccepted(); await held; }
  });
  const posts = vi.fn();
  const history = vi.fn();
  vi.stubGlobal("fetch", vi.fn(async (input: RequestInfo | URL): Promise<Response> => {
    const url = String(input);
    if (url === "/api/chat/completion") {
      posts();
      return response(frame(runId, 1, { type: "RUN_STARTED", threadId: runId })
        + frame(runId, 2, { type: "TEXT_MESSAGE_CONTENT", messageId: "answer", delta: "same original partial" })
        + frame(runId, 3, { type: terminal.type, ...terminal.fields }), runId);
    }
    if (url.endsWith("/cancel")) throw new TypeError("cancel request lost the native race");
    if (url === `/api/uar/sessions/${thread}`) { history(); return new Response(null, { status: 404 }); }
    throw new Error(`Unexpected HTTP request: ${url}`);
  }));
  const hook = renderHook(() => useChatRuntime(thread));
  await waitFor(() => expect(history).toHaveBeenCalledTimes(1));
  let pending: Promise<void> = Promise.resolve();
  act(() => { pending = useChatStreamStore.getState().startStream(thread, {
    message: "one raced intent", clientTurnId: turnId, persistedUserMessageId: userId,
  }); });
  try {
    await accepted;
    expect(await database.getRuns()).toMatchObject([{ id: runId, status: terminal.runStatus }]);
    await stop();
    expect((await database.getMessages(thread)).find((message) => message.id === originalId)).toMatchObject({ status: "interrupted" });
  } finally {
    releaseTerminal();
    await act(async () => { await pending; });
    finish.mockRestore();
  }
  const saved = await database.getMessages(thread);
  expect(saved.map((message) => message.id)).toEqual([userId, originalId]);
  expect(saved[1]).toMatchObject({ id: originalId, runId, status: terminal.status,
    content: expect.arrayContaining([{ type: "text", text: "same original partial" }]) });
  expect(useChatMessageStore.getState().messagesByThread[thread].map((message) => message.id)).toEqual([userId, originalId]);
  expect(await database.getRuns()).toMatchObject([{ id: runId, status: terminal.runStatus }]);
  expect((await database.getRunEvents(runId)).filter((event) => event.eventId === "3:0")).toHaveLength(1);
  expect(posts).toHaveBeenCalledTimes(1);
  await stop();
  expect(await database.getMessages(thread)).toEqual(saved);
  hook.unmount();
});

// Synthetic faults are limited to the named local storage boundary. Everything
// else uses the shipped hook, HTTP service, stream adapter and real repository.
for (const boundary of ["foreign", "append", "finish"] as const) {
  for (const identity of ["header", "first-event"] as const) {
    test.each(terminals)(`${boundary} rejection with ${identity} identity preserves recovery before $runStatus`, async (terminal) => {
      const thread = `boundary-${++serial}`;
      const runId = `boundary-native-${serial}`;
      const foreignRunId = `foreign-native-${serial}`;
      const turnId = `boundary-intent-${serial}`;
      const userId = `boundary-user-${serial}`;
      const originalId = `run-${runId}-assistant-0`;
      const now = new Date().toISOString();
      const localThread = { id: thread, title: "Rejected evidence", isEphemeral: false, createdAt: now, updatedAt: now };
      await database.upsertThread(localThread);
      useThreadRegistryStore.setState({ threads: { [thread]: localThread } });
      await database.insertMessage(thread, { id: userId, role: "user", clientTurnId: turnId, status: "complete", createdAt: new Date(),
        content: [{ type: "text", text: "recover only this intent" }] });
      const append = database.appendRunEvent.bind(database);
      const finish = database.finishRun.bind(database);
      const appendSpy = vi.spyOn(database, "appendRunEvent").mockImplementation(async (input) => {
        if (boundary === "append" && input.eventId === "3:0") throw new Error("synthetic terminal append unavailable");
        return append(input);
      });
      const finishSpy = vi.spyOn(database, "finishRun").mockImplementation(async (input) => {
        if (boundary === "finish" && input.nativeTerminal?.eventId === "3:0") throw new Error("synthetic native finish unavailable");
        return finish(input);
      });
      const posts = vi.fn();
      const resumes = vi.fn();
      const recoveries = vi.fn();
      const history = vi.fn();
      const onComplete = vi.fn();
      const onError = vi.fn();
      vi.stubGlobal("fetch", vi.fn(async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
        const url = String(input);
        if (url === "/api/chat/completion") {
          posts(JSON.parse(String(init?.body)));
          const body = frame(runId, 1, { type: "RUN_STARTED", threadId: runId })
            + frame(runId, 2, { type: "TEXT_MESSAGE_CONTENT", messageId: "answer", delta: "retained original partial" })
            + frame(boundary === "foreign" ? foreignRunId : runId, 3, {
              type: terminal.type, ...terminal.fields,
              result: { usage: { inputTokens: 70, outputTokens: 80, totalTokens: 150, model: "rejected-model" } },
            });
          return identity === "header" ? response(body, runId)
            : new Response(body, { headers: { "Content-Type": "text/event-stream" } });
        }
        if (url.startsWith("/api/uar/chat-turns/")) {
          recoveries(url);
          return Response.json({ client_turn_id: turnId, session_id: thread, user_message_id: userId,
            run_id: runId, state: "running", retry_allowed: false });
        }
        if (url.includes(`/runs/${runId}/stream`)) {
          resumes(url);
          return response(frame(runId, 4, { type: terminal.type, ...terminal.fields }), runId);
        }
        if (url === `/api/uar/sessions/${thread}`) { history(); return new Response(null, { status: 404 }); }
        if (url.endsWith("/cancel")) return new Response(null, { status: 202 });
        throw new Error(`Unexpected HTTP request: ${url}`);
      }));
      const hook = renderHook(() => useChatRuntime(thread));
      try {
        await waitFor(() => expect(history).toHaveBeenCalledTimes(1));
        vi.mocked(ingestAgUiEvent).mockClear();
        vi.mocked(ingestRuntimeEvent).mockClear();
        const payload = { message: "recover only this intent", clientTurnId: turnId, persistedUserMessageId: userId };
        await act(async () => { await useChatStreamStore.getState().startStream(thread, payload, { onComplete, onError }); });
        expect(onComplete).not.toHaveBeenCalled();
        expect(onError).toHaveBeenCalledTimes(1);
        expect(onError.mock.calls[0][0].name).toBe(boundary === "foreign" ? "RunEventIdentityError" : "RunEventStorageError");
        expect(readChatTurn(turnId)).toMatchObject({ id: turnId, run_id: runId, user_message_id: userId, state: "uncertain" });
        expect(pendingChatTurns(thread).map((receipt) => receipt.id)).toEqual([turnId]);
        const partial = await database.getMessages(thread);
        expect(partial.map((message) => message.id)).toEqual([userId, originalId]);
        expect(partial[1]).toMatchObject({ id: originalId, runId, status: "interrupted", content: [{ type: "text", text: "retained original partial" }] });
        expect(partial[1]?.usage).toBeUndefined();
        expect(partial[1]?.chunks?.some((chunk) => chunk.kind === "error" || chunk.kind === "usage")).toBe(false);
        expect(useChatMessageStore.getState().messagesByThread[thread].map((message) => message.id)).toEqual([userId, originalId]);
        expect(useChatMessageStore.getState().messagesByThread[thread][1]).toMatchObject({ status: "interrupted", runId,
          content: [{ type: "text", text: "retained original partial" }] });
        expect(await database.getRuns()).toMatchObject([{ id: runId, status: "interrupted", usage: null, phaseTimings: {} }]);
        expect(await database.getRunEvents(foreignRunId)).toEqual([]);
        const events = await database.getRunEvents(runId);
        expect(events.some((event) => event.eventId === "3:0")).toBe(boundary === "finish");
        expect(vi.mocked(ingestAgUiEvent).mock.calls.map(([, event]) => event.id)).toEqual(["1:0", "2:0"]);
        expect(vi.mocked(ingestRuntimeEvent).mock.calls).toEqual([]);
        expect(posts).toHaveBeenCalledTimes(1);
        expect(resumes).not.toHaveBeenCalled();
        expect(recoveries).not.toHaveBeenCalled();
        appendSpy.mockRestore();
        finishSpy.mockRestore();
        await act(async () => { await useChatStreamStore.getState().startStream(thread, payload, { onComplete, onError }); });
        expect(await database.getRuns()).toMatchObject([{ id: runId, status: terminal.runStatus }]);
        const recovered = await database.getMessages(thread);
        expect(recovered.map((message) => message.id)).toEqual([userId, originalId]);
        expect(recovered[1]).toMatchObject({ id: originalId, runId, status: terminal.status,
          content: expect.arrayContaining([{ type: "text", text: "retained original partial" }]) });
        expect(recovered[1]?.usage).not.toEqual({ inputTokens: 70, outputTokens: 80, totalTokens: 150 });
        expect(readChatTurn(turnId)).toMatchObject({ id: turnId, run_id: runId, user_message_id: userId, state: "settled" });
        expect(pendingChatTurns(thread)).toEqual([]);
        expect((await database.getRunEvents(runId)).filter((event) => event.eventId === "4:0")).toHaveLength(1);
        expect(posts).toHaveBeenCalledTimes(1);
        expect(recoveries).toHaveBeenCalledTimes(1);
        expect(resumes).toHaveBeenCalledTimes(1);
        expect(onComplete).toHaveBeenCalledTimes(terminal.type === "RUN_FINISHED" ? 1 : 0);
        await stop();
        expect(await database.getMessages(thread)).toEqual(recovered);
        expect(await database.getRuns()).toMatchObject([{ id: runId, status: terminal.runStatus }]);
      } finally {
        appendSpy.mockRestore();
        finishSpy.mockRestore();
        hook.unmount();
      }
    });
  }
}

// Native adapter → actual stream/message stores → production UarDb/PGlite.
// Runtime rendering is asserted through the shipped projection of the saved row;
// graph ingestion remains the existing substituted, unrelated seam above.
for (const terminal of ["RUN_FINISHED", "RUN_ERROR"] as const) {
  test.each([
    { label: "output-only", usage: { inputTokens: null, outputTokens: 5, totalTokens: null, model: "reported-model", costUsdEstimate: 0.02 }, counts: undefined },
    { label: "input-only", usage: { inputTokens: 2, outputTokens: null, totalTokens: null }, counts: undefined },
    { label: "unknown-total", usage: { inputTokens: 2, outputTokens: 5, totalTokens: null }, counts: undefined },
    { label: "all-null", usage: { inputTokens: null, outputTokens: null, totalTokens: null }, counts: undefined },
    { label: "absent", usage: undefined, counts: undefined },
    { label: "independent-cache", usage: { inputTokens: null, outputTokens: 5, totalTokens: null, uncachedInputTokens: 2,
      cacheReadInputTokens: 3, cacheCreationInputTokens: 1, model: "reported-model", costUsdEstimate: 0.02 }, counts: undefined },
    { label: "known-zero", usage: { inputTokens: 0, outputTokens: 0, totalTokens: 0 }, counts: { inputTokens: 0, outputTokens: 0, totalTokens: 0 } },
    { label: "complete-without-cache", usage: { inputTokens: 2, outputTokens: 5, totalTokens: 7, model: "reported-model", costUsdEstimate: 0.02 },
      counts: { inputTokens: 2, outputTokens: 5, totalTokens: 7 } },
  ])(`${terminal} durably preserves $label usage without inventing aggregate counts`, async ({ usage, counts }) => {
    const thread = `usage-native-${++serial}`;
    const runId = `usage-run-${serial}`;
    const turnId = `usage-intent-${serial}`;
    const userId = `usage-user-${serial}`;
    const now = new Date().toISOString();
    const localThread = { id: thread, title: "Native usage availability", isEphemeral: false, createdAt: now, updatedAt: now };
    await database.upsertThread(localThread);
    useThreadRegistryStore.setState({ threads: { [thread]: localThread } });
    await database.insertMessage(thread, { id: userId, role: "user", clientTurnId: turnId, status: "complete", createdAt: new Date(),
      content: [{ type: "text", text: "usage evidence" }] });
    const posts = vi.fn();
    const history = vi.fn();
    const fields = { ...(terminal === "RUN_ERROR" ? { code: "CANCELLED", message: "Run cancelled" } : {}),
      ...(usage ? { result: { usage } } : {}) };
    vi.stubGlobal("fetch", vi.fn(async (input: RequestInfo | URL): Promise<Response> => {
      const url = String(input);
      if (url === "/api/chat/completion") {
        posts();
        return response(frame(runId, 1, { type: "RUN_STARTED", threadId: runId })
          + frame(runId, 2, { type: "TEXT_MESSAGE_CONTENT", messageId: "answer", delta: "native usage answer" })
          + frame(runId, 3, { type: terminal, ...fields }), runId);
      }
      if (url.endsWith("/cancel")) return new Response(null, { status: 202 });
      if (url === `/api/uar/sessions/${thread}`) { history(); return new Response(null, { status: 404 }); }
      throw new Error(`Unexpected HTTP request: ${url}`);
    }));
    const hook = renderHook(() => useChatRuntime(thread));
    await waitFor(() => expect(history).toHaveBeenCalledTimes(1));
    await act(async () => { await useChatStreamStore.getState().startStream(thread, {
      message: "usage evidence", clientTurnId: turnId, persistedUserMessageId: userId,
    }); });
    const saved = await database.getMessages(thread);
    const assistant = saved.find((message) => message.id === `run-${runId}-assistant-0`);
    expect(assistant).toMatchObject({ runId, status: terminal === "RUN_FINISHED" ? "complete" : "cancelled",
      content: [{ type: "text", text: "native usage answer" }] });
    const liveAssistant = useChatMessageStore.getState().messagesByThread[thread].find((message) => message.id === assistant?.id);
    expect(liveAssistant?.usage).toEqual(counts);
    if (counts) expect(assistant?.chunks?.filter((chunk) => chunk.kind === "usage")).toMatchObject([{ ...counts }]);
    else expect(assistant?.chunks?.some((chunk) => chunk.kind === "usage")).toBe(false);
    if (usage && "model" in usage) expect(liveAssistant?.model).toBe(usage.model);
    const run = (await database.getRuns()).find((row) => row.id === runId);
    expect(run).toMatchObject({ status: terminal === "RUN_FINISHED" ? "finished" : "cancelled", usage: usage ?? null });
    expect(run?.costUsd).toBe(usage && "costUsdEstimate" in usage ? usage.costUsdEstimate : null);
    const events = await database.getRunEvents(runId);
    const savedTerminal = events.find((event) => event.eventId === "3:0");
    expect(savedTerminal?.payload).toEqual({ type: terminal, ...fields, runId,
      profile: "uar.agui/1", eventId: "3:0", sequence: 48 });
    expect(events.filter((event) => event.eventId === "3:0")).toHaveLength(1);
    const projection = savedTerminal && isUarAguiEvent(savedTerminal.payload)
      ? toRuntimeChunk(savedTerminal.payload, Date.now()) : undefined;
    if (counts) expect(projection).toMatchObject({ kind: "usage", ...counts });
    else {
      expect(projection).toMatchObject({ kind: "raw", payload: savedTerminal?.payload });
      expect(projection).not.toHaveProperty("totalTokens");
    }
    expect(readChatTurn(turnId)).toMatchObject({ run_id: runId, state: "settled" });
    // A later local Stop still cannot alter the terminal or its usage evidence.
    await stop();
    await useChatMessageStore.getState().finishStream(thread, "interrupted");
    expect(await database.getMessages(thread)).toEqual(saved);
    expect((await database.getRuns()).find((row) => row.id === runId)).toEqual(run);
    expect(posts).toHaveBeenCalledTimes(1);
    hook.unmount();
  });
}


// R037: local message/run persistence is real. Only HTTP and the already
// established graph-persistence binding are controlled at their host seams.
describe("completed action receipt recovery over settled chat history", () => {
  for (const metadata of [true, false]) {
    for (const observed of ["full", "abort", "storage", "eof"] as const) {
      test.each(terminals)(`${metadata ? "agent metadata" : "no agent metadata"}/${observed} preserves settled $status evidence`, async (terminal) => {
        const thread = `settled-action-${++serial}`;
        const runId = `settled-child-${serial}`;
        const source = `settled-source-${serial}`;
        const interaction = `settled-click-${serial}`;
        const originalId = `run-${runId}-assistant-0`;
        const now = new Date().toISOString();
        await database.upsertThread({ id: thread, title: "Settled receipt recovery", isEphemeral: false, createdAt: now, updatedAt: now });
        await database.insertMessage(thread, { id: `user-${serial}`, role: "user", createdAt: new Date(), status: "complete",
          content: [{ type: "text", text: "original action context" }] });
        const usage = { inputTokens: 2, outputTokens: 3, totalTokens: 5, model: "original-model", costUsdEstimate: 0.03 };
        const artifact = { artifactId: `rich-${serial}`, artifactType: "application/json", title: "Original rich result",
          content: "{\"retained\":true}", language: "json", sourceRunId: runId, metadata: { source_run_id: runId } };
        const prefix = frame(runId, 1, { type: "RUN_STARTED", threadId: runId, ...(metadata ? { input: { agentId: "original-agent" } } : {}) })
          + frame(runId, 2, { type: "TEXT_MESSAGE_CONTENT", messageId: "answer", delta: "original answer" })
          + frame(runId, 3, { type: "TEXT_MESSAGE_END", messageId: "answer" })
          + frame(runId, 4, { type: "TOOL_CALL_END", toolCallId: "original-tool", toolCallName: "native_echo", arguments: { text: "once" } })
          + frame(runId, 5, { type: "CUSTOM", name: "uar.artifact.available", value: artifact })
          + frame(runId, 6, { type: "TOOL_CALL_RESULT", toolCallId: "original-tool", content: "original result" })
          + frame(runId, 7, { type: "TEXT_MESSAGE_CONTENT", messageId: "answer", delta: " after rich result" })
          + frame(runId, 8, { type: "TEXT_MESSAGE_END", messageId: "answer" });
        const ending = frame(runId, 9, { type: terminal.type, ...terminal.fields, result: { usage } });
        let replay = false;
        let interrupt: (() => void) | undefined;
        const requests = vi.fn<typeof fetch>(async (request, init) => {
          const url = String(request);
          if (url === `/api/uar/runs/${source}/a2ui/actions/${interaction}`) return Response.json({ run_id: source,
            interaction_id: interaction, retry_protected: true, continuation_run_id: runId, outcome: "completed" });
          if (url === `/api/uar/runs/${source}`) return Response.json({ run_id: source, conversation_id: thread, status: "done" });
          if (url === `/api/uar/runs/${runId}/stream?last_event_id=0&stream_mode=agui_spec`) {
            expect(init?.method).toBe("GET");
            expect(init?.headers).toMatchObject({ "Last-Event-ID": "0" });
            if (replay && observed === "abort") return response(new ReadableStream<Uint8Array>({ start(controller) {
              controller.enqueue(encoder.encode(prefix));
              interrupt = () => controller.error(Object.assign(new Error("recovery observer lost"), { name: "AbortError" }));
            } }), runId);
            return response(prefix + (replay && observed === "eof" ? "" : ending), runId);
          }
          throw new Error(`Unexpected request: ${init?.method ?? "GET"} ${url}`);
        });
        vi.stubGlobal("fetch", requests);
        const lease = acquireChatStream();
        try {
          const initial = useChatStreamStore.getState().attachContinuation(thread, runId, lease);
          if (terminal.status === "complete") await initial;
          else await expect(initial).rejects.toThrow();
        } finally { lease.release(); }
        const saved = await database.getMessages(thread);
        expect(saved.filter((row) => row.role === "assistant")).toMatchObject([{ id: originalId, runId, status: terminal.status }]);
        const original = saved.find((row) => row.id === originalId)!;
        expect(original.content.filter((block) => block.type === "text").map((block) => block.text))
          .toEqual(["original answer", " after rich result"]);
        expect(original.chunks?.filter((chunk) => chunk.kind === "artifact")).toMatchObject([{ artifactId: artifact.artifactId }]);
        expect(original.chunks?.filter((chunk) => chunk.kind === "tool-call")).toMatchObject([{ toolCallId: "original-tool", result: "original result" }]);
        const runs = await database.getRuns();
        expect(runs).toMatchObject([{ id: runId, status: terminal.runStatus, usage }]);
        const events = await database.getRunEvents(runId);
        expect(events).toHaveLength(9);
        registerA2uiActionEntities();
        // Chat and native terminal were saved, but the final action receipt
        // write was lost. Recovery starts from the durable responding receipt.
        graphStore.getState().replaceEntity(A2UI_ACTION_ENTITY, a2uiActionId(source, interaction), {
          id: a2uiActionId(source, interaction), source_run_id: source, surface_id: "surface", interaction_id: interaction,
          continuation_run_id: runId, status: "responding",
        });
        const receiptWrites: string[] = [];
        let durableReceipt = structuredClone(latestActionReceipt(source, "surface")!);
        let loseFinalReceipt = observed === "full" && terminal.status === "complete";
        bindA2uiActionPersistence(async () => {
          const receipt = latestActionReceipt(source, "surface")!;
          receiptWrites.push(receipt.status);
          if (loseFinalReceipt && receipt.status === "completed") {
            loseFinalReceipt = false;
            throw new Error("final local action receipt write was lost");
          }
          durableReceipt = structuredClone(receipt);
        });
        const append = database.appendRunEvent.bind(database);
        const appendSpy = vi.spyOn(database, "appendRunEvent").mockImplementation(async (input) => {
          if (observed === "storage" && input.eventId === "6:0") throw new Error("local replay storage unavailable");
          return append(input);
        });
        const writeSpy = vi.spyOn(database, "insertMessage");
        replay = true;
        try {
          const recovery = reconcileA2uiAction(source, "surface");
          if (observed === "abort") {
            await waitFor(() => expect(appendSpy).toHaveBeenCalledWith(expect.objectContaining({ eventId: "8:0" })));
            expect(interrupt).toBeDefined();
            interrupt!();
          }
          await recovery;
          expect(latestActionReceipt(source, "surface")?.status)
            .toBe(observed === "full" && terminal.status === "complete" ? "completed" : "interrupted");
          expect(receiptWrites).toEqual(["responding", observed === "full" && terminal.status === "complete" ? "completed" : "interrupted"]);
          expect(useChatMessageStore.getState().messagesByThread[thread]).toEqual(saved);
          expect(await database.getMessages(thread)).toEqual(saved);
          expect(await database.getRuns()).toEqual(runs);
          expect(await database.getRunEvents(runId)).toEqual(events);
          expect(writeSpy).not.toHaveBeenCalled();
          expect(requests.mock.calls.filter(([, init]) => init?.method === "POST")).toHaveLength(0);
          expect(requests.mock.calls.filter(([url]) => String(url).includes("/stream?"))).toHaveLength(2);
          if (observed === "full" && terminal.status === "complete") {
            await reconcileA2uiAction(source, "surface");
            expect(requests.mock.calls.filter(([url]) => String(url).includes("/stream?"))).toHaveLength(2);
            expect(await database.getMessages(thread)).toEqual(saved);
            // A failed final receipt write leaves durable responding state.
            // Rehydrate that exact receipt and observe the same completed run again.
            expect(durableReceipt.status).toBe("responding");
            graphStore.getState().replaceEntity(A2UI_ACTION_ENTITY, durableReceipt.id, durableReceipt);
            await reconcileA2uiAction(source, "surface");
            expect(durableReceipt.status).toBe("completed");
            expect(receiptWrites).toEqual(["responding", "completed", "responding", "completed"]);
            expect(requests.mock.calls.filter(([url]) => String(url).includes("/stream?"))).toHaveLength(3);
            expect(requests.mock.calls.filter(([, init]) => init?.method === "POST")).toHaveLength(0);
            expect(useChatMessageStore.getState().messagesByThread[thread]).toEqual(saved);
            expect(await database.getMessages(thread)).toEqual(saved);
            expect(await database.getRuns()).toEqual(runs);
            expect(await database.getRunEvents(runId)).toEqual(events);
            expect(writeSpy).not.toHaveBeenCalled();
          }
        } finally { appendSpy.mockRestore(); writeSpy.mockRestore(); }
      });
    }
  }
});
