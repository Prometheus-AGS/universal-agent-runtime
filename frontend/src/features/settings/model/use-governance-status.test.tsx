import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { useGraphStore } from "@/platform/entities";
import {
  fetchGovernanceStatus,
  type GovernanceRuntimeStatus,
} from "../api/settings-api";
import { emitSettingsRealtimeConnected } from "../api/settings-change-bus";
import {
  __resetGovernanceStatusForTests,
  ingestGovernanceStatus,
  nextGovernanceRequestSequence,
} from "./governance-status";
import { useGovernanceStatus } from "./use-governance-status";

vi.mock("../api/settings-api", async (importOriginal) => {
  const original = await importOriginal<typeof import("../api/settings-api")>();
  return { ...original, fetchGovernanceStatus: vi.fn() };
});

const STATUS: GovernanceRuntimeStatus = {
  boot_instance_id: "boot-a",
  revision: 9,
  phase: "on",
  effective_state: "on",
  effective_enabled: true,
  may_disable: true,
  mutation_available: true,
  configured_host: "localhost",
  bound_addresses: ["127.0.0.1:1906"],
  jwt_required: false,
  reasons: [],
};

beforeEach(() => {
  vi.clearAllMocks();
  __resetGovernanceStatusForTests();
  useGraphStore.setState({ entities: {} });
});

describe("useGovernanceStatus", () => {
  test("confirms a newly adopted boot instance with one follow-up request", async () => {
    ingestGovernanceStatus(STATUS, 1);
    vi.mocked(fetchGovernanceStatus)
      .mockResolvedValueOnce({
        ...STATUS,
        boot_instance_id: "boot-b",
        revision: 1,
      })
      .mockResolvedValueOnce({
        ...STATUS,
        boot_instance_id: "boot-b",
        revision: 2,
      });

    const { result } = renderHook(() => useGovernanceStatus());

    await waitFor(() => {
      expect(result.current.status?.boot_instance_id).toBe("boot-b");
      expect(result.current.status?.revision).toBe(2);
      expect(result.current.loading).toBe(false);
    });
    expect(fetchGovernanceStatus).toHaveBeenCalledTimes(2);
  });

  test("invalidates stale status when revalidation fails", async () => {
    ingestGovernanceStatus(STATUS, 1);
    vi.mocked(fetchGovernanceStatus).mockRejectedValue(
      new Error("Governance status unavailable: 404"),
    );

    const { result } = renderHook(() => useGovernanceStatus());

    await waitFor(() => {
      expect(result.current.status).toBeNull();
      expect(result.current.error).toBe("Governance status unavailable: 404");
      expect(result.current.loading).toBe(false);
    });
  });

  test("becomes Unknown when a new boot cannot be confirmed", async () => {
    ingestGovernanceStatus(STATUS, 1);
    vi.mocked(fetchGovernanceStatus)
      .mockResolvedValueOnce({
        ...STATUS,
        boot_instance_id: "boot-b",
        revision: 1,
      })
      .mockRejectedValueOnce(new Error("Governance request timed out after 10 seconds"));

    const { result } = renderHook(() => useGovernanceStatus());

    await waitFor(() => {
      expect(result.current.status).toBeNull();
      expect(result.current.error).toBe(
        "Governance request timed out after 10 seconds",
      );
      expect(result.current.loading).toBe(false);
    });
    expect(fetchGovernanceStatus).toHaveBeenCalledTimes(2);
  });

  test("rejects a confirmation from the retired runtime", async () => {
    ingestGovernanceStatus(STATUS, 1);
    vi.mocked(fetchGovernanceStatus)
      .mockResolvedValueOnce({
        ...STATUS,
        boot_instance_id: "boot-b",
        revision: 1,
      })
      .mockResolvedValueOnce({ ...STATUS, revision: 10 });

    const { result } = renderHook(() => useGovernanceStatus());

    await waitFor(() => {
      expect(result.current.status).toBeNull();
      expect(result.current.error).toBe(
        "Governance restart confirmation did not match the adopted runtime",
      );
    });
  });

  test("revalidates after the realtime channel reconnects", async () => {
    vi.mocked(fetchGovernanceStatus).mockResolvedValue(STATUS);
    const { result } = renderHook(() => useGovernanceStatus());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(fetchGovernanceStatus).toHaveBeenCalledTimes(1);

    emitSettingsRealtimeConnected();

    await waitFor(() =>
      expect(fetchGovernanceStatus).toHaveBeenCalledTimes(2),
    );
  });

  test("does not surface a stale failure over a newer accepted status", async () => {
    let rejectRequest!: (reason: Error) => void;
    vi.mocked(fetchGovernanceStatus).mockReturnValue(
      new Promise((_resolve, reject) => {
        rejectRequest = reject;
      }),
    );
    const { result } = renderHook(() => useGovernanceStatus());
    await waitFor(() => expect(result.current.loading).toBe(true));

    act(() => {
      ingestGovernanceStatus(
        { ...STATUS, revision: 10 },
        nextGovernanceRequestSequence(),
      );
    });
    rejectRequest(new Error("stale request failed"));

    await waitFor(() => {
      expect(result.current.status?.revision).toBe(10);
      expect(result.current.error).toBeNull();
      expect(result.current.loading).toBe(false);
    });
  });
});
