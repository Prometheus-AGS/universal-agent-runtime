import { useGraphStore } from "@/platform/entities";
import type { GovernanceRuntimeStatus } from "../api/settings-api";

export const GOVERNANCE_STATUS_ENTITY_TYPE = "GovernanceRuntimeStatus";
export const GOVERNANCE_STATUS_ENTITY_ID = "active";

export interface GovernanceStatusEntity extends GovernanceRuntimeStatus {
  id: typeof GOVERNANCE_STATUS_ENTITY_ID;
  lastAcceptedRequestSequence: number;
}

export interface GovernanceStatusIngestResult {
  accepted: boolean;
  restarted: boolean;
}

let requestSequence = 0;
const retiredBootInstances = new Set<string>();

export function nextGovernanceRequestSequence(): number {
  requestSequence += 1;
  return requestSequence;
}

export function governanceStatusSnapshot(): GovernanceStatusEntity | null {
  return (
    (useGraphStore.getState().entities[GOVERNANCE_STATUS_ENTITY_TYPE]?.[
      GOVERNANCE_STATUS_ENTITY_ID
    ] as unknown as GovernanceStatusEntity | undefined) ?? null
  );
}

export function invalidateGovernanceStatus(
  failedRequestSequence: number,
): boolean {
  const current = governanceStatusSnapshot();
  if (
    current &&
    current.lastAcceptedRequestSequence > failedRequestSequence
  ) {
    return false;
  }
  useGraphStore
    .getState()
    .removeEntity(GOVERNANCE_STATUS_ENTITY_TYPE, GOVERNANCE_STATUS_ENTITY_ID);
  return true;
}

export function ingestGovernanceStatus(
  status: GovernanceRuntimeStatus,
  acceptedRequestSequence: number,
): GovernanceStatusIngestResult {
  const current = governanceStatusSnapshot();
  if (retiredBootInstances.has(status.boot_instance_id)) {
    return { accepted: false, restarted: false };
  }

  const sameBoot = current?.boot_instance_id === status.boot_instance_id;
  if (
    current &&
    (acceptedRequestSequence < current.lastAcceptedRequestSequence ||
      (sameBoot && status.revision < current.revision))
  ) {
    return { accepted: false, restarted: false };
  }

  const restarted = Boolean(current && !sameBoot);
  if (restarted && current) retiredBootInstances.add(current.boot_instance_id);
  useGraphStore.getState().upsertEntity(
    GOVERNANCE_STATUS_ENTITY_TYPE,
    GOVERNANCE_STATUS_ENTITY_ID,
    {
      ...status,
      id: GOVERNANCE_STATUS_ENTITY_ID,
      lastAcceptedRequestSequence: acceptedRequestSequence,
    },
  );
  return { accepted: true, restarted };
}

export function __resetGovernanceStatusForTests(): void {
  requestSequence = 0;
  retiredBootInstances.clear();
}
