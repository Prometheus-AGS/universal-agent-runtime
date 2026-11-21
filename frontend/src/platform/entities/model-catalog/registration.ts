import { registerSchema } from "@prometheus-ags/prometheus-entity-management";

import {
  CATALOG_MODEL_ENTITY,
  MODEL_CATALOG_LOAD_STATE_ENTITY,
  PROVIDER_ENTITY,
  PROVIDER_META_ENTITY,
  RUNTIME_PROVIDER_HEALTH_ENTITY,
} from "./contracts";

let registered = false;

/**
 * Register the `Provider`, `ProviderMeta`, `Model` and load-state schemas. No entity transport is
 * registered on purpose: the catalog is loaded by the domain actions (`loadModelCatalog`,
 * `loadProviderRegistry`) so that each payload is ingested atomically and a failed fetch writes
 * nothing. The relation `Provider.models -> Model` is keyed on `provider_id`.
 *
 * Called from `registerAllSchemas()` so the registration order of the entity bootstrap is unchanged.
 */
export function registerModelCatalogEntities(): void {
  if (registered) return;

  registerSchema({
    type: PROVIDER_ENTITY,
    relations: {
      models: {
        cardinality: "hasMany",
        targetType: CATALOG_MODEL_ENTITY,
        foreignKey: "provider_id",
        listKeyPrefix: (id) => [CATALOG_MODEL_ENTITY, { provider_id: id }],
      },
      health: {
        cardinality: "hasMany",
        targetType: RUNTIME_PROVIDER_HEALTH_ENTITY,
        foreignKey: "provider_id",
        listKeyPrefix: (id) => [RUNTIME_PROVIDER_HEALTH_ENTITY, { provider_id: id }],
      },
    },
  });

  registerSchema({
    type: CATALOG_MODEL_ENTITY,
    relations: {
      provider: {
        cardinality: "belongsTo",
        targetType: PROVIDER_ENTITY,
        foreignKey: "provider_id",
      },
    },
  });

  registerSchema({ type: PROVIDER_META_ENTITY });
  registerSchema({ type: MODEL_CATALOG_LOAD_STATE_ENTITY });
  registered = true;
}
