import { useGraphStore } from "@/platform/entities";
import { beforeEach, describe, expect, test, vi } from "vitest";
import * as compilerApi from "../api/compiler-api";
import { useCompilerStore } from "./compiler-store";

vi.mock("../api/compiler-api", () => ({ createCompilerSession: vi.fn(), fetchCompilerSessions: vi.fn() }));

beforeEach(() => {
  vi.resetAllMocks();
  useGraphStore.setState({ entities: {} } as never);
  useCompilerStore.setState({ loading: false, creating: false, error: null });
});

describe("compiler store", () => {
  test("creates an experimental session and reconciles it", async () => {
    vi.mocked(compilerApi.fetchCompilerSessions).mockResolvedValue({ sessions: [{ id: "c1", status: "running" }] });
    await useCompilerStore.getState().createSession();
    expect(compilerApi.createCompilerSession).toHaveBeenCalledOnce();
    expect(useGraphStore.getState().entities.CompilerSession?.c1).toMatchObject({ status: "running" });
  });

  test("surfaces compiler failures", async () => {
    vi.mocked(compilerApi.fetchCompilerSessions).mockRejectedValue(new Error("compiler unavailable"));
    await expect(useCompilerStore.getState().load()).rejects.toThrow("compiler unavailable");
    expect(useCompilerStore.getState().error).toBe("compiler unavailable");
  });
});
