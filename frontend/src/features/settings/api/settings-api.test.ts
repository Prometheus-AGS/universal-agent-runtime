import { afterEach, describe, expect, test, vi } from "vitest";

import { fetchSettingsNamespace, putSettingsNamespace } from "./settings-api";

afterEach(() => {
  vi.unstubAllEnvs();
  vi.unstubAllGlobals();
});

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
});
