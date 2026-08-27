import { afterEach, describe, expect, test, vi } from "vitest";

import {
  GOVERNANCE_REQUEST_TIMEOUT_MS,
  fetchGovernanceStatus,
  fetchSettingsNamespace,
  parseGovernanceRuntimeStatus,
  putSettingsNamespace,
} from "./settings-api";

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllEnvs();
  vi.unstubAllGlobals();
});

const GOVERNANCE_STATUS = {
  boot_instance_id: "boot-a",
  revision: 7,
  phase: "off",
  effective_state: "off",
  effective_enabled: false,
  may_disable: true,
  mutation_available: true,
  configured_host: "localhost",
  bound_addresses: ["127.0.0.1:1906"],
  jwt_required: false,
  reasons: [],
} as const;

describe("settings admin API", () => {
  test("sends the configured admin intent header for protected reads and writes", async () => {
    vi.stubEnv("VITE_UAR_ADMIN_KEY", "configured-admin-key");
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify([]), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ status: "updated", updated: [] }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );
    vi.stubGlobal("fetch", fetchMock);

    await fetchSettingsNamespace("prompt_caching");
    await putSettingsNamespace("prompt_caching", { enabled: true });

    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      "/api/uar/settings/prompt-caching",
      expect.objectContaining({
        headers: { "X-UAR-Admin-Key": "configured-admin-key" },
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      "/api/uar/settings/prompt-caching",
      expect.objectContaining({
        method: "PUT",
        headers: expect.objectContaining({
          "X-UAR-Admin-Key": "configured-admin-key",
        }),
      }),
    );
  });

  test("fetches and validates the authoritative governance status", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify(GOVERNANCE_STATUS), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      ),
    );

    await expect(fetchGovernanceStatus()).resolves.toEqual(GOVERNANCE_STATUS);
  });

  test("rejects missing and malformed governance status projections", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response("missing", { status: 404 }))
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            ...GOVERNANCE_STATUS,
            phase: "off",
            effective_state: "on",
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      );
    vi.stubGlobal("fetch", fetchMock);

    await expect(fetchGovernanceStatus()).rejects.toThrow(
      "Governance status unavailable: 404",
    );
    await expect(fetchGovernanceStatus()).rejects.toThrow(
      "Malformed governance status projection",
    );
    expect(() =>
      parseGovernanceRuntimeStatus({
        ...GOVERNANCE_STATUS,
        reasons: ["future_unknown_reason"],
      }),
    ).toThrow("Malformed governance status projection");
  });

  test("preserves On or Required while durable mutation is unavailable", () => {
    expect(
      parseGovernanceRuntimeStatus({
        ...GOVERNANCE_STATUS,
        phase: "on",
        effective_state: "on",
        effective_enabled: true,
        mutation_available: false,
        reasons: ["persistence_unavailable"],
      }),
    ).toMatchObject({ effective_state: "on", mutation_available: false });
    expect(
      parseGovernanceRuntimeStatus({
        ...GOVERNANCE_STATUS,
        phase: "on",
        effective_state: "required",
        effective_enabled: true,
        may_disable: false,
        mutation_available: false,
        reasons: ["jwt_required", "persistence_unavailable"],
      }),
    ).toMatchObject({ effective_state: "required", mutation_available: false });
  });

  test("rejects Required without a mandatory reason or with contradictory mutability", () => {
    expect(() =>
      parseGovernanceRuntimeStatus({
        ...GOVERNANCE_STATUS,
        phase: "on",
        effective_state: "required",
        effective_enabled: true,
        may_disable: false,
        mutation_available: false,
        reasons: ["persistence_unavailable"],
      }),
    ).toThrow("Malformed governance status projection");
    expect(() =>
      parseGovernanceRuntimeStatus({
        ...GOVERNANCE_STATUS,
        phase: "on",
        effective_state: "required",
        effective_enabled: true,
        may_disable: false,
        mutation_available: true,
        reasons: ["jwt_required", "persistence_unavailable"],
      }),
    ).toThrow("Malformed governance status projection");
  });

  test("aborts governance status confirmation after ten seconds", async () => {
    vi.useFakeTimers();
    vi.stubGlobal(
      "fetch",
      vi.fn().mockImplementation(
        (_input: RequestInfo | URL, init?: RequestInit) =>
          new Promise<Response>((_resolve, reject) => {
            init?.signal?.addEventListener("abort", () => {
              reject(new DOMException("Aborted", "AbortError"));
            });
          }),
      ),
    );

    const request = fetchGovernanceStatus();
    const rejection = expect(request).rejects.toThrow(
      "Governance request timed out after 10 seconds",
    );
    await vi.advanceTimersByTimeAsync(GOVERNANCE_REQUEST_TIMEOUT_MS);
    await rejection;
  });

  test("returns per-key governance results with an applied status token", async () => {
    const response = {
      status: "partial",
      results: [
        { key: "governance.default_mode", status: "updated" },
        {
          key: "governance.enabled",
          status: "dependency_failed",
          error: "policy prerequisite failed",
        },
      ],
      applied_status: { boot_instance_id: "boot-a", revision: 7 },
      governance_status: GOVERNANCE_STATUS,
    };
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify(response), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      ),
    );

    await expect(
      putSettingsNamespace("governance", {
        default_mode: "deny_all",
        enabled: true,
      }),
    ).resolves.toEqual(response);
  });

  test("rejects incomplete or incoherent governance mutation confirmations", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ status: "updated", results: [] }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            status: "updated",
            results: [],
            applied_status: { boot_instance_id: "boot-a", revision: 8 },
            governance_status: GOVERNANCE_STATUS,
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            status: "updated",
            results: [
              { key: "governance.enabled", status: "updated" },
              { key: "governance.enabled", status: "updated" },
            ],
            applied_status: { boot_instance_id: "boot-a", revision: 7 },
            governance_status: GOVERNANCE_STATUS,
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      );
    vi.stubGlobal("fetch", fetchMock);

    await expect(
      putSettingsNamespace("governance", { enabled: false }),
    ).rejects.toThrow("Malformed governance mutation response");
    await expect(
      putSettingsNamespace("governance", { enabled: false }),
    ).rejects.toThrow("Malformed governance mutation response");
    await expect(
      putSettingsNamespace("governance", { enabled: false }),
    ).rejects.toThrow("Malformed governance mutation response");
  });
});
