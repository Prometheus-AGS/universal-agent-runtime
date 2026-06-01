import { useEntityList } from "@prometheus-ags/prometheus-entity-management";
import type { McpStatusRow } from "@/entities/fetchers/mcp-status";

/** Live list of MCP server health rows from the entity graph. */
export function useMcpStatus() {
  return useEntityList<McpStatusRow>("McpStatus");
}
