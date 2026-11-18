import { memo } from "react";
import { ScrollText } from "lucide-react";
import { Link } from "react-router";
import { useAttemptField, useMemberWaitIds, useTaskField } from "@/platform/entities";
import { Collect } from "./collect";
import { CancelAttempt } from "./cancel-attempt";
import { DiagnosticBlock, ModelLine, TimeLine, UsageLine } from "./attempt-parts";
import { OutputView } from "./output-view";
import { attemptView, canCancelAttempt, pickWaitState } from "./run-model";
import { addWaitFact, NO_WAITS, useWaitFact, type RunContext } from "./run-hooks";
import { StatusLabel } from "./status-label";
import { TaskTitle } from "./task-parts";
import { WaitDetail } from "./wait-detail";

interface AttemptCardProps extends RunContext {
  attemptId: string;
  memberId: string;
  /** The runtime still counts this attempt as the member's current one (for a yielded attempt: its wait can resume). */
  open: boolean;
  /**
   * The lane header already states this attempt's status (it is the attempt the lane is read through), so the card
   * leaves it out and leads with the attempt's own facts. A status the console does not recognise is still shown.
   */
  statusInLane?: boolean;
}

function AttemptTask({ workspace, teamId, attemptId }: RunContext & { attemptId: string }) {
  const taskId = useAttemptField(workspace, teamId, attemptId, "taskId");
  if (taskId === undefined) return null;
  return (
    <p className="min-w-0 text-xs break-words text-muted-foreground">
      Task: <TaskTitle workspace={workspace} teamId={teamId} taskId={taskId} />
    </p>
  );
}

/**
 * A yielded attempt stays `yielded` for good, so what it shows comes from its wait and its task: waiting,
 * handed off, cancelled while waiting, wait invalidated, or (with no wait record) yielded with nothing more said.
 */
function YieldedStatus({ workspace, teamId, memberId, attemptId, open }: RunContext & { memberId: string; attemptId: string; open: boolean }) {
  const waitIds = useMemberWaitIds(workspace, teamId, memberId);
  const taskId = useAttemptField(workspace, teamId, attemptId, "taskId");
  const taskStatus = useTaskField(workspace, teamId, taskId ?? "", "status");
  const context = { workspace, teamId };
  return (
    <Collect ids={waitIds} context={context} useValue={useWaitFact} reduce={addWaitFact} initial={NO_WAITS}>
      {(waits) => (
        <StatusLabel view={attemptView("yielded", open, { state: pickWaitState(waits, attemptId), taskStatus: taskStatus ?? null })} />
      )}
    </Collect>
  );
}

function AttemptOutput({ workspace, teamId, attemptId, succeeded }: RunContext & { attemptId: string; succeeded: boolean }) {
  const output = useAttemptField(workspace, teamId, attemptId, "output");
  if (output !== null && output !== undefined && output !== "") return <OutputView label="Output" value={output} />;
  return succeeded ? <p className="text-sm text-muted-foreground">The attempt succeeded and returned no output.</p> : null;
}

function UncertainNote({ workspace, teamId, attemptId }: RunContext & { attemptId: string }) {
  const disposition = useAttemptField(workspace, teamId, attemptId, "effectDisposition");
  return (
    <p className="text-sm break-words">
      The runtime could not confirm what this attempt did{disposition ? ` (effect: ${disposition})` : ""}. Who is liable for
      its effects is unknown, and an uncertain attempt cannot be cancelled.
    </p>
  );
}

/**
 * One execution attempt. It subscribes to the status; each part below selects its own fields. A primitive field
 * selector (status, run id) does not re-render when its value is unchanged, but the graph allocates a new record and
 * id list on every write, so a part that selects a record, an array or an id list re-renders on every poll. `memo`
 * only stops re-renders caused by the parent. The status is not announced from this card; the member's lane carries
 * the one live region.
 */
export const AttemptCard = memo(function AttemptCard({ workspace, teamId, attemptId, memberId, open, statusInLane = false }: AttemptCardProps) {
  const status = useAttemptField(workspace, teamId, attemptId, "status");
  const runId = useAttemptField(workspace, teamId, attemptId, "runId");
  if (status === undefined) return null;
  const view = status === "yielded" ? null : attemptView(status, open);
  return (
    <li
      data-attempt-card=""
      tabIndex={-1}
      className="min-w-0 space-y-2 rounded-md bg-background/60 p-3 outline-none focus-visible:outline-2 focus-visible:outline-ring"
    >
      <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
        {statusInLane && view !== "unknown" ? null : view === null ? (
          <YieldedStatus workspace={workspace} teamId={teamId} memberId={memberId} attemptId={attemptId} open={open} />
        ) : (
          <StatusLabel view={view} raw={status} />
        )}
        <AttemptTask workspace={workspace} teamId={teamId} attemptId={attemptId} />
      </div>
      <ModelLine workspace={workspace} teamId={teamId} attemptId={attemptId} />
      <UsageLine workspace={workspace} teamId={teamId} attemptId={attemptId} />
      <TimeLine workspace={workspace} teamId={teamId} attemptId={attemptId} />
      {view === "uncertain" ? <UncertainNote workspace={workspace} teamId={teamId} attemptId={attemptId} /> : null}
      {status === "yielded" ? (
        <WaitDetail workspace={workspace} teamId={teamId} memberId={memberId} attemptId={attemptId} open={open} />
      ) : null}
      <AttemptOutput workspace={workspace} teamId={teamId} attemptId={attemptId} succeeded={view === "succeeded"} />
      <DiagnosticBlock workspace={workspace} teamId={teamId} attemptId={attemptId} />
      <div className="flex flex-wrap items-center gap-x-4 gap-y-1">
        {runId ? (
          <Link
            to={`/admin/runs?run=${encodeURIComponent(runId)}`}
            className="inline-flex min-h-11 items-center gap-1.5 text-sm underline underline-offset-4 focus-visible:outline-2 focus-visible:outline-ring"
          >
            <ScrollText aria-hidden="true" className="size-4" />
            Open run trace
          </Link>
        ) : null}
        {canCancelAttempt(status, null) ? <CancelAttempt workspace={workspace} teamId={teamId} attemptId={attemptId} /> : null}
      </div>
    </li>
  );
});
