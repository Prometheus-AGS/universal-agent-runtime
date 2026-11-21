import { useMemo } from "react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

import {
  CATALOG_MODEL_ENTITY,
  MODEL_CATALOG_LOAD_STATE_ENTITY,
  PROVIDER_ENTITY,
  PROVIDER_META_ENTITY,
  PROVIDER_META_ID,
  type CatalogModel,
  type ModelCatalogLoadId,
  type ModelCatalogLoadState,
  type Provider,
  type ProviderMeta,
  type ProviderMetaField,
} from "./contracts";

/**
 * Domain hooks over the catalog entities. Each subscribes to one slice or one derived primitive;
 * a control that needs one field uses `useProviderMetaField`, not a list hook.
 */

type EntityMap = Record<string, Record<string, unknown>> | undefined;

const EMPTY_MODELS: CatalogModel[] = [];
const EMPTY_PROVIDERS: Provider[] = [];

/** Every catalog model in the graph, in graph order. The array changes only when a Model is written. */
export function useCatalogModels(): CatalogModel[] {
  const models = useGraphStore((state) => state.entities[CATALOG_MODEL_ENTITY] as EntityMap);
  return useMemo(
    () => (models ? (Object.values(models) as CatalogModel[]) : EMPTY_MODELS),
    [models],
  );
}

/** One catalog model by its `provider/model` id; null while it is not in the graph. */
export function useCatalogModel(id: string | null | undefined): CatalogModel | null {
  return useGraphStore((state) =>
    id ? ((state.entities[CATALOG_MODEL_ENTITY]?.[id] as CatalogModel | undefined) ?? null) : null,
  );
}

/** The catalog models of one provider. Empty for an empty or unknown provider id. */
export function useCatalogModelsForProvider(providerId: string | null | undefined): CatalogModel[] {
  const models = useGraphStore((state) => state.entities[CATALOG_MODEL_ENTITY] as EntityMap);
  return useMemo(() => {
    if (!providerId || !models) return EMPTY_MODELS;
    const rows = (Object.values(models) as CatalogModel[]).filter(
      (model) => model.provider_id === providerId,
    );
    return rows.length > 0 ? rows : EMPTY_MODELS;
  }, [models, providerId]);
}

/** Every provider, configured first and then by display name. */
export function useCatalogProviders(): Provider[] {
  const providers = useGraphStore((state) => state.entities[PROVIDER_ENTITY] as EntityMap);
  return useMemo(() => {
    if (!providers) return EMPTY_PROVIDERS;
    return [...(Object.values(providers) as Provider[])].sort(
      (a, b) =>
        Number(b.configured === true) - Number(a.configured === true) ||
        String(a.display_name).localeCompare(String(b.display_name)),
    );
  }, [providers]);
}

/**
 * One field of the system default pair (`default_id` or `default_model`). Null until the
 * provider registry has loaded or while no default exists.
 */
export function useProviderMetaField(field: ProviderMetaField): string | null {
  return useGraphStore((state) => {
    const meta = state.entities[PROVIDER_META_ENTITY]?.[PROVIDER_META_ID] as ProviderMeta | undefined;
    return meta?.[field] ?? null;
  });
}

/** The last load failure for the models catalog or the provider registry; null when it succeeded. */
export function useModelCatalogLoadError(id: ModelCatalogLoadId): string | null {
  return useGraphStore((state) => {
    const loadState = state.entities[MODEL_CATALOG_LOAD_STATE_ENTITY]?.[id] as ModelCatalogLoadState | undefined;
    return loadState?.error ?? null;
  });
}

/** True once the models catalog or the provider registry has loaded successfully. */
export function useModelCatalogLoaded(id: ModelCatalogLoadId): boolean {
  return useGraphStore((state) => {
    const loadState = state.entities[MODEL_CATALOG_LOAD_STATE_ENTITY]?.[id] as ModelCatalogLoadState | undefined;
    return typeof loadState?.loaded_at === "string";
  });
}
