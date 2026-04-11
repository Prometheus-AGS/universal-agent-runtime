import { useEffect } from "react";

import { useCompilerSessionsStore } from "@/stores/compiler-sessions-store";

export function useCompilerSessions() {
  const sessions = useCompilerSessionsStore((s) => s.sessions);
  const loading = useCompilerSessionsStore((s) => s.loading);
  const error = useCompilerSessionsStore((s) => s.error);
  const creating = useCompilerSessionsStore((s) => s.creating);
  const load = useCompilerSessionsStore((s) => s.load);
  const createSession = useCompilerSessionsStore((s) => s.createSession);

  useEffect(() => {
    void load();
  }, [load]);

  return { sessions, loading, error, creating, load, createSession };
}
