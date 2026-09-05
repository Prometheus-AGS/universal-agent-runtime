import { create } from "zustand";
import { useGraphStore } from "@/platform/entities";

import {
  createAgent,
  deleteAgent,
  fetchAgentsList,
  generateAgentDefinition,
  patchAgent,
} from "../api/agents-api";
import { fetchKnowledgeBases } from "@/features/knowledge/api";
import { fetchSkillsList } from "@/features/skills/api";
import { fetchToolsDiscovery } from "@/features/tools/api";
import type { UarKnowledgeBase, UarSkill, UarTool } from "@/types";

interface AgentsAdminState {
  loading: boolean;
  error: string | null;
  availableSkills: UarSkill[];
  availableTools: UarTool[];
  availableKnowledgeBases: UarKnowledgeBase[];
  capabilitiesLoading: boolean;
  capabilitiesError: string | null;
  load: () => Promise<void>;
  save: (id: string | undefined, payload: Record<string, unknown>) => Promise<void>;
  remove: (id: string) => Promise<void>;
  patch: (id: string, values: Record<string, unknown>) => Promise<void>;
  loadCapabilities: () => Promise<void>;
  generate: (description: string) => Promise<unknown>;
}

function reconcileAgents(agents: Awaited<ReturnType<typeof fetchAgentsList>>) {
  const graph = useGraphStore.getState();
  const nextIds = new Set(agents.map((agent) => agent.id));
  for (const id of Object.keys(graph.entities.Agent ?? {})) {
    if (!nextIds.has(id)) graph.removeEntity("Agent", id);
  }
  for (const agent of agents) {
    graph.upsertEntity("Agent", agent.id, agent as unknown as Record<string, unknown>);
  }
}

export const useAgentsAdminStore = create<AgentsAdminState>((set, get) => ({
  loading: false,
  error: null,
  availableSkills: [],
  availableTools: [],
  availableKnowledgeBases: [],
  capabilitiesLoading: false,
  capabilitiesError: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      reconcileAgents(await fetchAgentsList());
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ loading: false });
    }
  },

  save: async (id, payload) => {
    // The form edits known fields; preserve independently assigned run-policy
    // extensions through the host's conditional merge rather than full replace.
    if (id) await patchAgent(id, payload);
    else await createAgent(payload);
    await get().load();
  },

  remove: async (id) => {
    await deleteAgent(id);
    useGraphStore.getState().removeEntity("Agent", id);
  },

  patch: async (id, values) => {
    await patchAgent(id, values);
    useGraphStore.getState().upsertEntity("Agent", id, values);
  },

  loadCapabilities: async () => {
    if (get().capabilitiesLoading) return;
    set({ capabilitiesLoading: true, capabilitiesError: null });
    try {
      const [skills, toolsResponse, knowledgeBases] = await Promise.all([
        fetchSkillsList(),
        fetchToolsDiscovery(),
        fetchKnowledgeBases(),
      ]);
      const toolsData = toolsResponse.data ?? toolsResponse;
      set({
        availableSkills: skills,
        availableTools: [...(toolsData.tools ?? []), ...(toolsData.built_in_tools ?? [])],
        availableKnowledgeBases: knowledgeBases,
      });
    } catch (error) {
      set({ capabilitiesError: (error as Error).message });
      throw error;
    } finally {
      set({ capabilitiesLoading: false });
    }
  },

  generate: generateAgentDefinition,
}));
