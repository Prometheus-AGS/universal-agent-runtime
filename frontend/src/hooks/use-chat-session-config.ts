import { useEffect } from "react";

import { useAgents } from "@/features/agents/model";
import { useChatSessionConfigStore } from "@/stores/chat-session-config-store";

/** Coordinate agent selection and session configuration for chat components. */
export function useChatSessionConfig(loadOnMount = false) {
  const agents = useAgents().items;
  const loadAgents = useChatSessionConfigStore((state) => state.loadAgents);
  const loadDefaultModelLabel = useChatSessionConfigStore(
    (state) => state.loadDefaultModelLabel,
  );

  useEffect(() => {
    if (!loadOnMount) return;
    void loadAgents();
    void loadDefaultModelLabel();
  }, [loadAgents, loadDefaultModelLabel, loadOnMount]);

  return {
    agents,
    modelLabel: useChatSessionConfigStore((state) => state.modelLabel),
    loadingAgents: useChatSessionConfigStore((state) => state.loadingAgents),
    saving: useChatSessionConfigStore((state) => state.saving),
    error: useChatSessionConfigStore((state) => state.error),
    loadDefaultModelLabel,
    save: useChatSessionConfigStore((state) => state.save),
    setModelLabel: useChatSessionConfigStore((state) => state.setModelLabel),
  };
}
