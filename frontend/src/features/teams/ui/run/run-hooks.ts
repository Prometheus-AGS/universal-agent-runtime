import {
  useAttemptField,
  useMemberAttemptIds,
  useMemberField,
  useMemberOpenAttemptId,
  useMemberWaitIds,
  usePeerMessageField,
  useTaskField,
  useTeamBudgetRemaining,
  useTeamRunSummaryField,
  useTeamTaskSlotsLeft,
  useWaitField,
} from "@/platform/entities";
import { attemptView, isChargedInFull, pickWaitState, type LaneCategory, type LaneEntry, type RemainingBudget } from "./run-model";

/** The team every run-view hook is scoped to. Workspace is always explicit. */
export interface RunContext {
  workspace: string;
  teamId: string;
}

/** One wait's owner and state, for the fold that tells a yielded attempt what became of its wait. */
export interface WaitFact {
  attemptId: string;
  state: string;
}
export function useWaitFact(waitId: string, { workspace, teamId }: RunContext): WaitFact {
  const attemptId = useWaitField(workspace, teamId, waitId, "attemptId");
  const state = useWaitField(workspace, teamId, waitId, "state");
  return { attemptId: attemptId ?? "", state: state ?? "" };
}
export const NO_WAITS: readonly WaitFact[] = [];
export function addWaitFact(waits: readonly WaitFact[], _id: string, fact: WaitFact): readonly WaitFact[] {
  return fact.attemptId === "" ? waits : [...waits, fact];
}

/** A member's wait ids, for the fold that gathers every wait of the team. */
export function useMemberWaits(memberId: string, { workspace, teamId }: RunContext): string[] {
  return useMemberWaitIds(workspace, teamId, memberId);
}
export function addWaitIds(acc: readonly string[], _id: string, waitIds: readonly string[]): readonly string[] {
  return waitIds.length === 0 ? acc : [...acc, ...waitIds];
}

/** The run context plus every wait of the team, so a yielded attempt can be read through its wait. */
export interface LaneContext extends RunContext {
  waits: readonly WaitFact[];
}

/**
 * The lane's attention category: the member's open attempt when it has one (queued, running, cancelling,
 * or yielded with a wait that can still resume), otherwise its latest attempt, otherwise idle. A yielded
 * attempt is read through its wait (from `waits`) and its task. The value is a string, so the fold above
 * passes on a stable value; the hooks inside still select id lists the graph reallocates on every write,
 * so this runs again on every poll.
 */
export function useMemberCategory(memberId: string, { workspace, teamId, waits }: LaneContext): LaneCategory {
  const attemptIds = useMemberAttemptIds(workspace, teamId, memberId);
  const openId = useMemberOpenAttemptId(workspace, teamId, memberId);
  const latestId = attemptIds.length > 0 ? attemptIds[attemptIds.length - 1] : null;
  const focusId = openId ?? latestId;
  const status = useAttemptField(workspace, teamId, focusId ?? "", "status");
  const taskId = useAttemptField(workspace, teamId, focusId ?? "", "taskId");
  const taskStatus = useTaskField(workspace, teamId, taskId ?? "", "status");
  if (focusId === null || status === undefined) return "idle";
  return attemptView(status, openId !== null, { state: pickWaitState(waits, focusId), taskStatus: taskStatus ?? null });
}

/** The cost the runtime keeps charged for an attempt that settled with no usage report; `costMicrounits` is null when no reservation is loaded. */
export interface UnreportedCharge {
  costMicrounits: number | null;
}

/**
 * Whether the member's lane attempt (its open one, else its latest) settled without a usage report, so the whole
 * reservation counts against the team budget. Null while the attempt is live, reported usage, or the member is idle.
 */
export function useUnreportedCharge(workspace: string, teamId: string, memberId: string): UnreportedCharge | null {
  const attemptIds = useMemberAttemptIds(workspace, teamId, memberId);
  const openId = useMemberOpenAttemptId(workspace, teamId, memberId);
  const focusId = openId ?? (attemptIds.length > 0 ? attemptIds[attemptIds.length - 1] : null);
  const status = useAttemptField(workspace, teamId, focusId ?? "", "status");
  const usage = useAttemptField(workspace, teamId, focusId ?? "", "usage");
  const reservation = useAttemptField(workspace, teamId, focusId ?? "", "reservation");
  if (focusId === null || !isChargedInFull(status, usage)) return null;
  return { costMicrounits: reservation?.costMicrounits ?? null };
}

export const NO_LANES: readonly LaneEntry[] = [];
export function addLane(entries: readonly LaneEntry[], id: string, category: LaneCategory): readonly LaneEntry[] {
  return [...entries, { id, category }];
}

/** One member's role and record status, for the roster fold. */
export interface MemberRole {
  role: string;
  status: string;
}
export function useMemberRole(memberId: string, { workspace, teamId }: RunContext): MemberRole {
  const role = useMemberField(workspace, teamId, memberId, "role");
  const status = useMemberField(workspace, teamId, memberId, "status");
  return { role: role ?? "", status: status ?? "" };
}

export interface RoleHolders {
  role: string;
  /** Members holding the role that the runtime will still claim for (not revoked or stopped). */
  available: number;
  total: number;
}

/** Mirrors the domain's claim rule: a revoked or stopped member is skipped. */
const isUnavailableStatus = (status: string) => status === "revoked" || status === "stopped";

export const NO_ROLES: readonly RoleHolders[] = [];
export function addRole(roles: readonly RoleHolders[], _id: string, member: MemberRole): readonly RoleHolders[] {
  if (member.role === "") return roles;
  const available = isUnavailableStatus(member.status) ? 0 : 1;
  const existing = roles.find((entry) => entry.role === member.role);
  if (!existing) return [...roles, { role: member.role, available, total: 1 }];
  return roles.map((entry) =>
    entry === existing ? { ...entry, available: entry.available + available, total: entry.total + 1 } : entry,
  );
}

/** Whether a peer message was sent by or addressed to the member (a single boolean per message). */
export function useMessageInvolves(messageId: string, { workspace, teamId, memberId }: RunContext & { memberId: string }): boolean {
  const sender = usePeerMessageField(workspace, teamId, messageId, "senderMemberId");
  const recipient = usePeerMessageField(workspace, teamId, messageId, "recipientMemberId");
  return sender === memberId || recipient === memberId;
}
export const NO_IDS: readonly string[] = [];
export function addIfInvolved(ids: readonly string[], id: string, involved: boolean): readonly string[] {
  return involved ? [...ids, id] : ids;
}

/** The team's budget currency (the reservation and usage are carried in microunits of it). */
export function useTeamCurrency(workspace: string, teamId: string): string | undefined {
  return useTeamRunSummaryField(workspace, teamId, "budget")?.currency;
}

/** What the team can still admit: remaining budget per dimension and remaining task slots (primitives only). */
export function useRemaining({ workspace, teamId }: RunContext): RemainingBudget {
  const tokens = useTeamBudgetRemaining(workspace, teamId, "tokens");
  const costMicrounits = useTeamBudgetRemaining(workspace, teamId, "costMicrounits");
  const elapsedSeconds = useTeamBudgetRemaining(workspace, teamId, "elapsedSeconds");
  const slots = useTeamTaskSlotsLeft(workspace, teamId);
  return { tokens, costMicrounits, elapsedSeconds, slots };
}
