/**
 * AgentConfigContext -- provides the currently selected agent's
 * configuration to deeply nested components (e.g. CapabilityToggles).
 */

import { createContext, useContext } from "react";
import type { AgentConfig } from "@/features/chat/agent-selector";

export const AgentConfigContext = createContext<AgentConfig | null>(null);

export function useAgentConfig(): AgentConfig | null {
  return useContext(AgentConfigContext);
}
