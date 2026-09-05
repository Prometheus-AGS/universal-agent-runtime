import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import type { PersistedRunSnapshot } from "@/platform/pglite/run-event-repository";
import { subscribePresentationHistory } from "./api";
import { decodePresentationProvenance, PRESENTATION_PROVENANCE_ENTITY, type PresentationProvenance } from "./contracts";

export const PRESENTATION_PROVENANCE_ADMISSION = crypto.randomUUID();
interface SubscriptionLease {
  users: number;
  generation: number;
  unsubscribe?: () => Promise<void>;
}
// Subscription mechanics only. All recorded observations live in the entity graph.
const leases = new Map<string, SubscriptionLease>();

function write(record: PresentationProvenance): void {
  graphStore.getState().replaceEntity(PRESENTATION_PROVENANCE_ENTITY, record.id, record);
}

function reset(runId: string, status: PresentationProvenance["status"]): void {
  write({ id: runId, admission_id: PRESENTATION_PROVENANCE_ADMISSION, status,
    source_event_id: null, source_sequence: null, observation: null });
}

function dispose(unsubscribe: (() => Promise<void>) | undefined): void {
  if (unsubscribe) void unsubscribe().catch(() => console.warn("[presentation-provenance] Could not release local history subscription"));
}

function start(runId: string, lease: SubscriptionLease): void {
  const generation = ++lease.generation;
  dispose(lease.unsubscribe);
  lease.unsubscribe = undefined;
  reset(runId, "loading");
  const active = () => leases.get(runId) === lease && lease.generation === generation;
  let callbackObserved = false;
  const ingest = (snapshot: PersistedRunSnapshot) => {
    if (!active()) return;
    const projection = decodePresentationProvenance(runId, snapshot);
    const previous = graphStore.getState().readEntity<PresentationProvenance>(PRESENTATION_PROVENANCE_ENTITY, runId);
    if (previous?.admission_id === PRESENTATION_PROVENANCE_ADMISSION
      && previous.status === projection.status && previous.source_event_id === projection.source_event_id
      && previous.source_sequence === projection.source_sequence) return;
    if (previous?.observation && projection.observation
      && JSON.stringify(previous.observation.published_templates) === JSON.stringify(projection.observation.published_templates)) {
      projection.observation.published_templates = previous.observation.published_templates;
    }
    write({ id: runId, admission_id: PRESENTATION_PROVENANCE_ADMISSION, ...projection });
  };
  // Begin asynchronously so a missing database follows the same recoverable error path.
  void Promise.resolve().then(() => subscribePresentationHistory(runId, (snapshot) => {
    callbackObserved = true;
    ingest(snapshot);
  })).then((subscription) => {
    if (!active()) { dispose(subscription.unsubscribe); return; }
    lease.unsubscribe = subscription.unsubscribe;
    // A live callback can arrive before the initial query promise resolves.
    if (!callbackObserved) ingest(subscription.initialSnapshot);
  }).catch(() => {
    if (active()) reset(runId, "error");
  });
}

export const presentationProvenanceActions = {
  subscribe(runId: string): () => void {
    let lease = leases.get(runId);
    if (!lease) {
      lease = { users: 0, generation: 0 };
      leases.set(runId, lease);
      start(runId, lease);
    }
    lease.users += 1;
    const acquired = lease;
    return () => {
      acquired.users -= 1;
      if (acquired.users !== 0 || leases.get(runId) !== acquired) return;
      leases.delete(runId);
      dispose(acquired.unsubscribe);
      reset(runId, "idle");
    };
  },
  retry(runId: string): void {
    const lease = leases.get(runId);
    if (lease) start(runId, lease);
  },
};
