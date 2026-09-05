import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { PRESENTATION_ADMISSION_ID, presentationActions, presentationListKey } from "../presentations/domain";
import { PRESENTATION_CATALOG_ENTITY, PRESENTATION_ENTITY, type Presentation, type PresentationCatalog } from "../presentations/contracts";
import { AssignmentApiError, fetchAssignment, saveAssignment } from "./api";
import { PRESENTATION_ASSIGNMENT_DRAFT_ENTITY, PRESENTATION_ASSIGNMENT_ENTITY, PRESENTATION_ASSIGNMENT_VIEW_ENTITY, copyPresentationSelection, presentationAssignmentId, presentationSelectionSchema,
  type PresentationAssignment, type PresentationAssignmentDraft, type PresentationAssignmentTarget, type PresentationAssignmentView, type PresentationSelectionMode } from "./contracts";

type GraphState = ReturnType<typeof graphStore.getState>;
const inFlight = new Map<string, Promise<void>>();
const protectedDraftIds = new Set<string>();
const pendingWrites = new Set<string>();

export const assignmentViewId = (target: PresentationAssignmentTarget) => presentationAssignmentId(PRESENTATION_ADMISSION_ID, target);

export function assignmentOwner(state: GraphState): string | null {
  const catalog = state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
  return catalog?.status === "ready" ? catalog.owner_id : null;
}

export function assignmentView(state: GraphState, target: PresentationAssignmentTarget): PresentationAssignmentView | undefined {
  return state.entities[PRESENTATION_ASSIGNMENT_VIEW_ENTITY]?.[assignmentViewId(target)] as PresentationAssignmentView | undefined;
}

export function assignmentDraft(state: GraphState, target: PresentationAssignmentTarget): PresentationAssignmentDraft | null {
  const owner = assignmentOwner(state);
  const view = assignmentView(state, target);
  const catalog = state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
  if (!owner || view?.status !== "ready" || view.owner_id !== owner || view.catalog_generation !== catalog?.generation) return null;
  const draft = state.entities[PRESENTATION_ASSIGNMENT_DRAFT_ENTITY]?.[presentationAssignmentId(owner, target)] as PresentationAssignmentDraft | undefined;
  return draft?.owner_id === owner && draft.admission_id === PRESENTATION_ADMISSION_ID ? draft : null;
}

function publishDraft(draft: PresentationAssignmentDraft): void {
  if (draft.dirty || draft.status === "saving") protectedDraftIds.add(draft.id);
  else protectedDraftIds.delete(draft.id);
  graphStore.getState().replaceEntity(PRESENTATION_ASSIGNMENT_DRAFT_ENTITY, draft.id, draft);
}

function fromRecord(record: PresentationAssignment): PresentationAssignmentDraft {
  return { ...record, selection: copyPresentationSelection(record.selection)!, admission_id: PRESENTATION_ADMISSION_ID,
    retained_ids: [...record.selection.ids], dirty: false, status: "idle", error: null, uncertain: false, conflict: false, recovered: false };
}

function current(owner: string, target: PresentationAssignmentTarget, generation: number): boolean {
  const state = graphStore.getState();
  const view = assignmentView(state, target);
  const catalog = state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
  return assignmentOwner(state) === owner && view?.owner_id === owner && view.generation === generation && view.catalog_generation === catalog?.generation;
}

function publishView(view: PresentationAssignmentView): void {
  graphStore.getState().replaceEntity(PRESENTATION_ASSIGNMENT_VIEW_ENTITY, view.id, view);
}

function markStaleAdmission(target: PresentationAssignmentTarget, generation: number): void {
  const view = assignmentView(graphStore.getState(), target);
  if (view?.generation === generation) {
    publishView({ ...view, owner_id: null, status: "error", error: "Owner verification changed. Reload assignment to restore access. Your draft is retained." });
  }
}

