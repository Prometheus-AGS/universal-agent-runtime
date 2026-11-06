import { registerSchema } from "@prometheus-ags/prometheus-entity-management";
import { A2UI_ACTION_ENTITY, A2UI_AUTHORITY_ENTITY } from "./contracts";
let registered = false;
export function registerA2uiActionEntities(): void {
  if (registered) return;
  registerSchema({ type: A2UI_ACTION_ENTITY });
  registerSchema({ type: A2UI_AUTHORITY_ENTITY });
  registered = true;
}
export { bindA2uiActionPersistence } from "./durability";
export { useA2uiActionStatus, useA2uiActionContinuation, useA2uiActionIntent, useA2uiSurfaceAuthority, useA2uiActionRecovery } from "./hooks";
