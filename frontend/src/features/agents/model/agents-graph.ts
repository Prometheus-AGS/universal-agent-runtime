import { useGraphStore } from "@/platform/entities";
import { fetchAgentsList } from "../api/agents-api";

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
