import { useGraphStore } from "@/platform/entities";
import { beforeEach, describe, expect, test, vi } from "vitest";

import * as toolsApi from "../api/tools-api";
import { useToolsAdminStore } from "./tools-admin-store";

vi.mock("../api/tools-api", () => ({
  executeTool: vi.fn(),
  fetchToolsDiscovery: vi.fn(),
}));

beforeEach(() => {
  vi.resetAllMocks();
  useGraphStore.setState({ entities: {} } as never);
  useToolsAdminStore.setState({ loading: false, error: null, executing: false, executionError: null });
});

describe("tools admin store", () => {
  test("loads discovery and executes through the governed service", async () => {
    vi.mocked(toolsApi.fetchToolsDiscovery).mockResolvedValue({
      tools: [{ name: "search", namespaced_name: "web::search" }],
      built_in_tools: [],
    } as never);
    vi.mocked(toolsApi.executeTool).mockResolvedValue({ result: { ok: true }, duration_ms: 5, success: true });

    await useToolsAdminStore.getState().load();
    await expect(useToolsAdminStore.getState().execute("web::search", { q: "release" })).resolves.toMatchObject({ success: true });
    expect(useGraphStore.getState().entities.Tool?.["web::search"]).toMatchObject({ _ns: "web" });
    expect(toolsApi.executeTool).toHaveBeenCalledWith("web::search", { q: "release" });
  });

  test("surfaces transport execution failures", async () => {
    vi.mocked(toolsApi.executeTool).mockRejectedValue(new Error("MCP transport closed"));
    await expect(useToolsAdminStore.getState().execute("web::search", {})).rejects.toThrow("MCP transport closed");
    expect(useToolsAdminStore.getState()).toMatchObject({ executing: false, executionError: "MCP transport closed" });
  });
});
