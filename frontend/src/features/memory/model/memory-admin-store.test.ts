import { useGraphStore } from "@/platform/entities";
import { beforeEach, describe, expect, test, vi } from "vitest";
import * as memoryApi from "../api/memory-api";
import { useMemoryAdminStore } from "./memory-admin-store";

vi.mock("../api/memory-api", () => ({
  bulkDeleteMemoriesApi: vi.fn(),
  deleteMemoryApi: vi.fn(),
  fetchMemoriesList: vi.fn(),
  fetchMemoryStats: vi.fn(),
}));

const memory = {
  id: "m1",
  content: "release",
  categories: [],
  scope: "Agent",
  memory_type: "fact",
  importance: 0.8,
  created_at: "2026-07-11T00:00:00Z",
};

beforeEach(() => {
  vi.resetAllMocks();
  useGraphStore.setState({ entities: {} } as never);
  useMemoryAdminStore.setState({ loading: false, deleting: false, error: null });
  vi.mocked(memoryApi.fetchMemoryStats).mockResolvedValue({ total: 1, by_scope: { Agent: 1 } });
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
