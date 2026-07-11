import { useAgentStatusStore } from "@/stores/agent-status-store";

export type { AgentStatusType } from "@/stores/agent-status-store";

/** Subscribe to the current agent execution status. */
export function useAgentStatus() {
  return {
    status: useAgentStatusStore((state) => state.status),
    toolName: useAgentStatusStore((state) => state.toolName),
  };
}
