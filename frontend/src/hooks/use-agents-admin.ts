import { useEffect } from "react";

import { useAgentsAdminStore } from "@/stores/agents-admin-store";

export function useAgentsAdmin() {
  const agents = useAgentsAdminStore((s) => s.agents);
  const loading = useAgentsAdminStore((s) => s.loading);
  const error = useAgentsAdminStore((s) => s.error);
  const load = useAgentsAdminStore((s) => s.load);
  const patchAgent = useAgentsAdminStore((s) => s.patchAgent);

  useEffect(() => {
    void load();
  }, [load]);

  return { agents, loading, error, load, patchAgent };
}
