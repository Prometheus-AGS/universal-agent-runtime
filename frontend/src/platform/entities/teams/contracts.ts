/**
 * Wire contracts for `/api/v1/collaboration`, mirrored from the Rust domain
 * types (camelCase). These describe what the runtime sends and accepts; the
 * graph-stored shapes live in `./entities`.
 *
 * Sources: src/uar/domain/{collaboration,team_planning,team_execution,team_wait}.rs
 * and src/uar/compiler/collaboration/{export,bindings}.rs.
 */

export const TEAMS_API_BASE = "/api/v1/collaboration";
export const COLLABORATION_PROFILE = "urn:prometheus:uar:collaboration:0.1.0-draft.2";
export const BINDING_CAPABILITY = "collaboration_deployment_bindings_v2";
export const WORKSPACE_HEADER = "x-uar-workspace-id";

export interface ImmutableDefinitionRef {
  id: string;
  version: string;
  digest: string;
}

/** Defaults offered by the routing test. Cost is in microunits of the budget currency. */
export const DEFAULT_ROUTING_RESERVATION: TeamReservation = {
  tokens: 128_000,
  costMicrounits: 1_000_000,
  elapsedSeconds: 120,
};

// ---------------------------------------------------------------------------
// Definitions, packages, export
// ---------------------------------------------------------------------------

export interface TeamMemberSpec {
  role: string;
  kind: string;
  min: number;
  max: number;
  definition: ImmutableDefinitionRef;
}

/** Budget and limits are free-form JSON on the wire; these are the fields the runtime reads. */
export interface TeamBudget {
  maxTokens?: number;
  maxCostMicrounits?: number;
  /** Admission requires "USD" on both the definition and the binding. */
  currency?: string;
  maxElapsedSeconds?: number;
}
export interface TeamLimits {
  concurrentTurns?: number;
  maxMembers?: number;
  maxDepth?: number;
  maxPendingTasks?: number;
}

export interface TeamInstructions {
  revision: number;
  digest: string;
  text: string;
}

/** `GET /team-definitions` item. */
export interface TeamDefinitionSummary {
  instructions?: TeamInstructions;
  id: string;
  version: string;
  digest: string;
  title: string;
  purpose: string;
  package: ImmutableDefinitionRef;
  members: TeamMemberSpec[];
  budget: TeamBudget;
  limits: TeamLimits;
}

export interface PackageFile {
  path: string;
  kind: string;
  definition: ImmutableDefinitionRef;
  byteDigest: string;
}

export interface PackageManifest {
  profile: string;
  kind: "PackageManifest";
  id: string;
  version: string;
  contentDigest: string;
  provenance: Record<string, unknown>;
  requiredCapabilities: string[];
  extensions: Record<string, unknown>;
  entrypoints: ImmutableDefinitionRef[];
  files: PackageFile[];
  lock: Array<{ requestedBy: string; reference: ImmutableDefinitionRef; resolvedPath: string }>;
  capabilityDeclarations: Array<{ capability: string; required: boolean }>;
  resolution: string;
}

/** `GET /packages` item and `GET /packages/{id}/versions/{version}`. Carries no definition documents. */
export interface TeamPackageRecord {
  identity: ImmutableDefinitionRef;
  manifest: PackageManifest;
  sourceJson: string;
  definitionKeys: string[];
  installedAt: string;
}
/** The graph keeps the record without `sourceJson` (large, duplicates `manifest`). */
export type TeamPackage = Omit<TeamPackageRecord, "sourceJson">;

export type ConversionDisposition = "exact" | "translated" | "optional-unsupported" | "required-unsupported";

/** Preflight diagnostic; `field` is an RFC 6901 pointer (for an agent model it points into the agent, e.g. `/models/0`). */
export interface ConversionDiagnostic {
  field: string;
  disposition: ConversionDisposition;
  message: string;
}

export interface CollaborationCommandReceipt {
  commandId: string;
  ownerId: string;
  requestDigest: string;
  operation: string;
  resourceId: string;
  catalogRevision: number;
  bindingRevision: number | null;
  committedAt: string;
}

export interface PackageSourceRequest {
  commandId: string;
  expectedCatalogRevision?: number;
  /** Exact UTF-8 manifest bytes as a JSON string. */
  manifest: string;
  /** Exact UTF-8 file bytes keyed by manifest-relative path. */
  files: Record<string, string>;
}

