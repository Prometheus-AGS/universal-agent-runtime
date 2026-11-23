export {
  CATALOG_MODEL_ENTITY,
  CATALOG_MODELS_LIST_KEY,
  MODEL_CATALOG_LOAD_ID,
  MODEL_CATALOG_LOAD_STATE_ENTITY,
  MODEL_CATALOG_SNAPSHOT_LOAD_ID,
  ModelCatalogIngestionError,
  PROVIDER_ENTITY,
  PROVIDER_LIST_KEY,
  PROVIDER_META_ENTITY,
  PROVIDER_META_ID,
  PROVIDER_REGISTRY_LOAD_ID,
  SystemDefaultModelError,
  catalogModelId,
  providerModelsListKey,
} from "./contracts";

export type {
  CatalogModel,
  ModelCatalogIngestionErrorCode,
  ModelCatalogLoadId,
  ModelCatalogLoadState,
  Provider,
  ProviderHealthEntry,
  ProviderHealthResponse,
  ProviderMeta,
  ProviderMetaField,
  SystemDefaultModelErrorInit,
  SystemDefaultModelErrorKind,
  SystemDefaultModelResult,
  SystemDefaultModelStage,
} from "./contracts";

export { projectCatalogModel, projectCatalogPayload } from "./projection";
export type { CatalogProviderInfo, ProjectCatalogModelOptions } from "./projection";

export { registerModelCatalogEntities } from "./registration";

export {
  ingestModelCatalog,
  ingestProviderHealth,
  ingestProviderRegistry,
  loadModelCatalog,
  loadProviderHealth,
  loadProviderRegistry,
  recordModelCatalogSnapshotAccepted,
  recordModelCatalogSnapshotRejected,
} from "./domain";

export { setSystemDefaultModel, setSystemDefaultProvider } from "./system-default";

export {
  useCatalogModel,
  useCatalogModels,
  useCatalogModelsForProvider,
  useCatalogProviders,
  useModelCatalogLoadError,
  useModelCatalogLoaded,
  useProviderMetaField,
} from "./use-model-catalog";

/**
 * Raw transport functions and `ProviderApiError`, exported only so the feature-level API modules
 * (`features/models/api`, `features/providers/api`) can re-export them for callers that have not
 * migrated to the domain actions. New code calls the actions above.
 */
export * as modelCatalogTransport from "./api/model-catalog-api";
