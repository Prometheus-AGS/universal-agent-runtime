import { useEffect } from "react";

import { useProviderDefault } from "@/entities/hooks/use-provider-default";
import { useProviders } from "@/entities/hooks/use-providers";
import { useProvidersStore } from "@/stores/providers-store";
import type { CatalogProviderSummary } from "@/types";

export function useProvidersAdmin() {
  const view = useProviders();
  const defaultId = useProviderDefault() ?? undefined;
  const refreshing = useProvidersStore((state) => state.refreshing);
  const saving = useProvidersStore((state) => state.saving);
  const removing = useProvidersStore((state) => state.removingId);
  const error = useProvidersStore((state) => state.error);
  const load = useProvidersStore((state) => state.load);
  const configure = useProvidersStore((state) => state.configure);
  const setDefault = useProvidersStore((state) => state.setDefault);
  const remove = useProvidersStore((state) => state.remove);

  useEffect(() => {
    void load();
  }, [load]);

  const catalog = view.items as unknown as CatalogProviderSummary[];
  return {
    catalog,
    configured: catalog.filter((provider) => provider.configured),
    defaultId,
    loading: refreshing && catalog.length === 0,
    refreshing,
    saving,
    removing,
    error,
    load,
    configure,
    setDefault,
    remove,
  };
}
