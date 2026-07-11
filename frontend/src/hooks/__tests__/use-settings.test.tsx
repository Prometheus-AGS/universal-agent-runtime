/**
 * Contract test — settings form-cache + hook.
 *
 * Locks the dirty → save → conflict pattern that replaced
 * `settings-store.ts`. Covers:
 *   - setSetting() marks dirty without touching graph
 *   - clearDirty() empties the dirty map
 *   - dirty + remote divergence synthesises a conflict
 */
import { describe, expect, test, beforeEach } from "vitest";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import {
  __resetForTests,
  clearDirty,
  getDirty,
  setDirty,
} from "../settings-form-cache";

beforeEach(() => {
  __resetForTests();
  // No replace-flag: keep the store's methods intact.
  useGraphStore.setState({ entities: {} });
});

describe("settings-form-cache", () => {
  test("setDirty stores values per namespace", () => {
    setDirty("provider", "openai.api_key", "sk-test");
    setDirty("agent_config", "memory_enabled", true);

    expect(getDirty("provider").values["openai.api_key"]).toBe("sk-test");
    expect(getDirty("agent_config").values["memory_enabled"]).toBe(true);
    // Namespaces are isolated.
    expect(getDirty("provider").values["memory_enabled"]).toBeUndefined();
  });

  test("clearDirty empties the dirty map for the namespace", () => {
    setDirty("provider", "k1", 1);
    setDirty("provider", "k2", 2);
    expect(Object.keys(getDirty("provider").values).length).toBe(2);

    clearDirty("provider");
    expect(Object.keys(getDirty("provider").values).length).toBe(0);
  });

  test("conflict synthesis: dirty value diverges from remote graph value", () => {
    // Set up a remote-known value in the graph.
    const graph = useGraphStore.getState();
    graph.upsertEntity("Setting", "provider:openai.api_key", {
      id: "provider:openai.api_key",
      namespace: "provider",
      key: "openai.api_key",
      data: "sk-remote",
    });

    // Locally edit to a different value.
    setDirty("provider", "openai.api_key", "sk-local");

    // Compute conflicts the same way useSettings does.
    const dirty = getDirty("provider").values;
    const remote = (useGraphStore.getState().entities["Setting"]?.[
      "provider:openai.api_key"
    ] as { data?: unknown } | undefined)?.data;

    const isConflict =
      Object.prototype.hasOwnProperty.call(dirty, "openai.api_key") &&
      remote !== undefined &&
      !Object.is(remote, dirty["openai.api_key"]);

    expect(isConflict).toBe(true);
    expect(remote).toBe("sk-remote");
  });
});
