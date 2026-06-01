/**
 * Contract test — UAR SSE adapter.
 *
 * `createUarSseAdapter` is the single boundary between SurrealDB live-query
 * events (delivered via SSE) and the entity graph. Its payload-shape
 * contract is consumed by every `useEntity*` reader transitively. A
 * regression in the event-name → `EntityChange.op` mapping would silently
 * swallow updates without any compile-time signal.
 */
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import type { ChangeSet } from "@prometheus-ags/prometheus-entity-management";

import { createUarSseAdapter } from "../uar-sse-adapter";

type Listener = (ev: MessageEvent | Event) => void;

class FakeEventSource {
  static instances: FakeEventSource[] = [];
  readonly url: string;
  onopen: ((ev: Event) => void) | null = null;
  onerror: ((ev: Event) => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  private listeners = new Map<string, Set<Listener>>();
  closed = false;

  constructor(url: string) {
    this.url = url;
    FakeEventSource.instances.push(this);
  }

  addEventListener(name: string, handler: Listener) {
    let set = this.listeners.get(name);
    if (!set) {
      set = new Set();
      this.listeners.set(name, set);
    }
    set.add(handler);
  }

  removeEventListener(name: string, handler: Listener) {
    this.listeners.get(name)?.delete(handler);
  }

  close() {
    this.closed = true;
  }

  /** Test helper: synthesize a server-sent event with named type + JSON payload. */
  dispatch(name: string, data: unknown) {
    const ev = new MessageEvent(name, {
      data: typeof data === "string" ? data : JSON.stringify(data),
    });
    for (const handler of this.listeners.get(name) ?? []) handler(ev);
  }

  /** Fire the synthetic `onopen` to drive the status transition. */
  fireOpen() {
    this.onopen?.(new Event("open"));
  }

  static reset() {
    FakeEventSource.instances = [];
  }
}

describe("createUarSseAdapter", () => {
  beforeEach(() => {
    FakeEventSource.reset();
    vi.stubGlobal("EventSource", FakeEventSource);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test("maps create event to EntityChange op=insert", () => {
    const adapter = createUarSseAdapter({
      topic: "providers",
      entityType: "Provider",
    });
    const handler = vi.fn();
    adapter.subscribe({ label: "test" }, handler as (cs: ChangeSet) => void);

    const es = FakeEventSource.instances[0]!;
    es.dispatch("create", { topic: "providers", id: "p1", data: { id: "p1" } });

    expect(handler).toHaveBeenCalledTimes(1);
    const cs = handler.mock.calls[0]![0] as ChangeSet;
    expect(cs.changes).toHaveLength(1);
    expect(cs.changes[0]!.op).toBe("insert");
    expect(cs.changes[0]!.type).toBe("Provider");
    expect(cs.changes[0]!.id).toBe("p1");
    expect(cs.changes[0]!.data).toEqual({ id: "p1" });
  });

  test("maps update event to EntityChange op=update", () => {
    const adapter = createUarSseAdapter({
      topic: "providers",
      entityType: "Provider",
    });
    const handler = vi.fn();
    adapter.subscribe({ label: "test" }, handler as (cs: ChangeSet) => void);
    const es = FakeEventSource.instances[0]!;

    es.dispatch("update", {
      topic: "providers",
      id: "p1",
      data: { id: "p1", display_name: "Alpha" },
    });

    expect(handler).toHaveBeenCalledTimes(1);
    expect((handler.mock.calls[0]![0] as ChangeSet).changes[0]!.op).toBe(
      "update",
    );
  });

  test("maps delete event to EntityChange op=delete", () => {
    const adapter = createUarSseAdapter({
      topic: "providers",
      entityType: "Provider",
    });
    const handler = vi.fn();
    adapter.subscribe({ label: "test" }, handler as (cs: ChangeSet) => void);
    const es = FakeEventSource.instances[0]!;

    es.dispatch("delete", { topic: "providers", id: "p1", data: { id: "p1" } });

    expect(handler).toHaveBeenCalledTimes(1);
    expect((handler.mock.calls[0]![0] as ChangeSet).changes[0]!.op).toBe(
      "delete",
    );
  });

  test("unsubscribe stops further deliveries", () => {
    const adapter = createUarSseAdapter({
      topic: "providers",
      entityType: "Provider",
    });
    const handler = vi.fn();
    const unsub = adapter.subscribe(
      { label: "test" },
      handler as (cs: ChangeSet) => void,
    );
    const es = FakeEventSource.instances[0]!;

    es.dispatch("create", { topic: "providers", id: "p1", data: { id: "p1" } });
    expect(handler).toHaveBeenCalledTimes(1);

    unsub();

    es.dispatch("create", { topic: "providers", id: "p2", data: { id: "p2" } });
    expect(handler).toHaveBeenCalledTimes(1); // no new call
  });

  test("status callback transitions to connected on EventSource onopen", () => {
    const adapter = createUarSseAdapter({
      topic: "providers",
      entityType: "Provider",
    });
    const statusSpy = vi.fn();
    adapter.onStatusChange?.(statusSpy);
    const noop = () => {};
    adapter.subscribe({ label: "test" }, noop);

    const es = FakeEventSource.instances[0]!;
    es.fireOpen();

    // The "connecting" status is emitted synchronously at subscribe time;
    // after fireOpen we expect "connected" to appear somewhere in the calls.
    const seen = statusSpy.mock.calls.map((c) => c[0]);
    expect(seen).toContain("connected");
  });
});
