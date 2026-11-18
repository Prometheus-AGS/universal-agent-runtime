/**
 * Pure display model for the team run view: attempt presentation (Decision 11 table), lane ordering,
 * output folding and the routing-test reservation. No platform imports, so every rule here is testable
 * without a graph.
 */

/** What an operator sees for one attempt. A `yielded` attempt is read through the state of its wait. */
export type AttemptView =
  | "queued"
  | "running"
  | "cancelling"
  | "waiting"
  | "handed_off"
  | "cancelled_waiting"
  | "wait_invalidated"
  | "yielded"
  | "succeeded"
  | "failed"
  | "cancelled"
  | "uncertain"
  | "unknown";

export type LaneCategory = AttemptView | "idle";
export type Tone = "danger" | "warning" | "success" | "progress" | "neutral";

export interface ViewPresentation {
  label: string;
  /** The attempt still has work in flight or a wait that can resume. */
  live: boolean;
  tone: Tone;
}

export const VIEW_PRESENTATION: Record<AttemptView, ViewPresentation> = {
  queued: { label: "Queued", live: true, tone: "neutral" },
  running: { label: "Running", live: true, tone: "progress" },
  cancelling: { label: "Cancelling", live: true, tone: "warning" },
  waiting: { label: "Waiting on tasks", live: true, tone: "warning" },
  handed_off: { label: "Handed off", live: false, tone: "neutral" },
  cancelled_waiting: { label: "Cancelled while waiting", live: false, tone: "neutral" },
  wait_invalidated: { label: "Wait invalidated", live: false, tone: "warning" },
  yielded: { label: "Yielded, wait not shown", live: false, tone: "warning" },
  succeeded: { label: "Succeeded", live: false, tone: "success" },
  failed: { label: "Failed", live: false, tone: "danger" },
  cancelled: { label: "Cancelled", live: false, tone: "neutral" },
  uncertain: { label: "Uncertain", live: false, tone: "warning" },
  unknown: { label: "Unrecognised status", live: false, tone: "warning" },
};

/** Wait states in which the runtime can still resume the yielded attempt. */
const LIVE_WAIT_STATES: ReadonlySet<string> = new Set(["yield_requested", "waiting", "blocked"]);

/** What is known about a yielded attempt's wait and task; `null` means no record is loaded for it. */
export interface WaitFacts {
  state: string | null;
  taskStatus: string | null;
}
export const NO_WAIT_FACTS: WaitFacts = { state: null, taskStatus: null };

/**
 * The wait state that describes an attempt: a live wait wins, otherwise the last settled one, otherwise null
 * (no wait record, or none loaded yet).
 */
export function pickWaitState(waits: readonly { attemptId: string; state: string }[], attemptId: string): string | null {
  let picked: string | null = null;
  for (const wait of waits) {
    if (wait.attemptId !== attemptId || wait.state === "") continue;
    if (LIVE_WAIT_STATES.has(wait.state)) return wait.state;
    picked = wait.state;
  }
  return picked;
}

/**
 * `yielded` is permanent, so liveness comes from the wait. A live wait is waiting; a `resumed` wait was handed
 * to a continuation; an `invalidated` wait on a cancelled task is a cancellation of a waiting attempt (the runtime
 * leaves the attempt `yielded` and invalidates its wait), and on any other task it is reported as invalidated
 * without a cause. With no wait record nothing says what became of the attempt, so no hand-off is asserted.
 */
function yieldedView(open: boolean, wait: WaitFacts): AttemptView {
  if (open || (wait.state !== null && LIVE_WAIT_STATES.has(wait.state))) return "waiting";
  switch (wait.state) {
    case "resumed":
      return "handed_off";
    case "invalidated":
      return wait.taskStatus === "cancelled" ? "cancelled_waiting" : "wait_invalidated";
    default:
      return "yielded";
  }
}

