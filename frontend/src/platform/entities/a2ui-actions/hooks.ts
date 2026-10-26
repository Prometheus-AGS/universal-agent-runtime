import { useEffect } from "react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { A2UI_ACTION_ENTITY, a2uiActionId, type A2uiActionReceipt } from "./contracts";
import { reconcileA2uiAction, submitA2uiAction } from "./domain";

export function useA2uiActionStatus(runId: string, surfaceId: string) {
  const status = useGraphStore((state) => (state.entities[A2UI_ACTION_ENTITY]?.[a2uiActionId(runId, surfaceId)] as A2uiActionReceipt | undefined)?.status);
  useEffect(() => { reconcileA2uiAction(runId, surfaceId); }, [runId, surfaceId, status]);
  return status;
}
export function useA2uiActionContinuation(runId: string, surfaceId: string) {
  return useGraphStore((state) => (state.entities[A2UI_ACTION_ENTITY]?.[a2uiActionId(runId, surfaceId)] as A2uiActionReceipt | undefined)?.continuation_run_id);
}
export function useA2uiActionIntent() { return submitA2uiAction; }
