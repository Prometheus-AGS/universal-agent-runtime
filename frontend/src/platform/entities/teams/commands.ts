import { batchGraphWrites } from "@prometheus-ags/entity-graph-core";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { TeamsApiError, preconditionError, toErrorInfo } from "./api/teams-errors";
import * as transport from "./api/teams-transport";
import {
  DEFAULT_ROUTING_RESERVATION,
  TEAM_ATTEMPT_FINAL_STATUSES,
  isAttemptInFlight,
  isWaitLive,
  type AdmitTeamTaskRequest,
  type BindingInstallResponse,
  type BindingPreflightResponse,
  type DeploymentBindingDocument,
  type MemberSlotRequest,
  type ImmutableDefinitionRef,
  type PackageInstallResponse,
  type PackagePreflightResponse,
  type PackageSourceRequest,
  type TeamExecutionAttempt,
  type TeamExecutionSummary,
  type TeamInstance,
  type TeamMember,
  type TeamReservation,
} from "./contracts";
import {
  TEAMS_SESSION_ID,
  TEAM_ROUTING_PROGRESS_ENTITY,
  owedRoutingStep,
  type RoutingCommandStep,
  type TeamRoutingProgress,
} from "./entities";
import { lists, routingGid } from "./keys";
import { admittedOwner } from "./store";
import { ensureOwner, loadBindings, loadPackages, loadTeamDefinitions, publishTeam, refreshTeamRun } from "./loads";

/**
 * Commands. The runtime is the authority: each command re-reads the team for its
 * current revisions, sends once, and publishes the returned record in one atomic
 * write. A conflict is reported as the runtime states it; the team is reloaded
 * and the command is NOT resubmitted. Every command accepts a caller-supplied
 * `commandId` (a retry of an uncertain write must reuse it).
 */

type Commandable = { commandId?: string };
export type PackageSourceInput = Omit<PackageSourceRequest, "commandId"> & Commandable;

/** Best-effort: a failed follow-up reload must not mask a command that succeeded. */
async function quietly(...tasks: Array<Promise<unknown>>): Promise<void> {
  await Promise.allSettled(tasks);
}

async function reloadOnConflict<T>(workspace: string, teamId: string, run: () => Promise<T>): Promise<T> {
  try {
    return await run();
  } catch (error) {
    if (error instanceof TeamsApiError && error.kind === "conflict") await quietly(refreshTeamRun(workspace, teamId));
    throw error;
  }
}

async function publishReturnedTeam(workspace: string, team: TeamInstance): Promise<void> {
  const owner = await ensureOwner();
  batchGraphWrites((batch) => publishTeam(batch, owner, workspace, team));
}

// --- Packages -----------------------------------------------------------------

export function preflightTeamPackage(input: PackageSourceInput): Promise<PackagePreflightResponse> {
  return transport.preflightPackage(input);
}

/** Install is permanent. On success the definitions and packages reload. */
export async function installTeamPackage(input: PackageSourceInput): Promise<PackageInstallResponse> {
  const response = await transport.installPackage(input);
  await quietly(loadTeamDefinitions(), loadPackages());
  return response;
}

// --- Bindings -----------------------------------------------------------------

export interface BindingCommandInput extends Commandable {
  binding: DeploymentBindingDocument;
  /** From `buildDeploymentBinding`: the existing revision, 0 for a new binding. */
  expectedRevision: number;
}

export function preflightTeamBinding(workspace: string, input: BindingCommandInput): Promise<BindingPreflightResponse> {
  return transport.preflightBinding(workspace, input);
}

export async function installTeamBinding(workspace: string, input: BindingCommandInput): Promise<BindingInstallResponse> {
  const response = await transport.installBinding(workspace, input);
  await quietly(loadBindings(workspace));
  return response;
}

// --- Teams and tasks -------------------------------------------------------------

export interface CreateTeamInput extends Commandable {
  deploymentBindingId: string;
  teamDefinition: ImmutableDefinitionRef;
  input: unknown;
  memberSlots?: MemberSlotRequest[];
}