/** `open` is true when the runtime still counts this attempt as the member's current one. */
export function attemptView(status: string, open: boolean, wait: WaitFacts = NO_WAIT_FACTS): AttemptView {
  switch (status) {
    case "queued":
      return "queued";
    case "running":
      return "running";
    case "cancellation_requested":
      return "cancelling";
    case "yielded":
      return yieldedView(open, wait);
    case "succeeded":
      return "succeeded";
    case "failed":
      return "failed";
    case "cancelled":
      return "cancelled";
    case "uncertain":
      return "uncertain";
    default:
      return "unknown";
  }
}

export function laneLabel(category: LaneCategory): string {
  return category === "idle" ? "No work yet" : VIEW_PRESENTATION[category].label;
}

/** Attention first: what needs a person comes before what is simply running or finished. */
export const LANE_ORDER: readonly LaneCategory[] = [
  "failed",
  "uncertain",
  "unknown",
  "wait_invalidated",
  "yielded",
  "waiting",
  "cancelling",
  "running",
  "queued",
  "succeeded",
  "handed_off",
  "cancelled",
  "cancelled_waiting",
  "idle",
];

export interface LaneEntry {
  id: string;
  category: LaneCategory;
}

/** Stable sort by attention rank; members with the same category keep their roster order. */
export function sortLanes(entries: readonly LaneEntry[]): LaneEntry[] {
  const rank = new Map(LANE_ORDER.map((category, index) => [category, index]));
  return entries
    .map((entry, index) => ({ entry, index }))
    .sort((a, b) => (rank.get(a.entry.category) ?? 0) - (rank.get(b.entry.category) ?? 0) || a.index - b.index)
    .map(({ entry }) => entry);
}

export interface LaneCount {
  category: LaneCategory;
  count: number;
}

/** Counts per category in attention order, omitting empty ones. */
export function countLanes(entries: readonly LaneEntry[]): LaneCount[] {
  const counts = new Map<LaneCategory, number>();
  for (const entry of entries) counts.set(entry.category, (counts.get(entry.category) ?? 0) + 1);
  return LANE_ORDER.filter((category) => counts.has(category)).map((category) => ({ category, count: counts.get(category) ?? 0 }));
}

/** More lanes than this and the counts appear above the list. */
export const LANE_SUMMARY_THRESHOLD = 6;

/** Whether a lane category carries a live attempt (the standing approvals line shows for these). */
export function isLiveCategory(category: LaneCategory): boolean {
  return category !== "idle" && VIEW_PRESENTATION[category].live;
}

/**
 * Cancel is offered for `queued` and `running`, and for a `yielded` attempt only while its wait is
 * `waiting` or `blocked`. `cancellation_requested`, `uncertain` (the runtime answers 409) and settled
 * attempts have no operator action.
 */
export function canCancelAttempt(status: string, waitState: string | null): boolean {
  if (status === "queued" || status === "running") return true;
  return status === "yielded" && (waitState === "waiting" || waitState === "blocked");
}

/** Statuses in which the runtime has finished with an attempt and settled its accounting. */
const SETTLED_STATUSES: ReadonlySet<string> = new Set(["succeeded", "failed", "cancelled", "uncertain"]);

export function isSettledStatus(status: string): boolean {
  return SETTLED_STATUSES.has(status);
}

/**
 * A settled attempt without a usage report keeps its whole reservation against the team budget
 * (`runtime/team_execution/execution.rs`). A cancel during a model call ends this way.
 */
export function isChargedInFull(status: string | undefined, usage: unknown): boolean {
  return status !== undefined && isSettledStatus(status) && (usage === null || usage === undefined);
}

/** `Cancelled · usage unknown · $1.00 charged (full reservation)`; `charged` is the formatted cost, or null when unknown. */
export function unreportedChargeSummary(label: string, charged: string | null): string {
  return charged === null
    ? `${label} · usage unknown · full reservation charged`
    : `${label} · usage unknown · ${charged} charged (full reservation)`;
}

export const WAIT_STATE_LABEL: Record<string, string> = {
  yield_requested: "Yield requested",
  waiting: "Waiting",
  blocked: "Blocked",
  resumed: "Resumed",
  invalidated: "Invalidated",
};

