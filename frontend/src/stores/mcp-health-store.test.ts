import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { fetchMcpHealth } from "@/services/mcp-api";
import { useMcpHealthStore } from "@/stores/mcp-health-store";

vi.mock("@/services/mcp-api", () => ({ fetchMcpHealth: vi.fn() }));

describe("MCP health store", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    useGraphStore.setState({ entities: {} } as never);
    useMcpHealthStore.setState({ loading: false, error: null });
  });

  test("reconciles connected and failed server health", async () => {
    vi.mocked(fetchMcpHealth).mockResolvedValue({
      servers: [
        { name: "search", status: "connected", transport: "http", tool_count: 2 },
        { name: "files", status: "error", transport: "stdio", tool_count: 0, error: "child exited" },
      ],
    });
    await useMcpHealthStore.getState().load();
    expect(useGraphStore.getState().entities.McpStatus?.search).toMatchObject({ status: "connected" });
    expect(useGraphStore.getState().entities.McpStatus?.files).toMatchObject({ status: "error", error: "child exited" });
  });

  test("surfaces endpoint and transport failures without stale success", async () => {
    vi.mocked(fetchMcpHealth).mockRejectedValue(new Error("503 transport probe failed"));
    await expect(useMcpHealthStore.getState().load()).rejects.toThrow("503 transport probe failed");
    expect(useMcpHealthStore.getState()).toMatchObject({
      loading: false,
      error: "MCP health unavailable: 503 transport probe failed",
    });
  });
});
