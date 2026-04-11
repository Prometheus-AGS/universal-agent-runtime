import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchAgentsList } from "@/services/agents-api";

/**
 * Fetch all agents (runtime + federated) and upsert each into the
 * entity graph as an `Agent` entity.
 */
export async function loadAgentsIntoGraph(): Promise<void> {
  const agents = await fetchAgentsList();
  const { upsertEntity } = useGraphStore.getState();

  for (const a of agents) {
    upsertEntity("Agent", a.id, a as unknown as Record<string, unknown>);
  }
}
