import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchModelsCatalog } from "@/services/models-api";
import type { CatalogModelsResponse } from "@/types";
import type { ModelEntity } from "@/entities/types";

/**
 * Fetch the full models catalog and upsert every (provider, model) pair
 * into the entity graph as a `Model` entity.
 *
 * Entity id format: `<provider_id>/<model_id>` (matches the natural unique
 * identifier the page UI already keys on).
 *
 * Page consumers should call this on mount; SSE-fed graph mutations keep
 * the graph fresh thereafter.
 */
export async function loadModelsIntoGraph(): Promise<void> {
  const data: CatalogModelsResponse = await fetchModelsCatalog();
  const { upsertEntity } = useGraphStore.getState();

  for (const [providerId, providerData] of Object.entries(data)) {
    for (const [modelId, model] of Object.entries(providerData.models)) {
      const key = `${providerId}/${modelId}`;
      const entity: ModelEntity & Record<string, unknown> = {
        id: key,
        name: model.name,
        provider_id: providerId,
        // Display extras that the page needs — augmenting ModelEntity loosely
        // via index signature; the strict fields are still satisfied.
        context: model.limit.context,
        tool_call: model.tool_call,
        reasoning: model.reasoning,
        vision: model.modalities.input.includes("image"),
        // Page-friendly extras (kept under stable keys so the page can read them).
        model_id: modelId,
        provider_name: providerData.display_name,
        provider_configured: providerData.configured,
        cost_input: model.cost.input,
        cost_output: model.cost.output,
        modalities_input: model.modalities.input,
        modalities_output: model.modalities.output,
        family: model.family,
        // CH-10: sourced benchmark scores (CH-09), for the model-comparison view.
        benchmarks: model.benchmarks,
      };
      upsertEntity("Model", key, entity);
    }
  }
}
