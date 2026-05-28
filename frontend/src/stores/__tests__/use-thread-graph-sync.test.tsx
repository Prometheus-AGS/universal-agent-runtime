/**
 * Contract test — Thread graph → thread-registry sync.
 *
 * Pins the SSE-driven reconciliation pattern. Three branches:
 *   - server insert of an unknown thread → registry creates persisted entry
 *   - server update of a known thread's title → setTitle called
 *   - server delete → removeThread called
 *
 * The local PGlite layer is mocked via `vi.mock` so the test stays in-memory.
 */
import { act, render } from "@testing-library/react";
import { describe, expect, test, beforeEach, vi } from "vitest";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { useThreadRegistryStore } from "@/stores/thread-registry-store";
import { useThreadGraphSync } from "@/stores/use-thread-graph-sync";

// PGlite is not available in the happy-dom test env; stub the DB layer.
vi.mock("@/lib/db", () => ({
  getDbInstance: () => ({
    getThreads: async () => [],
    upsertThread: async () => undefined,
    deleteThread: async () => undefined,
    touchThread: async () => undefined,
  }),
}));

function Harness() {
  useThreadGraphSync();
  return null;
}

beforeEach(() => {
  // Reset graph + registry.
  useGraphStore.setState({ entities: {} });
  useThreadRegistryStore.setState({ threads: {}, activeThreadId: null });
});

describe("useThreadGraphSync", () => {
  test("server insert of unknown thread → registry creates persisted entry", async () => {
    render(<Harness />);

    await act(async () => {
      useGraphStore.getState().upsertEntity("Thread", "sess-1", {
        id: "sess-1",
        title: "Hello from server",
        updated_at: new Date().toISOString(),
      });
    });

    const local = useThreadRegistryStore.getState().threads["sess-1"];
    expect(local).toBeDefined();
    expect(local?.title).toBe("Hello from server");
    expect(local?.isEphemeral).toBe(false);
  });

  test("server update of known thread's title → setTitle applied", async () => {
    // Pre-seed: locally-created ephemeral thread.
    useThreadRegistryStore.getState().registerThread("sess-2");
    expect(useThreadRegistryStore.getState().threads["sess-2"]?.title).toBe(
      "New conversation",
    );

    render(<Harness />);

    await act(async () => {
      useGraphStore.getState().upsertEntity("Thread", "sess-2", {
        id: "sess-2",
        title: "Renamed by server",
        updated_at: new Date().toISOString(),
      });
    });

    const local = useThreadRegistryStore.getState().threads["sess-2"];
    expect(local?.title).toBe("Renamed by server");
    expect(local?.isEphemeral).toBe(false);
  });

  test("server delete event → removeThread called", async () => {
    // Pre-seed: server-known thread.
    useThreadRegistryStore.getState().registerThread("sess-3");
    useThreadRegistryStore.getState().markPersisted("sess-3");

    render(<Harness />);

    // Establish the prior keyset by upserting then deleting.
    await act(async () => {
      useGraphStore.getState().upsertEntity("Thread", "sess-3", {
        id: "sess-3",
        title: "Doomed thread",
      });
    });
    expect(useThreadRegistryStore.getState().threads["sess-3"]).toBeDefined();

    await act(async () => {
      useGraphStore.getState().removeEntity("Thread", "sess-3");
    });

    expect(useThreadRegistryStore.getState().threads["sess-3"]).toBeUndefined();
  });
});
