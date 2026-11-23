import { graphStore, registerSchema, useGraphStore } from "@/platform/entities";
import { fetchAgentsList } from "@/features/agents/api/agents-api";
import { explicitAgentModel, type ExplicitModelSource } from "@/features/agents/model/explicit-model";

/**
 * Which agents keep their own model, so an operator changing the system default knows whom it
 * does not reach. One graph record holds the whole answer (the pinned rows and the count of
 * followers) so the two are never read from different loads. A load that fails replaces the record
 * with an error: pinned agents are then unknown, and the console never implies that all follow.
 */

export const PINNED_AGENTS_ENTITY = "SystemDefaultPinnedAgents" as const;
const PINNED_AGENTS_ID = "current" as const;

export interface PinnedAgentRow {
  id: string;
  title: string;
  /** The agent's own `provider/model`. */
  model: string;
}

export interface PinnedAgentsSnapshot extends Record<string, unknown> {
  id: typeof PINNED_AGENTS_ID;
  status: "ready" | "error";
  /** Pinned runtime agents, ordered by title. Empty and meaningless when `status` is `error`. */
  pinned: PinnedAgentRow[];
  /** Runtime agents that follow the system default. Zero and meaningless when `status` is `error`. */
  follow_count: number;
  error: string | null;
}

let registered = false;
let epoch = 0;

function register(): void {
  if (registered) return;
  registerSchema({ type: PINNED_AGENTS_ENTITY });
  registered = true;
}

type LoadedAgent = ExplicitModelSource & { id: string; metadata?: { title?: string } };

/** Splits one agents read into pinned rows and a follower count. Federated agents are not counted. */
export function summarizePinnedAgents(
  agents: readonly LoadedAgent[],
): Pick<PinnedAgentsSnapshot, "pinned" | "follow_count"> {
  const pinned: PinnedAgentRow[] = [];
  let followers = 0;
  for (const agent of agents) {
    if (agent._type === "federated") continue;
    const model = explicitAgentModel(agent);
    if (model) pinned.push({ id: agent.id, title: agent.metadata?.title ?? agent.id, model });
    else followers += 1;
  }
  pinned.sort((a, b) => a.title.localeCompare(b.title));
  return { pinned, follow_count: followers };
}

/**
 * Reads the agents and publishes the snapshot as one graph write. Never rejects; a failure is
 * published as `status: "error"`. The newest load wins.
 */
export async function loadPinnedAgents(): Promise<void> {
  register();
  const mine = ++epoch;
  let snapshot: PinnedAgentsSnapshot;
  try {
    const agents = (await fetchAgentsList()) as LoadedAgent[];
    snapshot = { id: PINNED_AGENTS_ID, status: "ready", error: null, ...summarizePinnedAgents(agents) };
  } catch (error) {
    snapshot = {
      id: PINNED_AGENTS_ID,
      status: "error",
      pinned: [],
      follow_count: 0,
      error: error instanceof Error ? error.message : "Agents could not be loaded",
    };
  }
  if (mine !== epoch) return;
  graphStore.getState().replaceEntity(PINNED_AGENTS_ENTITY, PINNED_AGENTS_ID, snapshot);
}

/** The latest snapshot; null until the first load has finished. */
export function usePinnedAgents(): PinnedAgentsSnapshot | null {
  return useGraphStore(
    (state) =>
      (state.entities[PINNED_AGENTS_ENTITY]?.[PINNED_AGENTS_ID] as PinnedAgentsSnapshot | undefined) ?? null,
  );
}
