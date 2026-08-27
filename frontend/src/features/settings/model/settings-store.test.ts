import { beforeEach, describe, expect, test, vi } from "vitest";
import { useGraphStore } from "@/platform/entities";
import {
  fetchGovernanceStatus,
  putSettingsNamespace,
} from "../api/settings-api";
import {
  __resetGovernanceStatusForTests,
  governanceStatusSnapshot,
  ingestGovernanceStatus,
  nextGovernanceRequestSequence,
} from "./governance-status";
import { useSettingsStore } from "./settings-store";

vi.mock("../api/settings-api", () => ({
  fetchGovernanceStatus: vi.fn(),
  fetchSettingsNamespace: vi.fn(),
  putSettingsNamespace: vi.fn(),
}));

beforeEach(() => {
  vi.clearAllMocks();
  __resetGovernanceStatusForTests();
  useGraphStore.setState({ entities: {} });
  useSettingsStore.setState({ statusByNamespace: {} });
});

describe("settings store governance mutation", () => {
  test("keeps successful keys and rolls back failed keys independently", async () => {
    const graph = useGraphStore.getState();
    graph.upsertEntity("Setting", "governance:governance.enabled", {
      id: "governance:governance.enabled",
      namespace: "governance",
      key: "governance.enabled",
      data: false,
    });
    graph.upsertEntity("Setting", "governance:governance.default_mode", {
      id: "governance:governance.default_mode",
      namespace: "governance",
      key: "governance.default_mode",
      data: "permit_all",
    });
    vi.mocked(putSettingsNamespace).mockResolvedValue({
      status: "partial",
      results: [
        { key: "governance.default_mode", status: "updated" },
        { key: "governance.enabled", status: "dependency_failed" },
      ],
      applied_status: { boot_instance_id: "boot-a", revision: 5 },
      governance_status: {
        boot_instance_id: "boot-a",
        revision: 5,
        phase: "off",
        effective_state: "off",
        effective_enabled: false,
        may_disable: true,
        mutation_available: true,
        configured_host: "localhost",
        bound_addresses: ["127.0.0.1:1906"],
        jwt_required: false,
        reasons: [],
      },
    });

    const response = await useSettingsStore.getState().save("governance", {
      "governance.enabled": true,
      "governance.default_mode": "deny_all",
    });

    expect(response.status).toBe("partial");
    expect(response.governance_outcome).toBe("partial");
    expect(
      useGraphStore.getState().entities.Setting?.[
        "governance:governance.enabled"
      ]?.data,
    ).toBe(false);
    expect(
      useGraphStore.getState().entities.Setting?.[
        "governance:governance.default_mode"
      ]?.data,
    ).toBe("deny_all");
    expect(governanceStatusSnapshot()?.revision).toBe(5);
  });

  test("reports changed elsewhere when a newer revision wins the race", async () => {
    let resolveMutation!: (value: Awaited<ReturnType<typeof putSettingsNamespace>>) => void;
    vi.mocked(putSettingsNamespace).mockReturnValue(
      new Promise((resolve) => {
        resolveMutation = resolve;
      }),
    );

    const save = useSettingsStore.getState().save("governance", {
      "governance.enabled": true,
    });
    ingestGovernanceStatus(
      {
        boot_instance_id: "boot-a",
        revision: 6,
        phase: "off",
        effective_state: "off",
        effective_enabled: false,
        may_disable: true,
        mutation_available: true,
        configured_host: "localhost",
        bound_addresses: ["127.0.0.1:1906"],
        jwt_required: false,
        reasons: [],
      },
      nextGovernanceRequestSequence(),
    );
    resolveMutation({
      status: "updated",
      results: [{ key: "governance.enabled", status: "updated" }],
      applied_status: { boot_instance_id: "boot-a", revision: 5 },
      governance_status: {
        boot_instance_id: "boot-a",
        revision: 5,
        phase: "on",
        effective_state: "on",
        effective_enabled: true,
        may_disable: true,
        mutation_available: true,
        configured_host: "localhost",
        bound_addresses: ["127.0.0.1:1906"],
        jwt_required: false,
        reasons: [],
      },
    });

    const response = await save;
    expect(response.governance_outcome).toBe("changed_elsewhere");
    expect(response.observed_governance_status?.revision).toBe(6);
  });

  test("confirms a restarted runtime before completing the save", async () => {
    ingestGovernanceStatus(
      {
        boot_instance_id: "boot-a",
        revision: 9,
        phase: "off",
        effective_state: "off",
        effective_enabled: false,
        may_disable: true,
        mutation_available: true,
        configured_host: "localhost",
        bound_addresses: ["127.0.0.1:1906"],
        jwt_required: false,
        reasons: [],
      },
      nextGovernanceRequestSequence(),
    );
    const restartedStatus = {
      boot_instance_id: "boot-b",
      revision: 1,
      phase: "on" as const,
      effective_state: "on" as const,
      effective_enabled: true,
      may_disable: true,
      mutation_available: true,
      configured_host: "localhost",
      bound_addresses: ["127.0.0.1:1906"],
      jwt_required: false,
      reasons: [],
    };
    vi.mocked(putSettingsNamespace).mockResolvedValue({
      status: "updated",
      results: [{ key: "governance.enabled", status: "updated" }],
      applied_status: { boot_instance_id: "boot-b", revision: 1 },
      governance_status: restartedStatus,
    });
    vi.mocked(fetchGovernanceStatus).mockResolvedValue(restartedStatus);

    const response = await useSettingsStore.getState().save("governance", {
      "governance.enabled": true,
    });

    expect(fetchGovernanceStatus).toHaveBeenCalledTimes(1);
    expect(response.governance_outcome).toBe("confirmed");
    expect(governanceStatusSnapshot()?.boot_instance_id).toBe("boot-b");
  });

  test("terminates as Unknown when a restarted runtime cannot be confirmed", async () => {
    const graph = useGraphStore.getState();
    graph.upsertEntity("Setting", "governance:governance.enabled", {
      id: "governance:governance.enabled",
      namespace: "governance",
      key: "governance.enabled",
      data: false,
    });
    ingestGovernanceStatus(
      {
        boot_instance_id: "boot-a",
        revision: 9,
        phase: "off",
        effective_state: "off",
        effective_enabled: false,
        may_disable: true,
        mutation_available: true,
        configured_host: "localhost",
        bound_addresses: ["127.0.0.1:1906"],
        jwt_required: false,
        reasons: [],
      },
      nextGovernanceRequestSequence(),
    );
    const restartedStatus = {
      boot_instance_id: "boot-b",
      revision: 1,
      phase: "on" as const,
      effective_state: "on" as const,
      effective_enabled: true,
      may_disable: true,
      mutation_available: true,
      configured_host: "localhost",
      bound_addresses: ["127.0.0.1:1906"],
      jwt_required: false,
      reasons: [],
    };
    vi.mocked(putSettingsNamespace).mockResolvedValue({
      status: "updated",
      results: [{ key: "governance.enabled", status: "updated" }],
      applied_status: { boot_instance_id: "boot-b", revision: 1 },
      governance_status: restartedStatus,
    });
    vi.mocked(fetchGovernanceStatus).mockRejectedValue(
      new Error("Governance request timed out after 10 seconds"),
    );

    const response = await useSettingsStore.getState().save("governance", {
      "governance.enabled": true,
    });

    expect(response.governance_outcome).toBe("unknown");
    expect(governanceStatusSnapshot()).toBeNull();
    expect(
      useGraphStore.getState().entities.Setting?.[
        "governance:governance.enabled"
      ]?.data,
    ).toBe(false);
    expect(
      useSettingsStore.getState().statusByNamespace.governance?.saving,
    ).toBe(false);
  });
});
