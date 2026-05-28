import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchToolsDiscovery } from "@/services/tools-api";
import type { UarTool } from "@/types";

/**
 * Tool graph row (page-local; not the `ToolGraphRow` exported from
 * `entities/types.ts`, which is the stricter shape used by `useTools`).
 *
 * `_ns` / `_key` / `_builtin` are kept as the legacy `ToolWithNs` shape
 * for back-compat with the tools-page grouping/filter logic.
 */
export interface ToolGraphRow extends UarTool {
  id: string;
  _ns: string;
  _key: string;
  _builtin: boolean;
}

function buildEntity(t: UarTool, builtin: boolean): ToolGraphRow {
  const fullName = t.namespaced_name ?? t.name;
  const parts = fullName.split("::");
  const ns = parts.length > 1 ? parts[0] : builtin ? "built-in" : "global";
  return {
    ...t,
    id: fullName,
    _ns: ns,
    _key: fullName,
    _builtin: builtin,
  };
}

/**
 * Fetch discovered MCP tools (plus built-ins) and upsert each into the
 * entity graph as a `Tool` entity. The tool registry is static after
 * server startup — see `add-push-channels-backend` assessment — so this
 * is a one-time fetch on page mount; no SSE subscription required.
 */
export async function loadToolsIntoGraph(): Promise<void> {
  const data = await fetchToolsDiscovery();
  const d = data.data ?? data;
  const all: ToolGraphRow[] = [
    ...(d.tools ?? []).map((t) => buildEntity(t as UarTool, false)),
    ...(d.built_in_tools ?? []).map((t) => buildEntity(t as UarTool, true)),
  ];

  const { upsertEntity } = useGraphStore.getState();
  for (const t of all) {
    upsertEntity("Tool", t.id, t as unknown as Record<string, unknown>);
  }
}
