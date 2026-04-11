import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchToolsDiscovery } from "@/services/tools-api";

/**
 * Fetch the tool discovery response and upsert every tool (MCP + built-in)
 * into the entity graph as a `Tool` entity.
 */
export async function loadToolsIntoGraph(): Promise<void> {
  const data = await fetchToolsDiscovery();
  const { upsertEntity } = useGraphStore.getState();

  // The response may be wrapped in a `data` envelope.
  const mcpTools = data.data?.tools ?? data.tools ?? [];
  const builtInTools = data.data?.built_in_tools ?? data.built_in_tools ?? [];

  for (const t of mcpTools) {
    const id = t.namespaced_name ?? t.name;
    upsertEntity("Tool", id, {
      id,
      name: t.name,
      description: t.description ?? "",
      namespace: t.source ?? "",
      input_schema: t.input_schema ?? {},
      transport: "mcp",
      built_in: false,
    });
  }

  for (const t of builtInTools) {
    upsertEntity("Tool", t.name, {
      id: t.name,
      name: t.name,
      description: t.description ?? "",
      namespace: "built-in",
      input_schema: t.parameters ?? {},
      transport: "internal",
      built_in: true,
    });
  }
}
