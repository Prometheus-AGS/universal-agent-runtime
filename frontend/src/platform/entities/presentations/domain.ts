import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { deletePresentation, fetchPresentations, PresentationApiError, savePresentation } from "./api/presentations-api";
import {
  PRESENTATION_CATALOG_ENTITY,
  PRESENTATION_DRAFT_ENTITY,
  PRESENTATION_ENTITY,
  STARTER_PRESENTATION_SOURCE,
  parsePresentationSource,
  type Presentation,
  type PresentationCatalog,
  type PresentationDraft,
  type PresentationDraftField,
} from "./contracts";

// Hydrated state cannot manufacture admission for this fresh application session.
export const PRESENTATION_ADMISSION_ID = crypto.randomUUID();
export const presentationListKey = (owner: string) => `presentations:${owner.length}:${owner}`;
export const presentationDraftId = (owner: string) => `presentation-editor:${owner.length}:${owner}`;
// IDs only: exit protection survives a failed re-admission without exposing records.
const exitProtectedDraftIds = new Set<string>();

function catalog(): PresentationCatalog | undefined {
  return graphStore.getState().readEntity<PresentationCatalog>(PRESENTATION_CATALOG_ENTITY, PRESENTATION_ADMISSION_ID) ?? undefined;
}

function publishCatalog(value: PresentationCatalog): void {
  graphStore.getState().replaceEntity(PRESENTATION_CATALOG_ENTITY, PRESENTATION_ADMISSION_ID, value);
}

function admittedOwner(): string | null {
  const current = catalog();
  return current?.status === "ready" ? current.owner_id : null;
}

function draftFor(owner: string): PresentationDraft | null {
  const draft = graphStore.getState().readEntity<PresentationDraft>(PRESENTATION_DRAFT_ENTITY, presentationDraftId(owner));
  return draft?.owner_id === owner ? draft : null;
}

function currentDraft(): PresentationDraft | null {
  const owner = admittedOwner();
  return owner ? draftFor(owner) : null;
}

export function warnBeforePresentationExit(event: BeforeUnloadEvent): void {
  const hasUnsavedWork = [...exitProtectedDraftIds].some((id) => {
    const draft = graphStore.getState().readEntity<PresentationDraft>(PRESENTATION_DRAFT_ENTITY, id);
    return draft?.dirty || draft?.status === "saving";
  });
  if (hasUnsavedWork) {
    event.preventDefault();
    event.returnValue = "";
  }
}

function publishDraft(draft: PresentationDraft): void {
  if (draft.dirty || draft.status === "saving") exitProtectedDraftIds.add(draft.id);
  else exitProtectedDraftIds.delete(draft.id);
  graphStore.getState().replaceEntity(PRESENTATION_DRAFT_ENTITY, draft.id, draft);
}

function draftFrom(owner: string, record?: Presentation): PresentationDraft {
  return {
    id: presentationDraftId(owner), owner_id: owner,
    presentation_id: record?.id ?? null, expected_revision: record?.revision ?? null,
    title: record?.content.title ?? "", description: record?.content.description ?? "",
    enabled: record?.content.enabled ?? true,
    source: record ? JSON.stringify(record.content.template, null, 2) : STARTER_PRESENTATION_SOURCE,
    dirty: false, status: "idle", error: null, uncertain: false, conflict: false,
  };
}

let loading: Promise<void> | null = null;

async function loadCatalog(): Promise<void> {
  if (currentDraft()?.status === "saving") return;
  const generation = (catalog()?.generation ?? 0) + 1;
  publishCatalog({ id: PRESENTATION_ADMISSION_ID, owner_id: null, generation, status: "loading", error: null, editor_open: false });
  try {
    const response = await fetchPresentations();
    if (catalog()?.generation !== generation) return;
    const state = graphStore.getState();
    state.upsertEntities(PRESENTATION_ENTITY, response.presentations.map((record) => ({ id: record.id, data: record })));
    state.setListResult(presentationListKey(response.owner_id), response.presentations.map((record) => record.id), { total: response.presentations.length });
    const recovered = draftFor(response.owner_id);
    if (recovered?.dirty) exitProtectedDraftIds.add(recovered.id);
    if (recovered?.status === "saving") {
      publishDraft({ ...recovered, dirty: true, status: "error", uncertain: true,
        error: "The previous save was interrupted. Check the catalog before allowing another save." });
    }
    publishCatalog({ id: PRESENTATION_ADMISSION_ID, owner_id: response.owner_id, generation, status: "ready", error: null, editor_open: false });
  } catch (error) {
    if (catalog()?.generation !== generation) return;
    publishCatalog({ id: PRESENTATION_ADMISSION_ID, owner_id: null, generation, status: "error", editor_open: false,
      error: error instanceof PresentationApiError ? error.message : "The catalog response could not be read. No records were loaded." });
  }
}

function reload(): Promise<void> {
  loading ??= loadCatalog().finally(() => { loading = null; });
  return loading;
}

function begin(id?: string): boolean {
  const owner = admittedOwner();
  const session = catalog();
  if (!owner || !session || currentDraft()?.dirty || currentDraft()?.status === "saving") return false;
  const record = id ? graphStore.getState().readEntity<Presentation>(PRESENTATION_ENTITY, id) : undefined;
  if (id && (!record || record.owner_id !== owner)) return false;
  if (id && !graphStore.getState().lists[presentationListKey(owner)]?.ids.includes(id)) return false;
  publishDraft(draftFrom(owner, record ?? undefined));
  publishCatalog({ ...session, editor_open: true });
  return true;
}

