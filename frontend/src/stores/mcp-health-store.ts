import { create } from "zustand";

import { fetchMcpHealth } from "@/services/mcp-api";

export interface McpServerHealth {
  name: string;
  transport: string;
  status: "connected" | "disconnected" | "error";
  tool_count: number;
  error?: string;
}

interface McpHealthState {
  servers: McpServerHealth[];
  loading: boolean;
  error: string | null;
}

interface McpHealthActions {
  load: () => Promise<void>;
}

export const useMcpHealthStore = create<McpHealthState & McpHealthActions>((set) => ({
  servers: [],
  loading: true,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const data = await fetchMcpHealth<
        | { servers?: McpServerHealth[] }
        | McpServerHealth[]
      >();
      const list = Array.isArray(data) ? data : data.servers ?? [];
      set({ servers: list, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },
}));
