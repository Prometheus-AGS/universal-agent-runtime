import { useCallback, useEffect, useRef } from "react";

import {
  loadModelCatalog,
  loadProviderRegistry,
  useModelCatalogLoadError,
  useModelCatalogLoaded,
} from "@/platform/entities";

export interface SystemDefaultData {
  /** The provider registry (the live default pair) has not loaded yet. */
  registryLoading: boolean;
  registryError: string | null;
  /** The model catalog behind the picker has not loaded yet. */
  catalogLoading: boolean;
  catalogError: string | null;
  /** Asks both sources again. */
  retry: () => void;
}

/**
 * Makes sure the data behind the system default panel is loaded: the provider registry, which
 * holds the live default pair, and the model catalog the picker lists. Pickers never load data
 * themselves, so the panel that owns them does, once per mount and again on Retry.
 */
export function useSystemDefaultData(): SystemDefaultData {
  const registryLoaded = useModelCatalogLoaded("providers");
  const registryError = useModelCatalogLoadError("providers");
  const catalogLoaded = useModelCatalogLoaded("models");
  const catalogError = useModelCatalogLoadError("models");
  const requested = useRef({ registry: false, catalog: false });

  // The live pair is read fresh on every visit: a pair restored from a stored snapshot is not proof.
  useEffect(() => {
    if (requested.current.registry) return;
    requested.current.registry = true;
    void loadProviderRegistry().catch(() => undefined);
  }, []);

  useEffect(() => {
    if (catalogLoaded || catalogError || requested.current.catalog) return;
    requested.current.catalog = true;
    void loadModelCatalog().catch(() => undefined);
  }, [catalogLoaded, catalogError]);

  const retry = useCallback(() => {
    void loadProviderRegistry().catch(() => undefined);
    void loadModelCatalog().catch(() => undefined);
  }, []);

  return {
    registryLoading: !registryLoaded && !registryError,
    registryError,
    catalogLoading: !catalogLoaded && !catalogError,
    catalogError,
    retry,
  };
}
