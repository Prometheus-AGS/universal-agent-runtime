import { beforeEach, describe, expect, test, vi } from "vitest";

import * as credentialsApi from "../api/credentials-api";
import { useCredentialsStore } from "./credentials-store";

vi.mock("../api/credentials-api", () => ({
  deleteCredential: vi.fn(),
  listCredentials: vi.fn(),
  putCredential: vi.fn(),
}));

beforeEach(() => {
  vi.resetAllMocks();
  useCredentialsStore.setState({
    state: "ok",
    credentials: [],
    loading: false,
    saving: false,
    removing: false,
    error: null,
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