/** `change-settings` becomes "Change settings"; the raw token stays visible elsewhere. */
export function humanizeToken(token: string): string {
  const text = token.replace(/[-_]+/g, " ").trim();
  return text.length === 0 ? token : text.charAt(0).toUpperCase() + text.slice(1);
}

// --- Output ------------------------------------------------------------------

const THINK_BLOCK = /<think>([\s\S]*?)(?:<\/think>|$)/gi;

/** Separates `<think>…</think>` blocks (an unterminated one runs to the end) from the answer. */
export function splitThinking(text: string): { answer: string; thoughts: string[] } {
  const thoughts: string[] = [];
  const answer = text
    .replace(THINK_BLOCK, (_match, thought: string) => {
      const trimmed = thought.trim();
      if (trimmed) thoughts.push(trimmed);
      return "";
    })
    .replace(/\n{3,}/g, "\n\n")
    .trim();
  return { answer, thoughts };
}

/** A string is shown as text; anything else as formatted JSON. */
export function outputText(output: unknown): string {
  if (output === null || output === undefined) return "";
  if (typeof output === "string") return output;
  try {
    return JSON.stringify(output, null, 2) ?? "";
  } catch {
    return String(output);
  }
}

/** About three lines of text; output beyond this folds behind "Show more". */
export const CLAMP_CHARACTERS = 240;
export const CLAMP_LINES = 3;

export function needsClamp(text: string): boolean {
  return text.length > CLAMP_CHARACTERS || text.split("\n").length > CLAMP_LINES;
}

// --- Time --------------------------------------------------------------------

