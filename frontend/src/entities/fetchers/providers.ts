import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchCatalog, fetchConfiguredProviders } from "@/services/providers-api";
import { fetchModelsCatalog, modelsRowsForProvider } from "@/services/models-api";
import type { ProviderEntity, ModelEntity } from "@/entities/types";

/**
 * Fetch the catalog and configured-providers lists, then upsert every
 * provider into the entity graph as a `Provider` entity.
 *
 * Configured providers augment catalog entries (e.g. base_url overrides).
 */
export async function loadProvidersIntoGraph(): Promise<void> {
  const [catalog, configured] = await Promise.all([
    fetchCatalog(),
    fetchConfiguredProviders(),
  ]);

  const { upsertEntity } = useGraphStore.getState();

  // Build a set of configured provider ids for quick lookup.
  const configuredIds = new Set(configured.providers.map((p) => p.id));

  // Upsert every catalog provider.
  for (const p of catalog.providers) {
    const entity: ProviderEntity = {
      id: p.id,
      display_name: p.display_name,
      base_url: p.base_url,
      configured: configuredIds.has(p.id) || p.configured,
      auth_env_var: p.auth_env_var,
      endpoints: p.endpoints,
      model_count: p.model_count,
      status: p.status,
      status_detail: p.status_detail,
    };
    upsertEntity("Provider", p.id, entity as unknown as Record<string, unknown>);
  }

  // Upsert any configured providers that were not in the catalog (custom).
  for (const p of configured.providers) {
    if (!catalog.providers.some((cp) => cp.id === p.id)) {
      const entity: ProviderEntity = {
        id: p.id,
        display_name: p.display_name ?? p.id,
        base_url: p.base_url,
        configured: true,
        auth_env_var: undefined,
        endpoints: [],
        model_count: p.models?.length ?? 0,
      };
      upsertEntity("Provider", p.id, entity as unknown as Record<string, unknown>);
    }
  }

  // Upsert the singleton ProviderMeta entity (id "current") with the
  // currently-default provider id and that provider's default model.
  // Consumers read this via useProviderDefault() / useHasWorkingSystemDefault().
  const defaultProvider = configured.providers.find((p) => p.id === configured.default_id);
  upsertEntity("ProviderMeta", "current", {
    id: "current",
    default_id: configured.default_id ?? null,
    default_model: defaultProvider?.default_model ?? null,
  });
}

/**
 * Fetch the full models catalog, extract rows for the given provider,
 * and upsert each as a `Model` entity in the graph.
 */
export async function loadModelsForProvider(providerId: string): Promise<void> {
  const data = await fetchModelsCatalog();
  const rows = modelsRowsForProvider(data, providerId);

  const { upsertEntity } = useGraphStore.getState();

  for (const row of rows) {
    const entity: ModelEntity = {
      id: row.id,
      name: row.name,
      provider_id: providerId,
      context: row.context,
      tool_call: row.tool_call,
      reasoning: row.reasoning,
      vision: false, // catalog does not expose vision per-model; default false
    };
    upsertEntity("Model", row.id, entity as unknown as Record<string, unknown>);
  }
}
