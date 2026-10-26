export const A2UI_ACTION_ENTITY = "A2uiActionReceipt";

export interface A2uiActionInput {
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
  continuation_run_id: string | null;
  status: "pending" | "responding" | "completed" | "rejected" | "uncertain" | "interrupted";
}

export function a2uiActionId(runId: string, surfaceId: string): string {
  return JSON.stringify([runId, surfaceId]);
}
