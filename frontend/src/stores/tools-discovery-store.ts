import { create } from "zustand";

import { fetchToolsDiscovery } from "@/services/tools-api";
import type { UarTool } from "@/types";

export interface ToolWithNs extends UarTool {
  _ns: string;
  _key: string;
}

interface ToolsDiscoveryState {
  tools: ToolWithNs[];
  loading: boolean;
  error: string | null;
}

interface ToolsDiscoveryActions {
  load: () => Promise<void>;
}

function addNs(t: UarTool, builtin: boolean): ToolWithNs {
  const fullName = t.namespaced_name ?? t.name;
  const parts = fullName.split("::");
  return { ...t, _ns: parts.length > 1 ? parts[0] : (builtin ? "built-in" : "global"), _key: fullName };
}

export const useToolsDiscoveryStore = create<ToolsDiscoveryState & ToolsDiscoveryActions>((set) => ({
  tools: [],
  loading: true,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const data = await fetchToolsDiscovery();
      const d = data.data ?? data;
      const all: ToolWithNs[] = [
        ...(d.tools ?? []).map((t) => addNs(t, false)),
        ...(d.built_in_tools ?? []).map((t) => addNs(t, true)),
      ];
      set({ tools: all, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },
}));
