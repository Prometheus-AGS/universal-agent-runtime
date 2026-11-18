import { batchGraphWrites, type GraphBatch } from "@prometheus-ags/entity-graph-core";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { TeamsApiError, toErrorInfo, type TeamsErrorInfo } from "./api/teams-errors";
import {
  TEAMS_RUNTIME_ID,
  TEAMS_SESSION_ID,
  TEAM_LOAD_STATE_ENTITY,
  TEAM_RUNTIME_ENTITY,
  type LoadStatus,
  type TeamLoadState,
  type TeamRuntimeState,
} from "./entities";

/** Internal graph access shared by loads, commands and the watcher. Not exported from the package index. */

export const loadStateId = (key: string) => `load:${key}`;

export const teamLoadKeys = {
  capabilities: "capabilities",
  executionOwner: "execution-owner",
  definitions: "definitions",
  packages: "packages",
  packageExport: (packageKey: string) => `package-export:${packageKey}`,
  bindings: (workspace: string) => `bindings:${workspace}`,
  teams: (workspace: string) => `teams:${workspace}`,
  team: (workspace: string, teamId: string) => `team:${workspace}:${teamId}`,
  run: (workspace: string, teamId: string) => `run:${workspace}:${teamId}`,
  artifacts: (workspace: string, teamId: string) => `artifacts:${workspace}:${teamId}`,
  peerMessages: (workspace: string, teamId: string) => `peer-messages:${workspace}:${teamId}`,
} as const;

export function emptyRuntime(): TeamRuntimeState {
  return {
    id: TEAMS_RUNTIME_ID, session: TEAMS_SESSION_ID, ownerId: null, instanceId: null, catalogBackend: null,
    capabilityNames: null, teamExecutionAvailable: null, ownsExecution: null, claim: null, currentFence: null,
    capabilitiesLoadedAt: null, executionOwnerLoadedAt: null,
  };
}

export function readRuntime(): TeamRuntimeState | null {
  const record = graphStore.getState().readEntity<TeamRuntimeState>(TEAM_RUNTIME_ENTITY, TEAMS_RUNTIME_ID);
  return record?.session === TEAMS_SESSION_ID ? record : null;
}

/** The owner admitted in this session, or null before capabilities load. */
export function admittedOwner(): string | null {
  return readRuntime()?.ownerId ?? null;
}

export function patchRuntime(batch: GraphBatch, patch: Partial<TeamRuntimeState>): void {
  const current = batch.readEntity<TeamRuntimeState>(TEAM_RUNTIME_ENTITY, TEAMS_RUNTIME_ID);
  const base = current?.session === TEAMS_SESSION_ID ? current : emptyRuntime();
  batch.replaceEntity(TEAM_RUNTIME_ENTITY, TEAMS_RUNTIME_ID, { ...base, ...patch } as unknown as Record<string, unknown>);
}

export function writeLoadState(batch: GraphBatch, key: string, status: LoadStatus, error: TeamsErrorInfo | null, now: number): void {
  const record: TeamLoadState = {
    id: loadStateId(key), session: TEAMS_SESSION_ID, status, error, completedAt: status === "loading" ? null : now,
  };
  batch.replaceEntity(TEAM_LOAD_STATE_ENTITY, record.id, record as unknown as Record<string, unknown>);
}

const inFlight = new Map<string, Promise<unknown>>();
let authFailure: TeamsErrorInfo | null = null;

/** The remembered 401, or null. While set, only `loadCapabilities` may reach the runtime. */
export function teamsAuthFailure(): TeamsErrorInfo | null {
  return authFailure;
}

/**
 * Run one load: mark it loading, fetch, then publish the data and the ready
 * state in a single atomic write. Concurrent calls for one key share a request.
 * After a 401 every other load is refused locally until `loadCapabilities` succeeds.
 */
export function runLoad<T>(
  key: string,
  fetcher: () => Promise<T>,
  publish: (batch: GraphBatch, data: T, now: number) => void,
): Promise<T> {
  const existing = inFlight.get(key);
  if (existing) return existing as Promise<T>;
  const task = Promise.resolve().then(async () => {
    batchGraphWrites((batch) => writeLoadState(batch, key, "loading", null, Date.now()));
    try {
      if (authFailure && key !== teamLoadKeys.capabilities) throw new TeamsApiError(authFailure);
      const data = await fetcher();
      const now = Date.now();
      batchGraphWrites((batch) => {
        publish(batch, data, now);
        writeLoadState(batch, key, "ready", null, now);
      });
      if (key === teamLoadKeys.capabilities) authFailure = null;
      return data;
    } catch (error) {
      const info = toErrorInfo(error);
      if (info.kind === "unauthorized") authFailure = info;
      batchGraphWrites((batch) => writeLoadState(batch, key, "error", info, Date.now()));
      throw error;
    } finally {
      inFlight.delete(key);
    }
  });
  inFlight.set(key, task);
  return task;
}
