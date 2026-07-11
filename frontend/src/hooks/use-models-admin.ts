import { useEffect } from "react";

import { useModels } from "@/entities/hooks/use-models";
import { useModelsStore } from "@/stores/models-store";

export function useModelsAdmin() {
  const view = useModels();
  const configured = useModelsStore((state) => state.configuredProviders);
  const refreshing = useModelsStore((state) => state.refreshing);
  const busyModelKey = useModelsStore((state) => state.busyModelKey);
  const error = useModelsStore((state) => state.error);
  const load = useModelsStore((state) => state.load);
  const addModel = useModelsStore((state) => state.addModel);
  const setDefaultModel = useModelsStore((state) => state.setDefaultModel);
  const removeModel = useModelsStore((state) => state.removeModel);

  useEffect(() => {
    void load();
  }, [load]);

  return {
    items: view.items,
    configured,
    refreshing,
    busyModelKey,
    error,
    load,
    addModel,
    setDefaultModel,
    removeModel,
  };
}