export interface PackagePreflightResponse {
  package: ImmutableDefinitionRef;
  definitions: ImmutableDefinitionRef[];
  diagnostics: ConversionDiagnostic[];
  activationSupported: boolean;
  requestDigest: string;
}
export interface PackageInstallResponse {
  preflight: PackagePreflightResponse;
  receipt: CollaborationCommandReceipt;
}

export interface PackageExportRequest {
  package: ImmutableDefinitionRef;
  target: { kind: "canonicalDraft2" };
}
export interface ConversionReport {
  id: string;
  version: string;
  diagnostics: Array<{ pointer: string; disposition: ConversionDisposition; reasonCode: string; message: string }>;
  activationBlocked: boolean;
}
/** `POST /packages:export`. `files` maps manifest path to the definition's source JSON as a string. */
export type PackageExportOutcome =
  | {
      status: "exported";
      export: {
        package: ImmutableDefinitionRef;
        manifest: string;
        files: Record<string, string>;
        conversionReports: ConversionReport[];
      };
    }
  | { status: "refused"; reports: ConversionReport[] };

// ---------------------------------------------------------------------------
// Deployment bindings
// ---------------------------------------------------------------------------

export interface ModelBindingDocument {
  requestedAlias: string;
  providerId: string;
  modelId: string;
  credentialRef: string;
}
export interface BindingStorage {
  backend: string;
  connectionRef: string;
  durableTransactions: boolean;
}

/** The full `DeploymentBinding` document the console assembles and digests. */
export interface DeploymentBindingDocument {
  profile: typeof COLLABORATION_PROFILE;
  kind: "DeploymentBinding";
  id: string;
  version: string;
  provenance: { source: string; authors: string[] };
  requiredCapabilities: string[];
  extensions: Record<string, unknown>;
  exportClass: "private-installed-state";
  package: ImmutableDefinitionRef;
  ownerId: string;
  workspaceId: string;
  runtimeInstanceId: string;
  revision: number;
  modelBindings: ModelBindingDocument[];
  skillBindings: unknown[];
  storage: BindingStorage;
  policyRevision: string;
  effectiveLimits: TeamLimits;
  effectiveBudget: TeamBudget;
  contextGrants: unknown[];
  representationGrantRefs: unknown[];
  status: "ready";
  effectiveBindingReceiptRef: null;
  contentDigest: string;
}

/** `GET /deployment-bindings` item. */
export interface DeploymentBinding {
  id: string;
  ownerId: string;
  workspaceId: string;
  revision: number;
  document: DeploymentBindingDocument;
  package: ImmutableDefinitionRef;
  activationSupported: boolean;
  preflightDiagnostics: ConversionDiagnostic[];
  effectiveBindingReceipt?: Record<string, unknown>;
  updatedAt: string;
}

export interface BindingCommandRequest {
  commandId: string;
  expectedRevision?: number;
  binding: DeploymentBindingDocument;
}
export interface BindingPreflightResponse {
  bindingId: string;
  package: ImmutableDefinitionRef;
  diagnostics: ConversionDiagnostic[];
  activationSupported: boolean;
  requestDigest: string;
}
export interface BindingInstallResponse {
  preflight: BindingPreflightResponse;
  binding: DeploymentBinding;
  receipt: CollaborationCommandReceipt;
}

// ---------------------------------------------------------------------------
// Team instances
// ---------------------------------------------------------------------------

export interface TeamBindingRef {
  id: string;
  revision: number;
}

/** `status` is written "inactive" at creation and only ever becomes "revoked"; it is not an activity signal. */
export interface TeamMember {
  id: string;
  role: string;
  kind: string;
  ordinal: number;
  definition: ImmutableDefinitionRef;
  revision: number;
  status: string;
}

/**
 * Observed task statuses: queued, ready, waiting, blocked, succeeded, failed, cancelled.
 * Admission leaves a task `ready` (it bumps the revision); claim does not change status.
 */
