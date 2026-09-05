import { registerSchema } from "@prometheus-ags/prometheus-entity-management";
import { PRESENTATION_PROVENANCE_ENTITY } from "./contracts";

let registered = false;
export function registerPresentationProvenanceEntities(): void {
  if (registered) return;
  registerSchema({ type: PRESENTATION_PROVENANCE_ENTITY });
  registered = true;
}
