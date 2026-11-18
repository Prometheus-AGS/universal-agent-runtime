import { useCallback, useEffect } from "react";
import { teamLoadKeys, useTeamActions, useTeamLoadError } from "@/platform/entities";

/** Execution summaries are fetched per team (one request each), so the list prefetches only its newest teams. */
export const LIST_ACTIVITY_PREFETCH_LIMIT = 20;

function quiet(task: Promise<unknown>): Promise<void> {
  return task.then(
    () => undefined,
    () => undefined, // The failure is recorded in the load state and shown where the data would be.
  );
}

/**
 * Loads what the page needs: runtime facts and the catalog once, then the workspace's bindings
 * and teams whenever the workspace changes. After a 401 the domain refuses further loads locally.
 */
export function useTeamsBootstrap(workspace: string): { refresh: () => void } {
  const actions = useTeamActions();
  const unauthorized = useTeamLoadError(teamLoadKeys.capabilities)?.kind === "unauthorized";

  useEffect(() => {
    void quiet(
      actions.loadCapabilities().then(() =>
        Promise.allSettled([actions.loadExecutionOwner(), actions.loadTeamDefinitions(), actions.loadPackages()]),
      ),
    );
  }, [actions]);

  useEffect(() => {
    void quiet(
      actions.ensureOwner().then(() => Promise.allSettled([actions.loadBindings(workspace), actions.loadTeams(workspace)])),
    );
  }, [actions, workspace]);

  const refresh = useCallback(() => {
    if (unauthorized) return; // A 401 stops team requests until the page is reloaded.
    void quiet(
      actions.loadCapabilities().then(() =>
        Promise.allSettled([
          actions.loadExecutionOwner(),
          actions.loadTeamDefinitions(),
          actions.loadPackages(),
          actions.loadBindings(workspace),
          actions.loadTeams(workspace),
        ]),
      ),
    );
  }, [actions, workspace, unauthorized]);

  return { refresh };
}

/** Fetch run summaries for the newest teams once per list so the list can show who is working. */
export function useTeamActivityPrefetch(workspace: string, teamIds: readonly string[]): void {
  const actions = useTeamActions();
  const key = teamIds.slice(0, LIST_ACTIVITY_PREFETCH_LIMIT).join("\n");
  useEffect(() => {
    if (!key) return;
    void quiet(actions.loadTeamExecutions(workspace, key.split("\n")));
  }, [actions, workspace, key]);
}
