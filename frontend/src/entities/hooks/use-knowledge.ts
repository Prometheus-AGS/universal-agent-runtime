import { useMemo } from "react";
import { useEntityView, useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import type { ViewDescriptor } from "@prometheus-ags/prometheus-entity-management";
import type { KnowledgeBaseEntity, DocumentEntity } from "@/entities/types";

const EMPTY_KNOWLEDGE_BASES: KnowledgeBaseEntity[] = [];
const EMPTY_DOCUMENTS: DocumentEntity[] = [];

/**
 * Live, filterable view of all KnowledgeBase entities in the graph.
 *
 * Sorted alphabetically by name.
 * Supports optional free-text search across name and description.
 */
export function useKnowledgeBases(searchTerm?: string) {
  const view = useMemo<ViewDescriptor>(
    () => ({
      sort: [{ field: "name", direction: "asc" as const }],
      search:
        searchTerm && searchTerm.length > 0
          ? { query: searchTerm, fields: ["name", "description"], minChars: 1 }
          : undefined,
    }),
    [searchTerm],
  );

  return useEntityView<KnowledgeBaseEntity>({
    type: "KnowledgeBase",
    baseQueryKey: ["knowledge-bases", searchTerm ?? ""],
    view,
    mode: "local",
  });
}

/**
 * Returns a single KnowledgeBase entity by id, read directly from the graph.
 *
 * Uses `useGraphStore` selector so the component re-renders only when
 * that KB's data changes. Returns null when the KB is not loaded.
 */
export function useKnowledgeBase(
  id: string | undefined,
): KnowledgeBaseEntity | null {
  return useGraphStore((state) => {
    if (!id) return null;

    const kbMap = state.entities["KnowledgeBase"];
    if (!kbMap) return null;

    const entity = kbMap[id];
    if (!entity) return null;

    return entity as unknown as KnowledgeBaseEntity;
  });
}

/**
 * Returns all Document entities for a given knowledge base, read directly
 * from the graph.
 *
 * Uses `useGraphStore` selector so the component re-renders only when that
 * KB's document slice changes. Returns a stable empty array when no
 * documents are loaded yet (avoids the Zustand infinite-render bug).
 */
export function useDocuments(kbId: string | undefined): DocumentEntity[] {
  return useGraphStore((state) => {
    if (!kbId) return EMPTY_DOCUMENTS;

    const docMap = state.entities["Document"];
    if (!docMap) return EMPTY_DOCUMENTS;

    const results: DocumentEntity[] = [];
    for (const id of Object.keys(docMap)) {
      const entity = docMap[id];
      if (entity && (entity as Record<string, unknown>)["kb_id"] === kbId) {
        results.push(entity as unknown as DocumentEntity);
      }
    }

    return results.length > 0 ? results : EMPTY_DOCUMENTS;
  });
}

/**
 * Returns all KnowledgeBase entities from the graph without view processing.
 *
 * Useful for dropdowns and selection lists where the full `useEntityView`
 * machinery is unnecessary. Returns a stable empty array when no KBs exist.
 */
export function useAllKnowledgeBases(): KnowledgeBaseEntity[] {
  return useGraphStore((state) => {
    const kbMap = state.entities["KnowledgeBase"];
    if (!kbMap) return EMPTY_KNOWLEDGE_BASES;

    const results = Object.values(kbMap) as unknown as KnowledgeBaseEntity[];
    return results.length > 0 ? results : EMPTY_KNOWLEDGE_BASES;
  });
}
