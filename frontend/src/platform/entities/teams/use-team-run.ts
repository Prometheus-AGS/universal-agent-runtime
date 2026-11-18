import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import type { GraphState } from "@prometheus-ags/entity-graph-core";
import type { TeamsErrorInfo } from "./api/teams-errors";
import { isAttemptInFlight, isWaitLive, isTeamRunLive } from "./contracts";
import {
  TEAMS_SESSION_ID,
  TEAM_ARTIFACT_ENTITY,
  TEAM_ATTEMPT_ENTITY,
  TEAM_DEFINITION_ENTITY,
  TEAM_INSTANCE_ENTITY,
  TEAM_MEMBER_ENTITY,
  TEAM_PEER_MESSAGE_ENTITY,
  TEAM_ROUTING_PROGRESS_ENTITY,
  TEAM_RUN_SUMMARY_ENTITY,
  TEAM_RUN_WATCH_ENTITY,
  TEAM_TASK_ENTITY,
  TEAM_WAIT_ENTITY,
  definitionKey,
  owedRoutingStep,
  type RoutingCommandStep,
  type TeamArtifactEntity,
  type TeamAttemptEntity,
  type TeamDefinitionEntity,
  type TeamInstanceEntity,
  type TeamMemberEntity,
  type TeamPeerMessageEntity,
  type TeamRoutingProgress,
  type TeamRunSummaryEntity,
  type TeamRunWatchState,
  type TeamTaskEntity,
  type TeamWaitEntity,
} from "./entities";
import { childGid, definitionGid, lists, routingGid, teamGid } from "./keys";
import { entityOf, idsOf, ownerOf, useScopedField, useScopedIds } from "./selectors";

/**
 * Team, member, task, attempt, wait, artifact, message and run-watch hooks.
 * Each subscribes to one field or one derived primitive; split controls that need
 * independent fields into independent components. Ids are domain ids; the
 * workspace is always an argument.
 */

type Dimension = "tokens" | "costMicrounits" | "elapsedSeconds";
const BUDGET_KEY = { tokens: "maxTokens", costMicrounits: "maxCostMicrounits", elapsedSeconds: "maxElapsedSeconds" } as const;

// --- Teams ----------------------------------------------------------------------

/** Team ids of the workspace, newest first. */
export function useTeamIds(workspace: string): string[] {
  return useScopedIds((owner) => lists.teams(owner, workspace));
}

export function useTeamField<K extends keyof TeamInstanceEntity>(workspace: string, teamId: string, field: K): TeamInstanceEntity[K] | undefined {
  return useScopedField<TeamInstanceEntity, K>(TEAM_INSTANCE_ENTITY, (owner) => teamGid(owner, workspace, teamId), field);
}

// --- Members --------------------------------------------------------------------

export function useMemberIds(workspace: string, teamId: string): string[] {
  return useScopedIds((owner) => lists.members(owner, workspace, teamId));
}

export function useMemberField<K extends keyof TeamMemberEntity>(workspace: string, teamId: string, memberId: string, field: K): TeamMemberEntity[K] | undefined {
  return useScopedField<TeamMemberEntity, K>(TEAM_MEMBER_ENTITY, (owner) => childGid(owner, workspace, teamId, memberId), field);
}

// --- Tasks ----------------------------------------------------------------------

export function useUnassignedTaskIds(workspace: string, teamId: string): string[] {
  return useScopedIds((owner) => lists.unassignedTasks(owner, workspace, teamId));
}
/** Tasks assigned to one member. */
export function useMemberTaskIds(workspace: string, teamId: string, memberId: string): string[] {
  return useScopedIds((owner) => lists.tasksByMember(owner, workspace, teamId, memberId));
}

export function useTaskField<K extends keyof TeamTaskEntity>(workspace: string, teamId: string, taskId: string, field: K): TeamTaskEntity[K] | undefined {
  return useScopedField<TeamTaskEntity, K>(TEAM_TASK_ENTITY, (owner) => childGid(owner, workspace, teamId, taskId), field);
}

/** The first dependency that failed or was cancelled (so this task will never run), or null. */
export function useTaskBlockedBy(workspace: string, teamId: string, taskId: string): string | null {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    if (!owner) return null;
    const task = entityOf<TeamTaskEntity>(state, TEAM_TASK_ENTITY, childGid(owner, workspace, teamId, taskId));
    for (const dependency of task?.dependsOn ?? []) {
      const status = entityOf<TeamTaskEntity>(state, TEAM_TASK_ENTITY, childGid(owner, workspace, teamId, dependency))?.status;
      if (status === "failed" || status === "cancelled") return dependency;
    }
    return null;
  });
}

/** The member whose attempt delegated this task, from the run's `team_delegate` receipts, or null. */
export function useTaskDelegatorMemberId(workspace: string, teamId: string, taskId: string): string | null {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    const run = owner ? entityOf<TeamRunSummaryEntity>(state, TEAM_RUN_SUMMARY_ENTITY, teamGid(owner, workspace, teamId)) : undefined;
    return run?.delegations.find((receipt) => receipt.taskId === taskId)?.senderMemberId ?? null;
  });
}

