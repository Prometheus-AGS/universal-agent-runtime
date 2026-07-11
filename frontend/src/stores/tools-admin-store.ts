import { create } from "zustand";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

import { executeTool, fetchToolsDiscovery, type ExecuteToolResult } from "@/services/tools-api";
import type { UarTool } from "@/types";

interface ToolsAdminStore {
  loading: boolean;
  error: string | null;
  executing: boolean;
  executionError: string | null;
  load: () => Promise<void>;
  execute: (name: string, arguments_: Record<string, unknown>) => Promise<ExecuteToolResult>;
}

export const useToolsAdminStore = create<ToolsAdminStore>((set) => ({
  loading: false,
  error: null,
  executing: false,
  executionError: null,
  load: async () => {
    set({ loading: true, error: null });
    try {
      const response = await fetchToolsDiscovery();
      const data = response.data ?? response;
      const graph = useGraphStore.getState();
      const buildRow = (tool: UarTool, builtin: boolean) => {
        const key = tool.namespaced_name ?? tool.name;
        const parts = key.split("::");
        return {
          ...tool,
          id: key,
          _ns: parts.length > 1 ? parts[0] : builtin ? "built-in" : "global",
          _key: key,
          _builtin: builtin,
        };
      };
      const rows = [
        ...(data.tools ?? []).map((tool) => buildRow(tool, false)),
        ...(data.built_in_tools ?? []).map((tool) => buildRow(tool, true)),
      ];
      const ids = new Set(rows.map((tool) => tool._key));
      for (const id of Object.keys(graph.entities.Tool ?? {})) {
        if (!ids.has(id)) graph.removeEntity("Tool", id);
      }
      for (const tool of rows) graph.upsertEntity("Tool", tool._key, tool);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ loading: false });
    }
  },
  execute: async (name, arguments_) => {
    set({ executing: true, executionError: null });
    try {
      return await executeTool(name, arguments_);
    } catch (error) {
      set({ executionError: (error as Error).message });
      throw error;
    } finally {
      set({ executing: false });
    }
  },
}));
