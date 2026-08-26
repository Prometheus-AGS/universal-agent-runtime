import { afterEach, describe, expect, test, vi } from "vitest";

import { putUserSettings } from "./user-settings-api";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("user settings API", () => {
  test("serializes an explicit null to clear the prompt-caching override", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          user_id: "tenant:user",
          prompt_caching_enabled: null,
          preferred_scope: "session",
          updated_at: "2026-08-25T00:01:00Z",
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    );
    vi.stubGlobal("fetch", fetchMock);

    await putUserSettings(
      { Authorization: "Bearer ey.test.jwt" },
      { prompt_caching_enabled: null },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/uar/user/settings",
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ prompt_caching_enabled: null }),
      }),
    );
  });
});
