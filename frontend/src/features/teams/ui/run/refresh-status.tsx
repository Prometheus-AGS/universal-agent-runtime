import { useEffect, useState } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  teamLoadKeys,
  useTeamActions,
  useTeamIsLive,
  useTeamLoadError,
  useTeamRunLastUpdatedAt,
  useTeamRunWatchError,
  useTeamRunWatchInterval,
  useTeamRunWatchStatus,
  type TeamsErrorInfo,
} from "@/platform/entities";
import { formatAge } from "./run-model";
import type { RunContext } from "./run-hooks";

const TICK_MS = 1_000;

/** "Updated 4 s ago", re-rendered once a second. It is not a live region: a ticking clock would chatter. */
function UpdatedAgo({ at }: { at: number | null }) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), TICK_MS);
    return () => clearInterval(timer);
  }, []);
  if (at === null) return <span>Not updated yet</span>;
  return <span>Updated {formatAge(now - at)}</span>;
}

function failureText(error: TeamsErrorInfo, retryMs: number | null): string {
  const retry = retryMs === null ? "" : ` Retrying every ${Math.round(retryMs / 1000)} s.`;
  if (error.kind === "rate_limited") return `The runtime is rate limiting requests, so the view is behind.${retry}`;
  if (error.kind === "unauthorized") return "A verified owner is required, so refreshing has stopped. Reload after signing in.";
  return `Refreshing failed: ${error.message}${retry}`;
}

/**
 * When the view was last updated, whether it is still refreshing, a visible error state when a refresh fails or is
 * rate limited, and a manual refresh. A failing refresh never clears what is already shown.
 */
export function RefreshStatus({ workspace, teamId }: RunContext) {
  const actions = useTeamActions();
  const updatedAt = useTeamRunLastUpdatedAt(workspace, teamId);
  const watchError = useTeamRunWatchError(workspace, teamId);
  const loadError = useTeamLoadError(teamLoadKeys.run(workspace, teamId));
  const watchStatus = useTeamRunWatchStatus(workspace, teamId);
  const interval = useTeamRunWatchInterval(workspace, teamId);
  const live = useTeamIsLive(workspace, teamId);
  const [busy, setBusy] = useState(false);
  const error = watchError ?? loadError;

  const refresh = async () => {
    setBusy(true);
    try {
      await actions.refreshTeamRunNow(workspace, teamId);
    } catch {
      // Recorded in the run load state, which the error line above reads.
    } finally {
      setBusy(false);
    }
  };

  // A stopped watch wins over `live`: after a 401 the last data still shows running work, but nothing refreshes it.
  const mode =
    watchStatus === "stopped"
      ? live
        ? "Not refreshing. Work was running when refreshing stopped, so this view may be out of date."
        : "Not refreshing: no member is working."
      : live
        ? `Refreshing every ${Math.round((interval ?? 3_000) / 1000)} s while work is running.`
        : "Checking for work.";

  return (
    <div className="min-w-0 space-y-2">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-1">
        <p className="text-sm text-muted-foreground">
          <UpdatedAgo at={updatedAt} />. {mode}
        </p>
        <Button type="button" variant="secondary" className="min-h-11" aria-busy={busy || undefined} onClick={() => void refresh()}>
          <RefreshCw aria-hidden="true" className={busy ? "motion-safe:animate-spin" : undefined} />
          Refresh now
        </Button>
      </div>
      {error ? (
        <p role="status" className="flex items-start gap-2 text-sm text-destructive">
          <AlertTriangle aria-hidden="true" className="mt-0.5 size-4 shrink-0" />
          <span className="min-w-0 break-words">{failureText(error, watchError ? interval : null)}</span>
        </p>
      ) : null}
    </div>
  );
}
