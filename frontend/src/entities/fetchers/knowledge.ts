import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchKnowledgeBases } from "@/services/knowledge-api";

/**
 * Fetch all knowledge bases and upsert each into the entity graph
 * as a `KnowledgeBase` entity.
 */
export async function loadKnowledgeBasesIntoGraph(): Promise<void> {
  const kbs = await fetchKnowledgeBases();
  const { upsertEntity } = useGraphStore.getState();

  for (const kb of kbs) {
    upsertEntity("KnowledgeBase", kb.id, kb as unknown as Record<string, unknown>);
  }
}
