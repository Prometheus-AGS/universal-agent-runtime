import { create } from "zustand";

import type { A2uiArtifactSchema, A2uiTestTriggerPayload } from "@/features/a2ui/a2ui-protocol";
import { fetchA2uiSchemas, triggerA2uiTest } from "@/services/a2ui-api";

interface A2uiSchemasState {
  schemas: A2uiArtifactSchema[];
  loading: boolean;
  error: string | null;
  triggering: boolean;
  triggerError: string | null;
}

interface A2uiSchemasActions {
  load: () => Promise<void>;
  trigger: (runId: string, payload: A2uiTestTriggerPayload) => Promise<boolean>;
}

export const useA2uiSchemasStore = create<A2uiSchemasState & A2uiSchemasActions>((set) => ({
  schemas: [],
  loading: true,
  error: null,
  triggering: false,
  triggerError: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const schemas = await fetchA2uiSchemas();
      set({ schemas, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },

  trigger: async (runId, payload) => {
    set({ triggering: true, triggerError: null });
    try {
      await triggerA2uiTest(runId, payload);
      set({ triggering: false });
      return true;
    } catch (error) {
      set({
        triggering: false,
        triggerError: error instanceof Error ? error.message : "Failed to trigger A2UI test",
      });
      return false;
    }
  },
}));
