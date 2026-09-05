import { registerEntityTransport, registerSchema } from "@prometheus-ags/prometheus-entity-management";
import { fetchPresentations } from "./api/presentations-api";
import { warnBeforePresentationExit } from "./domain";
import { PRESENTATION_CATALOG_ENTITY, PRESENTATION_DRAFT_ENTITY, PRESENTATION_ENTITY, type Presentation } from "./contracts";

let registered = false;

export function registerPresentationEntities(): void {
  if (registered) return;
  registerSchema({ type: PRESENTATION_ENTITY });
  registerSchema({ type: PRESENTATION_DRAFT_ENTITY });
  registerSchema({ type: PRESENTATION_CATALOG_ENTITY });
  registerEntityTransport<Presentation>(PRESENTATION_ENTITY, {
    identify: (record) => record.id,
    authoritative: false,
    list: async () => {
      const catalog = await fetchPresentations();
      return { rows: catalog.presentations, total: catalog.presentations.length, nextCursor: null };
    },
  });
  // Drafts never acquire a remote mutation/replay transport.
  window.addEventListener("beforeunload", warnBeforePresentationExit);
  registered = true;
}
