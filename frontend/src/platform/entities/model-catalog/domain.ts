import { batchGraphWrites } from "@prometheus-ags/entity-graph-core";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";

import type { CatalogModelsResponse, CatalogResponse, ProvidersResponse } from "@/types";
import {
  fetchCatalog,
  fetchConfiguredProviders,
  fetchModelsCatalog,
  fetchProviderHealth,
  type ProviderHealthEntry,
  type ProviderHealthResponse,
} from "./api/model-catalog-api";
import {
  CATALOG_MODELS_LIST_KEY,
  CATALOG_MODEL_ENTITY,
  MODEL_CATALOG_LOAD_ID,
  MODEL_CATALOG_LOAD_STATE_ENTITY,
  PROVIDER_ENTITY,
  PROVIDER_LIST_KEY,
  PROVIDER_META_ENTITY,
  PROVIDER_META_ID,
  PROVIDER_REGISTRY_LOAD_ID,
  RUNTIME_PROVIDER_HEALTH_ENTITY,
  providerModelsListKey,
  type ModelCatalogLoadId,
  type Provider,
  type ProviderMeta,
} from "./contracts";
import { projectCatalogPayload } from "./projection";

/**
 * Ingestion and load actions. Each ingestion action validates and projects the whole payload
 * first, then publishes it with one `batchGraphWrites`, so a rejected record or a failed fetch
 * leaves the graph unchanged and a payload never appears half-applied.
 */

const nowIso = (): string => new Date().toISOString();

