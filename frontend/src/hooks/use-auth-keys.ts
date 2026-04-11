import { useEffect } from "react";

import { useAuthKeysStore } from "@/stores/auth-keys-store";

export function useAuthKeys() {
  const keys = useAuthKeysStore((s) => s.keys);
  const loading = useAuthKeysStore((s) => s.loading);
  const error = useAuthKeysStore((s) => s.error);
  const saving = useAuthKeysStore((s) => s.saving);
  const revoking = useAuthKeysStore((s) => s.revoking);
  const load = useAuthKeysStore((s) => s.load);
  const createKey = useAuthKeysStore((s) => s.createKey);
  const revokeKey = useAuthKeysStore((s) => s.revokeKey);

  useEffect(() => {
    void load();
  }, [load]);

  return { keys, loading, error, saving, revoking, load, createKey, revokeKey };
}
