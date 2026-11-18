import {
  TEAMS_API_BASE,
  WORKSPACE_HEADER,
  type AdmitTeamTaskRequest,
  type AssignTeamTaskRequest,
  type BindingCommandRequest,
  type BindingInstallResponse,
  type BindingPreflightResponse,
  type CollaborationCapabilities,
  type CreateTeamRequest,
  type CreateTeamTaskRequest,
  type DeploymentBinding,
  type ExecutionOwnerView,
  type ImmutableDefinitionRef,
  type PackageExportOutcome,
  type PackageInstallResponse,
  type PackagePreflightResponse,
  type PackageSourceRequest,
  type PeerMessageRecord,
  type TeamArtifact,
  type TeamControlRequest,
  type TeamDefinitionSummary,
  type TeamExecutionAttempt,
  type TeamExecutionSummary,
  type TeamInstance,
  type TeamPackageRecord,
} from "../contracts";
import { TeamsApiError, classifyFailure } from "./teams-errors";

/**
 * Transport for `/api/v1/collaboration`. It owns every request; nothing else
 * in the teams domain calls `fetch`. Workspace-scoped calls take the workspace
 * as an argument and send `x-uar-workspace-id`. Command ids are generated here
 * unless the caller passes one (a retry of an uncertain command must reuse it).
 */

export interface ReadOptions {
  signal?: AbortSignal;
}

function authorizationHeaders(): Record<string, string> {
  const credential = import.meta.env.VITE_UAR_API_KEY ?? "";
  if (!credential) return {};
  return credential.startsWith("ey") ? { Authorization: `Bearer ${credential}` } : { "x-api-key": credential };
}

function workspaceHeader(workspace: string): Record<string, string> {
  const value = workspace.trim();
  if (!value) {
    throw new TeamsApiError({
      kind: "workspace_required", status: 0, code: "workspace_required", detail: null,
      message: "Enter a workspace id first.", capability: null, uncertain: false,
    });
  }
  if (!/^[\x21-\x7e]+$/.test(value)) {
    throw new TeamsApiError({
      kind: "invalid", status: 0, code: "workspace_invalid", detail: null,
      message: "A workspace id may contain only printable ASCII characters without spaces.", capability: null, uncertain: false,
    });
  }
  return { [WORKSPACE_HEADER]: value };
}

async function request<T>(
  method: "GET" | "POST",
  path: string,
  scope: Record<string, string>,
  body?: unknown,
  signal?: AbortSignal,
): Promise<T> {
  const writing = method !== "GET";
  let response: Response;
  try {
    response = await fetch(`${TEAMS_API_BASE}${path}`, {
      method,
      headers: { "Content-Type": "application/json", ...authorizationHeaders(), ...scope },
      body: body === undefined ? undefined : JSON.stringify(body),
      cache: "no-store",
      signal,
    });
  } catch (error) {
    if (signal?.aborted) throw error;
    throw new TeamsApiError({
      kind: "network", status: 0, code: "network", detail: null, capability: null, uncertain: writing,
      message: writing
        ? "The connection ended before the result was confirmed. Reload before sending the command again, or retry it with the same command id."
        : "Could not reach the runtime. Check the connection and reload.",
    });
  }
  if (!response.ok) {
    const failure: unknown = response.status === 429 ? null : await response.json().catch(() => null);
    throw classifyFailure(response.status, failure, writing);
  }
  try {
    return (await response.json()) as T;
  } catch {
    throw new TeamsApiError({
      kind: "malformed", status: response.status, code: "malformed_response", detail: null, capability: null, uncertain: writing,
      message: "The runtime returned an unreadable response.",
    });
  }
}

function malformed(what: string): TeamsApiError {
  return new TeamsApiError({
    kind: "malformed", status: 0, code: "malformed_response", detail: null, capability: null, uncertain: false,
    message: `The runtime returned an unexpected shape for ${what}.`,
  });
}

