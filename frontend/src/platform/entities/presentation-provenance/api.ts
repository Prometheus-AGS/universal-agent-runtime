import { getDbInstance } from "@/platform/pglite/client";
import type { PersistedRunSnapshot, PersistedRunSnapshotSubscription } from "@/platform/pglite/run-event-repository";

export function subscribePresentationHistory(runId: string, onSnapshot: (snapshot: PersistedRunSnapshot) => void): Promise<PersistedRunSnapshotSubscription> {
  return getDbInstance().subscribeRunSnapshot(runId, onSnapshot);
}
