import { modelCatalogTransport } from "@/platform/entities";

/**
 * The provider transport moved to the model-catalog platform domain
 * (`platform/entities/model-catalog/api`). These re-exports remain for callers that have not
 * migrated to the domain actions; new code calls `loadProviderRegistry`, `setSystemDefaultModel`
 * and the other actions from `@/platform/entities`.
 */
export type {
  ProviderHealthEntry,
  ProviderHealthResponse,
} from "@/platform/entities";
export type CreateProviderBody = modelCatalogTransport.CreateProviderBody;

export const fetchCatalog = modelCatalogTransport.fetchCatalog;
export const fetchConfiguredProviders = modelCatalogTransport.fetchConfiguredProviders;
export const fetchProviderHealth = modelCatalogTransport.fetchProviderHealth;
export const createProvider = modelCatalogTransport.createProvider;
export const updateProvider = modelCatalogTransport.updateProvider;
export const setDefaultProvider = modelCatalogTransport.setDefaultProvider;
export const deleteProvider = modelCatalogTransport.deleteProvider;