async function load(target: PresentationAssignmentTarget, discard = false): Promise<void> {
  const id = assignmentViewId(target);
  if (pendingWrites.has(id)) return;
  const generation = (assignmentView(graphStore.getState(), target)?.generation ?? 0) + 1;
  publishView({ id, generation, catalog_generation: null, owner_id: null, status: "loading", error: null });
  await presentationActions.ensureLoaded();
  const owner = assignmentOwner(graphStore.getState());
  if (assignmentView(graphStore.getState(), target)?.generation !== generation) return;
  if (!owner) {
    publishView({ id, generation, catalog_generation: null, owner_id: null, status: "error", error: "Reload the catalog to verify its owner. Your assignment draft is retained." });
    return;
  }
  const catalog_generation = (graphStore.getState().entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog).generation;
  publishView({ id, generation, catalog_generation, owner_id: owner, status: "loading", error: null });
  try {
    const record = await fetchAssignment(owner, target);
    if (!current(owner, target, generation)) { markStaleAdmission(target, generation); return; }
    const state = graphStore.getState();
    state.replaceEntity(PRESENTATION_ASSIGNMENT_ENTITY, record.id, record);
    const previous = state.readEntity<PresentationAssignmentDraft>(PRESENTATION_ASSIGNMENT_DRAFT_ENTITY, record.id);
    if (!discard && previous?.owner_id === owner && (previous.dirty || previous.status === "saving")) {
      publishDraft({ ...previous, admission_id: PRESENTATION_ADMISSION_ID,
        recovered: previous.recovered || previous.admission_id !== PRESENTATION_ADMISSION_ID,
        uncertain: previous.uncertain || previous.status === "saving",
        status: previous.status === "saving" ? "error" : previous.status,
        error: previous.status === "saving" ? "A previous save was interrupted. Check saved assignment before saving again." : previous.error });
    } else publishDraft({ ...fromRecord(record), retained_ids: !discard && previous?.owner_id === owner && record.selection.mode !== "selected" ? previous.retained_ids : [...record.selection.ids] });
    publishView({ id, generation, catalog_generation, owner_id: owner, status: "ready", error: null });
  } catch (error) {
    if (!current(owner, target, generation)) { markStaleAdmission(target, generation); return; }
    publishView({ id, generation, catalog_generation, owner_id: null, status: "error", error: error instanceof AssignmentApiError ? error.message : "Assignment response is invalid. No policy was loaded." });
  }
}

function ensureLoaded(target: PresentationAssignmentTarget): Promise<void> {
  if (assignmentDraft(graphStore.getState(), target)) return Promise.resolve();
  const id = assignmentViewId(target);
  const pending = inFlight.get(id);
  if (pending) return pending;
  const operation = load(target).finally(() => { if (inFlight.get(id) === operation) inFlight.delete(id); });
  inFlight.set(id, operation);
  return operation;
}

function editable(target: PresentationAssignmentTarget): PresentationAssignmentDraft | null {
  const draft = assignmentDraft(graphStore.getState(), target);
  return draft && draft.status !== "saving" && !draft.uncertain && !draft.conflict && !draft.recovered ? draft : null;
}

function setMode(target: PresentationAssignmentTarget, mode: PresentationSelectionMode): void {
  const draft = editable(target);
  if (!draft) return;
  const retained = draft.selection.mode === "selected" ? draft.selection.ids : draft.retained_ids;
  publishDraft({ ...draft, retained_ids: [...retained], selection: { ...draft.selection, mode, ids: mode === "selected" ? [...retained] : [] }, dirty: true, status: "idle", error: null });
}

function toggle(target: PresentationAssignmentTarget, id: string, excluded = false): void {
  const draft = editable(target);
  if (!draft || (!excluded && draft.selection.mode !== "selected")) return;
  const field = excluded ? "denied_ids" : "ids";
  const removing = draft.selection[field].includes(id);
  const state = graphStore.getState();
  const record = state.readEntity<Presentation>(PRESENTATION_ENTITY, id);
  if (!removing && (!record || record.owner_id !== draft.owner_id || !record.content.enabled || !state.lists[presentationListKey(draft.owner_id)]?.ids.includes(id))) return;
  publishDraft({ ...draft, selection: { ...draft.selection, [field]: removing ? draft.selection[field].filter((value) => value !== id) : [...draft.selection[field], id] }, dirty: true, status: "idle", error: null });
}

function reset(target: PresentationAssignmentTarget): void {
  const draft = editable(target);
  if (draft) publishDraft({ ...draft, selection: { mode: "inherit", ids: [], denied_ids: [] }, retained_ids: [], dirty: true, status: "idle", error: null });
}