// --- Attempts and waits ------------------------------------------------------------

/** A member's attempts, oldest first. */
export function useMemberAttemptIds(workspace: string, teamId: string, memberId: string): string[] {
  return useScopedIds((owner) => lists.attemptsByMember(owner, workspace, teamId, memberId));
}
export function useAttemptField<K extends keyof TeamAttemptEntity>(workspace: string, teamId: string, attemptId: string, field: K): TeamAttemptEntity[K] | undefined {
  return useScopedField<TeamAttemptEntity, K>(TEAM_ATTEMPT_ENTITY, (owner) => childGid(owner, workspace, teamId, attemptId), field);
}

export function useMemberWaitIds(workspace: string, teamId: string, memberId: string): string[] {
  return useScopedIds((owner) => lists.waitsByMember(owner, workspace, teamId, memberId));
}
export function useWaitField<K extends keyof TeamWaitEntity>(workspace: string, teamId: string, waitId: string, field: K): TeamWaitEntity[K] | undefined {
  return useScopedField<TeamWaitEntity, K>(TEAM_WAIT_ENTITY, (owner) => childGid(owner, workspace, teamId, waitId), field);
}

function liveParts(state: GraphState, owner: string, workspace: string, teamId: string) {
  const attempts = idsOf(state, lists.attempts(owner, workspace, teamId))
    .map((id) => entityOf<TeamAttemptEntity>(state, TEAM_ATTEMPT_ENTITY, id)).filter((a): a is TeamAttemptEntity => Boolean(a));
  const waits = idsOf(state, lists.waits(owner, workspace, teamId))
    .map((id) => entityOf<TeamWaitEntity>(state, TEAM_WAIT_ENTITY, id)).filter((w): w is TeamWaitEntity => Boolean(w));
  return { attempts, waits };
}

/**
 * True while any attempt is queued, running or cancellation_requested, or any wait is
 * yield_requested, waiting or blocked. This, not team or member `status`, is activity.
 */
export function useTeamIsLive(workspace: string, teamId: string): boolean {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    if (!owner) return false;
    const { attempts, waits } = liveParts(state, owner, workspace, teamId);
    return isTeamRunLive(attempts, waits.map((wait) => ({ state: wait.state })));
  });
}

export function useTeamAttemptCount(workspace: string, teamId: string): number {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    return owner ? idsOf(state, lists.attempts(owner, workspace, teamId)).length : 0;
  });
}

/**
 * The member's current attempt: its newest queued, running or cancellation_requested attempt,
 * or the yielded attempt of a wait that can still resume. Null when the member is idle.
 */
export function useMemberOpenAttemptId(workspace: string, teamId: string, memberId: string): string | null {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    if (!owner) return null;
    const { attempts, waits } = liveParts(state, owner, workspace, teamId);
    const liveWaitAttempts = new Set(waits.filter((wait) => isWaitLive(wait.state)).map((wait) => wait.attemptId));
    const mine = attempts.filter((attempt) => attempt.memberId === memberId);
    const open = [...mine].reverse().find((attempt) => isAttemptInFlight(attempt.status) || liveWaitAttempts.has(attempt.attemptId));
    return open?.attemptId ?? null;
  });
}

// --- Budget and capacity ---------------------------------------------------------------

/** The team's effective budget for one dimension (min of definition and binding), or null. */
export function useTeamBudgetLimit(workspace: string, teamId: string, dimension: Dimension): number | null {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    const run = owner ? entityOf<TeamRunSummaryEntity>(state, TEAM_RUN_SUMMARY_ENTITY, teamGid(owner, workspace, teamId)) : undefined;
    return run?.budget[BUDGET_KEY[dimension]] ?? null;
  });
}

/**
 * Budget left for new admissions: the limit minus every attempt's usage (or its reservation while usage
 * is unsettled), which is how the runtime counts it. Cost is in microunits.
 */
export function useTeamBudgetRemaining(workspace: string, teamId: string, dimension: Dimension): number | null {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    if (!owner) return null;
    const run = entityOf<TeamRunSummaryEntity>(state, TEAM_RUN_SUMMARY_ENTITY, teamGid(owner, workspace, teamId));
    const limit = run?.budget[BUDGET_KEY[dimension]];
    if (limit === undefined) return null;
    let consumed = 0;
    for (const attempt of liveParts(state, owner, workspace, teamId).attempts) {
      consumed += (attempt.usage ?? attempt.reservation)[dimension];
    }
    return Math.max(0, limit - consumed);
  });
}

/** Tasks that can still be created: the team definition's `maxPendingTasks` minus every task ever created. */
export function useTeamTaskSlotsLeft(workspace: string, teamId: string): number | null {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    if (!owner) return null;
    const team = entityOf<TeamInstanceEntity>(state, TEAM_INSTANCE_ENTITY, teamGid(owner, workspace, teamId));
    if (!team) return null;
    const definition = entityOf<TeamDefinitionEntity>(state, TEAM_DEFINITION_ENTITY, definitionGid(owner, definitionKey(team.definition)));
    const cap = definition?.limits.maxPendingTasks;
    return cap === undefined ? null : Math.max(0, cap - idsOf(state, lists.tasks(owner, workspace, teamId)).length);
  });
}

