import { useEffect } from "react";

import { useProviderModelsStore } from "@/stores/provider-models-store";
import type { ProviderModelRow } from "@/types";

const EMPTY_MODELS: ProviderModelRow[] = [];

export function useProviderModels(providerId: string) {
  const loadForProvider = useProviderModelsStore((s) => s.loadForProvider);
  const models = useProviderModelsStore((s) => s.rowsByProvider[providerId] ?? EMPTY_MODELS);
  const loadingModels = useProviderModelsStore((s) => s.loadingProviderId === providerId);

  useEffect(() => {
    void loadForProvider(providerId);
  }, [providerId, loadForProvider]);

  return { models, loadingModels };
}