function expectArray<T>(value: unknown, what: string): T[] {
  if (!Array.isArray(value) || value.some((item) => item === null || typeof item !== "object")) throw malformed(what);
  return value as T[];
}

function expectObject<T extends object>(value: unknown, what: string, ...required: Array<keyof T & string>): T {
  if (value === null || typeof value !== "object" || Array.isArray(value)) throw malformed(what);
  for (const key of required) if (!(key in value)) throw malformed(what);
  return value as T;
}

const enc = encodeURIComponent;
const teamPath = (teamId: string) => `/team-instances/${enc(teamId)}`;
const command = (commandId?: string) => commandId ?? crypto.randomUUID();

// --- Catalog-wide reads and package commands (no workspace) -----------------

export async function fetchCapabilities(options: ReadOptions = {}): Promise<CollaborationCapabilities> {
  const value = await request<unknown>("GET", "/capabilities", {}, undefined, options.signal);
  return expectObject<CollaborationCapabilities>(value, "capabilities", "capabilities", "instance", "bindingOwnerId", "collaboration");
}

export async function fetchExecutionOwner(options: ReadOptions = {}): Promise<ExecutionOwnerView> {
  const value = await request<unknown>("GET", "/execution-owner", {}, undefined, options.signal);
  return expectObject<ExecutionOwnerView>(value, "the execution owner", "ownsExecution", "currentFence");
}

export async function fetchTeamDefinitions(options: ReadOptions = {}): Promise<TeamDefinitionSummary[]> {
  return expectArray(await request<unknown>("GET", "/team-definitions", {}, undefined, options.signal), "team definitions");
}

export async function fetchPackages(options: ReadOptions = {}): Promise<TeamPackageRecord[]> {
  return expectArray(await request<unknown>("GET", "/packages", {}, undefined, options.signal), "packages");
}

/** The only source of definition documents (coordinator role, requested model aliases). */
export async function exportPackage(pkg: ImmutableDefinitionRef, options: ReadOptions = {}): Promise<PackageExportOutcome> {
  const value = await request<unknown>(
    "POST", "/packages:export", {}, { package: pkg, target: { kind: "canonicalDraft2" } }, options.signal,
  );
  return expectObject<PackageExportOutcome>(value, "the package export", "status");
}

export async function preflightPackage(input: Omit<PackageSourceRequest, "commandId"> & { commandId?: string }): Promise<PackagePreflightResponse> {
  const value = await request<unknown>("POST", "/packages:preflight", {}, { ...input, commandId: command(input.commandId) });
  return expectObject<PackagePreflightResponse>(value, "package preflight", "activationSupported", "diagnostics");
}

export async function installPackage(input: Omit<PackageSourceRequest, "commandId"> & { commandId?: string }): Promise<PackageInstallResponse> {
  const value = await request<unknown>("POST", "/packages:install", {}, { ...input, commandId: command(input.commandId) });
  return expectObject<PackageInstallResponse>(value, "package install", "preflight", "receipt");
}

// --- Workspace-scoped bindings ------------------------------------------------

export async function fetchBindings(workspace: string, options: ReadOptions = {}): Promise<DeploymentBinding[]> {
  const scope = workspaceHeader(workspace);
  return expectArray(await request<unknown>("GET", "/deployment-bindings", scope, undefined, options.signal), "bindings");
}

type BindingInput = Omit<BindingCommandRequest, "commandId"> & { commandId?: string };

export async function preflightBinding(workspace: string, input: BindingInput): Promise<BindingPreflightResponse> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("POST", "/deployment-bindings:preflight", scope, { ...input, commandId: command(input.commandId) });
  return expectObject<BindingPreflightResponse>(value, "binding preflight", "activationSupported", "diagnostics");
}

export async function installBinding(workspace: string, input: BindingInput): Promise<BindingInstallResponse> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("POST", "/deployment-bindings", scope, { ...input, commandId: command(input.commandId) });
  return expectObject<BindingInstallResponse>(value, "binding install", "preflight", "binding");
}