/** "4 s ago", "3 min ago", "2 h ago". Negative differences (clock skew) read as "0 s ago". */
export function formatAge(milliseconds: number): string {
  const seconds = Math.max(0, Math.floor(milliseconds / 1000));
  if (seconds < 120) return `${seconds} s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 120) return `${minutes} min ago`;
  return `${Math.floor(minutes / 60)} h ago`;
}

// --- Routing-test reservation --------------------------------------------------

export interface ReservationDraft {
  tokens: string;
  /** Cost in currency units (for example dollars), not microunits. */
  cost: string;
  elapsedSeconds: string;
}

export interface ReservationValue {
  tokens: number;
  costMicrounits: number;
  elapsedSeconds: number;
}

export type ReservationParse = { ok: true; value: ReservationValue } | { ok: false; problem: string };

const MICROUNITS_PER_UNIT = 1_000_000;

function positiveInteger(text: string): number | null {
  if (!/^\d+$/.test(text.trim())) return null;
  const value = Number(text.trim());
  return Number.isSafeInteger(value) && value > 0 ? value : null;
}

/** Parses the three inputs. The cost field is currency units; the wire carries microunits. */
export function parseReservation(draft: ReservationDraft): ReservationParse {
  const tokens = positiveInteger(draft.tokens);
  if (tokens === null) return { ok: false, problem: "Tokens must be a whole number above zero." };
  const seconds = positiveInteger(draft.elapsedSeconds);
  if (seconds === null) return { ok: false, problem: "Elapsed seconds must be a whole number above zero." };
  const cost = Number(draft.cost.trim());
  if (draft.cost.trim() === "" || !Number.isFinite(cost) || cost < 0) {
    return { ok: false, problem: "Cost must be a number, zero or more." };
  }
  return { ok: true, value: { tokens, costMicrounits: Math.round(cost * MICROUNITS_PER_UNIT), elapsedSeconds: seconds } };
}

export function reservationToDraft(value: ReservationValue): ReservationDraft {
  return {
    tokens: String(value.tokens),
    cost: String(value.costMicrounits / MICROUNITS_PER_UNIT),
    elapsedSeconds: String(value.elapsedSeconds),
  };
}

/** What each field of a draft means on its own, so a valid field can be echoed with units while another is wrong. */
export interface ReservationHints {
  tokens: number | null;
  costMicrounits: number | null;
  elapsedSeconds: number | null;
}

export function reservationHints(draft: ReservationDraft): ReservationHints {
  const cost = Number(draft.cost.trim());
  return {
    tokens: positiveInteger(draft.tokens),
    costMicrounits: draft.cost.trim() === "" || !Number.isFinite(cost) || cost < 0 ? null : Math.round(cost * MICROUNITS_PER_UNIT),
    elapsedSeconds: positiveInteger(draft.elapsedSeconds),
  };
}

export interface RemainingBudget {
  tokens: number | null;
  costMicrounits: number | null;
  elapsedSeconds: number | null;
  slots: number | null;
}

/** What is left once a test is admitted: the reservation comes off every dimension and one task slot is used for good. */
export function remainingAfter(value: ReservationValue, remaining: RemainingBudget): RemainingBudget {
  const take = (left: number | null, amount: number): number | null => (left === null ? null : Math.max(0, left - amount));
  return {
    tokens: take(remaining.tokens, value.tokens),
    costMicrounits: take(remaining.costMicrounits, value.costMicrounits),
    elapsedSeconds: take(remaining.elapsedSeconds, value.elapsedSeconds),
    slots: take(remaining.slots, 1),
  };
}

/** The first dimension a reservation would exceed, worded for the control's reason text, or null. */
export function reservationOverrun(
  value: ReservationValue,
  remaining: RemainingBudget,
  formatCost: (microunits: number) => string,
): string | null {
  if (remaining.tokens !== null && value.tokens > remaining.tokens) {
    return `The token reservation exceeds the ${remaining.tokens.toLocaleString()} tokens left in the team budget.`;
  }
  if (remaining.costMicrounits !== null && value.costMicrounits > remaining.costMicrounits) {
    return `The cost reservation exceeds the ${formatCost(remaining.costMicrounits)} left in the team budget.`;
  }
  if (remaining.elapsedSeconds !== null && value.elapsedSeconds > remaining.elapsedSeconds) {
    return `The time reservation exceeds the ${remaining.elapsedSeconds.toLocaleString()} s left in the team budget.`;
  }
  return null;
}

// --- Routing-test steps ----------------------------------------------------------

export type StepName = "create" | "claim" | "admit";
export const STEP_ORDER: readonly StepName[] = ["create", "claim", "admit"];
export const STEP_LABEL: Record<StepName, string> = { create: "Create task", claim: "Claim", admit: "Admit" };

/** How a step ended without succeeding: the runtime said no, the outcome is unknown, or nothing left the console. */
export type StepFailure = "refused" | "unknown_outcome" | "not_sent";
export const STEP_FAILURE_LABEL: Record<StepFailure, string> = {
  refused: "Refused",
  unknown_outcome: "Outcome unknown",
  not_sent: "Not sent",
};

/**
 * A local precondition (`precondition`) stopped before anything was sent; an `uncertain` write or a network
 * failure may have been committed; anything else is the runtime's own answer.
 */
export function classifyStepFailure(error: { kind: string; uncertain: boolean }): StepFailure {
  if (error.kind === "precondition") return "not_sent";
  if (error.uncertain || error.kind === "network") return "unknown_outcome";
  return "refused";
}

export type StepState = "done" | "running" | "failed" | "next" | "later";

/** Step of the stored progress record that has been reached: how many of the three are done. */
export function stepsDone(step: string, failedStep: StepName | null): number {
  switch (step) {
    case "created":
      return 1;
    case "claimed":
      return 2;
    case "admitted":
      return 3;
    case "failed":
      return failedStep === null ? 0 : STEP_ORDER.indexOf(failedStep);
    default:
      return 0;
  }
}

export function stepState(index: number, done: number, step: string, running: StepName | null): StepState {
  if (index < done) return "done";
  if (index > done) return "later";
  if (running === STEP_ORDER[index]) return "running";
  return step === "failed" ? "failed" : "next";
}