export async function createTeam(workspace: string, input: CreateTeamInput): Promise<TeamInstance> {
  const team = await transport.createTeam(workspace, { ...input, memberSlots: input.memberSlots ?? [] });
  await publishReturnedTeam(workspace, team);
  return team;
}

const unavailableMember = (status: string) => status === "revoked" || status === "stopped";

/**
 * Admission refuses a member with an in-flight or unresolved attempt, or a live wait
 * (`team_execution/admission.rs`); a claim for such a member would fail at the next step.
 */
function busyMemberIds(summary: TeamExecutionSummary): Set<string> {
  const busy = new Set<string>();
  for (const attempt of summary.attempts) {
    const unresolved = attempt.status === "uncertain" && !TEAM_ATTEMPT_FINAL_STATUSES.includes(attempt.executionOutcome ?? "");
    if (isAttemptInFlight(attempt.status) || unresolved) busy.add(attempt.memberId);
  }
  for (const wait of summary.waits) if (isWaitLive(wait.state)) busy.add(wait.authority.memberId);
  return busy;
}

/** The first member of the role that is not busy; the first available holder when every holder is busy. */
function pickMemberForRole(members: readonly TeamMember[], role: string, summary: TeamExecutionSummary): TeamMember | undefined {
  const holders = members.filter((member) => member.role === role && !unavailableMember(member.status)).sort((a, b) => a.ordinal - b.ordinal);
  const busy = busyMemberIds(summary);
  return holders.find((member) => !busy.has(member.id)) ?? holders[0];
}

export interface CancelAttemptInput extends Commandable {
  reason: string;
}

/**
 * A cancel the runtime already committed leaves the attempt `cancellation_requested`/`cancelled`
 * (queued or running), or a `yielded` attempt with its wait `invalidated`, carrying the request's
 * reason. A lost response is retried with the same commandId, but the runtime digests the expected
 * team revision too, which the commit bumped; resending would be refused as another request.
 */
function committedCancel(summary: TeamExecutionSummary, attemptId: string, reason: string): TeamExecutionAttempt | null {
  const attempt = summary.attempts.find((candidate) => candidate.id === attemptId);
  if (!attempt) return null;
  // Already cancelling or cancelled: what the operator asked for holds. The reason is not compared here,
  // because settlement replaces it (observed: `team_usage_or_effect_outcome_uncertain` after a mid-call cancel).
  if (attempt.status === "cancellation_requested" || attempt.status === "cancelled") return attempt;
  // A wait is also invalidated for reasons other than a cancel, so only the operator's own reason counts.
  if (attempt.stateReason !== reason) return null;
  const invalidated = summary.waits.some((wait) => wait.authority.attemptId === attemptId && wait.state === "invalidated");
  return attempt.status === "yielded" && invalidated ? attempt : null;
}

export async function cancelAttempt(workspace: string, teamId: string, attemptId: string, input: CancelAttemptInput): Promise<TeamExecutionAttempt> {
  return reloadOnConflict(workspace, teamId, async () => {
    const [fresh, summary] = await Promise.all([transport.fetchTeam(workspace, teamId), transport.fetchExecution(workspace, teamId)]);
    const committed = committedCancel(summary, attemptId, input.reason);
    if (committed) {
      await quietly(refreshTeamRun(workspace, teamId));
      return committed;
    }
    const attempt = await transport.cancelAttempt(workspace, teamId, attemptId, {
      commandId: input.commandId, expectedTeamRevision: fresh.revision, reason: input.reason,
    });
    await quietly(refreshTeamRun(workspace, teamId));
    return attempt;
  });
}

// --- Routing test: create, claim, admit with resumable progress ----------------------

export interface RoutingTestInput {
  workspace: string;
  teamId: string;
  /** The role the test goes to; the coordinator role by default in the UI. */
  role: string;
  request: string;
  title?: string;
  reservation?: TeamReservation;
  contextArtifactIds?: string[];
  /** Claim for this member instead of the first idle holder of the role. */
  memberId?: string;
  taskId?: string;
}

