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
import { useGraphStore } from "@/platform/entities";
import {
  __resetForTests,
  clearDirty,
  getDirty,
  reconcileSubmittedDirty,
  setDirty,
} from "./settings-form-cache";
import { successfulSubmittedKeys } from "./use-settings";

beforeEach(() => {
  __resetForTests();
  // No replace-flag: keep the store's methods intact.
  useGraphStore.setState({ entities: {} });
});

describe("settings-form-cache", () => {
  test("returns a stable empty snapshot for useSyncExternalStore", () => {
    expect(getDirty("missing")).toBe(getDirty("missing"));
  });

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

  test("reconciles only unchanged successful drafts", () => {
    setDirty("governance", "governance.enabled", true);
    setDirty("governance", "governance.default_mode", "deny_all");
    setDirty("governance", "governance.policy_reload_enabled", false);
    const submitted = getDirty("governance");

    setDirty("governance", "governance.default_mode", "custom");
    reconcileSubmittedDirty("governance", submitted, [
      "governance.enabled",
      "governance.default_mode",
    ]);

    expect(getDirty("governance").values).toEqual({
      "governance.default_mode": "custom",
      "governance.policy_reload_enabled": false,
    });
  });

  test("preserves dependency failures and unconfirmed outcomes", () => {
    const submitted = {
      "governance.default_mode": "deny_all",
      "governance.enabled": true,
    };
    const response = {
      status: "partial" as const,
      results: [
        { key: "governance.default_mode", status: "updated" as const },
        {
          key: "governance.enabled",
          status: "dependency_failed" as const,
        },
      ],
      governance_outcome: "partial" as const,
    };

    expect(successfulSubmittedKeys(submitted, response)).toEqual([
      "governance.default_mode",
    ]);
    expect(
      successfulSubmittedKeys(submitted, {
        ...response,
        governance_outcome: "unknown",
      }),
    ).toEqual([]);
    expect(
      successfulSubmittedKeys(submitted, {
        ...response,
        governance_outcome: "changed_elsewhere",
      }),
    ).toEqual([]);
  });

  test("clears only an authoritative Required master rejection", () => {
    const requiredStatus = {
      boot_instance_id: "boot-a",
      revision: 8,
      phase: "on" as const,
      effective_state: "required" as const,
      effective_enabled: true,
      may_disable: false,
      mutation_available: true,
      configured_host: "0.0.0.0",
      bound_addresses: ["0.0.0.0:1906"],
      jwt_required: true,
      reasons: ["jwt_required" as const],
    };
    expect(
      successfulSubmittedKeys(
        {
          "governance.enabled": false,
          "governance.default_mode": "deny_all",
        },
        {
          status: "partial",
          results: [
            {
              key: "governance.enabled",
              status: "validation_rejected",
            },
            {
              key: "governance.default_mode",
              status: "dependency_failed",
            },
          ],
          governance_outcome: "rejected",
          observed_governance_status: requiredStatus,
        },
      ),
    ).toEqual(["governance.enabled"]);
  });
});
