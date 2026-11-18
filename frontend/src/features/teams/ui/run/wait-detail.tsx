import { useMemberWaitIds, useTaskField, useWaitField } from "@/platform/entities";
import { CancelAttempt } from "./cancel-attempt";
import { canCancelAttempt, humanizeToken, WAIT_STATE_LABEL } from "./run-model";
import type { RunContext } from "./run-hooks";
import { TaskTitle } from "./task-parts";

interface WaitScope extends RunContext {
  memberId: string;
  attemptId: string;
  /** The runtime still counts this yielded attempt as the member's current one. */
  open: boolean;
}

function TargetTask({ workspace, teamId, taskId }: RunContext & { taskId: string }) {
  const status = useTaskField(workspace, teamId, taskId, "status");
  return (
    <li className="break-words">
      <TaskTitle workspace={workspace} teamId={teamId} taskId={taskId} />
      <span className="text-xs text-muted-foreground"> ({status ? humanizeToken(status) : "status unknown"})</span>
    </li>
  );
}

function WaitMatch({ workspace, teamId, waitId, attemptId, open }: RunContext & { waitId: string; attemptId: string; open: boolean }) {
  const owner = useWaitField(workspace, teamId, waitId, "attemptId");
  const state = useWaitField(workspace, teamId, waitId, "state");
  const targets = useWaitField(workspace, teamId, waitId, "targetTaskIds");
  const continuation = useWaitField(workspace, teamId, waitId, "continuationAttemptId");
  const reasonCode = useWaitField(workspace, teamId, waitId, "reasonCode");
  if (owner !== attemptId || state === undefined) return null;
  const waitLabel = WAIT_STATE_LABEL[state] ?? humanizeToken(state);
  const cancellable = open && canCancelAttempt("yielded", state);
  return (
    <div className="min-w-0 space-y-2">
      <p className="text-sm">
        <span className="text-xs text-muted-foreground">Wait: </span>
        {waitLabel}
        {reasonCode ? <span className="font-mono text-xs text-muted-foreground"> ({reasonCode})</span> : null}
      </p>
      {targets && targets.length > 0 && (
        <div>
          <p className="text-xs text-muted-foreground">{open ? "Waiting for" : "Waited for"}</p>
          <ul className="list-disc space-y-0.5 pl-5 text-sm">
            {targets.map((taskId) => (
              <TargetTask key={taskId} workspace={workspace} teamId={teamId} taskId={taskId} />
            ))}
          </ul>
        </div>
      )}
      {continuation ? (
        <p className="text-sm break-all">
          <span className="text-xs text-muted-foreground">Resumed as attempt </span>
          <span className="font-mono text-xs">{continuation}</span>
        </p>
      ) : null}
      {cancellable ? <CancelAttempt workspace={workspace} teamId={teamId} attemptId={attemptId} /> : null}
      {open && state === "yield_requested" ? (
        <p className="text-xs text-muted-foreground">Cancel becomes available once the runtime registers the wait.</p>
      ) : null}
    </div>
  );
}

/**
 * The wait that belongs to one yielded attempt. A wait is looked up through its member's waits; the one whose
 * attempt matches renders and the rest render nothing. A yielded attempt whose wait has not loaded shows no
 * wait line.
 */
export function WaitDetail({ workspace, teamId, memberId, attemptId, open }: WaitScope) {
  const waitIds = useMemberWaitIds(workspace, teamId, memberId);
  return (
    <>
      {waitIds.map((waitId) => (
        <WaitMatch key={waitId} workspace={workspace} teamId={teamId} waitId={waitId} attemptId={attemptId} open={open} />
      ))}
    </>
  );
}
