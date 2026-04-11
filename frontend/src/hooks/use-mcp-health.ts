import { useEffect, useRef } from "react";

import { useMcpHealthStore } from "@/stores/mcp-health-store";

const AUTO_REFRESH_MS = 30_000;

export function useMcpHealth() {
  const servers = useMcpHealthStore((s) => s.servers);
  const loading = useMcpHealthStore((s) => s.loading);
  const error = useMcpHealthStore((s) => s.error);
  const load = useMcpHealthStore((s) => s.load);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    void load();
    intervalRef.current = setInterval(() => {
      void load();
    }, AUTO_REFRESH_MS);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [load]);

  return { servers, loading, error, load };
}