export function useTeamRunSummaryField<K extends keyof TeamRunSummaryEntity>(workspace: string, teamId: string, field: K): TeamRunSummaryEntity[K] | undefined {
  return useScopedField<TeamRunSummaryEntity, K>(TEAM_RUN_SUMMARY_ENTITY, (owner) => teamGid(owner, workspace, teamId), field);
}

// --- Artifacts and peer messages -----------------------------------------------------------

export function useMemberArtifactIds(workspace: string, teamId: string, memberId: string): string[] {
  return useScopedIds((owner) => lists.artifactsByMember(owner, workspace, teamId, memberId));
}
export function useArtifactField<K extends keyof TeamArtifactEntity>(workspace: string, teamId: string, artifactId: string, field: K): TeamArtifactEntity[K] | undefined {
  return useScopedField<TeamArtifactEntity, K>(TEAM_ARTIFACT_ENTITY, (owner) => childGid(owner, workspace, teamId, artifactId), field);
}

/** Peer message ids of the team, oldest first. */
export function usePeerMessageIds(workspace: string, teamId: string): string[] {
  return useScopedIds((owner) => lists.peerMessages(owner, workspace, teamId));
}
export function usePeerMessageField<K extends keyof TeamPeerMessageEntity>(workspace: string, teamId: string, messageId: string, field: K): TeamPeerMessageEntity[K] | undefined {
  return useScopedField<TeamPeerMessageEntity, K>(TEAM_PEER_MESSAGE_ENTITY, (owner) => childGid(owner, workspace, teamId, messageId), field);
}

// --- Run watch ------------------------------------------------------------------------------

function watchOf(state: GraphState, workspace: string, teamId: string): TeamRunWatchState | undefined {
  const owner = ownerOf(state);
  const watch = owner ? entityOf<TeamRunWatchState>(state, TEAM_RUN_WATCH_ENTITY, teamGid(owner, workspace, teamId)) : undefined;
  return watch?.session === TEAMS_SESSION_ID ? watch : undefined;
}

/** Epoch milliseconds of the last successful refresh in this session, or null. Drive "Updated N s ago" from it. */
export function useTeamRunLastUpdatedAt(workspace: string, teamId: string): number | null {
  return useGraphStore((state) => watchOf(state, workspace, teamId)?.lastUpdatedAt ?? null);
}
export function useTeamRunWatchError(workspace: string, teamId: string): TeamsErrorInfo | null {
  return useGraphStore((state) => watchOf(state, workspace, teamId)?.lastError ?? null);
}
/** `stopped` once the run is not live (or on a 401); `watching` while polling or backing off. */
export function useTeamRunWatchStatus(workspace: string, teamId: string): "watching" | "stopped" | "idle" {
  return useGraphStore((state) => watchOf(state, workspace, teamId)?.status ?? "idle");
}
/** Current poll interval in ms (3000 live, 15000 after an error or a 429). */
export function useTeamRunWatchInterval(workspace: string, teamId: string): number | null {
  return useGraphStore((state) => watchOf(state, workspace, teamId)?.intervalMs ?? null);
}

// --- Routing test progress ---------------------------------------------------------------------

/** Ids of the routing-test tasks started from this console for the team. */
export function useRoutingTestTaskIds(workspace: string, teamId: string): string[] {
  return useScopedIds((owner) => lists.routing(owner, workspace, teamId));
}
export function useRoutingProgressField<K extends keyof TeamRoutingProgress>(workspace: string, teamId: string, taskId: string, field: K): TeamRoutingProgress[K] | undefined {
  return useScopedField<TeamRoutingProgress, K>(TEAM_ROUTING_PROGRESS_ENTITY, (owner) => routingGid(owner, workspace, teamId, taskId), field);
}
/** The step still owed (`create`, `claim`, `admit`), or null when admitted or unknown. */
export function useRoutingOwedStep(workspace: string, teamId: string, taskId: string): RoutingCommandStep | null {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    const progress = owner ? entityOf<TeamRoutingProgress>(state, TEAM_ROUTING_PROGRESS_ENTITY, routingGid(owner, workspace, teamId, taskId)) : undefined;
    return progress ? owedRoutingStep(progress) : null;
  });
}
/** The command in flight in this session, or null (a persisted value from an earlier session is ignored). */
export function useRoutingRunningStep(workspace: string, teamId: string, taskId: string): RoutingCommandStep | null {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    const progress = owner ? entityOf<TeamRoutingProgress>(state, TEAM_ROUTING_PROGRESS_ENTITY, routingGid(owner, workspace, teamId, taskId)) : undefined;
    return progress?.runningSession === TEAMS_SESSION_ID ? progress.running : null;
  });
}
