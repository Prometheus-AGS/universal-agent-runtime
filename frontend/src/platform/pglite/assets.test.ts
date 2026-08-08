import { afterEach, describe, expect, test, vi } from "vitest";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

describe("PGlite schema seed selection", () => {
  test("does not fetch or load the seed for an existing UAR database", async () => {
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);
    vi.stubGlobal("indexedDB", {
      databases: vi.fn().mockResolvedValue([{ name: "/pglite/uar-threads" }]),
    });

    const { loadPgliteSeedForFreshDatabase } = await import("./assets");

    await expect(loadPgliteSeedForFreshDatabase()).resolves.toBeUndefined();
    expect(fetchMock).not.toHaveBeenCalled();
  });

  test("loads the checked seed when no UAR database exists", async () => {
    const seed = new Blob(["seed"]);
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      blob: vi.fn().mockResolvedValue(seed),
    });
    vi.stubGlobal("fetch", fetchMock);
    vi.stubGlobal("indexedDB", {
      databases: vi.fn().mockResolvedValue([]),
    });

    const { loadPgliteSeedForFreshDatabase } = await import("./assets");

    await expect(loadPgliteSeedForFreshDatabase()).resolves.toBe(seed);
    expect(fetchMock).toHaveBeenCalledOnce();
  });
});