function saveProgress(progress: TeamRoutingProgress): void {
  const id = progress.id;
  batchGraphWrites((batch) => {
    batch.replaceEntity(TEAM_ROUTING_PROGRESS_ENTITY, id, progress as unknown as Record<string, unknown>);
    const key = lists.routing(progress.ownerId, progress.workspace, progress.teamId);
    const current = graphStore.getState().lists[key]?.ids ?? [];
    batch.setListIds(key, current.includes(id) ? current : [...current, id], TEAM_ROUTING_PROGRESS_ENTITY);
  });
}

function readProgress(owner: string, workspace: string, teamId: string, taskId: string): TeamRoutingProgress | null {
  return graphStore.getState().readEntity<TeamRoutingProgress>(TEAM_ROUTING_PROGRESS_ENTITY, routingGid(owner, workspace, teamId, taskId));
}

async function performStep(progress: TeamRoutingProgress, step: RoutingCommandStep): Promise<TeamRoutingProgress> {
  const { workspace, teamId, taskId, ownerId } = progress;
  const touch = { updatedAt: Date.now(), lastError: null, failedStep: null, running: null, runningSession: null } as const;
  if (step === "create") {
    const fresh = await transport.fetchTeam(workspace, teamId);
    if (!fresh.tasks.some((task) => task.id === taskId)) {
      const team = await transport.createTask(workspace, teamId, {
        commandId: progress.createCommandId, taskId, expectedTeamRevision: fresh.revision, title: progress.title,
        role: progress.role, input: { request: progress.request }, outputContract: {}, dependsOn: [],
      });
      batchGraphWrites((batch) => publishTeam(batch, ownerId, workspace, team));
    }
    return { ...progress, ...touch, step: "created" };
  }
  if (step === "claim") {
    const [fresh, summary] = await Promise.all([transport.fetchTeam(workspace, teamId), transport.fetchExecution(workspace, teamId)]);
    const task = fresh.tasks.find((candidate) => candidate.id === taskId);
    if (!task) throw preconditionError("task_not_found", "The test task is not in the team.");
    if (task.assigneeMemberId) return { ...progress, ...touch, step: "claimed", memberId: task.assigneeMemberId };
    if (task.status !== "ready") throw preconditionError("task_not_ready", `Only a ready task can be claimed; this one is ${task.status}.`);
    const member = progress.memberId
      ? fresh.members.find((candidate) => candidate.id === progress.memberId)
      : pickMemberForRole(fresh.members, task.role, summary);
    if (!member || member.role !== task.role || unavailableMember(member.status)) {
      throw preconditionError("no_member_for_role", `No available member holds the role "${task.role}".`);
    }
    const team = await transport.claimTask(workspace, teamId, taskId, {
      commandId: progress.claimCommandId, expectedTeamRevision: fresh.revision, expectedTaskRevision: task.revision, memberId: member.id,
    });
    batchGraphWrites((batch) => publishTeam(batch, ownerId, workspace, team));
    return { ...progress, ...touch, step: "claimed", memberId: member.id };
  }
  // admit: look for an admission that already committed before sending anything new.
  const summary = await transport.fetchExecution(workspace, teamId);
  const committed = summary.attempts.filter((attempt) => attempt.taskId === taskId && !attempt.continuationOfWaitId).at(-1);
  let request: AdmitTeamTaskRequest;
  if (committed && (committed.status !== "queued" || !progress.admitRequest)) {
    await quietly(refreshTeamRun(workspace, teamId));
    return { ...progress, ...touch, step: "admitted", attemptId: committed.id };
  }
  if (committed && progress.admitRequest) {
    request = progress.admitRequest; // committed, never dispatched: resend byte-identically with the same commandId
  } else {
    const fresh = await transport.fetchTeam(workspace, teamId);
    const task = fresh.tasks.find((candidate) => candidate.id === taskId);
    if (!task?.assigneeMemberId) throw preconditionError("task_not_claimed", "Claim the task before admitting it.");
    request = {
      commandId: progress.admitCommandId, expectedTeamRevision: fresh.revision, expectedTaskRevision: task.revision,
      memberId: task.assigneeMemberId, reservation: progress.reservation, contextArtifactIds: progress.contextArtifactIds,
    };
    saveProgress({ ...progress, admitRequest: request, running: "admit", runningSession: TEAMS_SESSION_ID, updatedAt: Date.now() });
  }
  const attempt = await transport.admitTask(workspace, teamId, taskId, request);
  await quietly(refreshTeamRun(workspace, teamId));
  return { ...progress, ...touch, admitRequest: request, step: "admitted", attemptId: attempt.id };
}

