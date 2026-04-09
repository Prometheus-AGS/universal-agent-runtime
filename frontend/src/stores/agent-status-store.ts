import { create } from "zustand";

export type AgentStatusType = "thinking" | "executing" | "searching" | "idle";

interface AgentStatusState {
  status: AgentStatusType;
  toolName?: string;
}

interface AgentStatusActions {
  setThinking: () => void;
  setExecuting: (toolName: string) => void;
  setSearching: () => void;
  setIdle: () => void;
}

type AgentStatusStore = AgentStatusState & AgentStatusActions;

export const useAgentStatusStore = create<AgentStatusStore>((set) => ({
  status: "idle",
  toolName: undefined,
  setThinking: () => set({ status: "thinking", toolName: undefined }),
  setExecuting: (toolName) => set({ status: "executing", toolName }),
  setSearching: () => set({ status: "searching", toolName: undefined }),
  setIdle: () => set({ status: "idle", toolName: undefined }),
}));
