import { formatDateTime } from "./format";

/**
 * The four readiness facts of Decision 2, derived from runtime load state. Pure so the
 * wording, the 401 behaviour and the action gates are testable without a graph.
 */

export type FactState = "ok" | "blocked" | "unknown" | "checking";
export type FactId = "identity" | "workspace" | "instance" | "ownership";
export type TeamAction = "install" | "bind" | "launch" | "routing";

export interface ReadinessFact {
  id: FactId;
  label: string;
  state: FactState;
  summary: string;
  /** What stops working while this fact is not satisfied. */
  blocks: string;
}

type Load = "idle" | "loading" | "ready" | "error";
interface LoadFailure {
  kind: string;
  message: string;
}

export interface ReadinessInputs {
  workspace: string;
  ownerId: string | null;
  capabilitiesStatus: Load;
  capabilitiesError: LoadFailure | null;
  instanceId: string | null;
  teamExecutionAvailable: boolean | null;
  ownershipStatus: Load;
  ownershipError: LoadFailure | null;
  ownsExecution: boolean | null;
  claimEpoch: number | null;
  claimAcquiredAt: string | null;
}

const BLOCKS: Record<FactId, string> = {
  identity: "Everything on this page.",
  workspace: "Bindings, teams and routing tests.",
  instance: "Bindings, and so launching teams.",
  ownership: "Team bindings and routing tests.",
};

const LABELS: Record<FactId, string> = {
  identity: "Verified owner",
  workspace: "Workspace",
  instance: "Runtime instance id",
  ownership: "Execution ownership",
};

const RECOVERY_DOCUMENT = "docs/operations/supervisor-recovery.md";

function fact(id: FactId, state: FactState, summary: string): ReadinessFact {
  return { id, label: LABELS[id], state, summary, blocks: BLOCKS[id] };
}

function identityFact(input: ReadinessInputs): ReadinessFact {
  if (input.ownerId) return fact("identity", "ok", "The runtime recognises you as a verified owner.");
  const error = input.capabilitiesError;
  if (error?.kind === "unauthorized") {
    return fact("identity", "blocked", "A verified owner is required. Open the console through the identity host, then reload this page.");
  }
  if (error) return fact("identity", "unknown", `The owner could not be checked: ${error.message}`);
  return fact("identity", "checking", "Checking the verified owner.");
}

function instanceFact(input: ReadinessInputs): ReadinessFact {
  if (input.instanceId === null) return fact("instance", "checking", "Reading the runtime's instance id.");
  if (input.instanceId === "") {
    return fact("instance", "blocked", "The host has not configured a runtime instance id. Bindings need one; the host operator sets it.");
  }
  return fact("instance", "ok", `Instance id ${input.instanceId}.`);
}

function ownershipFact(input: ReadinessInputs): ReadinessFact {
  // The server reports team execution as unavailable whenever it does not own execution, so the
  // ownership answer must be read first or the "another incarnation holds it" state is unreachable.
  if (input.ownsExecution === false) {
    const holder = input.claimEpoch === null
      ? "No execution claim is held."
      : `The claim is held at epoch ${input.claimEpoch}, acquired ${formatDateTime(input.claimAcquiredAt)}, by another incarnation.`;
    return fact(
      "ownership",
      "blocked",
      `${holder} The host operator must recover ownership (see ${RECOVERY_DOCUMENT}). The console cannot reclaim it.`,
    );
  }
  if (input.teamExecutionAvailable === false) {
    return fact("ownership", "blocked", "The runtime reports team execution as unavailable.");
  }
  if (input.ownsExecution === true) {
    const epoch = input.claimEpoch === null ? "" : ` at epoch ${input.claimEpoch}`;
    return fact("ownership", "ok", `This runtime holds team execution${epoch}.`);
  }
  if (input.ownershipError) return fact("ownership", "unknown", `Ownership could not be read: ${input.ownershipError.message}`);
  return fact("ownership", "checking", "Reading who holds team execution.");
}

function dependent(id: FactId, identity: ReadinessFact): ReadinessFact {
  if (identity.state === "checking") return fact(id, "checking", "Waiting for the owner check.");
  return fact(id, "unknown", "Not checked: there is no verified owner.");
}

export function deriveReadiness(input: ReadinessInputs): ReadinessFact[] {
  const identity = identityFact(input);
  if (identity.state !== "ok") {
    return [identity, dependent("workspace", identity), dependent("instance", identity), dependent("ownership", identity)];
  }
  const workspace = input.workspace.trim()
    ? fact("workspace", "ok", `Workspace "${input.workspace}" is selected.`)
    : fact("workspace", "blocked", "Choose a workspace.");
  return [identity, workspace, instanceFact(input), ownershipFact(input)];
}

const REQUIRED: Record<TeamAction, readonly FactId[]> = {
  install: ["identity"],
  bind: ["identity", "workspace", "instance", "ownership"],
  launch: ["identity", "workspace", "instance"],
  routing: ["identity", "workspace", "instance", "ownership"],
};

/** The first unmet precondition for an action, worded for the control's adjacent reason text; null when allowed. */
export function gateReason(facts: readonly ReadinessFact[], action: TeamAction): string | null {
  for (const id of REQUIRED[action]) {
    const blocking = facts.find((entry) => entry.id === id && entry.state !== "ok");
    if (!blocking) continue;
    if (blocking.state === "checking") return `${blocking.label} is still being checked.`;
    if (blocking.state === "unknown" && id !== "identity") return `${blocking.label} is not known yet. ${blocking.summary}`;
    return `${blocking.label}: ${blocking.summary}`;
  }
  return null;
}
