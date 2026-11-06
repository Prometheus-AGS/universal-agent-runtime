import { useEffect } from "react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { A2UI_ACTION_ENTITY, A2UI_AUTHORITY_ENTITY, type A2uiActionReceipt, type A2uiSurfaceAuthority } from "./contracts";
import { reconcileA2uiAction, submitA2uiAction, loadA2uiSurfaceAuthority } from "./domain";
function latest(entities: Record<string, unknown> | undefined, runId: string, surfaceId: string) {
  return (Object.values(entities ?? {}) as A2uiActionReceipt[]).filter((row) => row.source_run_id === runId && row.surface_id === surfaceId).at(-1);
}
export function useA2uiActionStatus(runId: string, surfaceId: string) {
  const status = useGraphStore((state) => latest(state.entities[A2UI_ACTION_ENTITY], runId, surfaceId)?.status);
  useEffect(() => { reconcileA2uiAction(runId, surfaceId); }, [runId, surfaceId]);
  return status;
}
export function useA2uiActionContinuation(runId: string, surfaceId: string) {
  return useGraphStore((state) => latest(state.entities[A2UI_ACTION_ENTITY], runId, surfaceId)?.continuation_run_id);
}
export function useA2uiSurfaceAuthority(runId: string) {
  const authority = useGraphStore((state) => state.entities[A2UI_AUTHORITY_ENTITY]?.[runId] as A2uiSurfaceAuthority | undefined);
  useEffect(() => { if (runId) void loadA2uiSurfaceAuthority(runId); }, [runId]);
  return authority;
}
export function useA2uiActionIntent() { return submitA2uiAction; }
export function useA2uiActionRecovery() {
  return (runId: string, surfaceId: string) => { reconcileA2uiAction(runId, surfaceId); void loadA2uiSurfaceAuthority(runId); };
}
