import {
  useTeamAttemptCount,
  useTeamIsLive,
  useTeamRunSummaryField,
} from "@/platform/entities";
import { cn } from "@/lib/utils";
import { StatusLabel } from "./run/status-label";

/**
 * What the team is doing, with the same icon-and-text pattern the member lanes use. A running team uses the
 * lanes' "Running" label; the other states are quiet text. Activity comes from the members' attempts, never
 * from the record status, which the runtime writes once at creation.
 * `row` is the short form for list rows, `detail` the longer form beside the title.
 */
export function TeamActivity({ workspace, teamId, variant, className }: {
  workspace: string;
  teamId: string;
  variant: "row" | "detail";
  className?: string;
}) {
  const live = useTeamIsLive(workspace, teamId);
  const loadedAt = useTeamRunSummaryField(workspace, teamId, "loadedAt");
  const attempts = useTeamAttemptCount(workspace, teamId);
  const size = variant === "row" ? "text-xs" : "text-sm";
  if (live) return <StatusLabel view="running" className={cn(size, className)} />;
  const text =
    loadedAt === undefined
      ? variant === "row" ? "Not loaded" : "Activity not loaded yet"
      : attempts === 0
        ? "No work yet"
        : variant === "row" ? "Idle" : `Idle, ${attempts} ${attempts === 1 ? "attempt" : "attempts"} so far`;
  return <span className={cn("shrink-0", size, "text-muted-foreground", className)}>{text}</span>;
}
