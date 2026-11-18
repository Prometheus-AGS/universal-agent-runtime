import type {
  ConversionDiagnostic,
  DeploymentBindingDocument,
  EffectiveTeamModelReceipt,
  ExecutionClaim,
  ImmutableDefinitionRef,
  PackageManifest,
  TargetOutcome,
  TeamBindingRef,
  TeamBudget,
  TeamExecutionDiagnostic,
  TeamExecutionFence,
  TeamInstructions,
  TeamLimits,
  TeamMemberSpec,
  TeamReservation,
  TeamPeerCommandReceipt,
  ContinuationReceipt,
  AttributedPayload,
  AdmitTeamTaskRequest,
  MessageDelivery,
} from "./contracts";
import type { TeamsErrorInfo } from "./api/teams-errors";
import type { PackageExportModel } from "./export-model";

/**
 * Graph-stored shapes. Every record's `id` is the graph id: qualified by the
 * authenticated owner (and workspace for workspace-scoped records) because the
 * graph is persisted. The server's own id sits in a named field (`teamId`, ...).
 * Hooks resolve the owner from the per-session runtime record, so a persisted
 * graph never shows one principal's records to another.
 */

export const TEAM_RUNTIME_ENTITY = "TeamRuntime";
export const TEAM_LOAD_STATE_ENTITY = "TeamLoadState";
export const TEAM_DEFINITION_ENTITY = "TeamDefinition";
export const TEAM_PACKAGE_ENTITY = "TeamPackage";
export const TEAM_PACKAGE_EXPORT_ENTITY = "TeamPackageExport";
export const TEAM_BINDING_ENTITY = "TeamDeploymentBinding";
export const TEAM_INSTANCE_ENTITY = "TeamInstance";
export const TEAM_MEMBER_ENTITY = "TeamMember";
export const TEAM_TASK_ENTITY = "TeamTask";
export const TEAM_ATTEMPT_ENTITY = "TeamAttempt";
export const TEAM_WAIT_ENTITY = "TeamWait";
export const TEAM_RUN_SUMMARY_ENTITY = "TeamRunSummary";
export const TEAM_ARTIFACT_ENTITY = "TeamArtifact";
export const TEAM_PEER_MESSAGE_ENTITY = "TeamPeerMessage";
export const TEAM_RUN_WATCH_ENTITY = "TeamRunWatch";
export const TEAM_ROUTING_PROGRESS_ENTITY = "TeamRoutingProgress";

/** Fresh per application session: hydrated state cannot manufacture a ready runtime or a live watch. */
export const TEAMS_SESSION_ID = crypto.randomUUID();
export const TEAMS_RUNTIME_ID = "current";

const part = (value: string) => `${value.length}:${value}`;
/** Catalog-wide record id: `<owner>|<key>`, each length-prefixed. */
export const catalogEntityId = (owner: string, key: string) => `${part(owner)}|${part(key)}`;
/** Workspace-scoped record id: `<owner>|<workspace>|<key...>`. */
export const scopedEntityId = (owner: string, workspace: string, ...keys: string[]) =>
  [part(owner), part(workspace), ...keys.map(part)].join("|");
/** The last domain id inside a graph id built by the helpers above (`teamId`, `memberId`, `definitionKey`, ...). */
export function lastIdSegment(graphId: string): string {
  let rest = graphId;
  let last = graphId;
  while (rest.length > 0) {
    const colon = rest.indexOf(":");
    const length = colon > 0 ? Number(rest.slice(0, colon)) : Number.NaN;
    if (!Number.isInteger(length)) return last;
    last = rest.slice(colon + 1, colon + 1 + length);
    rest = rest.slice(colon + 1 + length + 1);
  }
  return last;
}

/** `id@version`, the readable key of a definition or package (the URL form). */
export const definitionKey = (ref: Pick<ImmutableDefinitionRef, "id" | "version">) => `${ref.id}@${ref.version}`;

export const listKey = (name: string, owner: string, workspace?: string, ...keys: string[]) =>
  `teams:${name}:${workspace === undefined ? part(owner) : scopedEntityId(owner, workspace, ...keys)}`;

export type LoadStatus = "idle" | "loading" | "ready" | "error";

export interface TeamRuntimeState {
  id: string;
  session: string;
  ownerId: string | null;
  instanceId: string | null;
  catalogBackend: string | null;
  capabilityNames: string[] | null;
  teamExecutionAvailable: boolean | null;
  ownsExecution: boolean | null;
  claim: ExecutionClaim | null;
  currentFence: TeamExecutionFence | null;
  capabilitiesLoadedAt: number | null;
  executionOwnerLoadedAt: number | null;
}

export interface TeamLoadState {
  id: string;
  session: string;
  status: LoadStatus;
  error: TeamsErrorInfo | null;
  completedAt: number | null;
}

export interface TeamDefinitionEntity {
  id: string;
  ownerId: string;
  definitionKey: string;
  definitionId: string;
  version: string;
  digest: string;
  title: string;
  purpose: string;
  package: ImmutableDefinitionRef;
  members: TeamMemberSpec[];
  budget: TeamBudget;
  limits: TeamLimits;
  instructions: TeamInstructions | null;
}

export interface TeamPackageEntity {
  id: string;
  ownerId: string;
  packageKey: string;
  packageId: string;
  version: string;
  digest: string;
  manifest: PackageManifest;
  definitionKeys: string[];
  installedAt: string;
}

export interface TeamPackageExportEntity {
  id: string;
  ownerId: string;
  packageKey: string;
  model: PackageExportModel;
  loadedAt: number;
}

