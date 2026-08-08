import { useMcpHealthStore } from "./mcp-health-store";

export function useMcpHealth() {
  const loading = useMcpHealthStore((state) => state.loading);
  const error = useMcpHealthStore((state) => state.error);
  const load = useMcpHealthStore((state) => state.load);
  return { loading, error, load };
}