/** Performs every owed step in order, or only the first one when `single` (a retry of one named step). */
async function advance(initial: TeamRoutingProgress, single = false): Promise<TeamRoutingProgress> {
  let current = initial;
  for (let owed = owedRoutingStep(current); owed; owed = owedRoutingStep(current)) {
    const stored = readProgress(current.ownerId, current.workspace, current.teamId, current.taskId) ?? current;
    current = { ...stored, running: owed, runningSession: TEAMS_SESSION_ID, lastError: null, updatedAt: Date.now() };
    saveProgress(current);
    try {
      current = await performStep(current, owed);
    } catch (error) {
      const info = toErrorInfo(error);
      const persisted = readProgress(current.ownerId, current.workspace, current.teamId, current.taskId) ?? current;
      current = { ...persisted, step: "failed", failedStep: owed, running: null, runningSession: null, lastError: info, updatedAt: Date.now() };
      saveProgress(current);
      if (info.kind === "conflict") await quietly(refreshTeamRun(current.workspace, current.teamId));
      return current;
    }
    saveProgress(current);
    if (single) break;
  }
  return current;
}

/**
 * Create, claim and admit one test task. Progress lives in the graph; a failure
 * stops at the failing step and is reported there (this resolves, it does not throw).
 * `resumeRoutingTest` performs only the one step still owed, with the same command id.
 */
export async function startRoutingTest(input: RoutingTestInput): Promise<TeamRoutingProgress> {
  const owner = await ensureOwner();
  const taskId = input.taskId ?? crypto.randomUUID();
  const progress: TeamRoutingProgress = {
    id: routingGid(owner, input.workspace, input.teamId, taskId), ownerId: owner, workspace: input.workspace,
    teamId: input.teamId, taskId, role: input.role, title: input.title?.trim() || "Routing test",
    request: input.request, reservation: input.reservation ?? DEFAULT_ROUTING_RESERVATION,
    contextArtifactIds: input.contextArtifactIds ?? [], memberId: input.memberId ?? null,
    createCommandId: crypto.randomUUID(), claimCommandId: crypto.randomUUID(), admitCommandId: crypto.randomUUID(),
    attemptId: null, admitRequest: null, step: "pending", failedStep: null, running: null, runningSession: null, lastError: null, updatedAt: Date.now(),
  };
  saveProgress(progress);
  return advance(progress);
}

export async function resumeRoutingTest(workspace: string, teamId: string, taskId: string): Promise<TeamRoutingProgress | null> {
  const owner = await ensureOwner();
  const progress = readProgress(owner, workspace, teamId, taskId);
  if (!progress || (progress.running && progress.runningSession === TEAMS_SESSION_ID)) return progress;
  return advance(progress, true);
}

/** Stop tracking a routing test (the task itself cannot be deleted). */
export function dismissRoutingTest(workspace: string, teamId: string, taskId: string): void {
  const owner = admittedOwner();
  if (!owner) return;
  const id = routingGid(owner, workspace, teamId, taskId);
  const key = lists.routing(owner, workspace, teamId);
  batchGraphWrites((batch) => {
    batch.removeEntity(TEAM_ROUTING_PROGRESS_ENTITY, id);
    batch.setListIds(key, (graphStore.getState().lists[key]?.ids ?? []).filter((candidate) => candidate !== id), TEAM_ROUTING_PROGRESS_ENTITY);
  });
}
