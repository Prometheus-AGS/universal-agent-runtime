import { batchGraphWrites, type GraphBatch } from "@prometheus-ags/entity-graph-core";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import {
  exportPackage,
  fetchArtifacts,
  fetchBindings,
  fetchCapabilities,
  fetchExecution,
  fetchExecutionOwner,
  fetchPackages,
  fetchPeerMessages,
  fetchTeam,
  fetchTeamDefinitions,
  fetchTeams,
  type ReadOptions,
} from "./api/teams-transport";
import type {
  CollaborationCapabilities,
  ExecutionOwnerView,
  ImmutableDefinitionRef,
  TeamExecutionSummary,
  TeamInstance,
} from "./contracts";
import { isTeamRunLive } from "./contracts";
import { TEAM_MEMBER_ENTITY, definitionKey } from "./entities";
import type { PackageExportModel } from "./export-model";
import {
  prependTeam,
  writeArtifacts,
  writeBindings,
  writeDefinitions,
  writeExecution,
  writePackageExport,
  writePackages,
  writePeerMessages,
  writeTeam,
  writeTeams,
} from "./ingest";
import { lists, teamGid } from "./keys";
import { TeamsApiError, toErrorInfo } from "./api/teams-errors";
import { admittedOwner, patchRuntime, runLoad, teamLoadKeys, teamsAuthFailure, writeLoadState } from "./store";

/**
 * Load actions. Each fetches, then publishes the whole response with one
 * atomic graph write (never one write per row). Workspace is always an argument.
 */

let admission: Promise<string> | null = null;

/** The authenticated owner for this session, loading capabilities first when needed. */
export function ensureOwner(): Promise<string> {
  const owner = admittedOwner();
  if (owner) return Promise.resolve(owner);
  const blocked = teamsAuthFailure();
  if (blocked) return Promise.reject(new TeamsApiError(blocked));
  admission ??= loadCapabilities()
    .then((capabilities) => capabilities.bindingOwnerId)
    .finally(() => {
      admission = null;
    });
  return admission;
}

async function withOwner<T>(read: (owner: string) => Promise<T>): Promise<{ owner: string; data: T }> {
  const owner = await ensureOwner();
  return { owner, data: await read(owner) };
}

function graphMemberIds(owner: string, workspace: string, teamId: string): string[] {
  const state = graphStore.getState();
  const ids = state.lists[lists.members(owner, workspace, teamId)]?.ids ?? [];
  return ids.flatMap((id) => {
    const memberId = state.entities[TEAM_MEMBER_ENTITY]?.[id]?.memberId;
    return typeof memberId === "string" ? [memberId] : [];
  });
}

export function loadCapabilities(options?: ReadOptions): Promise<CollaborationCapabilities> {
  return runLoad(teamLoadKeys.capabilities, () => fetchCapabilities(options), (batch, capabilities, now) =>
    patchRuntime(batch, {
      ownerId: capabilities.bindingOwnerId,
      instanceId: capabilities.instance.id,
      catalogBackend: capabilities.catalogStorage?.backend ?? null,
      capabilityNames: capabilities.capabilities,
      teamExecutionAvailable: capabilities.collaboration.activation.teamExecution,
      capabilitiesLoadedAt: now,
    }));
}

/** Read-only view of who holds team execution. The console never reclaims ownership. */
export function loadExecutionOwner(options?: ReadOptions): Promise<ExecutionOwnerView> {
  return runLoad(teamLoadKeys.executionOwner, () => fetchExecutionOwner(options), (batch, view, now) =>
    patchRuntime(batch, {
      ownsExecution: view.ownsExecution, claim: view.claim, currentFence: view.currentFence, executionOwnerLoadedAt: now,
    }));
}

export async function loadTeamDefinitions(options?: ReadOptions): Promise<void> {
  await runLoad(teamLoadKeys.definitions, () => withOwner(() => fetchTeamDefinitions(options)),
    (batch, { owner, data }) => writeDefinitions(batch, owner, data));
}

export async function loadPackages(options?: ReadOptions): Promise<void> {
  await runLoad(teamLoadKeys.packages, () => withOwner(() => fetchPackages(options)),
    (batch, { owner, data }) => writePackages(batch, owner, data));
}

/**
 * Export the package and keep the parsed coordinator role and model requirements.
 * A refused export is stored with its reasons (status "refused"), not thrown.
 */
export async function loadPackageExport(pkg: ImmutableDefinitionRef, options?: ReadOptions): Promise<PackageExportModel> {
  const packageKey = definitionKey(pkg);
  const result: { model?: PackageExportModel } = {};
  await runLoad(teamLoadKeys.packageExport(packageKey), () => withOwner(() => exportPackage(pkg, options)),
    (batch, { owner, data }, now) => {
      result.model = writePackageExport(batch, owner, packageKey, data, now);
    });
  if (!result.model) throw new Error("The package export was not stored.");
  return result.model;
}

export async function loadBindings(workspace: string, options?: ReadOptions): Promise<void> {
  await runLoad(teamLoadKeys.bindings(workspace), () => withOwner(() => fetchBindings(workspace, options)),
    (batch, { owner, data }) => writeBindings(batch, owner, workspace, data));
}

export async function loadTeams(workspace: string, options?: ReadOptions): Promise<void> {
  await runLoad(teamLoadKeys.teams(workspace), () => withOwner(() => fetchTeams(workspace, options)),
    (batch, { owner, data }) => writeTeams(batch, owner, workspace, data));
}

