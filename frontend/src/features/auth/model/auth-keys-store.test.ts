import { beforeEach, describe, expect, test, vi } from "vitest";

import * as authApi from "../api/auth-api";
import { useAuthKeysStore } from "./auth-keys-store";

vi.mock("../api/auth-api", () => ({
  createAuthKey: vi.fn(),
  deleteAuthKey: vi.fn(),
  fetchAuthKeys: vi.fn(),
}));
beforeEach(() => {
  vi.resetAllMocks();
  useAuthKeysStore.setState({ keys: [], loading: false, saving: false, revoking: false, error: null });
});

describe("auth key store", () => {
  test("creates a key and reconciles the authoritative list", async () => {
    vi.mocked(authApi.createAuthKey).mockResolvedValue({ raw_key: "uar_secret" });
    vi.mocked(authApi.fetchAuthKeys).mockResolvedValue([{ id: "key-1", name: "release" }]);

    await expect(useAuthKeysStore.getState().createKey("release")).resolves.toEqual({
      raw_key: "uar_secret",
    });
    expect(useAuthKeysStore.getState().keys).toEqual([{ id: "key-1", name: "release" }]);
  });

  test("retains a key and surfaces authorization failure when revoke is denied", async () => {
    useAuthKeysStore.setState({ keys: [{ id: "key-1", name: "release" }] });
    vi.mocked(authApi.deleteAuthKey).mockRejectedValue(new Error("403"));

    await useAuthKeysStore.getState().revokeKey("key-1");
    expect(useAuthKeysStore.getState().keys).toHaveLength(1);
    expect(useAuthKeysStore.getState().error).toBe("403");
  });
});
