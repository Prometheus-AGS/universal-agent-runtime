import { useAgentsAdminStore } from "./agents-admin-store";
import { useShallow } from "zustand/react/shallow";

export function useAgentsAdmin() {
  return useAgentsAdminStore(useShallow((state) => ({
    loading: state.loading,
    error: state.error,
    availableSkills: state.availableSkills,
    availableTools: state.availableTools,
    availableKnowledgeBases: state.availableKnowledgeBases,
    capabilitiesLoading: state.capabilitiesLoading,
    capabilitiesError: state.capabilitiesError,
    load: state.load,
    save: state.save,
    remove: state.remove,
    patch: state.patch,
    loadCapabilities: state.loadCapabilities,
    generate: state.generate,
  })));
}
