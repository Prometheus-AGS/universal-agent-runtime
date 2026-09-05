import { useEffect } from "react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { PRESENTATION_PROVENANCE_ENTITY, type PresentationObservation, type PresentationProvenance } from "./contracts";
import { PRESENTATION_PROVENANCE_ADMISSION, presentationProvenanceActions } from "./domain";

export function usePresentationProvenanceSubscription(runId: string): void {
  useEffect(() => presentationProvenanceActions.subscribe(runId), [runId]);
}

export function usePresentationProvenanceStatus(runId: string): PresentationProvenance["status"] {
  return useGraphStore((state) => {
    const record = state.entities[PRESENTATION_PROVENANCE_ENTITY]?.[runId] as PresentationProvenance | undefined;
    return record?.admission_id === PRESENTATION_PROVENANCE_ADMISSION ? record.status : "loading";
  });
}

export function usePresentationProvenanceField<K extends keyof PresentationObservation>(runId: string, field: K): PresentationObservation[K] | undefined {
  return useGraphStore((state) => {
    const record = state.entities[PRESENTATION_PROVENANCE_ENTITY]?.[runId] as PresentationProvenance | undefined;
    return record?.admission_id === PRESENTATION_PROVENANCE_ADMISSION && record.status === "ready"
      ? record.observation?.[field] : undefined;
  });
}

export function usePresentationProvenanceActions() { return presentationProvenanceActions; }
