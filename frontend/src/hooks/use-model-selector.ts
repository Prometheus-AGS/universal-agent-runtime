import { useEffect } from "react";

import { useModels } from "@/entities/hooks/use-models";
import { useModelsStore } from "@/stores/models-store";

export function useModelSelector() {
  const { items } = useModels();
  const loading = useModelsStore((state) => state.refreshing);
  const error = useModelsStore((state) => state.error);
  const load = useModelsStore((state) => state.load);

  useEffect(() => {
    void load();
  }, [load]);

  return { items, loading, error };
}