function loadedState(id: ModelCatalogLoadId): Record<string, unknown> {
  return { id, error: null, loaded_at: nowIso() };
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function recordLoadFailure(id: ModelCatalogLoadId, error: unknown): void {
  batchGraphWrites((batch) => {
    batch.upsertEntity(MODEL_CATALOG_LOAD_STATE_ENTITY, id, { id, error: errorMessage(error) });
  });
}

function entityIds(type: string): string[] {
  return Object.keys(graphStore.getState().entities[type] ?? {});
}

/**
 * Publish the whole models catalog: upsert every row, remove rows the payload no longer lists
 * and set the all-models and per-provider id lists, in one publication.
 *
 * @throws ModelCatalogIngestionError when any record is malformed. Nothing is written.
 */
export function ingestModelCatalog(payload: CatalogModelsResponse): number {
  const rows = projectCatalogPayload(payload);
  const nextIds = new Set<string>();
  const idsByProvider = new Map<string, string[]>();
  for (const row of rows) {
    nextIds.add(row.id);
    const ids = idsByProvider.get(row.provider_id);
    if (ids) ids.push(row.id);
    else idsByProvider.set(row.provider_id, [row.id]);
  }

  const previous = graphStore.getState().entities[CATALOG_MODEL_ENTITY] ?? {};
  const removedIds: string[] = [];
  const previousProviderIds = new Set<string>();
  for (const [id, entity] of Object.entries(previous)) {
    if (typeof entity.provider_id === "string") previousProviderIds.add(entity.provider_id);
    if (!nextIds.has(id)) removedIds.push(id);
  }

  batchGraphWrites((batch) => {
    batch.upsertEntities(
      CATALOG_MODEL_ENTITY,
      rows.map((row) => ({ id: row.id, data: row })),
    );
    for (const id of removedIds) batch.removeEntity(CATALOG_MODEL_ENTITY, id);
    batch.setListIds(CATALOG_MODELS_LIST_KEY, [...nextIds], CATALOG_MODEL_ENTITY);
    for (const [providerId, ids] of idsByProvider) {
      batch.setListIds(providerModelsListKey(providerId), ids, CATALOG_MODEL_ENTITY);
    }
    for (const providerId of previousProviderIds) {
      if (!idsByProvider.has(providerId)) {
        batch.setListIds(providerModelsListKey(providerId), [], CATALOG_MODEL_ENTITY);
      }
    }
    batch.upsertEntity(
      MODEL_CATALOG_LOAD_STATE_ENTITY,
      MODEL_CATALOG_LOAD_ID,
      loadedState(MODEL_CATALOG_LOAD_ID),
    );
  });
  return rows.length;
}

/** The provider the registry names as default, and its `default_model`, as one record. */
export function providerMetaFrom(configured: ProvidersResponse): ProviderMeta {
  const defaultProvider = configured.providers.find((provider) => provider.id === configured.default_id);
  return {
    id: PROVIDER_META_ID,
    default_id: configured.default_id ?? null,
    default_model: defaultProvider?.default_model ?? null,
  };
}

/**
 * Publish the provider registry: `Provider` rows (the catalog merged with the configured set),
 * the `ProviderMeta` default pair and the ordered provider list, in one publication. Providers
 * the registry no longer lists are removed.
 */
export function ingestProviderRegistry(catalog: CatalogResponse, configured: ProvidersResponse): void {
  const configuredIds = new Set(configured.providers.map((provider) => provider.id));
  const providers = new Map<string, Provider>();

  for (const provider of catalog.providers) {
    providers.set(provider.id, {
      id: provider.id,
      display_name: provider.display_name,
      base_url: provider.base_url,
      configured: configuredIds.has(provider.id) || provider.configured,
      auth_env_var: provider.auth_env_var,
      endpoints: provider.endpoints,
      model_count: provider.model_count,
      status: provider.status,
      status_detail: provider.status_detail,
    });
  }
  for (const provider of configured.providers) {
    if (providers.has(provider.id)) continue;
    providers.set(provider.id, {
      id: provider.id,
      display_name: provider.display_name ?? provider.id,
      base_url: provider.base_url,
      configured: true,
      endpoints: [],
      model_count: provider.models?.length ?? 0,
    });
  }

  const removedIds = entityIds(PROVIDER_ENTITY).filter((id) => !providers.has(id));
  const meta = providerMetaFrom(configured);

  batchGraphWrites((batch) => {
    batch.upsertEntities(
      PROVIDER_ENTITY,
      [...providers.values()].map((provider) => ({ id: provider.id, data: provider })),
    );
    for (const id of removedIds) batch.removeEntity(PROVIDER_ENTITY, id);
    batch.replaceEntity(PROVIDER_META_ENTITY, PROVIDER_META_ID, meta);
    batch.setListIds(PROVIDER_LIST_KEY, [...providers.keys()], PROVIDER_ENTITY);
    batch.upsertEntity(
      MODEL_CATALOG_LOAD_STATE_ENTITY,
      PROVIDER_REGISTRY_LOAD_ID,
      loadedState(PROVIDER_REGISTRY_LOAD_ID),
    );
  });
}

/** Publish every provider's health row in one publication. */
export function ingestProviderHealth(providers: ProviderHealthResponse["providers"]): void {
  const updatedAt = nowIso();
  const entries = Object.entries(providers).map(([providerId, health]: [string, ProviderHealthEntry]) => ({
    id: providerId,
    data: {
      id: providerId,
      provider_id: providerId,
      status: health.healthy ? "healthy" : health.cooldown_remaining_secs ? "degraded" : "offline",
      error: health.consecutive_errors ? `${health.consecutive_errors} consecutive error(s)` : undefined,
      updated_at: updatedAt,
    },
  }));
  batchGraphWrites((batch) => {
    batch.upsertEntities(RUNTIME_PROVIDER_HEALTH_ENTITY, entries);
  });
}

/**
 * Registry epoch. A load publishes only when no newer load has started and no system default
 * change has been published since it began; an older response must not overwrite fresher data.
 */
let registryEpoch = 0;
let catalogEpoch = 0;

/** Called when a write publishes provider metadata, so older in-flight registry loads are dropped. */
export function invalidateInFlightRegistryLoads(): void {
  registryEpoch += 1;
}

/**
 * Fetch the models catalog, then publish it. A failed fetch or a rejected record leaves the
 * graph unchanged, records the failure in the load state and rethrows.
 */
export async function loadModelCatalog(): Promise<void> {
  const epoch = ++catalogEpoch;
  try {
    const payload = await fetchModelsCatalog();
    if (epoch !== catalogEpoch) return;
    ingestModelCatalog(payload);
  } catch (error) {
    if (epoch === catalogEpoch) recordLoadFailure(MODEL_CATALOG_LOAD_ID, error);
    throw error;
  }
}

/**
 * Fetch the provider catalog and the configured registry, then publish both. A failed fetch
 * leaves the graph unchanged, records the failure in the load state and rethrows.
 */
export async function loadProviderRegistry(): Promise<void> {
  const epoch = ++registryEpoch;
  try {
    const [catalog, configured] = await Promise.all([fetchCatalog(), fetchConfiguredProviders()]);
    if (epoch !== registryEpoch) return;
    ingestProviderRegistry(catalog, configured);
  } catch (error) {
    if (epoch === registryEpoch) recordLoadFailure(PROVIDER_REGISTRY_LOAD_ID, error);
    throw error;
  }
}

/** Fetch provider health and publish it as one batch. Returns the response for workflow flags. */
export async function loadProviderHealth(): Promise<ProviderHealthResponse> {
  const response = await fetchProviderHealth();
  ingestProviderHealth(response.providers);
  return response;
}
