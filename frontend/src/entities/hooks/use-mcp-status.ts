import { useGraphEntities } from "@/entities/hooks/use-graph-entities";
import type { McpStatusRow } from "@/entities/fetchers/mcp-status";

/** Live list of MCP server health rows from the entity graph. */
export function useMcpStatus(): { items: McpStatusRow[] } {
  return { items: useGraphEntities<McpStatusRow>("McpStatus") };
}
