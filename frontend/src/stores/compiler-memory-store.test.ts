import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { beforeEach, describe, expect, test, vi } from "vitest";
import * as compilerApi from "@/services/compiler-api";
import * as memoryApi from "@/services/memory-api";
import { useCompilerStore } from "@/stores/compiler-store";
import { useMemoryAdminStore } from "@/stores/memory-admin-store";

vi.mock("@/services/compiler-api", () => ({ createCompilerSession: vi.fn(), fetchCompilerSessions: vi.fn() }));
vi.mock("@/services/memory-api", () => ({
  bulkDeleteMemoriesApi: vi.fn(), deleteMemoryApi: vi.fn(), fetchMemoriesList: vi.fn(), fetchMemoryStats: vi.fn(),
}));

const memory = { id: "m1", content: "release", categories: [], scope: "Agent", memory_type: "fact", importance: 0.8, created_at: "2026-07-11T00:00:00Z" };

beforeEach(() => {
  vi.resetAllMocks();
  useGraphStore.setState({ entities: {} } as never);
  useCompilerStore.setState({ loading: false, creating: false, error: null });
  useMemoryAdminStore.setState({ loading: false, deleting: false, error: null });
  vi.mocked(memoryApi.fetchMemoryStats).mockResolvedValue({ total: 1, by_scope: { Agent: 1 } });
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

describe("memory admin store", () => {
  test("loads filtered memory and stats", async () => {
    vi.mocked(memoryApi.fetchMemoriesList).mockResolvedValue({ total: 1, items: [memory] });
    await useMemoryAdminStore.getState().load({ userId: "", agentId: "a1", searchQ: "", searchMode: false });
    expect(useGraphStore.getState().entities.Memory?.m1).toMatchObject(memory);
    expect(useGraphStore.getState().entities.MemoryMeta?.current).toMatchObject({ total: 1 });
  });

  test("retains memory when deletion is denied", async () => {
    useGraphStore.getState().upsertEntity("Memory", memory.id, memory);
    vi.mocked(memoryApi.deleteMemoryApi).mockRejectedValue(new Error("403 delete denied"));
    await expect(useMemoryAdminStore.getState().remove(memory)).rejects.toThrow("403 delete denied");
    expect(useGraphStore.getState().entities.Memory?.m1).toMatchObject(memory);
  });
});
