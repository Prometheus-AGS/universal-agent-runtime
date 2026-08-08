import { create } from "zustand";
import { useGraphStore } from "@/platform/entities";

import { fetchMcpHealth } from "../api/mcp-api";
import type { McpStatusRow } from "./mcp-status";

interface McpHealthStore {
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
}

export const useMcpHealthStore = create<McpHealthStore>((set) => ({
  loading: false,
  error: null,
  load: async () => {
    set({ loading: true, error: null });
    try {
      const response = await fetchMcpHealth<{ servers?: McpStatusRow[] } | McpStatusRow[]>();
      const servers = Array.isArray(response) ? response : response.servers ?? [];
      const graph = useGraphStore.getState();
      const ids = new Set(servers.map((server) => server.name));
      for (const id of Object.keys(graph.entities.McpStatus ?? {})) {
        if (!ids.has(id)) graph.removeEntity("McpStatus", id);
      }
      for (const server of servers) {
        graph.upsertEntity("McpStatus", server.name, { ...server, id: server.name });
      }
    } catch (error) {
      set({ error: `MCP health unavailable: ${(error as Error).message}` });
      throw error;
    } finally {
      set({ loading: false });
    }
  },
}));
