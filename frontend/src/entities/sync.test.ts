import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { useGraphStore } from "@/platform/entities";
import type { ChangeSet } from "@/platform/entities";

import { createEmbeddedSseAdapter } from "./sync";

class ControlledSseStream {
  private controller: ReadableStreamDefaultController<Uint8Array> | undefined;
  private closed = false;
  readonly body = new ReadableStream<Uint8Array>({
    start: (controller) => { this.controller = controller; },
  });

  constructor(readonly signal: AbortSignal | null | undefined) {
    signal?.addEventListener("abort", () => this.fail(), { once: true });
  }

  send(name: string, data: unknown) {
    if (this.closed) return;
    const payload = typeof data === "string" ? data : JSON.stringify(data);
    this.controller?.enqueue(new TextEncoder().encode(`event: ${name}\ndata: ${payload}\n\n`));
  }

  fail() {
    if (this.closed) return;
    this.closed = true;
    this.controller?.error(new Error("fixture SSE connection ended"));
  }
}

describe("createEmbeddedSseAdapter", () => {
  let streams: ControlledSseStream[];
  let unsubscribe: (() => void) | undefined;

  beforeEach(() => {
    streams = [];
    vi.stubGlobal("fetch", vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      expect(input).toBe("/api/uar/sync/stream");
      const stream = new ControlledSseStream(init?.signal);
      streams.push(stream);
      return new Response(stream.body, { headers: { "Content-Type": "text/event-stream" } });
    }));
    useGraphStore.setState({ entities: {} });
  });

  afterEach(() => {
    unsubscribe?.();
    unsubscribe = undefined;
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  test("maps only valid named entity.change payloads", async () => {
    const handler = vi.fn<(changeset: ChangeSet) => void>();
    const adapter = createEmbeddedSseAdapter("/api/uar/sync/stream");
    unsubscribe = adapter.subscribe({ label: "embedded" }, handler);
    const source = streams[0]!;

    source.send("message", {
      table: "knowledge_bases",
      action: "update",
      id: "ignored-message",
      record: { id: "ignored-message" },
    });
    source.send("entity.change", "not-json");
    source.send("entity.change", {
      table: "unknown_table",
      action: "update",
      id: "ignored-table",
      record: { id: "ignored-table" },
    });
    source.send("entity.change", {
      table: "knowledge_bases",
      action: "unknown",
      id: "ignored-action",
      record: { id: "ignored-action" },
    });
    source.send("entity.change", {
      table: "knowledge_bases",
      action: "update",
      record: { name: "Missing id" },
    });
    source.send("entity.change", {
      table: "knowledge_bases",
      action: "update",
      id: "scalar-record",
      record: "not-an-object",
    });
    source.send("entity.change", {
      table: "knowledge_bases",
      action: "update",
      id: "kb-1",
      record: { id: "kb-1", name: "Live knowledge" },
      ts: "2026-08-20T07:00:00Z",
    });

    await vi.waitFor(() => expect(handler).toHaveBeenCalledTimes(1));
    expect(handler).toHaveBeenCalledWith({
      changes: [
        {
          op: "update",
          type: "KnowledgeBase",
          id: "kb-1",
          data: { id: "kb-1", name: "Live knowledge" },
        },
      ],
      timestamp: "2026-08-20T07:00:00Z",
    });
  });

  test("reconciles an authoritative snapshot after reconnect", async () => {
    vi.useFakeTimers();
    const statuses: string[] = [];
    const handler = vi.fn<(changeset: ChangeSet) => void>();
    const adapter = createEmbeddedSseAdapter("/api/uar/sync/stream", {
      reconnectBaseDelay: 10,
      maxReconnectDelay: 100,
    });
    adapter.onStatusChange?.((status) => statuses.push(status));
    unsubscribe = adapter.subscribe({ label: "embedded" }, handler);
    useGraphStore.getState().upsertEntity("KnowledgeBase", "stale-kb", {
      id: "stale-kb",
      name: "Stale predecessor",
    });

    const first = streams[0]!;
    await vi.advanceTimersByTimeAsync(0);
    expect(statuses).toEqual(["connecting", "connected"]);
    first.fail();
    await vi.advanceTimersByTimeAsync(100);
    expect(statuses).toEqual(["connecting", "connected", "error"]);
    expect(streams).toHaveLength(1);
    first.send("entity.change", {
      table: "knowledge_bases",
      action: "update",
      id: "stale-kb",
      record: { id: "stale-kb", name: "Stale predecessor" },
    });
    expect(handler).not.toHaveBeenCalled();

    window.dispatchEvent(new Event("online"));
    expect(streams).toHaveLength(2);
    const second = streams[1]!;
    second.send("entity.snapshot", {
      table: "knowledge_bases",
      complete: true,
      records: [{ id: "kb-1", name: "Recovered knowledge" }],
      ts: "2026-08-20T08:00:00Z",
    });

    await vi.waitFor(() => expect(handler).toHaveBeenCalledTimes(1));
    expect(handler).toHaveBeenCalledWith({
      changes: [
        {
          op: "upsert",
          type: "KnowledgeBase",
          id: "kb-1",
          data: { id: "kb-1", name: "Recovered knowledge" },
        },
        { op: "delete", type: "KnowledgeBase", id: "stale-kb" },
      ],
      affectedListKeys: [""],
      timestamp: "2026-08-20T08:00:00Z",
    });
    expect(statuses).toEqual([
      "connecting",
      "connected",
      "error",
      "connecting",
      "connected",
    ]);
  });

  test("rejects a malformed snapshot without deleting local entities", async () => {
    const handler = vi.fn<(changeset: ChangeSet) => void>();
    const adapter = createEmbeddedSseAdapter("/api/uar/sync/stream");
    unsubscribe = adapter.subscribe({ label: "embedded" }, handler);
    useGraphStore.getState().upsertEntity("KnowledgeBase", "local-kb", {
      id: "local-kb",
      name: "Keep me",
    });

    streams[0]!.send("entity.snapshot", {
      table: "knowledge_bases",
      complete: true,
      records: [{ name: "Missing id" }],
    });

    await vi.waitFor(() => expect(handler).toHaveBeenCalledTimes(1));
    expect(handler).toHaveBeenCalledWith({ changes: [], affectedListKeys: [""] });
    expect(useGraphStore.getState().entities.KnowledgeBase?.["local-kb"]).toMatchObject({
      name: "Keep me",
    });
  });

  test("unsubscribe aborts the stream and removes explicit recovery listeners", async () => {
    vi.useFakeTimers();
    const statuses: string[] = [];
    const handler = vi.fn<(changeset: ChangeSet) => void>();
    const adapter = createEmbeddedSseAdapter("/api/uar/sync/stream", {
      reconnectBaseDelay: 10,
    });
    adapter.onStatusChange?.((status) => statuses.push(status));
    unsubscribe = adapter.subscribe({ label: "embedded" }, handler);

    const source = streams[0]!;
    await vi.advanceTimersByTimeAsync(0);
    expect(statuses).toEqual(["connecting", "connected"]);
    unsubscribe();
    unsubscribe = undefined;
    source.send("entity.change", {
      table: "knowledge_bases",
      action: "update",
      id: "late-kb",
      record: { id: "late-kb", name: "Late delivery" },
    });
    window.dispatchEvent(new Event("online"));
    await vi.runAllTimersAsync();

    expect(streams).toHaveLength(1);
    expect(source.signal?.aborted).toBe(true);
    expect(handler).not.toHaveBeenCalled();
    expect(statuses.at(-1)).toBe("disconnected");
  });
});
