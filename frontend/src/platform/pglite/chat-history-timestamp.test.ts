import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { PGlite } from "@electric-sql/pglite";
import { live, type PGliteWithLive } from "@electric-sql/pglite/live";
import { setDbInstance, UarDb } from "./client";
import { RunEventPersistence } from "./run-event-persistence";
import { useChatMessageStore } from "@/stores/chat-message-store";
import { toRuntimeChunk } from "@/platform/agui/runtime-chunk-projection";
import type { UarAguiEvent } from "@/platform/agui/agui-schema";
import { MIGRATIONS } from "./migrations";
import { missingServerMessages } from "@/features/chat/model/session-backfill";
import { toChunks } from "@/features/chat/model/to-chunks";
import type { RichMessage } from "@/types/chat-content";
import { createElement } from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { UarAguiAdapter } from "@/platform/agui/agui-adapter";
import { ChunkRenderer } from "@/features/chat/ui/chunks/chunk-renderer";
import { A2UI_CATALOG_ID, A2UI_PROFILE, A2UI_VERSION } from "@/features/a2ui/a2ui-protocol";

const known = "2026-11-02T00:00:00.000Z";
const epoch = new Date(0);

describe("canonical history timestamp persistence", () => {
  let engine: PGlite & PGliteWithLive;
  let database: UarDb;

  // The shipped constructor is private because open() owns browser assets and
  // IndexedDB. This fixture constructs only the real production adapter around
  // the real migrated engine; no production test seam or query mock is added.
  const adapter = (db: PGlite & PGliteWithLive): UarDb => Reflect.construct(UarDb, [db]) as UarDb;

  beforeEach(async () => {
    engine = await PGlite.create({ extensions: { live } });
    for (const migration of MIGRATIONS) await engine.exec(migration.up);
    database = adapter(engine);
    setDbInstance(database);
    useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
    await database.upsertThread({ id: "thread", title: "Timestamp history", isEphemeral: false,
      createdAt: known, updatedAt: known });
  });

  afterEach(async () => { vi.unstubAllGlobals(); await engine.close(); });

  for (const variant of ["caption", "image-only", "reordered", "mixed", "identical-legacy", "modern-text", "client-turn"] as const) {
    test(`mismatched user identity never overwrites durable original history (${variant})`, async () => {
      const first = { type: "image" as const, url: "data:image/png;base64,eA==", mime: "image/png", dataBase64: null };
      const second = { ...first, url: "data:image/png;base64,eQ==" };
      const caption = { type: "text" as const, text: "inspect" };
      const user: RichMessage = { id: "original-user", role: "user", status: "complete", createdAt: new Date(known),
        content: variant === "modern-text" || variant === "client-turn" ? [caption]
          : variant === "image-only" ? [first] : variant === "reordered" ? [caption, first, second] : [caption, first],
        ...(variant === "client-turn" ? { clientTurnId: "original-turn" } : {}) };
      user.chunks = toChunks(user.content, { messageId: user.id, at: known, finalized: true });
      const answer: RichMessage = { id: "original-answer", role: "assistant", status: "complete", runId: "original-run",
        createdAt: new Date(known), content: [{ type: "text", text: "Original answer." },
          { type: "artifact", id: "original-artifact", kind: "text/plain", content: "original evidence" }, first] };
      answer.chunks = toChunks(answer.content, { messageId: answer.id, at: known, runId: answer.runId, finalized: true });
      answer.chunks.push({ kind: "tool-approval", id: "original-approval", at: known, seq: 5, runId: answer.runId,
        approvalId: "approved", toolCallId: "call", toolName: "inspect", args: { retained: true }, historical: true,
        historicalState: "approved", decision: "approved", decidedAt: known });
      await database.insertMessage("thread", user);
      await database.insertMessage("thread", answer);
      const original = await adapter(engine).getMessages("thread");
      const remoteContent = variant === "modern-text" || variant === "client-turn" ? [caption]
        : variant === "image-only" ? [second] : variant === "reordered" ? [caption, second, first]
          : variant === "mixed" ? [first, caption] : variant === "identical-legacy" ? [caption, first] : [caption, second];
      const request = vi.fn<typeof fetch>().mockImplementation(async () => new Response(JSON.stringify({ history_version: 1, history: [
        { message_id: variant === "modern-text" ? "different-user" : "legacy:thread:0:user", role: "user", content: remoteContent,
          status: "complete", created_at: known, ...(variant === "client-turn" ? { client_turn_id: "different-turn" } : {}) },
        { message_id: "different-answer", role: "assistant", content: "Must not replace original answer.", status: "complete", created_at: known },
      ] }), { status: 200 }));
      vi.stubGlobal("fetch", request);
      const writes = vi.spyOn(database, "insertMessage");
      const store = useChatMessageStore.getState();
      await store.loadMessagesFromDb("thread");
      await store.backfillFromServer("thread");
      expect(useChatMessageStore.getState().messagesByThread.thread).toEqual(original);
      expect(writes).not.toHaveBeenCalled();
      expect(await adapter(engine).getMessages("thread")).toEqual(original);
      useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
      const reloadedAdapter = adapter(engine);
      const reloadWrites = vi.spyOn(reloadedAdapter, "insertMessage");
      setDbInstance(reloadedAdapter);
      await store.loadMessagesFromDb("thread");
      await store.backfillFromServer("thread");
      const reloaded = useChatMessageStore.getState().messagesByThread.thread;
      expect(reloaded).toEqual(original);
      expect(reloaded.map(row => row.id)).toEqual([user.id, answer.id]);
      expect(reloaded[0].content).toEqual(user.content);
      expect(reloaded[0].chunks).toEqual(user.chunks);
      expect(reloaded[1].chunks).toEqual(answer.chunks);
      expect(await adapter(engine).getMessages("thread")).toEqual(original);
      expect(reloadWrites).not.toHaveBeenCalled();
      expect(request).toHaveBeenCalledTimes(2);
      for (const [url, options] of request.mock.calls) {
        expect(url).toBe("/api/uar/sessions/thread");
        expect(options).toEqual({ cache: "no-store" });
      }
      writes.mockRestore();
      reloadWrites.mockRestore();
    });
  }

  for (const variant of ["canonical", "missing-generation", "missing-metadata", "missing-profile", "unsupported-profile", "generic"] as const) {
    test(`new native display survives cached interruption, portable reload and shipped policy (${variant})`, async () => {
      const unavailable = vi.fn<typeof fetch>().mockRejectedValue(new Error("server unavailable"));
      vi.stubGlobal("fetch", unavailable);
      const runId = `new-publication-${variant}`;
      const artifactId = `host-artifact-${variant}`;
      const generation = "host-publication-G7";
      const content = JSON.stringify([
        { version: A2UI_VERSION, createSurface: { surfaceId: "parcel", catalogId: A2UI_CATALOG_ID } },
        { version: A2UI_VERSION, updateComponents: { surfaceId: "parcel", components: [
          { id: "heading", component: "Text", text: "Recovered parcel", variant: "h2" },
          { id: "label", component: "Text", text: "Calculate parcel" },
          { id: "submit", component: "Button", child: "label", action: { event: { name: "calculate" } } },
          { id: "root", component: "Column", children: ["heading", "submit"] },
        ] } },
      ]);
      const metadata = { source_run_id: runId, protocol_version: A2UI_VERSION,
        host_annotation: { original: true },
        ...(variant === "missing-profile" || variant === "generic" ? {} : {
          profile: variant === "unsupported-profile" ? "uar.a2ui/unsupported" : A2UI_PROFILE,
        }),
        ...(variant === "missing-generation" ? {} : { surface_generations: { parcel: generation } }),
      };
      // Exact native adapters.rs vocabulary: profile is nested, not top-level.
      const value = { artifactId, artifactType: variant === "generic" ? "application/json" : "a2ui/surface",
        title: "Host publication", content, language: "json", sourceRunId: runId,
        metadata: variant === "missing-metadata" ? null : metadata };
      const writer = new RunEventPersistence(database, { threadId: "thread", fallbackRunId: runId, nativeRunId: runId });
      const nativeAdapter = new UarAguiAdapter();
      let sequence = 0;
      const persist = async (type: string, fields: Record<string, unknown> = {}) => {
        const event = { profile: "uar.agui/1", type, runId, eventId: `${runId}:${sequence}:8`,
          sequence: sequence * 16 + 8, ...fields };
        sequence += 1;
        const adapted = nativeAdapter.ingest(type, JSON.stringify(event), sequence);
        expect(adapted?.eventRow).toBeDefined();
        await writer.ingest(adapted!.eventRow!);
        return adapted!;
      };
      const store = useChatMessageStore.getState();
      await persist("RUN_STARTED");
      await persist("TEXT_MESSAGE_CONTENT", { messageId: `${runId}:assistant`, delta: "Saved partial." });
      store.beginStream("thread", runId);
      store.appendTextDelta("thread", runId, "Saved partial.");
      await store.finishStream("thread", "interrupted");
      const cached = (await database.getMessages("thread"))[0];
      expect(cached.chunks!.map(chunk => chunk.kind)).toEqual(["markdown"]);
      // The host display and terminal are journaled after the message snapshot.
      // No live display reduction or message finalization fills this gap.
      const publication = await persist("CUSTOM", { name: "uar.artifact.available", value });
      await persist("RUN_FINISHED");
      expect(publication.eventRow!.payload.value).toEqual(value);
      const originalJournal = await database.getRunEvents(runId);
      useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
      setDbInstance(adapter(engine));
      await store.loadMessagesFromDb("thread");
      const restored = useChatMessageStore.getState().messagesByThread.thread;
      expect(restored).toHaveLength(1);
      expect(restored[0]).toMatchObject({ id: cached.id, status: "complete", runId });
      expect(restored[0].chunks![0]).toEqual(cached.chunks![0]);
      const display = restored[0].chunks!.find(chunk => variant === "generic"
        ? chunk.kind === "artifact" : chunk.kind === "a2ui-display")!;
      expect(display).toBeDefined();
      expect(restored[0].chunks!.filter(chunk => chunk.id === display.id)).toHaveLength(1);
      if (variant === "generic") {
        expect(display).toMatchObject({ kind: "artifact", artifactId, mime: "application/json", content });
      } else {
        expect(display).toMatchObject({ kind: "a2ui-display", toolCallId: artifactId,
          profile: variant === "missing-metadata" ? "" : "profile" in metadata ? metadata.profile : "",
          ...(variant === "missing-metadata" ? {} : { version: A2UI_VERSION }),
          component: "a2ui/surface", payload: value });
        expect(display.kind === "a2ui-display" && display.payload).toEqual(value);
        const portable = restored[0].content.find(block => block.type === "artifact" && block.id === artifactId)!;
        expect(portable).toMatchObject({ type: "artifact", id: artifactId, kind: "application/vnd.uar.a2ui+json" });
        if (portable.type !== "artifact") throw new Error("Missing portable native display");
        expect(JSON.parse(portable.content)).toMatchObject({ payload: value,
          ...(variant === "missing-metadata" ? {} : { version: A2UI_VERSION }) });
        const rebuilt = toChunks([portable], { messageId: cached.id, at: known, runId })[0];
        expect(rebuilt).toMatchObject({ kind: "a2ui-display", toolCallId: artifactId,
          component: "a2ui/surface", ...(variant === "missing-metadata" ? {} : { version: A2UI_VERSION }), payload: value });
        if (variant === "missing-metadata") {
          expect(display.kind === "a2ui-display" && display.version).toBeUndefined();
          expect(rebuilt.kind === "a2ui-display" && rebuilt.version).toBeUndefined();
        }
        if (variant === "missing-generation") {
          expect(display.kind === "a2ui-display" && display.payload).not.toHaveProperty("metadata.surface_generations");
          expect(rebuilt.kind === "a2ui-display" && rebuilt.payload).not.toHaveProperty("metadata.surface_generations");
        }
      }
      const persisted = await adapter(engine).getMessages("thread");
      expect(persisted[0].content).toEqual(restored[0].content);
      expect(persisted[0].chunks).toEqual(restored[0].chunks);
      useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
      setDbInstance(adapter(engine));
      await store.loadMessagesFromDb("thread");
      expect(useChatMessageStore.getState().messagesByThread.thread).toEqual(persisted);
      expect(await database.getRunEvents(runId)).toEqual(originalJournal);
      expect(unavailable).not.toHaveBeenCalled();

      // Mount the shipped dispatcher and policy component. Only its ordinary
      // authority read may occur; unavailable history cannot enable a POST.
      const reloaded = useChatMessageStore.getState().messagesByThread.thread[0].chunks!
        .find(chunk => chunk.id === display.id)!;
      render(createElement(ChunkRenderer, { chunk: reloaded }));
      if (variant === "generic") {
        expect(screen.queryByRole("region", { name: "A2UI display artifact" })).toBeNull();
        expect(screen.getByLabelText("JSON")).toHaveTextContent("Recovered parcel");
        expect(unavailable).not.toHaveBeenCalled();
      } else if (variant === "missing-metadata" || variant === "missing-profile" || variant === "unsupported-profile") {
        expect(screen.getByRole("alert")).toHaveTextContent("could not be rendered");
        expect(screen.getByText(variant !== "unsupported-profile"
          ? "The artifact is missing its required UAR A2UI profile."
          : "Unsupported A2UI profile: uar.a2ui/unsupported")).toBeVisible();
        expect(screen.queryByRole("button", { name: "Calculate parcel", exact: true })).toBeNull();
        expect(unavailable).not.toHaveBeenCalled();
      } else {
        expect(screen.getByRole("heading", { name: "Recovered parcel" })).toBeVisible();
        const button = screen.getByRole("button", { name: "Calculate parcel", exact: true });
        expect(button).toBeDisabled();
        await screen.findByText(variant === "missing-generation"
          ? "This surface has no publication identity. Request a fresh surface before acting."
          : "Surface history is unavailable. Reload the conversation to recover.", { exact: true });
        fireEvent.click(button);
        await waitFor(() => expect(unavailable).toHaveBeenCalledTimes(1));
        expect(String(unavailable.mock.calls[0][0])).toContain(`/runs/${runId}/a2ui/surface-authority`);
        expect(unavailable.mock.calls.every(call => call[1]?.method !== "POST")).toBe(true);
      }
    });
  }

  for (const outcome of ["success", "error", "cancelled", "partial"] as const) {
    test(`local journal reconciles cached rich interruption with server unavailable (${outcome})`, async () => {
      const unavailable = vi.fn().mockRejectedValue(new Error("server unavailable"));
      vi.stubGlobal("fetch", unavailable);
      const runId = `local-${outcome}`;
      const writer = new RunEventPersistence(database, { threadId: "thread", fallbackRunId: runId, nativeRunId: runId });
      const nativeAdapter = new UarAguiAdapter();
      let sequence = 0;
      const persist = async (type: string, fields: Record<string, unknown> = {}): Promise<UarAguiEvent> => {
        const event: UarAguiEvent = { profile: "uar.agui/1", type, eventId: `${sequence}:0`, sequence: sequence * 16, runId, threadId: runId, ...fields };
        sequence += 1;
        const adapted = nativeAdapter.ingest(type, JSON.stringify(event), sequence);
        expect(adapted?.eventRow).toBeDefined();
        await writer.ingest(adapted!.eventRow!);
        return event;
      };
      const store = useChatMessageStore.getState();
      store.beginStream("thread", runId);
      await persist("RUN_STARTED", { input: { agentId: "native-agent" } });
      await persist("TEXT_MESSAGE_CONTENT", { messageId: `${runId}:assistant`, delta: "Checking evidence." });
      store.appendTextDelta("thread", runId, "Checking evidence.");
      await persist("TOOL_CALL_END", { toolCallId: "lookup", toolCallName: "lookup", arguments: { query: "fact" } });
      store.addToolCall("thread", runId, { type: "tool-call", toolCallId: "lookup", toolName: "lookup", args: { query: "fact" }, status: "running" });
      const approval = await persist("CUSTOM", { name: "uar.tool.approval_required", value: { approvalId: "approval", toolCallId: "lookup", name: "lookup", arguments: "{\"query\":\"fact\"}", riskReason: "operator decision" } });
      const approvalChunk = toRuntimeChunk(approval, new Date(known).getTime())!;
      if (approvalChunk.kind === "tool-approval") {
        approvalChunk.decision = "approved";
        approvalChunk.decidedAt = known;
      }
      store.addRuntimeChunk("thread", runId, approvalChunk);
      await persist("TOOL_CALL_RESULT", { toolCallId: "lookup", toolCallName: "lookup", content: "fact" });
      store.updateToolCall("thread", "lookup", { status: "complete", result: "fact" });
      await persist("CUSTOM", { name: "uar.artifact.available", value: { artifactId: "evidence", artifactType: "text", content: "retained artifact", sourceRunId: runId } });
      store.addToolCall("thread", runId, { type: "tool-call", toolCallId: "evidence", toolName: "__a2ui_display__", args: { artifactType: "text" }, result: "retained artifact", status: "complete" });
      const publicationId = `a2ui-surface-${runId}-surface-1`;
      store.addToolCall("thread", runId, { type: "tool-call", toolCallId: publicationId, toolName: "__a2ui_display__",
        args: { artifactType: "a2ui/surface", language: "a2ui", metadata: {
          profile: "uar.a2ui/1", protocol_version: "v0.9.1", source_run_id: runId,
        } }, result: "retained publication generation", status: "complete" });
      await persist("TEXT_MESSAGE_CONTENT", { messageId: `${runId}:assistant`, delta: "Final" });
      store.appendTextDelta("thread", runId, "Final");
      await store.finishStream("thread", "interrupted");
      const cached = (await database.getMessages("thread"))[0];
      const cachedIds = cached.chunks!.map(chunk => chunk.id);
      // Later journal evidence is durable separately; do not call chat stream
      // finalization after it, reproducing the writer-before-message window.
      // adapters.rs ContextAction + enrich_agui_spec_payload preserve these
      // null identities. Only the writer's attached outer run owns this row.
      const context = await persist("CUSTOM", { name: "uar.context.updated", runId: null, threadId: null,
        value: { strategy: "sliding_window", messagesRemoved: 2, tokensSaved: 128, wasApplied: true, summaryGenerated: false } });
      const contextRow = (await database.getRunEvents(runId)).find(row => row.eventId === context.eventId)!;
      expect(contextRow).toMatchObject({ runId, type: "CUSTOM", wireSequence: context.sequence, payload: context });
      expect(contextRow.payload.runId).toBeNull();
      expect(contextRow.payload.threadId).toBeNull();
      await persist("TEXT_MESSAGE_CONTENT", { messageId: `${runId}:assistant`, delta: " answer." });
      if (outcome !== "partial") await persist(outcome === "success" ? "RUN_FINISHED" : "RUN_ERROR", {
        ...(outcome === "cancelled" ? { code: "CANCELLED", message: "Run cancelled" } : outcome === "error" ? { code: "PROVIDER_ERROR", message: "native failure" } : {}),
        ...(outcome === "error" ? {} : { result: { usage: { inputTokens: null, outputTokens: 5, totalTokens: null, costUsdEstimate: null, model: "native-model" } } }),
      });
      useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
      setDbInstance(adapter(engine));
      await store.loadMessagesFromDb("thread");
      const restored = useChatMessageStore.getState().messagesByThread.thread;
      expect(restored).toHaveLength(1);
      expect(restored[0].id).toBe(cached.id);
      expect(restored[0].status).toBe(outcome === "success" ? "complete" : outcome === "error" ? "failed" : outcome === "cancelled" ? "cancelled" : "interrupted");
      expect(restored[0].content.filter(block => block.type === "text")).toEqual([
        { type: "text", text: "Checking evidence." }, { type: "text", text: "Final answer." },
      ]);
      expect(restored[0].content.filter(block => block.type !== "text" && !(block.type === "artifact" && block.kind === "application/vnd.uar.error+json")))
        .toEqual(cached.content.filter(block => block.type !== "text"));
      expect(restored[0].chunks!.slice(0, cachedIds.length).map(chunk => chunk.id)).toEqual(cachedIds);
      expect(restored[0].chunks!.filter(chunk => chunk.kind === "tool-call")).toHaveLength(1);
      expect(restored[0].chunks!.filter(chunk => chunk.kind === "artifact")).toHaveLength(1);
      expect(restored[0].chunks!.filter(chunk => chunk.kind === "a2ui-display")).toEqual(cached.chunks!.filter(chunk => chunk.kind === "a2ui-display"));
      expect(restored[0].chunks!.find(chunk => chunk.kind === "a2ui-display")).toMatchObject({
        toolCallId: publicationId, profile: "uar.a2ui/1", version: "v0.9.1",
        payload: { metadata: { source_run_id: runId }, content: "retained publication generation" },
      });
      expect(restored[0].chunks!.filter(chunk => chunk.kind === "tool-approval")).toHaveLength(1);
      expect(restored[0].chunks!.find(chunk => chunk.kind === "tool-approval")).toMatchObject({ historical: true, decision: "approved", decidedAt: known });
      expect(restored[0].chunks!.filter(chunk => chunk.kind === "usage")).toHaveLength(0);
      expect(restored[0].usage).toBeUndefined();
      expect(restored[0].chunks!.filter(chunk => chunk.kind === "context-update")).toHaveLength(0);
      const persisted = await adapter(engine).getMessages("thread");
      expect(persisted[0].content).toEqual(restored[0].content);
      expect(persisted[0].chunks).toEqual(restored[0].chunks);
      await store.loadMessagesFromDb("thread");
      expect(useChatMessageStore.getState().messagesByThread.thread).toEqual(persisted);
      expect((await database.getRunEvents(runId)).find(row => row.eventId === context.eventId)).toEqual(contextRow);
      expect(unavailable).not.toHaveBeenCalled();
    });
  }

  for (const originalStatus of ["interrupted", "complete", "failed", "cancelled"] as const) {
    test(`local fixed native message ID preserves ordered bubbles and settled ${originalStatus}`, async () => {
      const unavailable = vi.fn().mockRejectedValue(new Error("server unavailable"));
      vi.stubGlobal("fetch", unavailable);
      const runId = "split";
      const originals: RichMessage[] = [
        { id: "run-split-assistant-0", runId, role: "assistant", status: originalStatus, createdAt: new Date(known), content: [
          { type: "text", text: "Before." }, { type: "toolUse", id: "lookup", name: "lookup", inputJson: "{}" },
          { type: "toolResult", toolUseId: "lookup", outputJson: "retained", isError: false },
        ] },
        { id: "run-split-assistant-1", runId, role: "assistant", status: "interrupted", createdAt: new Date(known), content: [{ type: "text", text: "Final" }] },
      ];
      for (const message of originals) {
        message.chunks = toChunks(message.content, { messageId: message.id, at: known, runId });
        await database.insertMessage("thread", message);
      }
      const writer = new RunEventPersistence(database, { threadId: "thread", fallbackRunId: runId, nativeRunId: runId });
      const frames: Array<{ type: string; [key: string]: unknown }> = [
        { type: "RUN_STARTED", input: { agentId: "native-agent" } },
        { type: "TEXT_MESSAGE_CONTENT", messageId: "split:assistant", delta: "Before." },
        { type: "TOOL_CALL_END", toolCallId: "lookup", toolCallName: "lookup", arguments: {} },
        { type: "TOOL_CALL_RESULT", toolCallId: "lookup", content: "retained" },
        { type: "CUSTOM", name: "uar.context.updated", runId: null, threadId: null,
          value: { strategy: "sliding_window", messagesRemoved: 2, tokensSaved: 128, wasApplied: true, summaryGenerated: false } },
        { type: "TEXT_MESSAGE_CONTENT", messageId: "split:assistant", delta: "Final answer." },
        { type: "RUN_FINISHED" },
      ];
      const nativeAdapter = new UarAguiAdapter();
      for (const [index, frame] of frames.entries()) {
        const event: UarAguiEvent = { runId, threadId: runId, ...frame, profile: "uar.agui/1", eventId: `${index}:0`, sequence: index * 16 };
        const adapted = nativeAdapter.ingest(event.type, JSON.stringify(event), index);
        expect(adapted?.eventRow).toBeDefined();
        await writer.ingest(adapted!.eventRow!);
      }
      await useChatMessageStore.getState().loadMessagesFromDb("thread");
      const restored = useChatMessageStore.getState().messagesByThread.thread;
      expect(restored.map(message => message.id)).toEqual(originals.map(message => message.id));
      expect(restored[0].status).toBe(originalStatus === "interrupted" ? "complete" : originalStatus);
      expect(restored[0].content).toEqual(originals[0].content);
      expect(restored[1].status).toBe("complete");
      expect(restored[1].content).toEqual([{ type: "text", text: "Final answer." }]);
      expect(restored[0].chunks!.map(chunk => chunk.id)).toEqual(originals[0].chunks!.map(chunk => chunk.id));
      expect(restored[1].chunks![0].id).toBe(originals[1].chunks![0].id);
      const firstReadback = await adapter(engine).getMessages("thread");
      await useChatMessageStore.getState().loadMessagesFromDb("thread");
      expect(useChatMessageStore.getState().messagesByThread.thread).toEqual(firstReadback);
      expect(unavailable).not.toHaveBeenCalled();
    });
  }

  test("portable coalesced text retains separate rich approval segments during local recovery", async () => {
    const runId = "approval-split";
    const writer = new RunEventPersistence(database, { threadId: "thread", fallbackRunId: runId, nativeRunId: runId });
    const frames: UarAguiEvent[] = [
      { type: "RUN_STARTED", profile: "uar.agui/1", eventId: "start", sequence: 0, runId },
      { type: "TEXT_MESSAGE_CONTENT", profile: "uar.agui/1", eventId: "before", sequence: 16, runId, messageId: `${runId}:assistant`, delta: "Before." },
      { type: "CUSTOM", profile: "uar.agui/1", eventId: "approval", sequence: 32, runId,
        name: "uar.tool.approval_required", value: { approvalId: "approval", toolCallId: "lookup", name: "lookup", arguments: "{}" } },
      { type: "TEXT_MESSAGE_CONTENT", profile: "uar.agui/1", eventId: "partial", sequence: 48, runId, messageId: `${runId}:assistant`, delta: "Final" },
      { type: "TEXT_MESSAGE_CONTENT", profile: "uar.agui/1", eventId: "later", sequence: 64, runId, messageId: `${runId}:assistant`, delta: " answer." },
      { type: "RUN_FINISHED", profile: "uar.agui/1", eventId: "terminal", sequence: 80, runId },
    ];
    const store = useChatMessageStore.getState();
    store.beginStream("thread", runId);
    store.appendTextDelta("thread", runId, "Before.");
    store.addRuntimeChunk("thread", runId, toRuntimeChunk(frames[2], new Date(known).getTime())!);
    store.appendTextDelta("thread", runId, "Final");
    await store.finishStream("thread", "interrupted");
    const cached = (await database.getMessages("thread"))[0];
    expect(cached.content).toEqual([{ type: "text", text: "Before.Final" }]);
    expect(cached.chunks!.map(chunk => chunk.kind)).toEqual(["markdown", "tool-approval", "markdown"]);
    for (const event of frames) await writer.ingest({ id: event.eventId, type: event.type, sequence: event.sequence,
      runId, receivedAt: new Date(known).getTime(), phase: null, payload: event });
    await store.loadMessagesFromDb("thread");
    const restored = useChatMessageStore.getState().messagesByThread.thread[0];
    expect(restored.content).toEqual([{ type: "text", text: "Before.Final answer." }]);
    expect(restored.chunks!.slice(0, 3).map(chunk => chunk.id)).toEqual(cached.chunks!.map(chunk => chunk.id));
    expect(restored.chunks!.slice(0, 3)).toMatchObject([
      { kind: "markdown", source: "Before." }, { kind: "tool-approval", historical: true }, { kind: "markdown", source: "Final answer." },
    ]);
    expect(restored.status).toBe("complete");
    await store.loadMessagesFromDb("thread");
    expect(useChatMessageStore.getState().messagesByThread.thread[0]).toEqual((await database.getMessages("thread"))[0]);
  });

  for (const fault of ["foreign-run", "foreign-owner", "foreign-outer-run", "wrong-message", "malformed", "missing-start", "coalesced-only", "unmatched-cache",
    "unknown-runless", "runless-text", "runless-state", "runless-action", "runless-artifact", "runless-terminal", "unprofiled-terminal", "uncorrelated-terminal",
    "foreign-context-run", "foreign-context-thread", "malformed-context", "context-before-start", "context-after-terminal", "context-sequence", "context-event-id", "context-type"] as const) {
    test(`local recovery refuses ${fault} evidence without replacement dispatch`, async () => {
      const unavailable = vi.fn().mockRejectedValue(new Error("server unavailable"));
      vi.stubGlobal("fetch", unavailable);
      const runId = "refusal";
      const original: RichMessage = { id: "run-refusal-assistant-0", runId, role: "assistant", status: "interrupted",
        createdAt: new Date(known), content: [{ type: "text", text: fault === "unmatched-cache" ? "independent cached text" : "partial" }] };
      original.chunks = toChunks(original.content, { messageId: original.id, at: known, runId });
      await database.insertMessage("thread", original);
      if (fault === "foreign-owner") await database.upsertThread({ id: "other-thread", title: "Other owner", isEphemeral: false, createdAt: known, updatedAt: known });
      const outerRunId = fault === "foreign-outer-run" ? "foreign-outer" : runId;
      await database.startRun({ id: outerRunId, threadId: fault === "foreign-owner" ? "other-thread" : "thread", startedAt: known });
      const append = async (eventId: string, sequence: number, type: string, fields: Record<string, unknown> = {}) => {
        await database.appendRunEvent({ runId: outerRunId, eventId, wireSequence: sequence, type, kind: "message", at: known,
          payload: { profile: "uar.agui/1", runId, eventId, sequence, type, ...fields } });
      };
      const appendContext = (eventId: string, sequence: number, fields: Record<string, unknown> = {}) => append(eventId, sequence, "CUSTOM", {
        name: "uar.context.updated", runId: null, threadId: null,
        value: { strategy: "sliding_window", messagesRemoved: 2, tokensSaved: 128, wasApplied: true, summaryGenerated: false }, ...fields,
      });
      if (fault === "context-before-start") await appendContext("early-context", 0);
      if (fault !== "missing-start") await append("start", 0, "RUN_STARTED");
      if (fault === "coalesced-only") await append("delta:coalesced", 16, "TEXT_MESSAGE_END", {
        messageId: "refusal:assistant", coalesced: true, sourceEventCount: 1, content: "partial newer",
      });
      else await append("delta", 16, "TEXT_MESSAGE_CONTENT", {
        messageId: fault === "wrong-message" ? "different:assistant" : "refusal:assistant", delta: "partial newer",
        ...(fault === "malformed" ? { sequence: "16" } : {}),
        ...(fault === "runless-text" ? { runId: null } : {}),
      });
      if (fault === "unknown-runless") await append("side-data", 24, "CUSTOM", { runId: null, threadId: null, name: "uar.unknown", value: {} });
      if (fault === "runless-state") await append("state", 24, "STATE_DELTA", { runId: null, delta: [] });
      if (fault === "runless-action") await append("action", 24, "CUSTOM", { runId: null, name: "uar.tool.approval_required", value: { approvalId: "foreign-approval" } });
      if (fault === "runless-artifact") await append("artifact", 24, "CUSTOM", { runId: null, name: "uar.artifact.available", value: { artifactId: "foreign-artifact", artifactType: "text", content: "not owned" } });
      if (fault === "foreign-context-run") await appendContext("context", 24, { runId: "foreign" });
      if (fault === "foreign-context-thread") await appendContext("context", 24, { threadId: "foreign" });
      if (fault === "malformed-context") await appendContext("context", 24, { value: { strategy: "sliding_window", messagesRemoved: "2" } });
      if (fault === "context-sequence") await appendContext("context", 24, { sequence: 8 });
      if (fault === "context-event-id") await appendContext("context", 24, { eventId: "different-context" });
      if (fault === "context-type") await appendContext("context", 24, { type: "RUN_FINISHED" });
      await append("terminal", 32, "RUN_FINISHED", {
        ...(fault === "foreign-run" ? { runId: "foreign" } : fault === "runless-terminal" ? { runId: null } : {}),
        ...(fault === "unprofiled-terminal" ? { profile: undefined } : {}),
        ...(fault === "uncorrelated-terminal" ? { eventId: "different-terminal" } : {}),
      });
      if (fault === "context-after-terminal") await appendContext("late-context", 48);
      await database.finishRun({ id: outerRunId, status: "finished", finishedAt: known, nativeTerminal: { runId: outerRunId, eventId: "terminal" } });
      await useChatMessageStore.getState().loadMessagesFromDb("thread");
      const restored = useChatMessageStore.getState().messagesByThread.thread;
      expect(restored).toHaveLength(1);
      expect(restored[0]).toMatchObject({ id: original.id, status: "interrupted", content: original.content, chunks: original.chunks });
      await useChatMessageStore.getState().loadMessagesFromDb("thread");
      expect(useChatMessageStore.getState().messagesByThread.thread).toEqual(restored);
      expect(unavailable).not.toHaveBeenCalled();
    });
  }

  for (const variant of ["same", "prefix", "changed", "changed-final", "unanchored", "tool-only"] as const) {
    test(`warm canonical text keeps rich chronology through real adapter reload (${variant})`, async () => {
      const id = "run-warm-assistant-0";
      const original: RichMessage = { id, runId: "warm", role: "assistant", status: "complete",
        createdAt: epoch, createdAtKnown: false, content: [
          { type: "text", text: "Checking now." },
          { type: "toolUse", id: "inspect", name: "inspect", inputJson: "{}" },
          { type: "toolResult", toolUseId: "inspect", outputJson: "found", isError: false },
          { type: "artifact", id: "retained", kind: "text/plain", content: "evidence" },
          { type: "image", url: "data:image/png;base64,eA==", mime: "image/png", dataBase64: null },
          { type: "text", text: "Evidence checked." },
        ] };
      original.chunks = toChunks(original.content, { messageId: id, runId: "warm", at: known, finalized: true });
      original.chunks.splice(original.chunks.length - 1, 0, { kind: "tool-approval", id: "original-approval", at: known,
        seq: 5, runId: "warm", approvalId: "approved", toolCallId: "inspect", toolName: "inspect", args: { retained: true },
        reason: "Original host decision", decision: "approved", decidedAt: known, historical: true, historicalState: "approved" });
      // Portable streaming text can coalesce across a rich-only approval. Its
      // separate rich markdown anchors remain the source for rendered ordering.
      if (variant === "changed") original.content = [
        { type: "text", text: "Checking now.Evidence checked." }, ...original.content.filter(block => block.type !== "text"),
      ];
      await database.insertMessage("thread", original);
      const intro = variant === "changed" ? "Rechecked now." : variant === "unanchored" ? "Revised explanation."
        : variant === "tool-only" ? "" : "Checking now.";
      const conclusion = variant === "changed-final" ? "Updated evidence." : variant === "same" || variant === "changed" ? "Evidence checked." : "";
      const canonicalText = intro + conclusion;
      const history = [
        { message_id: "user", role: "user", content: "inspect", status: "complete", created_at: known },
        { message_id: id, run_id: "warm", role: "assistant", content: variant === "same" ? [{ type: "text", text: canonicalText }] : canonicalText, status: "complete", created_at: known },
        { message_id: "run-warm-assistant-1", run_id: "warm", role: "assistant", content: "Final answer.", status: "complete", created_at: known },
        { message_id: "later-user", role: "user", content: "next", status: "complete", created_at: known },
      ];
      const request = vi.fn<typeof fetch>().mockImplementation(async () => new Response(JSON.stringify({ history_version: 1,
        history, history_runs_availability: "available", history_runs: { warm: {
          usage_availability: "available", usage: { input_tokens: 2, output_tokens: 3, total_tokens: 5 },
          approvals_availability: "available", approvals: [{ approval_id: "approved", tool_call_id: "inspect", tool_name: "inspect",
            state: "approved", decision: { approved: true, decided_at: known } }],
        } },
      }), { status: 200 }));
      vi.stubGlobal("fetch", request);
      const store = useChatMessageStore.getState();
      await store.loadMessagesFromDb("thread");
      await store.backfillFromServer("thread");
      const restored = useChatMessageStore.getState().messagesByThread.thread;
      expect(restored.map(message => message.id)).toEqual(history.map(message => message.message_id));
      expect(restored.map(message => message.historyPosition)).toEqual([0, 1, 2, 3]);
      expect(restored[1]).toMatchObject({ id, runId: "warm", createdAt: new Date(known), createdAtKnown: true });
      expect(restored[1].content.filter(block => block.type === "text").map(block => block.text).join("")).toBe(canonicalText);
      const portable = restored[1].content.filter(block => block.type !== "artifact" || block.id !== `${id}:history`)
        .map(block => block.type === "text" ? block.text : block.type);
      expect(portable).toEqual(variant === "changed"
        ? [canonicalText, "toolUse", "toolResult", "artifact", "image"]
        : [...(intro ? [intro] : []), "toolUse", "toolResult", "artifact", "image", ...(conclusion ? [conclusion] : [])]);
      const rich = restored[1].chunks!.filter(chunk => chunk.kind !== "usage"
        && !(chunk.kind === "artifact" && chunk.mime === "application/vnd.uar.history-provenance+json"));
      expect(rich.map(chunk => chunk.kind === "markdown" ? chunk.source : chunk.kind))
        .toEqual([...(intro ? [intro] : []), "tool-call", "artifact", "image", "tool-approval", ...(conclusion ? [conclusion] : [])]);
      expect(rich.filter(chunk => chunk.kind !== "markdown").map(chunk => chunk.id))
        .toEqual(original.chunks.filter(chunk => chunk.kind !== "markdown").map(chunk => chunk.id));
      expect(rich.find(chunk => chunk.kind === "tool-approval")).toMatchObject({ id: "original-approval", args: { retained: true },
        reason: "Original host decision", decision: "approved", decidedAt: known, historical: true, historicalState: "approved" });
      expect(restored[2].content.filter(block => block.type === "text")).toEqual([{ type: "text", text: "Final answer." }]);
      expect(restored.flatMap(message => message.chunks ?? []).filter(chunk => chunk.kind === "usage")).toHaveLength(1);
      await store.backfillFromServer("thread");
      expect(useChatMessageStore.getState().messagesByThread.thread).toEqual(restored);
      useChatMessageStore.setState({ messagesByThread: {}, streamingByThread: {} });
      setDbInstance(adapter(engine));
      await store.loadMessagesFromDb("thread");
      expect(useChatMessageStore.getState().messagesByThread.thread).toEqual(await adapter(engine).getMessages("thread"));
      expect(useChatMessageStore.getState().messagesByThread.thread).toEqual(restored);
      await store.backfillFromServer("thread");
      expect(await adapter(engine).getMessages("thread")).toEqual(restored);
      expect(request).toHaveBeenCalledTimes(3);
      for (const [url, options] of request.mock.calls) {
        expect(url).toBe("/api/uar/sessions/thread");
        expect(options).toEqual({ cache: "no-store" });
      }
    });
  }

  for (const timestamp of [known, null]) {
    test(`persists canonical timestamp and availability across adapter reload (${timestamp})`, async () => {
      const original: RichMessage = { id: "run-r-assistant-0", runId: "r", role: "assistant",
        content: [{ type: "text", text: "local aggregate" }], createdAt: timestamp ? epoch : new Date(known),
        createdAtKnown: !timestamp, status: "complete" };
      await database.insertMessage("thread", original);
      const canonical = missingServerMessages([], [
        { message_id: "user", role: "user", content: "question", status: "complete", created_at: known },
        { message_id: original.id, run_id: "r", role: "assistant", content: "preamble", status: "complete", created_at: timestamp },
        { message_id: "run-r-assistant-1", run_id: "r", role: "assistant", content: "final answer", status: "complete", created_at: timestamp },
        { message_id: "later-user", role: "user", content: "next question", status: "complete", created_at: known },
      ]);
      // Match the store's production canonical projection before persistence.
      for (const message of canonical) message.chunks = toChunks(message.content, {
        messageId: message.id, runId: message.runId, at: message.createdAt.toISOString(), finalized: true,
      });
      // Insert out of chronology to require production history_position ordering.
      for (const index of [3, 2, 0, 1]) await database.insertMessage("thread", canonical[index]);
      const first = await database.getMessages("thread");
      const reloaded = await adapter(engine).getMessages("thread");
      expect(reloaded).toEqual(first);
      expect(reloaded.map(row => row.id)).toEqual(canonical.map(row => row.id));
      const restored = reloaded[1];
      expect(restored.createdAt.toISOString()).toBe(new Date(timestamp ?? 0).toISOString());
      expect(restored.createdAtKnown).toBe(Boolean(timestamp));
      expect(restored.historyPosition).toBe(1);
      expect(restored.content).toContainEqual({ type: "text", text: "preamble" });
      expect(reloaded[2].content).toContainEqual({ type: "text", text: "final answer" });
      expect(restored.chunks?.find(chunk => chunk.kind === "markdown")).toMatchObject({
        at: new Date(timestamp ?? 0).toISOString(), source: "preamble" });
      const evidence = restored.content.find(block => block.type === "artifact" && block.kind === "application/vnd.uar.history-provenance+json");
      expect(evidence?.type === "artifact" && JSON.parse(evidence.content).created_at_availability)
        .toBe(timestamp ? "available" : "unavailable");
      // Unavailable canonical epoch remains explicitly hidden by the production
      // createdAtKnown contract consumed by richMessageToThreadMessageLike.
      if (!timestamp) expect(restored.createdAtKnown).toBe(false);
      await database.insertMessage("thread", canonical[1]);
      expect(await adapter(engine).getMessages("thread")).toEqual(reloaded);
    });
  }

  test("ordinary unpositioned reinsertion preserves its original creation time", async () => {
    const original: RichMessage = { id: "ordinary", role: "assistant", content: [{ type: "text", text: "original" }],
      createdAt: new Date(known), status: "complete" };
    await database.insertMessage("thread", original);
    await database.insertMessage("thread", { ...original, createdAt: epoch, content: [{ type: "text", text: "updated content" }] });
    const reloaded = await adapter(engine).getMessages("thread");
    expect(reloaded).toHaveLength(1);
    expect(reloaded[0]).toMatchObject({ id: original.id, historyPosition: 0, createdAtKnown: true });
    expect(reloaded[0].createdAt.toISOString()).toBe(known);
    expect(reloaded[0].content).toEqual([{ type: "text", text: "updated content" }]);
  });
});
