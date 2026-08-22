import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { useGraphStore } from "@/platform/entities";
import type { ChangeSet } from "@/platform/entities";

import { createEmbeddedSseAdapter } from "./sync";

type Listener = (event: Event) => void;

class FakeEventSource {
  static instances: FakeEventSource[] = [];

  readonly url: string;
  onopen: ((event: Event) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  closed = false;
  private readonly listeners = new Map<string, Set<Listener>>();

  constructor(url: string) {
    this.url = url;
    FakeEventSource.instances.push(this);
  }

  addEventListener(name: string, listener: Listener) {
    let listeners = this.listeners.get(name);
    if (!listeners) {
      listeners = new Set();
      this.listeners.set(name, listeners);
    }
    listeners.add(listener);
  }

  removeEventListener(name: string, listener: Listener) {
    this.listeners.get(name)?.delete(listener);
  }

  close() {
    this.closed = true;
  }

  dispatch(name: string, data: unknown) {
    const event = new MessageEvent(name, {
      data: typeof data === "string" ? data : JSON.stringify(data),
    });
    for (const listener of this.listeners.get(name) ?? []) listener(event);
  }

  fireOpen() {
    this.onopen?.(new Event("open"));
  }

  fireError() {
    this.onerror?.(new Event("error"));
  }

  static reset() {
    FakeEventSource.instances = [];
  }
}

describe("createEmbeddedSseAdapter", () => {
  beforeEach(() => {
    FakeEventSource.reset();
    vi.stubGlobal("EventSource", FakeEventSource);
    useGraphStore.setState({ entities: {} });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  test("maps only valid named entity.change payloads", () => {
    const handler = vi.fn<(changeset: ChangeSet) => void>();
    const adapter = createEmbeddedSseAdapter("/api/uar/sync/stream");
    adapter.subscribe({ label: "embedded" }, handler);
    const source = FakeEventSource.instances[0]!;

    source.dispatch("message", {
      table: "knowledge_bases",
      action: "update",
      id: "ignored-message",
      record: { id: "ignored-message" },
    });
    source.dispatch("entity.change", "not-json");
    source.dispatch("entity.change", {
      table: "unknown_table",
      action: "update",
      id: "ignored-table",
      record: { id: "ignored-table" },
    });
    source.dispatch("entity.change", {
      table: "knowledge_bases",
      action: "unknown",
      id: "ignored-action",
      record: { id: "ignored-action" },
    });
    source.dispatch("entity.change", {
      table: "knowledge_bases",
      action: "update",
      record: { name: "Missing id" },
    });
    source.dispatch("entity.change", {
      table: "knowledge_bases",
      action: "update",
      id: "scalar-record",
      record: "not-an-object",
    });
    source.dispatch("entity.change", {
      table: "knowledge_bases",
      action: "update",
      id: "kb-1",
      record: { id: "kb-1", name: "Live knowledge" },
      ts: "2026-08-20T07:00:00Z",
    });

    expect(handler).toHaveBeenCalledTimes(1);
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
    adapter.subscribe({ label: "embedded" }, handler);
    useGraphStore.getState().upsertEntity("KnowledgeBase", "stale-kb", {
      id: "stale-kb",
      name: "Stale predecessor",
    });

    const first = FakeEventSource.instances[0]!;
    first.fireOpen();
    first.fireError();
    expect(first.closed).toBe(true);
    expect(FakeEventSource.instances).toHaveLength(1);
    first.dispatch("entity.change", {
      table: "knowledge_bases",
      action: "update",
      id: "stale-kb",
      record: { id: "stale-kb", name: "Stale predecessor" },
    });
    expect(handler).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(10);
    expect(FakeEventSource.instances).toHaveLength(2);
    const second = FakeEventSource.instances[1]!;
    second.fireOpen();
    second.dispatch("entity.snapshot", {
      table: "knowledge_bases",
      records: [{ id: "kb-1", name: "Recovered knowledge" }],
      ts: "2026-08-20T08:00:00Z",
    });

    expect(handler).toHaveBeenCalledTimes(1);
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

  test("rejects a malformed snapshot without deleting local entities", () => {
    const handler = vi.fn<(changeset: ChangeSet) => void>();
    const adapter = createEmbeddedSseAdapter("/api/uar/sync/stream");
    adapter.subscribe({ label: "embedded" }, handler);
    useGraphStore.getState().upsertEntity("KnowledgeBase", "local-kb", {
      id: "local-kb",
      name: "Keep me",
    });

    FakeEventSource.instances[0]!.dispatch("entity.snapshot", {
      table: "knowledge_bases",
      records: [{ name: "Missing id" }],
    });

    expect(handler).not.toHaveBeenCalled();
    expect(useGraphStore.getState().entities.KnowledgeBase?.["local-kb"]).toMatchObject({
      name: "Keep me",
    });
  });

  test("unsubscribe cancels a pending reconnect", async () => {
    vi.useFakeTimers();
    const statuses: string[] = [];
    const handler = vi.fn<(changeset: ChangeSet) => void>();
    const adapter = createEmbeddedSseAdapter("/api/uar/sync/stream", {
      reconnectBaseDelay: 10,
    });
    adapter.onStatusChange?.((status) => statuses.push(status));
    const unsubscribe = adapter.subscribe({ label: "embedded" }, handler);

    const source = FakeEventSource.instances[0]!;
    source.fireOpen();
    source.fireError();
    unsubscribe();
    source.dispatch("entity.change", {
      table: "knowledge_bases",
      action: "update",
      id: "late-kb",
      record: { id: "late-kb", name: "Late delivery" },
    });
    await vi.runAllTimersAsync();

    expect(FakeEventSource.instances).toHaveLength(1);
    expect(source.closed).toBe(true);
    expect(handler).not.toHaveBeenCalled();
    expect(statuses.at(-1)).toBe("disconnected");
  });
});