// --- Workspace-scoped teams -----------------------------------------------------

export async function fetchTeams(workspace: string, options: ReadOptions = {}): Promise<TeamInstance[]> {
  const scope = workspaceHeader(workspace);
  return expectArray(await request<unknown>("GET", "/team-instances", scope, undefined, options.signal), "teams");
}

export async function fetchTeam(workspace: string, teamId: string, options: ReadOptions = {}): Promise<TeamInstance> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("GET", teamPath(teamId), scope, undefined, options.signal);
  return expectObject<TeamInstance>(value, "a team", "id", "members", "tasks");
}

export async function createTeam(workspace: string, input: Omit<CreateTeamRequest, "commandId"> & { commandId?: string }): Promise<TeamInstance> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("POST", "/team-instances", scope, { ...input, commandId: command(input.commandId) });
  return expectObject<TeamInstance>(value, "a created team", "id", "members", "tasks");
}

export async function createTask(workspace: string, teamId: string, input: Omit<CreateTeamTaskRequest, "commandId"> & { commandId?: string }): Promise<TeamInstance> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("POST", `${teamPath(teamId)}/tasks`, scope, { ...input, commandId: command(input.commandId) });
  return expectObject<TeamInstance>(value, "a team after task creation", "id", "tasks");
}

export async function claimTask(workspace: string, teamId: string, taskId: string, input: Omit<AssignTeamTaskRequest, "commandId"> & { commandId?: string }): Promise<TeamInstance> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("POST", `${teamPath(teamId)}/tasks/${enc(taskId)}/claim`, scope, { ...input, commandId: command(input.commandId) });
  return expectObject<TeamInstance>(value, "a team after a claim", "id", "tasks");
}

/** Admission can commit and then fail at dispatch; retry only with the same command id. */
export async function admitTask(workspace: string, teamId: string, taskId: string, input: Omit<AdmitTeamTaskRequest, "commandId"> & { commandId?: string }): Promise<TeamExecutionAttempt> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("POST", `${teamPath(teamId)}/tasks/${enc(taskId)}/admit`, scope, { ...input, commandId: command(input.commandId) });
  return expectObject<TeamExecutionAttempt>(value, "an admitted attempt", "id", "status");
}

export async function cancelAttempt(workspace: string, teamId: string, attemptId: string, input: Omit<TeamControlRequest, "commandId"> & { commandId?: string }): Promise<TeamExecutionAttempt> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("POST", `${teamPath(teamId)}/attempts/${enc(attemptId)}/cancel`, scope, { ...input, commandId: command(input.commandId) });
  return expectObject<TeamExecutionAttempt>(value, "a cancelled attempt", "id", "status");
}

// --- Workspace-scoped run reads ---------------------------------------------------

export async function fetchExecution(workspace: string, teamId: string, options: ReadOptions = {}): Promise<TeamExecutionSummary> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("GET", `${teamPath(teamId)}/execution`, scope, undefined, options.signal);
  return expectObject<TeamExecutionSummary>(value, "team execution", "attempts", "waits");
}

export async function fetchArtifacts(workspace: string, teamId: string, options: ReadOptions = {}): Promise<TeamArtifact[]> {
  const scope = workspaceHeader(workspace);
  return expectArray(await request<unknown>("GET", `${teamPath(teamId)}/artifacts`, scope, undefined, options.signal), "artifacts");
}

export async function fetchPeerMessages(workspace: string, teamId: string, options: ReadOptions = {}): Promise<PeerMessageRecord[]> {
  const scope = workspaceHeader(workspace);
  const value = await request<unknown>("GET", `${teamPath(teamId)}/peer-messages`, scope, undefined, options.signal);
  return expectArray(expectObject<{ messages: unknown }>(value, "peer messages", "messages").messages, "peer messages");
}