export interface TeamBindingEntity {
  id: string;
  ownerId: string;
  workspace: string;
  bindingId: string;
  revision: number;
  package: ImmutableDefinitionRef;
  packageKey: string;
  document: DeploymentBindingDocument;
  activationSupported: boolean;
  preflightDiagnostics: ConversionDiagnostic[];
  effectiveBindingReceipt: Record<string, unknown> | null;
  updatedAt: string;
}

export interface TeamInstanceEntity {
  id: string;
  ownerId: string;
  workspace: string;
  teamId: string;
  revision: number;
  /** Written "inactive" at creation and never advanced; not an activity signal. */
  status: string;
  definition: ImmutableDefinitionRef;
  package: ImmutableDefinitionRef;
  binding: TeamBindingRef;
  input: unknown;
  createdAt: string;
  updatedAt: string;
}

export interface TeamMemberEntity {
  id: string;
  ownerId: string;
  workspace: string;
  teamId: string;
  memberId: string;
  role: string;
  kind: string;
  ordinal: number;
  definition: ImmutableDefinitionRef;
  revision: number;
  status: string;
}

export interface TeamTaskEntity {
  id: string;
  ownerId: string;
  workspace: string;
  teamId: string;
  taskId: string;
  title: string;
  role: string;
  input: unknown;
  outputContract: unknown;
  output: unknown;
  dependsOn: string[];
  status: string;
  revision: number;
  assigneeMemberId: string | null;
  reviewerMemberId: string | null;
  stateReason: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface TeamAttemptEntity {
  id: string;
  ownerId: string;
  workspace: string;
  teamId: string;
  attemptId: string;
  runId: string;
  taskId: string;
  memberId: string;
  status: string;
  executionOutcome: string | null;
  effectDisposition: string;
  accountingState: string;
  reservation: TeamReservation;
  usage: TeamReservation | null;
  output: unknown;
  stateReason: string | null;
  diagnostic: TeamExecutionDiagnostic | null;
  effectiveModels: EffectiveTeamModelReceipt[];
  continuationOfWaitId: string | null;
  contextArtifactIds: string[];
  queueSequence: number;
  createdAt: string;
  updatedAt: string;
}

export interface TeamWaitEntity {
  id: string;
  ownerId: string;
  workspace: string;
  teamId: string;
  waitId: string;
  /** The yielded attempt this wait belongs to. */
  attemptId: string;
  taskId: string;
  memberId: string;
  targetTaskIds: string[];
  predicate: string;
  /** yield_requested, waiting, blocked (live); resumed, invalidated (settled). */
  state: string;
  continuationAttemptId: string | null;
  reasonCode: string | null;
  wakeOutcomes: TargetOutcome[];
  continuationInput: AttributedPayload;
}

/** The execution summary's team-wide figures plus the links between tasks and attempts. */
export interface TeamRunSummaryEntity {
  id: string;
  ownerId: string;
  workspace: string;
  teamId: string;
  committed: TeamReservation;
  reserved: TeamReservation;
  uncertainAttemptIds: string[];
  budget: TeamBudget;
  limits: TeamLimits;
  /** `team_delegate` receipts: the only link from a delegated task back to its delegator. */
  delegations: TeamPeerCommandReceipt[];
  continuations: ContinuationReceipt[];
  loadedAt: number;
}

export interface TeamArtifactEntity {
  id: string;
  ownerId: string;
  workspace: string;
  teamId: string;
  artifactId: string;
  taskId: string;
  memberId: string;
  attemptId: string;
  content: unknown;
  createdAt: string;
}

export interface TeamPeerMessageEntity {
  id: string;
  ownerId: string;
  workspace: string;
  teamId: string;
  messageId: string;
  senderMemberId: string;
  senderAttemptId: string;
  recipientMemberId: string;
  recipientTaskId: string | null;
  mode: string;
  payload: AttributedPayload;
  acceptedAt: string;
  delivery: MessageDelivery | null;
}

export interface TeamRunWatchState {
  id: string;
  session: string;
  status: "watching" | "stopped";
  live: boolean;
  intervalMs: number;
  lastUpdatedAt: number | null;
  lastError: TeamsErrorInfo | null;
  consecutiveFailures: number;
}

export type RoutingStep = "pending" | "created" | "claimed" | "admitted" | "failed";
export type RoutingCommandStep = "create" | "claim" | "admit";

/**
 * The routing test's progress for one task. The owed step cannot be derived from
 * the task alone: admission leaves the task `ready`, and a commit followed by a
 * failed dispatch is only retryable with the same command id.
 */
export interface TeamRoutingProgress {
  id: string;
  ownerId: string;
  workspace: string;
  teamId: string;
  taskId: string;
  role: string;
  title: string;
  request: string;
  reservation: TeamReservation;
  contextArtifactIds: string[];
  memberId: string | null;
  createCommandId: string;
  claimCommandId: string;
  admitCommandId: string;
  attemptId: string | null;
  /** The exact admit request first sent, kept so a committed-but-undispatched admission is retried byte-identically (the runtime digests expected revisions too). */
  admitRequest: AdmitTeamTaskRequest | null;
  step: RoutingStep;
  /** The command that failed when `step` is `failed`; the one to retry. */
  failedStep: RoutingCommandStep | null;
  /** The command in flight right now, if any. */
  running: RoutingCommandStep | null;
  /** Session that started `running`; a persisted value from an earlier session is stale. */
  runningSession: string | null;
  lastError: TeamsErrorInfo | null;
  updatedAt: number;
}

/** The step still owed, or null when the test is admitted. */
export function owedRoutingStep(progress: Pick<TeamRoutingProgress, "step" | "failedStep">): RoutingCommandStep | null {
  switch (progress.step) {
    case "pending":
      return "create";
    case "created":
      return "claim";
    case "claimed":
      return "admit";
    case "failed":
      return progress.failedStep;
    case "admitted":
      return null;
  }
}
