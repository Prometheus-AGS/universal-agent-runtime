import { useEntityList } from "@prometheus-ags/prometheus-entity-management";
import type { ModelEntity } from "@/entities/types";

/**
 * Live list of all Model entities in the graph.
 *
 * Hydration is the page's responsibility (call `loadModelsIntoGraph()` on
 * mount). SSE-fed graph mutations keep the view fresh thereafter.
 */
export function useModels() {
  return useEntityList<ModelEntity>("Model");
}
