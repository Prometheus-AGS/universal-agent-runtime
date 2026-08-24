import { useConfiguredModels } from "@/platform/entities";

export function useModelSelector() {
  const { items, isLoading, error } = useConfiguredModels();
  return { items, loading: isLoading, error: error?.message ?? null };
}
