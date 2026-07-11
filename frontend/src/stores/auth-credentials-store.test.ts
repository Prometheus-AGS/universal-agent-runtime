import { beforeEach, describe, expect, test, vi } from "vitest";

import * as authApi from "@/services/auth-api";
import * as credentialsApi from "@/services/credentials-api";
import { useAuthKeysStore } from "@/stores/auth-keys-store";
import { useCredentialsStore } from "@/stores/credentials-store";

vi.mock("@/services/auth-api", () => ({
  createAuthKey: vi.fn(),
  deleteAuthKey: vi.fn(),
  fetchAuthKeys: vi.fn(),
}));
vi.mock("@/services/credentials-api", () => ({
  deleteCredential: vi.fn(),
  listCredentials: vi.fn(),
  putCredential: vi.fn(),
}));

beforeEach(() => {
  vi.resetAllMocks();
  useAuthKeysStore.setState({ keys: [], loading: false, saving: false, revoking: false, error: null });
  useCredentialsStore.setState({
    state: "ok",
    credentials: [],
    loading: false,
    saving: false,
    removing: false,
    error: null,
  });
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

describe("credentials store", () => {
  test("distinguishes disabled storage and sorts active credentials", async () => {
    vi.mocked(credentialsApi.listCredentials).mockResolvedValueOnce({
      state: "disabled",
      credentials: [],
    });
    await useCredentialsStore.getState().load();
    expect(useCredentialsStore.getState().state).toBe("disabled");

    vi.mocked(credentialsApi.listCredentials).mockResolvedValueOnce({
      state: "ok",
      credentials: [
        { provider_id: "openai", api_key_hint: "1234", created_at: "", updated_at: "" },
        { provider_id: "anthropic", api_key_hint: "5678", created_at: "", updated_at: "" },
      ],
    });
    await useCredentialsStore.getState().load();
    expect(useCredentialsStore.getState().credentials.map((item) => item.provider_id)).toEqual([
      "anthropic",
      "openai",
    ]);
  });

  test("does not remove credential metadata when deletion fails", async () => {
    useCredentialsStore.setState({
      credentials: [{ provider_id: "openai", api_key_hint: "1234", created_at: "", updated_at: "" }],
    });
    vi.mocked(credentialsApi.deleteCredential).mockRejectedValue(new Error("401 unauthorized"));

    await expect(useCredentialsStore.getState().remove("openai")).resolves.toBe(false);
    expect(useCredentialsStore.getState().credentials).toHaveLength(1);
    expect(useCredentialsStore.getState().error).toBe("401 unauthorized");
  });
});
