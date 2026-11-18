import { batchGraphWrites } from "@prometheus-ags/entity-graph-core";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { toErrorInfo } from "./api/teams-errors";
import { TEAMS_SESSION_ID, TEAM_RUN_WATCH_ENTITY, type TeamRunWatchState } from "./entities";
import { teamGid } from "./keys";
import { ensureOwner, loadTeamArtifacts, loadTeamPeerMessages, refreshTeamRun } from "./loads";

/** Poll cadence while the run is live, and after an error or a 429 (the runtime has one global 10 req/s bucket). */
export const WATCH_LIVE_INTERVAL_MS = 3_000;
export const WATCH_BACKOFF_INTERVAL_MS = 15_000;

interface Watcher {
  holders: number;
  stop: () => void;
  kick: () => void;
}
const watchers = new Map<string, Watcher>();
const watcherKey = (workspace: string, teamId: string) => `${workspace.length}:${workspace}|${teamId}`;

function writeWatch(owner: string, workspace: string, teamId: string, patch: Partial<TeamRunWatchState>): void {
  const id = teamGid(owner, workspace, teamId);
  const current = graphStore.getState().readEntity<TeamRunWatchState>(TEAM_RUN_WATCH_ENTITY, id);
  const base: TeamRunWatchState = current?.session === TEAMS_SESSION_ID
    ? current
    : { id, session: TEAMS_SESSION_ID, status: "stopped", live: false, intervalMs: WATCH_LIVE_INTERVAL_MS, lastUpdatedAt: null, lastError: null, consecutiveFailures: 0 };
  batchGraphWrites((batch) => batch.replaceEntity(TEAM_RUN_WATCH_ENTITY, id, { ...base, ...patch } as unknown as Record<string, unknown>));
}

/**
 * Watch one team run. Reads the team and `/execution` every 3 s while the run is
 * live (`isTeamRunLive`), reads artifacts and peer messages only when the set of
 * attempts or waits changes, backs off to 15 s after an error or a 429, and stops
 * when the run is no longer live, on a 401, or when the returned function is called.
 * Calls for the same team share one loop. Last-updated time and the last error are
 * published to the graph for `useTeamRunWatch*`.
 */
export function watchTeamRun(workspace: string, teamId: string): () => void {
  const key = watcherKey(workspace, teamId);
  const existing = watchers.get(key);
  if (existing) {
    existing.holders += 1;
    return release(key);
  }
  const abort = new AbortController();
  let timer: ReturnType<typeof setTimeout> | null = null;
  let stopped = false;
  let running = false;
  let signature: string | null = null;
  let failures = 0;
  let admitted: string | null = null;

  const schedule = (delay: number, owner: string) => {
    if (stopped) return;
    timer = setTimeout(() => void tick(owner), delay);
  };

  const tick = async (owner: string): Promise<void> => {
    if (stopped || running) return;
    running = true;
    timer = null;
    try {
      const first = signature === null;
      const refreshed = await refreshTeamRun(workspace, teamId, { sideRecords: first }, { signal: abort.signal });
      if (!first && refreshed.signature !== signature) {
        await Promise.all([
          loadTeamArtifacts(workspace, teamId, { signal: abort.signal }),
          loadTeamPeerMessages(workspace, teamId, { signal: abort.signal }),
        ]);
      }
      // A tick that joined a shared load it cannot abort may finish after the last holder left.
      if (stopped) return;
      signature = refreshed.signature;
      failures = 0;
      writeWatch(owner, workspace, teamId, {
        status: refreshed.live ? "watching" : "stopped", live: refreshed.live, intervalMs: WATCH_LIVE_INTERVAL_MS,
        lastUpdatedAt: Date.now(), lastError: null, consecutiveFailures: 0,
      });
      if (refreshed.live) schedule(WATCH_LIVE_INTERVAL_MS, owner);
    } catch (error) {
      if (stopped || abort.signal.aborted) return;
      const info = toErrorInfo(error);
      failures += 1;
      const fatal = info.kind === "unauthorized";
      writeWatch(owner, workspace, teamId, {
        status: fatal ? "stopped" : "watching", live: false, intervalMs: WATCH_BACKOFF_INTERVAL_MS, lastError: info,
        consecutiveFailures: failures,
      });
      if (!fatal) schedule(WATCH_BACKOFF_INTERVAL_MS, owner);
    } finally {
      running = false;
    }
  };

  /** Admit the owner, then tick. A rejection is written to the watch record and retried, never left as a dead loop. */
  const begin = (announce: boolean): void => {
    void ensureOwner().then((owner) => {
      if (stopped) return undefined;
      admitted = owner;
      if (announce) writeWatch(owner, workspace, teamId, { status: "watching", live: true, lastError: null, consecutiveFailures: 0 });
      return tick(owner);
    }).catch((error: unknown) => {
      if (stopped) return;
      const info = toErrorInfo(error);
      const fatal = info.kind === "unauthorized";
      failures += 1;
      if (admitted) {
        writeWatch(admitted, workspace, teamId, {
          status: fatal ? "stopped" : "watching", live: false, intervalMs: WATCH_BACKOFF_INTERVAL_MS, lastError: info,
          consecutiveFailures: failures,
        });
      }
      // Without an owner there is no record to write; the "capabilities" load state records why.
      if (!fatal) timer = setTimeout(() => begin(false), WATCH_BACKOFF_INTERVAL_MS);
    });
  };

  const watcher: Watcher = {
    holders: 1,
    stop: () => {
      stopped = true;
      abort.abort();
      if (timer) clearTimeout(timer);
      timer = null;
    },
    kick: () => {
      if (stopped || running) return;
      if (timer) clearTimeout(timer);
      timer = null;
      begin(false);
    },
  };
  watchers.set(key, watcher);
  begin(true);
  return release(key);
}

function release(key: string): () => void {
  let released = false;
  return () => {
    if (released) return;
    released = true;
    const watcher = watchers.get(key);
    if (!watcher) return;
    watcher.holders -= 1;
    if (watcher.holders > 0) return;
    watcher.stop();
    watchers.delete(key);
  };
}

/**
 * Manual refresh: wakes the loop for an actively watched run (resuming it if it
 * had stopped); otherwise performs one refresh of the run, artifacts and messages.
 */
export async function refreshTeamRunNow(workspace: string, teamId: string): Promise<void> {
  const watcher = watchers.get(watcherKey(workspace, teamId));
  if (watcher) {
    watcher.kick();
    return;
  }
  await refreshTeamRun(workspace, teamId, { sideRecords: true });
}
