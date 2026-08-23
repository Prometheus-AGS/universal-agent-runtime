import { useGraphStore } from "@/platform/entities";
import { fetchAgentsList } from "../api/agents-api";

export const AGENT_LOAD_STATE_ENTITY = "AgentLoadState" as const;
export const AGENT_LOAD_STATE_ID = "agent-list" as const;

export interface AgentLoadState extends Record<string, unknown> {
  id: string;
  status: "loading" | "ready" | "error";
  error: string | null;
}

function publishAgentLoadState(status: AgentLoadState["status"], error: string | null): void {
  useGraphStore.getState().replaceEntity(AGENT_LOAD_STATE_ENTITY, AGENT_LOAD_STATE_ID, {
    id: AGENT_LOAD_STATE_ID,
    status,
    error,
  });
}

/**
 * Fetch all agents (runtime + federated) and upsert each into the
 * entity graph as an `Agent` entity.
 */
export async function loadAgentsIntoGraph(): Promise<void> {
  publishAgentLoadState("loading", null);
  try {
    const agents = await fetchAgentsList();
    const { upsertEntity } = useGraphStore.getState();

    for (const a of agents) {
      upsertEntity("Agent", a.id, a as unknown as Record<string, unknown>);
    }
    publishAgentLoadState("ready", null);
  } catch (error) {
    publishAgentLoadState("error", (error as Error).message);
    throw error;
  }
}
