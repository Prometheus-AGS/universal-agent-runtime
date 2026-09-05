import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { PRESENTATION_ENTITY, type Presentation } from "../presentations/contracts";
import { presentationListKey } from "../presentations/domain";
import { assignmentDraft, assignmentOwner, assignmentView, presentationAssignmentActions } from "./domain";
import type { PresentationAssignmentDraft, PresentationAssignmentTarget } from "./contracts";

const EMPTY_IDS: string[] = [];
type DraftField = "dirty" | "status" | "error" | "uncertain" | "conflict" | "recovered";

export function usePresentationAssignmentReady(target: PresentationAssignmentTarget): boolean {
  return useGraphStore((state) => assignmentDraft(state, target) !== null);
}

export function usePresentationAssignmentLoadError(target: PresentationAssignmentTarget): string | null {
  return useGraphStore((state) => assignmentView(state, target)?.error ?? null);
}

export function usePresentationAssignmentField<K extends DraftField>(target: PresentationAssignmentTarget, field: K): PresentationAssignmentDraft[K] | undefined {
  return useGraphStore((state) => assignmentDraft(state, target)?.[field]);
}

export function usePresentationAssignmentMode(target: PresentationAssignmentTarget) {
  return useGraphStore((state) => assignmentDraft(state, target)?.selection.mode);
}

export function usePresentationAssignmentIds(target: PresentationAssignmentTarget, excluded = false): string[] {
  return useGraphStore((state) => assignmentDraft(state, target)?.selection[excluded ? "denied_ids" : "ids"] ?? EMPTY_IDS);
}

export function usePresentationAssignmentMarked(target: PresentationAssignmentTarget, id: string, excluded = false): boolean {
  return useGraphStore((state) => assignmentDraft(state, target)?.selection[excluded ? "denied_ids" : "ids"].includes(id) ?? false);
}

export function usePresentationAssignmentRetainedCount(target: PresentationAssignmentTarget): number {
  return useGraphStore((state) => {
    const draft = assignmentDraft(state, target);
    return draft && draft.selection.mode !== "selected" ? draft.retained_ids.length : 0;
  });
}

export function usePresentationAssignmentMatchCount(target: PresentationAssignmentTarget, search: string): number {
  return useGraphStore((state) => {
    const draft = assignmentDraft(state, target);
    const owner = assignmentOwner(state);
    if (!draft || !owner) return 0;
    const listed = state.lists[presentationListKey(owner)]?.ids ?? EMPTY_IDS;
    const ids = new Set([...listed, ...draft.selection.ids, ...draft.selection.denied_ids]);
    let count = 0;
    for (const id of ids) {
      const record = state.entities[PRESENTATION_ENTITY]?.[id] as Presentation | undefined;
      const title = record?.owner_id === owner && listed.includes(id) ? record.content.title : id;
      if (`${title} ${id}`.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase())) count++;
    }
    return count;
  });
}

export function usePresentationAssignmentActions() { return presentationAssignmentActions; }
