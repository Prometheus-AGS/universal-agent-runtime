import { z } from "zod";

export const presentationSelectionSchema = z.object({
  mode: z.enum(["inherit", "auto", "all", "selected", "none"]).default("inherit"),
  ids: z.array(z.string()).default([]),
  denied_ids: z.array(z.string()).default([]),
}).strict();

export type PresentationSelection = z.infer<typeof presentationSelectionSchema>;
export type PresentationSelectionMode = PresentationSelection["mode"];

export function copyPresentationSelection(value: PresentationSelection | null | undefined): PresentationSelection | null {
  return value ? { mode: value.mode, ids: [...value.ids], denied_ids: [...value.denied_ids] } : null;
}

export const PRESENTATION_ASSIGNMENT_ENTITY = "PresentationAssignment";
export const PRESENTATION_ASSIGNMENT_DRAFT_ENTITY = "PresentationAssignmentDraft";
export const PRESENTATION_ASSIGNMENT_VIEW_ENTITY = "PresentationAssignmentView";

export type PresentationAssignmentTarget = { scope: "global" } | { scope: "agent"; agentId: string };

export interface PresentationAssignment extends Record<string, unknown> {
  id: string;
  owner_id: string;
  target: PresentationAssignmentTarget;
  policy: Record<string, unknown>;
  selection: PresentationSelection;
}

export interface PresentationAssignmentDraft extends PresentationAssignment {
  admission_id: string;
  retained_ids: string[];
  dirty: boolean;
  status: "idle" | "saving" | "error";
  error: string | null;
  uncertain: boolean;
  conflict: boolean;
  recovered: boolean;
}

export interface PresentationAssignmentView extends Record<string, unknown> {
  id: string;
  owner_id: string | null;
  generation: number;
  catalog_generation: number | null;
  status: "loading" | "ready" | "error";
  error: string | null;
}

export function presentationAssignmentId(owner: string, target: PresentationAssignmentTarget): string {
  return JSON.stringify([owner, target.scope, target.scope === "agent" ? target.agentId : ""]);
}