export interface TeamTask {
  id: string;
  title: string;
  role: string;
  input: unknown;
  outputContract: unknown;
  output: unknown;
  dependsOn: string[];
  status: string;
  revision: number;
  assigneeMemberId?: string | null;
  ownershipEpoch?: number;
  reviewerMemberId?: string | null;
  stateReason?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface TeamInstance {
  id: string;
  ownerId: string;
  workspaceId: string;
  revision: number;
  status: string;
  definition: ImmutableDefinitionRef;
  package: ImmutableDefinitionRef;
  binding: TeamBindingRef;
  input: unknown;
  members: TeamMember[];
  tasks: TeamTask[];
  createdAt: string;
  updatedAt: string;
}

export interface MemberSlotRequest {
  role: string;
  count: number;
}
export interface CreateTeamRequest {
  commandId: string;
  deploymentBindingId: string;
  teamDefinition: ImmutableDefinitionRef;
  input: unknown;
  memberSlots: MemberSlotRequest[];
}
export interface CreateTeamTaskRequest {
  commandId: string;
  taskId: string;
  expectedTeamRevision: number;
  title: string;
  role: string;
  input: unknown;
  /** Must be a JSON Schema object; `{}` is accepted. */
  outputContract: Record<string, unknown>;
  dependsOn: string[];
}
export interface AssignTeamTaskRequest {
  commandId: string;
  expectedTeamRevision: number;
  expectedTaskRevision: number;
  memberId: string;
}
export interface TeamReservation {
  tokens: number;
  costMicrounits: number;
  elapsedSeconds: number;
}
export interface AdmitTeamTaskRequest {
  commandId: string;
  expectedTeamRevision: number;
  expectedTaskRevision: number;
  memberId: string;
  reservation: TeamReservation;
  contextArtifactIds: string[];
}
export interface TeamControlRequest {
  commandId: string;
  expectedTeamRevision: number;
  reason: string;
}

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

/** `queued`, `running` and `cancellation_requested` are in flight. `yielded` is a permanent record. */
export const TEAM_ATTEMPT_STATUSES = [
  "queued",
  "running",
  "cancellation_requested",
  "yielded",
  "succeeded",
  "failed",
  "cancelled",
  "uncertain",
] as const;
export type TeamAttemptStatus = (typeof TEAM_ATTEMPT_STATUSES)[number];

export const TEAM_ATTEMPT_IN_FLIGHT_STATUSES: readonly string[] = ["queued", "running", "cancellation_requested"];
/** Outcomes nothing further can happen to. `uncertain` may still become `yielded`; `yielded` is replaced by a new attempt on resume. */
export const TEAM_ATTEMPT_FINAL_STATUSES: readonly string[] = ["succeeded", "failed", "cancelled"];
export const TEAM_WAIT_LIVE_STATES: readonly string[] = ["yield_requested", "waiting", "blocked"];

export function isAttemptInFlight(status: string): boolean {
  return TEAM_ATTEMPT_IN_FLIGHT_STATUSES.includes(status);
}
export function isWaitLive(state: string): boolean {
  return TEAM_WAIT_LIVE_STATES.includes(state);
}
/**
 * The polling rule. A run is live while any attempt is in flight or any wait can still resume.
 * A status-only terminal test is wrong: `yielded` stays on the original attempt after resumption.
 */
export function isTeamRunLive(
  attempts: ReadonlyArray<{ status: string }>,
  waits: ReadonlyArray<{ state: string }>,
): boolean {
  return attempts.some((attempt) => isAttemptInFlight(attempt.status)) || waits.some((wait) => isWaitLive(wait.state));
}

export interface TeamExecutionFence {
  catalogId: string;
  serviceInstanceId: string;
  incarnationId: string;
  epoch: number;
}
export interface TeamExecutionDiagnostic {
  code: string;
  field?: string;
  retryable: boolean;
  action: string;
  protectedDiagnosticRef?: string;
  sourceStage?: string;
  category?: string;
  httpStatus?: number;
  collaborationCode?: string;
}
export interface EffectiveTeamModelReceipt {
  route: { providerId: string; modelId: string };
  wireModelAlias: string;
  pricingIdentity?: { providerId: string; modelId: string; catalogRevision: string };
  endpointKind: string;
  settingsRevision: number;
  support: string;
  limits: Record<string, unknown>;
  fit: Record<string, unknown>;
}
export interface TeamExecutionAttempt {
  id: string;
  runId: string;
  rootId: string;
  approvalScopeId: string;
  queueSequence: number;
  continuationOfWaitId?: string;
  ownerId: string;
  workspaceId: string;
  teamId: string;
  taskId: string;
  memberId: string;
  memberRevision: number;
  ownershipEpoch: number;
  bindingRevision: number;
  executionEpoch: number;
  executionFence: TeamExecutionFence | null;
  effectiveModels: EffectiveTeamModelReceipt[];
  effectDisposition: string;
  accountingState: string;
  status: string;
  executionOutcome: string | null;
  reservation: TeamReservation;
  contextArtifactIds: string[];
  usage: TeamReservation | null;
  usageRevision: number;
  output: unknown;
  stateReason: string | null;
  diagnostic?: TeamExecutionDiagnostic;
  createdAt: string;
  updatedAt: string;
}

export interface AttributedPayload {
  text: string;
  artifactIds: string[];
}
export interface TargetOutcome {
  taskId: string;
  memberId: string;
  attemptId: string;
  executionOutcome: string;
  effectDisposition: string;
  artifactIds: string[];
}
export interface AttemptAuthority {
  ownerId: string;
  workspaceId: string;
  teamId: string;
  taskId: string;
  memberId: string;
  attemptId: string;
  runId: string;
  taskOwnershipEpoch: number;
  memberRevision: number;
}
/** States: yield_requested, waiting, blocked (live); resumed, invalidated (settled). */
export interface TeamWait {
  waitId: string;
  authority: AttemptAuthority;
  targetTaskIds: string[];
  predicate: string;
  continuationInput: AttributedPayload;
  continuationReservation: TeamReservation;
  state: string;
  continuationAttemptId?: string;
  wakeOutcomes: TargetOutcome[];
  reasonCode?: string;
}
export interface ContinuationReceipt {
  waitId: string;
  uniquenessKey: string;
  previousAttemptId: string;
  rootId: string;
  taskId: string;
  continuationAttemptId: string;
  runId: string;
  selectedArtifactIds: string[];
  targetOutcomes: TargetOutcome[];
  committedAt: string;
}
/** Operations seen: team_delegate, team_send, team_wait. A `team_delegate` receipt links a delegated task to its delegator. */
export interface TeamPeerCommandReceipt {
  commandId: string;
  requestDigest: string;
  operation: string;
  senderMemberId: string;
  senderAttemptId: string;
  acceptedAt: string;
  messageId?: string;
  taskId?: string;
  attemptId?: string;
  waitId?: string;
}
export interface TeamExecutionSummary {
  attempts: TeamExecutionAttempt[];
  committed: TeamReservation;
  reserved: TeamReservation;
  uncertainAttempts: string[];
  commandReceipts: TeamPeerCommandReceipt[];
  waits: TeamWait[];
  continuations: ContinuationReceipt[];
  contextReceipts: unknown[];
  budget: TeamBudget;
  limits: TeamLimits;
}

export interface TeamArtifact {
  id: string;
  ownerId: string;
  workspaceId: string;
  teamId: string;
  taskId: string;
  memberId: string;
  attemptId: string;
  content: unknown;
  createdAt: string;
}
export interface TeamPeerMessage {
  messageId: string;
  senderMemberId: string;
  senderAttemptId: string;
  recipientMemberId: string;
  recipientTaskId?: string;
  mode: string;
  payload: AttributedPayload;
  acceptedAt: string;
}
export interface MessageDelivery {
  messageId: string;
  recipientMemberId: string;
  recipientTaskId?: string;
  status: string;
  selectedAttemptId?: string;
  selectedAt?: string;
  consumedAt?: string;
  rejectionCode?: string;
}
/** `GET /team-instances/{id}/peer-messages` item. */
export interface PeerMessageRecord {
  message: TeamPeerMessage;
  delivery: MessageDelivery | null;
}

// ---------------------------------------------------------------------------
// Runtime readiness
// ---------------------------------------------------------------------------

export interface ExecutionClaim extends TeamExecutionFence {
  state: string;
  acquiredAt: string;
  auditReceiptId: string;
  releasedAt?: string;
}
/** `GET /execution-owner`. Read-only in the console. */
export interface ExecutionOwnerView {
  claim: ExecutionClaim | null;
  currentFence: TeamExecutionFence;
  ownsExecution: boolean;
}

/** The slice of `GET /capabilities` the console reads. Top-level keys other than `executionProfile` are snake_case. */
export interface CollaborationCapabilities {
  capabilities: string[];
  instance: { id: string; profile: string };
  collaboration: {
    activation: {
      teamPlanning: boolean;
      taskOwnership: boolean;
      teamMailbox: boolean;
      teamInstance: boolean;
      teamExecution: boolean;
      workflow: boolean;
    };
  };
  bindingOwnerId: string;
  catalogStorage: { backend: string };
}
