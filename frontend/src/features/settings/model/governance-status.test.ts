import { beforeEach, describe, expect, test } from "vitest";
import { useGraphStore } from "@/platform/entities";
import type { GovernanceRuntimeStatus } from "../api/settings-api";
import {
  __resetGovernanceStatusForTests,
  governanceStatusSnapshot,
  ingestGovernanceStatus,
  invalidateGovernanceStatus,
  nextGovernanceRequestSequence,
} from "./governance-status";

const STATUS: GovernanceRuntimeStatus = {
  boot_instance_id: "boot-a",
  revision: 4,
  phase: "off",
  effective_state: "off",
  effective_enabled: false,
  may_disable: true,
  mutation_available: true,
  configured_host: "localhost",
  bound_addresses: ["127.0.0.1:1906"],
  jwt_required: false,
  reasons: [],
};

beforeEach(() => {
  __resetGovernanceStatusForTests();
  useGraphStore.setState({ entities: {} });
});

describe("governance status entity acceptance", () => {
  test("rejects stale same-process revisions and request sequences", () => {
    expect(ingestGovernanceStatus(STATUS, 2).accepted).toBe(true);
    expect(
      ingestGovernanceStatus({ ...STATUS, revision: 3 }, 3).accepted,
    ).toBe(false);
    expect(
      ingestGovernanceStatus({ ...STATUS, revision: 5 }, 1).accepted,
    ).toBe(false);
    expect(governanceStatusSnapshot()?.revision).toBe(4);
  });

  test("adopts a newer-request process restart and retires the old process", () => {
    ingestGovernanceStatus(STATUS, 1);
    const restarted = ingestGovernanceStatus(
      { ...STATUS, boot_instance_id: "boot-b", revision: 1 },
      2,
    );
    expect(restarted).toEqual({ accepted: true, restarted: true });
    expect(governanceStatusSnapshot()?.boot_instance_id).toBe("boot-b");
    expect(ingestGovernanceStatus({ ...STATUS, revision: 99 }, 3).accepted).toBe(
      false,
    );
  });

  test("allocates monotonically increasing request sequences", () => {
    expect(nextGovernanceRequestSequence()).toBe(1);
    expect(nextGovernanceRequestSequence()).toBe(2);
  });

  test("a stale failure cannot invalidate a newer accepted status", () => {
    ingestGovernanceStatus(STATUS, 2);

    expect(invalidateGovernanceStatus(1)).toBe(false);
    expect(governanceStatusSnapshot()?.revision).toBe(4);
    expect(invalidateGovernanceStatus(2)).toBe(true);
    expect(governanceStatusSnapshot()).toBeNull();
  });
});