async function save(target: PresentationAssignmentTarget): Promise<boolean> {
  const draft = editable(target);
  const view = assignmentView(graphStore.getState(), target);
  if (!draft || !draft.dirty || !view) return false;
  pendingWrites.add(view.id);
  publishDraft({ ...draft, status: "saving", error: null });
  try {
    const baseline: PresentationAssignment = { ...draft, selection: copyPresentationSelection(
      // The baseline is saved intent, not the edited selection.
      presentationSelectionSchema.parse(draft.policy.presentations ?? {}))! };
    if (target.scope === "agent") {
      const latest = await fetchAssignment(draft.owner_id, target);
      if (JSON.stringify(latest.selection) !== JSON.stringify(baseline.selection)) {
        throw new AssignmentApiError("The agent's assignment changed. Reload saved assignment before saving your draft.", 409);
      }
    }
    if (!current(draft.owner_id, target, view.generation)) throw new AssignmentApiError("Owner admission changed. Reload before saving.", 403);
    const saved = await saveAssignment(baseline, draft.selection);
    if (!current(draft.owner_id, target, view.generation)) {
      publishDraft({ ...draft, status: "error", uncertain: true, error: "Admission changed during save. Check saved assignment after verifying the owner." });
      markStaleAdmission(target, view.generation);
      return false;
    }
    graphStore.getState().replaceEntity(PRESENTATION_ASSIGNMENT_ENTITY, saved.id, saved);
    publishDraft({ ...fromRecord(saved), retained_ids: saved.selection.mode === "selected" ? [...saved.selection.ids] : [...draft.retained_ids] });
    return true;
  } catch (error) {
    const apiError = error instanceof AssignmentApiError ? error : null;
    publishDraft({ ...draft, status: "error", error: apiError?.message ?? "Save could not be confirmed. Check saved assignment before saving again.", uncertain: apiError?.uncertain ?? true, conflict: apiError?.status === 409 });
    if (current(draft.owner_id, target, view.generation) && (apiError?.status === 401 || apiError?.status === 403)) {
      publishView({ ...view, owner_id: null, status: "error", error: apiError.message });
    } else if (!current(draft.owner_id, target, view.generation)) markStaleAdmission(target, view.generation);
    return false;
  } finally {
    pendingWrites.delete(view.id);
  }
}

async function checkSaved(target: PresentationAssignmentTarget): Promise<void> {
  const draft = assignmentDraft(graphStore.getState(), target);
  const view = assignmentView(graphStore.getState(), target);
  if (!draft || !view || draft.status === "saving" || !draft.uncertain) return;
  pendingWrites.add(view.id);
  publishDraft({ ...draft, status: "saving", error: null });
  try {
    const saved = await fetchAssignment(draft.owner_id, target);
    if (!current(draft.owner_id, target, view.generation)) {
      publishDraft({ ...draft, status: "error", error: "Owner admission changed. Your draft remains retained." });
      markStaleAdmission(target, view.generation);
      return;
    }
    graphStore.getState().replaceEntity(PRESENTATION_ASSIGNMENT_ENTITY, saved.id, saved);
    publishDraft({ ...draft, policy: saved.policy, status: "idle", uncertain: false, conflict: false,
      error: "Saved assignment checked. Your draft is retained; review it before saving again." });
  } catch (error) {
    publishDraft({ ...draft, status: "error", error: error instanceof AssignmentApiError ? error.message : "Saved assignment is still unavailable. Your draft is retained." });
    if (current(draft.owner_id, target, view.generation) && error instanceof AssignmentApiError && (error.status === 401 || error.status === 403)) {
      publishView({ ...view, owner_id: null, status: "error", error: error.message });
    } else if (!current(draft.owner_id, target, view.generation)) markStaleAdmission(target, view.generation);
  } finally {
    pendingWrites.delete(view.id);
  }
}

export function warnBeforeAssignmentExit(event: BeforeUnloadEvent): void {
  if ([...protectedDraftIds].some((id) => {
    const draft = graphStore.getState().readEntity<PresentationAssignmentDraft>(PRESENTATION_ASSIGNMENT_DRAFT_ENTITY, id);
    return draft?.dirty || draft?.status === "saving";
  })) { event.preventDefault(); event.returnValue = ""; }
}

export const presentationAssignmentActions = {
  ensureLoaded, reload: (target: PresentationAssignmentTarget) => load(target),
  discard: (target: PresentationAssignmentTarget) => load(target, true), setMode, toggle, reset, save, checkSaved,
  resume: (target: PresentationAssignmentTarget) => {
    const draft = assignmentDraft(graphStore.getState(), target);
    if (draft && draft.status !== "saving") publishDraft({ ...draft, recovered: false });
  },
};
