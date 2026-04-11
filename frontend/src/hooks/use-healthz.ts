import { useEffect } from "react";

import { useHealthStore } from "@/stores/health-store";

export function useHealthz() {
  const health = useHealthStore((s) => s.health);
  const load = useHealthStore((s) => s.load);

  useEffect(() => {
    void load();
  }, [load]);

  return { health, load };
}
