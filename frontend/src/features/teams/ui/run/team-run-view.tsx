import { useEffect, useId } from "react";
import { useTeamActions } from "@/platform/entities";
import { MemberLanes } from "./member-lanes";
import { RefreshStatus } from "./refresh-status";
import { RoutingTestConsole } from "./routing-test";

/**
 * The run view of one team: refresh status, one lane per member, then the routing-test console (lanes first, so
 * they are on screen without scrolling past a form). It starts the
 * domain's run watch on mount and stops it on unmount. The watch is shared per team (reference counted), reads
 * every 3 s while work is live, backs off to 15 s after an error or a 429, and stops when nothing is running;
 * ingest never moves focus or scroll. This shell subscribes to no records; each child subscribes at its own
 * boundary. A poll still re-renders every child that selects a record or an id list, because the graph allocates a
 * new record and id list on every write; only primitive field selectors skip a re-render when the value is unchanged.
 */
export function TeamRunView({ workspace, teamId }: { workspace: string; teamId: string }) {
  const actions = useTeamActions();
  const headingId = useId();

  useEffect(() => actions.watchTeamRun(workspace, teamId), [actions, workspace, teamId]);

  return (
    <section aria-labelledby={headingId} className="min-w-0 space-y-4">
      <div className="space-y-2">
        <h3 id={headingId} className="text-sm font-semibold">Run</h3>
        <RefreshStatus workspace={workspace} teamId={teamId} />
      </div>
      <MemberLanes workspace={workspace} teamId={teamId} />
      <RoutingTestConsole workspace={workspace} teamId={teamId} />
    </section>
  );
}
