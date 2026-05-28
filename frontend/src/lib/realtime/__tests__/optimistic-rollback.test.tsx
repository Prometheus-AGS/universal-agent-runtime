/**
 * Contract test — optimistic rollback.
 *
 * Locks the snapshot → optimistic mutation → rollback-on-failure pattern
 * used by:
 *   - Provider `setDefault` (singleton patch)
 *   - Provider `removeProvider` (entity delete)
 *   - Agent `patchAgent` (shallow merge)
 *   - Agent `deleteAgent` (entity delete)
 *
 * The pattern is currently inlined in each page; this test pins the
 * underlying graph primitives (upsertEntity / removeEntity + snapshot
 * capture) so an extraction to a shared helper can be done with confidence.
 */
import { describe, expect, test, vi } from "vitest";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { optimisticUpsert, optimisticRemove } from "../optimistic";

describe("optimistic rollback", () => {
  test("upsert: rejected server call restores the snapshot", async () => {
    const graph = useGraphStore.getState();
    graph.upsertEntity("Agent", "a1", { id: "a1", memory_enabled: true });

    const serverCall = vi.fn().mockRejectedValue(new Error("forced"));
    await expect(
      optimisticUpsert("Agent", "a1", { memory_enabled: false }, serverCall),
    ).rejects.toThrow("forced");

    // Server rejected — graph must have rolled back to the snapshot.
    const after = useGraphStore.getState().entities["Agent"]?.["a1"] as
      | { memory_enabled?: boolean }
      | undefined;
    expect(after?.memory_enabled).toBe(true);
  });

  test("upsert: successful server call preserves the optimistic patch", async () => {
    const graph = useGraphStore.getState();
    graph.upsertEntity("Agent", "a2", { id: "a2", memory_enabled: true });

    const serverCall = vi.fn().mockResolvedValue(undefined);
    await optimisticUpsert("Agent", "a2", { memory_enabled: false }, serverCall);

    const after = useGraphStore.getState().entities["Agent"]?.["a2"] as
      | { memory_enabled?: boolean }
      | undefined;
    expect(after?.memory_enabled).toBe(false);
  });

  test("remove: rejected server call re-upserts the snapshot", async () => {
    const graph = useGraphStore.getState();
    graph.upsertEntity("Provider", "p1", {
      id: "p1",
      display_name: "Alpha",
      configured: true,
    });

    const serverCall = vi.fn().mockRejectedValue(new Error("forced"));
    await expect(
      optimisticRemove("Provider", "p1", serverCall),
    ).rejects.toThrow("forced");

    const after = useGraphStore.getState().entities["Provider"]?.["p1"] as
      | { display_name?: string; configured?: boolean }
      | undefined;
    expect(after).toBeDefined();
    expect(after?.display_name).toBe("Alpha");
    expect(after?.configured).toBe(true);
  });

  test("remove: successful server call leaves the row deleted", async () => {
    const graph = useGraphStore.getState();
    graph.upsertEntity("Provider", "p2", {
      id: "p2",
      display_name: "Bravo",
      configured: true,
    });

    const serverCall = vi.fn().mockResolvedValue(undefined);
    await optimisticRemove("Provider", "p2", serverCall);

    const after = useGraphStore.getState().entities["Provider"]?.["p2"];
    expect(after).toBeUndefined();
  });

  test("ProviderMeta singleton rollback restores prior default_id", async () => {
    const graph = useGraphStore.getState();
    graph.upsertEntity("ProviderMeta", "current", {
      id: "current",
      default_id: "p1",
    });

    // Pattern from providers-page setDefault: optimistically flip the
    // singleton, then roll back if the service call fails.
    const previousDefault =
      (useGraphStore.getState().entities["ProviderMeta"]?.["current"] as {
        default_id?: string | null;
      } | undefined)?.default_id ?? null;

    useGraphStore
      .getState()
      .upsertEntity("ProviderMeta", "current", {
        id: "current",
        default_id: "p2",
      });

    const serverCall = vi.fn().mockRejectedValue(new Error("forced"));
    try {
      await serverCall();
    } catch {
      useGraphStore.getState().upsertEntity("ProviderMeta", "current", {
        id: "current",
        default_id: previousDefault,
      });
    }

    const after = useGraphStore.getState().entities["ProviderMeta"]?.["current"] as
      | { default_id?: string | null }
      | undefined;
    expect(after?.default_id).toBe("p1");
  });
});
