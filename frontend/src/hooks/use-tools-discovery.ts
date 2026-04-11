import { useEffect } from "react";

import { useToolsDiscoveryStore } from "@/stores/tools-discovery-store";

export function useToolsDiscovery() {
  const tools = useToolsDiscoveryStore((s) => s.tools);
  const loading = useToolsDiscoveryStore((s) => s.loading);
  const error = useToolsDiscoveryStore((s) => s.error);
  const load = useToolsDiscoveryStore((s) => s.load);

  useEffect(() => {
    void load();
  }, [load]);

  return { tools, loading, error, load };
}
