import { graphStore, registerEntityTransport, registerSchema } from "@prometheus-ags/prometheus-entity-management";
import { z } from "zod";
import { fetchAssignment } from "./api";
import { assignmentOwner, warnBeforeAssignmentExit } from "./domain";
import { PRESENTATION_ASSIGNMENT_DRAFT_ENTITY, PRESENTATION_ASSIGNMENT_ENTITY, PRESENTATION_ASSIGNMENT_VIEW_ENTITY, type PresentationAssignment } from "./contracts";

let registered = false;
export function registerPresentationAssignmentEntities(): void {
  if (registered) return;
  registerSchema({ type: PRESENTATION_ASSIGNMENT_ENTITY });
  registerSchema({ type: PRESENTATION_ASSIGNMENT_DRAFT_ENTITY });
  registerSchema({ type: PRESENTATION_ASSIGNMENT_VIEW_ENTITY });
  registerEntityTransport<PresentationAssignment>(PRESENTATION_ASSIGNMENT_ENTITY, {
    identify: (record) => record.id, authoritative: false,
    list: async () => ({ rows: [], total: 0, nextCursor: null }),
    get: async (id) => {
      const [owner, scope, agentId] = z.tuple([z.string(), z.enum(["global", "agent"]), z.string()]).parse(JSON.parse(id));
      if (assignmentOwner(graphStore.getState()) !== owner) throw new Error("Assignment owner not admitted");
      const record = await fetchAssignment(owner, scope === "global" ? { scope } : { scope, agentId });
      if (assignmentOwner(graphStore.getState()) !== owner) throw new Error("Assignment owner changed");
      return record;
    },
  });
  window.addEventListener("beforeunload", warnBeforeAssignmentExit);
  registered = true;
}
