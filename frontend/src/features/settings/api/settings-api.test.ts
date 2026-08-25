import { afterEach, describe, expect, test, vi } from "vitest";
import { fetchSettingsNamespace } from "./settings-api";

describe("fetchSettingsNamespace", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test.each([
    ["provider", "/api/uar/settings/providers"],
    ["context_management", "/api/uar/settings/context-management"],
    ["server", "/api/uar/settings/server"],
  ])("requests the canonical route for %s", async (namespace, route) => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response("[]", {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );
    vi.stubGlobal("fetch", fetchMock);

    await expect(fetchSettingsNamespace(namespace)).resolves.toEqual([]);

    expect(fetchMock).toHaveBeenCalledOnce();
    expect(fetchMock).toHaveBeenCalledWith(route);
  });

  test("preserves non-success response propagation", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(new Response(null, { status: 404 })),
    );

    await expect(fetchSettingsNamespace("provider")).rejects.toThrow("404");
  });
});
