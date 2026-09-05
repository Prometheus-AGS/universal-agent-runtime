import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { PRESENTATION_CATALOG_ENTITY, PRESENTATION_DRAFT_ENTITY, PRESENTATION_ENTITY, type Presentation, type PresentationCatalog, type PresentationDraft } from "./contracts";
import { PRESENTATION_ADMISSION_ID, presentationActions, presentationDraftId, presentationListKey } from "./domain";

const EMPTY_IDS: string[] = [];
type CatalogField = "owner_id" | "status" | "error" | "editor_open";
type DraftField = "title" | "description" | "enabled" | "source" | "dirty" | "status" | "error" | "uncertain" | "conflict" | "presentation_id" | "expected_revision";
type RowFields = Pick<Presentation["content"], "title" | "description" | "enabled"> & Pick<Presentation, "revision">;

export function usePresentationCatalogField<K extends CatalogField>(field: K): PresentationCatalog[K] | undefined {
  return useGraphStore((state) => (state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined)?.[field]);
}

export function usePresentationIds(): string[] {
  return useGraphStore((state) => {
    const session = state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
    return session?.status === "ready" && session.owner_id
      ? state.lists[presentationListKey(session.owner_id)]?.ids ?? EMPTY_IDS : EMPTY_IDS;
  });
}

export function usePresentationMatchCount(search: string): number {
  return useGraphStore((state) => {
    const session = state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
    if (session?.status !== "ready" || !session.owner_id) return 0;
    const ids = state.lists[presentationListKey(session.owner_id)]?.ids ?? EMPTY_IDS;
    return ids.reduce((count, id) => {
      const record = state.entities[PRESENTATION_ENTITY]?.[id] as Presentation | undefined;
      return count + (record?.owner_id === session.owner_id && record.content.title.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase()) ? 1 : 0);
    }, 0);
  });
}

export function usePresentationField<K extends keyof RowFields>(id: string, field: K): RowFields[K] | undefined {
  return useGraphStore((state) => {
    const session = state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
    const record = state.entities[PRESENTATION_ENTITY]?.[id] as Presentation | undefined;
    if (session?.status !== "ready" || !record || record.owner_id !== session.owner_id) return undefined;
    if (!session.owner_id || !state.lists[presentationListKey(session.owner_id)]?.ids.includes(id)) return undefined;
    const fields: RowFields = { title: record.content.title, description: record.content.description, enabled: record.content.enabled, revision: record.revision };
    return fields[field];
  });
}

export function usePresentationDraftField<K extends DraftField>(field: K): PresentationDraft[K] | undefined {
  return useGraphStore((state) => {
    const session = state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
    if (session?.status !== "ready" || !session.owner_id) return undefined;
    const draft = state.entities[PRESENTATION_DRAFT_ENTITY]?.[presentationDraftId(session.owner_id)] as PresentationDraft | undefined;
    return draft?.owner_id === session.owner_id ? draft[field] : undefined;
  });
}

export function usePresentationActions() { return presentationActions; }
