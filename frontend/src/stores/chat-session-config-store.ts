import { create } from "zustand";

import { loadAgentsIntoGraph } from "@/features/agents/model";
import { fetchConfiguredProviders } from "@/features/providers/api";
import { saveSessionAgentConfig } from "@/services/session-config-api";

interface ChatSessionConfigState {
  modelLabel: string | null;
  loadingAgents: boolean;
  saving: boolean;
  error: string | null;
}

interface ChatSessionConfigActions {
  loadAgents: () => Promise<void>;
  loadDefaultModelLabel: () => Promise<void>;
  save: (threadId: string, config: Record<string, unknown>) => Promise<boolean>;
  setModelLabel: (label: string) => void;
}

function configuredModelLabel(data: Awaited<ReturnType<typeof fetchConfiguredProviders>>) {
  const defaultId = data.default_id;
  if (defaultId) {
    const provider = (data.providers ?? []).find((item) => item.id === defaultId);
    const model = provider?.models?.[0]?.id;
    return model ? `${defaultId}/${model}` : defaultId;
  }
  return (data.providers ?? []).length > 0 ? "Using default model" : null;
}

export const useChatSessionConfigStore = create<
  ChatSessionConfigState & ChatSessionConfigActions
>((set) => ({
  modelLabel: null,
  loadingAgents: false,
  saving: false,
  error: null,

  loadAgents: async () => {
    set({ loadingAgents: true, error: null });
    try {
      await loadAgentsIntoGraph();
    } catch (error) {
      set({ error: (error as Error).message });
    } finally {
      set({ loadingAgents: false });
    }
  },

  loadDefaultModelLabel: async () => {
    try {
      const providers = await fetchConfiguredProviders();
      set({ modelLabel: configuredModelLabel(providers) });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  save: async (threadId, config) => {
    set({ saving: true, error: null });
    try {
      await saveSessionAgentConfig(threadId, config);
      return true;
    } catch (error) {
      set({ error: (error as Error).message });
      return false;
    } finally {
      set({ saving: false });
    }
  },

  setModelLabel: (modelLabel) => set({ modelLabel }),
}));
