import { describe, expect, test } from "vitest";

import { deriveReadiness, gateReason, type ReadinessInputs } from "./readiness";

const READY: ReadinessInputs = {
  workspace: "default",
  ownerId: "owner-1",
  capabilitiesStatus: "ready",
  capabilitiesError: null,
  instanceId: "uar-1",
  teamExecutionAvailable: true,
  ownershipStatus: "ready",
  ownershipError: null,
  ownsExecution: true,
  claimEpoch: 7,
  claimAcquiredAt: "2026-11-18T00:00:00Z",
};

const states = (input: ReadinessInputs) => deriveReadiness(input).map((entry) => `${entry.id}:${entry.state}`);

describe("deriveReadiness", () => {
  test("is all ok for a configured runtime that owns execution", () => {
    expect(states(READY)).toEqual(["identity:ok", "workspace:ok", "instance:ok", "ownership:ok"]);
  });

  test("on 401 identity is blocked and every other fact is unknown", () => {
    const unauthorized = { ...READY, ownerId: null, capabilitiesStatus: "error" as const, capabilitiesError: { kind: "unauthorized", message: "x" } };
    expect(states(unauthorized)).toEqual(["identity:blocked", "workspace:unknown", "instance:unknown", "ownership:unknown"]);
    expect(deriveReadiness(unauthorized)[0].summary).toContain("identity host");
  });

  test("an empty instance id blocks bindings and names the reason", () => {
    const facts = deriveReadiness({ ...READY, instanceId: "" });
    expect(facts[2].state).toBe("blocked");
    expect(gateReason(facts, "bind")).toContain("instance id");
    expect(gateReason(facts, "launch")).toContain("instance id");
    expect(gateReason(facts, "install")).toBeNull();
  });

  test("another holder blocks binding and routing but not launch, and offers no reclaim", () => {
    // The server forces teamExecution to false whenever it does not own execution.
    const facts = deriveReadiness({ ...READY, teamExecutionAvailable: false, ownsExecution: false, claimEpoch: 6 });
    expect(facts[3].state).toBe("blocked");
    expect(facts[3].summary).toContain("epoch 6");
    expect(facts[3].summary).toContain("supervisor-recovery.md");
    expect(gateReason(facts, "bind")).not.toBeNull();
    expect(gateReason(facts, "routing")).not.toBeNull();
    expect(gateReason(facts, "launch")).toBeNull();
  });

  test("before the first answers everything is being checked", () => {
    const checking = { ...READY, ownerId: null, capabilitiesStatus: "loading" as const, instanceId: null, ownsExecution: null };
    expect(states(checking)).toEqual(["identity:checking", "workspace:checking", "instance:checking", "ownership:checking"]);
  });
});
