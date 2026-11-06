export const A2UI_ACTION_ENTITY = "A2uiActionReceipt";

export interface A2uiActionInput {
  interaction_id?: string;
  /** Exact rendered publication identity. Missing legacy publications are read-only. */
  surface_generation?: string;
  name: string;
  surfaceId: string;
  sourceComponentId: string;
  timestamp: string;
  context: Record<string, unknown>;
  a2uiClientDataModel: unknown;
}

export interface A2uiActionReceipt extends Record<string, unknown> {
  id: string;
  source_run_id: string;
  surface_id: string;
  interaction_id?: string;
  surface_generation?: string;
  input?: A2uiActionInput;
  continuation_run_id: string | null;
  status: "pending" | "responding" | "completed" | "rejected" | "uncertain" | "interrupted";
}

export function a2uiActionId(runId: string, surfaceId: string): string {
  return JSON.stringify([runId, surfaceId]);
}

export interface A2uiSurfaceAuthority extends Record<string, unknown> {
  id: string;
  status: "complete" | "truncated" | "expired" | "unavailable";
  authoritative: boolean;
  recovery_hint: string;
  surfaces: { surface_id: string; surface_generation: string; source_run_id: string }[];
}
export const A2UI_AUTHORITY_ENTITY = "A2uiSurfaceAuthority";
