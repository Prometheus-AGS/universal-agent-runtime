import { useGraphEntities } from "@/entities/hooks/use-graph-entities";
import type { ModelEntity } from "@/entities/types";

/**
 * Live list of all Model entities in the graph.
 *
 * Hydration is the page's responsibility (call `loadModelsIntoGraph()` on
 * mount). SSE-fed graph mutations keep the view fresh thereafter.
 */
export function useModels(): { items: ModelEntity[] } {
  return { items: useGraphEntities<ModelEntity>("Model") };
}
