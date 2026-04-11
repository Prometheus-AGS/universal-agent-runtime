import { create } from "zustand";

import { fetchAgentsList, patchAgent as patchAgentApi } from "@/services/agents-api";
import type { UarAgent } from "@/types";

interface AgentsAdminState {
  agents: UarAgent[];
  loading: boolean;
  error: string | null;
}

interface AgentsAdminActions {
  load: () => Promise<void>;
  patchAgent: (agentId: string, body: Record<string, unknown>) => Promise<void>;
}

export const useAgentsAdminStore = create<AgentsAdminState & AgentsAdminActions>((set) => ({
  agents: [],
  loading: true,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const agents = await fetchAgentsList();
      set({ agents, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },

  patchAgent: async (agentId, body) => {
    await patchAgentApi(agentId, body);
  },
}));
