import { useMemo } from "react";
import {
  teamLoadKeys,
  useTeamExecutionClaimField,
  useTeamLoadError,
  useTeamLoadStatus,
  useTeamsOwnerId,
  useTeamsRuntimeField,
} from "@/platform/entities";
import { deriveReadiness, gateReason, type ReadinessFact, type TeamAction } from "./readiness";

/** The four readiness facts for a workspace. Only the strip and the gates call this; each is a small boundary. */
export function useReadiness(workspace: string): ReadinessFact[] {
  const ownerId = useTeamsOwnerId();
  const capabilitiesStatus = useTeamLoadStatus(teamLoadKeys.capabilities);
  const capabilitiesError = useTeamLoadError(teamLoadKeys.capabilities);
  const instanceId = useTeamsRuntimeField("instanceId");
  const teamExecutionAvailable = useTeamsRuntimeField("teamExecutionAvailable");
  const ownershipStatus = useTeamLoadStatus(teamLoadKeys.executionOwner);
  const ownershipError = useTeamLoadError(teamLoadKeys.executionOwner);
  const ownsExecution = useTeamsRuntimeField("ownsExecution");
  const claimEpoch = useTeamExecutionClaimField("epoch");
  const claimAcquiredAt = useTeamExecutionClaimField("acquiredAt");

  return useMemo(
    () =>
      deriveReadiness({
        workspace,
        ownerId,
        capabilitiesStatus,
        capabilitiesError,
        instanceId,
        teamExecutionAvailable,
        ownershipStatus,
        ownershipError,
        ownsExecution,
        claimEpoch,
        claimAcquiredAt,
      }),
    [
      workspace,
      ownerId,
      capabilitiesStatus,
      capabilitiesError,
      instanceId,
      teamExecutionAvailable,
      ownershipStatus,
      ownershipError,
      ownsExecution,
      claimEpoch,
      claimAcquiredAt,
    ],
  );
}

/** The reason an action is unavailable right now, or null. Re-renders only when that string changes. */
export function useActionGate(workspace: string, action: TeamAction): string | null {
  const facts = useReadiness(workspace);
  return useMemo(() => gateReason(facts, action), [facts, action]);
}