function edit<K extends PresentationDraftField>(field: K, value: PresentationDraft[K]): void {
  const draft = currentDraft();
  if (!draft || draft.status === "saving") return;
  publishDraft({ ...draft, [field]: value, dirty: true, status: "idle", error: null });
}

function failure(draft: PresentationDraft, generation: number, error: unknown): void {
  const apiError = error instanceof PresentationApiError ? error : null;
  // Retain the submitted draft even on revoked admission, without exposing it.
  publishDraft({ ...draft, status: "error", dirty: true,
    error: apiError?.message ?? "The result could not be confirmed. Reload the catalog before saving again.",
    uncertain: apiError?.uncertain ?? true, conflict: apiError?.status === 409 });
  const session = catalog();
  if (session?.generation === generation && (apiError?.status === 401 || apiError?.status === 403)) {
    publishCatalog({ ...session, owner_id: null, editor_open: false, status: "error", error: apiError.message });
  }
}

async function save(): Promise<boolean> {
  const draft = currentDraft();
  const session = catalog();
  if (!draft || !session || draft.status === "saving" || draft.uncertain || draft.conflict) return false;
  const parsed = parsePresentationSource(draft.source);
  if (!draft.title.trim() || !parsed.template) {
    publishDraft({ ...draft, status: "error", error: !draft.title.trim() ? "Enter a title before saving." : parsed.error });
    return false;
  }
  publishDraft({ ...draft, status: "saving", error: null });
  try {
    const record = await savePresentation(draft.presentation_id, draft.expected_revision, {
      title: draft.title, description: draft.description, enabled: draft.enabled, template: parsed.template,
    });
    if (record.owner_id !== draft.owner_id || (draft.presentation_id && record.id !== draft.presentation_id)) {
      throw new PresentationApiError("The saved record identity could not be verified. Reload the catalog.", 403, true);
    }
    if (catalog()?.generation !== session.generation || admittedOwner() !== draft.owner_id) return false;
    const state = graphStore.getState();
    state.replaceEntity(PRESENTATION_ENTITY, record.id, record);
    const key = presentationListKey(draft.owner_id);
    const ids = state.lists[key]?.ids ?? [];
    const next = ids.includes(record.id) ? ids : [...ids, record.id];
    state.setListResult(key, next, { total: next.length });
    publishDraft({ ...draftFrom(draft.owner_id, record), status: "saved" });
    return true;
  } catch (error) {
    failure(draft, session.generation, error);
    return false;
  }
}

async function remove(): Promise<boolean> {
  const draft = currentDraft();
  const session = catalog();
  if (!draft?.presentation_id || !draft.expected_revision || !session || draft.status === "saving" || draft.uncertain || draft.conflict) return false;
  publishDraft({ ...draft, status: "saving", error: null });
  try {
    await deletePresentation(draft.presentation_id, draft.expected_revision);
    if (catalog()?.generation !== session.generation || admittedOwner() !== draft.owner_id) return false;
    const state = graphStore.getState();
    state.removeEntity(PRESENTATION_ENTITY, draft.presentation_id);
    state.removeIdFromAllLists(PRESENTATION_ENTITY, draft.presentation_id);
    state.removeEntity(PRESENTATION_DRAFT_ENTITY, draft.id);
    exitProtectedDraftIds.delete(draft.id);
    publishCatalog({ ...session, editor_open: false });
    return true;
  } catch (error) {
    failure(draft, session.generation, error);
    return false;
  }
}

export const presentationActions = {
  ensureLoaded: () => catalog()?.status === "ready" ? Promise.resolve() : reload(),
  reload, begin, edit, save, remove,
  resume: () => {
    const session = catalog();
    if (session?.status === "ready" && currentDraft()) publishCatalog({ ...session, editor_open: true });
  },
  close: (discard: boolean) => {
    const session = catalog();
    const draft = currentDraft();
    if (!session || draft?.status === "saving") return;
    if (discard && draft) {
      graphStore.getState().removeEntity(PRESENTATION_DRAFT_ENTITY, draft.id);
      exitProtectedDraftIds.delete(draft.id);
    }
    publishCatalog({ ...session, editor_open: false });
  },
  acknowledgeUncertainResult: () => {
    const draft = currentDraft();
    if (draft?.uncertain && draft.status !== "saving") publishDraft({ ...draft, uncertain: false, error: null, status: "idle" });
  },
  reloadSavedVersion: async () => {
    const draft = currentDraft();
    if (!draft?.presentation_id || draft.status === "saving") return;
    await reload();
    if (admittedOwner() !== draft.owner_id) return;
    const record = graphStore.getState().readEntity<Presentation>(PRESENTATION_ENTITY, draft.presentation_id);
    const ids = graphStore.getState().lists[presentationListKey(draft.owner_id)]?.ids ?? [];
    const session = catalog();
    if (record?.owner_id === draft.owner_id && ids.includes(record.id) && session) {
      publishDraft(draftFrom(draft.owner_id, record));
      publishCatalog({ ...session, editor_open: true });
    }
  },
};