export function publishTeam(batch: GraphBatch, owner: string, workspace: string, team: TeamInstance): void {
  const listed = graphStore.getState().lists[lists.teams(owner, workspace)]?.ids.includes(teamGid(owner, workspace, team.id));
  if (listed) writeTeam(batch, owner, workspace, team);
  else prependTeam(batch, owner, workspace, team);
}

export interface TeamRunRefresh {
  live: boolean;
  /** Attempt ids with statuses and wait states; changes when the set of attempts or waits changes. */
  signature: string;
}

export function runSignature(summary: Pick<TeamExecutionSummary, "attempts" | "waits">): string {
  return [
    ...summary.attempts.map((attempt) => `a:${attempt.id}:${attempt.status}`),
    ...summary.waits.map((wait) => `w:${wait.waitId}:${wait.state}`),
  ].sort().join("|");
}

/**
 * One atomic refresh of a team run: the team (revisions, tasks), its execution
 * summary (attempts, waits, budget) and, when asked, artifacts and peer messages.
 */
export async function refreshTeamRun(
  workspace: string, teamId: string, include: { sideRecords?: boolean } = {}, options?: ReadOptions,
): Promise<TeamRunRefresh> {
  const refresh = await runLoad(
    teamLoadKeys.run(workspace, teamId),
    () => withOwner(async () => {
      const [team, execution, artifacts, messages] = await Promise.all([
        fetchTeam(workspace, teamId, options),
        fetchExecution(workspace, teamId, options),
        include.sideRecords ? fetchArtifacts(workspace, teamId, options) : Promise.resolve(null),
        include.sideRecords ? fetchPeerMessages(workspace, teamId, options) : Promise.resolve(null),
      ]);
      return { team, execution, artifacts, messages };
    }),
    (batch, { owner, data }, now) => {
      const memberIds = data.team.members.map((member) => member.id);
      publishTeam(batch, owner, workspace, data.team);
      writeExecution(batch, owner, workspace, teamId, data.execution, memberIds, now);
      if (data.artifacts) writeArtifacts(batch, owner, workspace, teamId, data.artifacts, memberIds);
      if (data.messages) writePeerMessages(batch, owner, workspace, teamId, data.messages);
    },
  );
  // Loads with the same key share one request. If this call joined a refresh that was
  // started without artifacts and messages, read them now instead of reporting none.
  if (include.sideRecords && refresh.data.artifacts === null) {
    await Promise.all([loadTeamArtifacts(workspace, teamId, options), loadTeamPeerMessages(workspace, teamId, options)]);
  }
  return {
    live: isTeamRunLive(refresh.data.execution.attempts, refresh.data.execution.waits),
    signature: runSignature(refresh.data.execution),
  };
}

export async function loadTeamArtifacts(workspace: string, teamId: string, options?: ReadOptions): Promise<void> {
  await runLoad(teamLoadKeys.artifacts(workspace, teamId), () => withOwner(() => fetchArtifacts(workspace, teamId, options)),
    (batch, { owner, data }) => writeArtifacts(batch, owner, workspace, teamId, data, graphMemberIds(owner, workspace, teamId)));
}

export async function loadTeamPeerMessages(workspace: string, teamId: string, options?: ReadOptions): Promise<void> {
  await runLoad(teamLoadKeys.peerMessages(workspace, teamId), () => withOwner(() => fetchPeerMessages(workspace, teamId, options)),
    (batch, { owner, data }) => writePeerMessages(batch, owner, workspace, teamId, data));
}

/**
 * Load execution summaries for several teams (list ordering needs to know which are live).
 * At most `concurrency` requests are in flight at a time; that bounds parallelism, not the
 * request rate the runtime limits. One team's failed read does not discard the others: the
 * summaries that arrived are published in one write, then the failure is recorded in the load
 * state and thrown. Calls share a request only when they ask for the same set of teams.
 */
export async function loadTeamExecutions(workspace: string, teamIds: readonly string[], concurrency = 2): Promise<void> {
  const ids = [...new Set(teamIds)].sort();
  const key = `executions:${workspace}:${ids.join("\n")}`;
  const { data: { failures } } = await runLoad(key, () => withOwner(async () => {
    const results: Array<{ teamId: string; summary: TeamExecutionSummary }> = [];
    const failed: unknown[] = [];
    for (let index = 0; index < ids.length; index += concurrency) {
      const chunk = ids.slice(index, index + concurrency);
      const settled = await Promise.allSettled(chunk.map(async (teamId) => ({ teamId, summary: await fetchExecution(workspace, teamId) })));
      for (const outcome of settled) {
        if (outcome.status === "fulfilled") results.push(outcome.value);
        else failed.push(outcome.reason);
      }
    }
    // A 401 holds for every team: fail the whole load so access is withdrawn like any other load.
    const unauthorized = failed.find((reason) => toErrorInfo(reason).kind === "unauthorized");
    if (unauthorized !== undefined) throw unauthorized;
    return { results, failures: failed };
  }), (batch, { owner, data }, now) => {
    for (const { teamId, summary } of data.results) {
      writeExecution(batch, owner, workspace, teamId, summary, graphMemberIds(owner, workspace, teamId), now);
    }
  });
  if (failures.length > 0) {
    const info = toErrorInfo(failures[0]);
    batchGraphWrites((batch) => writeLoadState(batch, key, "error", info, Date.now()));
    throw new TeamsApiError(info);
  }
}
