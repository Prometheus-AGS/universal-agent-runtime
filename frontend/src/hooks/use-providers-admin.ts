import { useEffect } from "react";

import { useProvidersAdminStore } from "@/stores/providers-admin-store";

export function useProvidersAdmin() {
  const catalog = useProvidersAdminStore((s) => s.catalog);
  const configured = useProvidersAdminStore((s) => s.configured);
  const defaultId = useProvidersAdminStore((s) => s.defaultId);
  const loading = useProvidersAdminStore((s) => s.loading);
  const error = useProvidersAdminStore((s) => s.error);
  const saving = useProvidersAdminStore((s) => s.saving);
  const removing = useProvidersAdminStore((s) => s.removing);
  const load = useProvidersAdminStore((s) => s.load);
  const configureProvider = useProvidersAdminStore((s) => s.configureProvider);
  const setDefault = useProvidersAdminStore((s) => s.setDefault);
  const removeProvider = useProvidersAdminStore((s) => s.removeProvider);
  const clearError = useProvidersAdminStore((s) => s.clearError);

  useEffect(() => {
    void load();
  }, [load]);

  return {
    catalog,
    configured,
    defaultId,
    loading,
    error,
    saving,
    removing,
    load,
    configureProvider,
    setDefault,
    removeProvider,
    clearError,
  };
}
