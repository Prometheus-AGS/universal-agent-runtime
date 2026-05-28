import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchMcpHealth } from "@/services/mcp-api";

export interface McpStatusRow {
  id: string;
  name: string;
  transport: string;
  status: "connected" | "disconnected" | "error";
  tool_count: number;
  error?: string;
}

/**
 * Fetch MCP server health and upsert each as a `McpStatus` entity. The
 * server's health endpoint is the source of truth; this fetcher is the
 * graph's hydration source. The mcp-health-page calls it on mount and
 * via a 30 s polling interval.
 *
 * No SSE topic — health probes are server-process-local; broadcasting
 * them through the realtime bus would require a manual-publish channel
 * that isn't worth the complexity at this stage.
 */
export async function loadMcpStatusIntoGraph(): Promise<void> {
  const data = await fetchMcpHealth<
    | { servers?: McpStatusRow[] }
    | McpStatusRow[]
  >();
  const list = Array.isArray(data) ? data : data.servers ?? [];
  const { upsertEntity } = useGraphStore.getState();
  for (const s of list) {
    const row: McpStatusRow = { ...s, id: s.name };
    upsertEntity("McpStatus", row.id, row as unknown as Record<string, unknown>);
  }
}
