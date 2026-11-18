import { ArrowRightLeft, OctagonX } from "lucide-react";
import {
  useMemberField,
  useTaskBlockedBy,
  useTaskDelegatorMemberId,
  useTaskField,
} from "@/platform/entities";
import { humanizeToken } from "./run-model";
import type { RunContext } from "./run-hooks";

interface TaskRef extends RunContext {
  taskId: string;
}

export function TaskTitle({ workspace, teamId, taskId }: TaskRef) {
  const title = useTaskField(workspace, teamId, taskId, "title");
  return <>{title && title !== "" ? title : taskId}</>;
}

export function MemberRoleName({ workspace, teamId, memberId }: RunContext & { memberId: string }) {
  const role = useMemberField(workspace, teamId, memberId, "role");
  return <>{role ?? memberId}</>;
}

function TaskStatus({ workspace, teamId, taskId }: TaskRef) {
  const status = useTaskField(workspace, teamId, taskId, "status");
  return <span className="text-xs text-muted-foreground">{status ? humanizeToken(status) : "Status unknown"}</span>;
}

/** "Blocked by <task>" for a task whose dependency failed or was cancelled; renders nothing otherwise. */
export function BlockedBy({ workspace, teamId, taskId }: TaskRef) {
  const blockedBy = useTaskBlockedBy(workspace, teamId, taskId);
  if (blockedBy === null) return null;
  return (
    <p className="flex items-start gap-1.5 text-sm text-destructive">
      <OctagonX aria-hidden="true" className="mt-0.5 size-4 shrink-0" />
      <span className="min-w-0 break-words">
        Blocked by <TaskTitle workspace={workspace} teamId={teamId} taskId={blockedBy} />, which did not finish.
      </span>
    </p>
  );
}

function DelegatedBy({ workspace, teamId, taskId }: TaskRef) {
  const delegator = useTaskDelegatorMemberId(workspace, teamId, taskId);
  if (delegator === null) return null;
  return (
    <p className="flex items-start gap-1.5 text-xs text-muted-foreground">
      <ArrowRightLeft aria-hidden="true" className="mt-0.5 size-3 shrink-0" />
      <span>
        Delegated by <MemberRoleName workspace={workspace} teamId={teamId} memberId={delegator} />
      </span>
    </p>
  );
}

function DependsOn({ workspace, teamId, taskId }: TaskRef) {
  const dependsOn = useTaskField(workspace, teamId, taskId, "dependsOn");
  if (!dependsOn || dependsOn.length === 0) return null;
  return (
    <p className="text-xs break-words text-muted-foreground">
      Depends on:{" "}
      {dependsOn.map((dependency, index) => (
        <span key={dependency}>
          {index > 0 ? ", " : ""}
          <TaskTitle workspace={workspace} teamId={teamId} taskId={dependency} />
        </span>
      ))}
    </p>
  );
}

/** One task: title, status, who delegated it, what it depends on, and whether a failed dependency blocks it. */
export function TaskRow({ workspace, teamId, taskId }: TaskRef) {
  return (
    <li className="min-w-0 space-y-0.5 rounded-md bg-background/70 px-3 py-2 text-sm">
      <p className="flex flex-wrap items-baseline gap-x-2">
        <span className="min-w-0 font-medium break-words">
          <TaskTitle workspace={workspace} teamId={teamId} taskId={taskId} />
        </span>
        <TaskStatus workspace={workspace} teamId={teamId} taskId={taskId} />
      </p>
      <DelegatedBy workspace={workspace} teamId={teamId} taskId={taskId} />
      <DependsOn workspace={workspace} teamId={teamId} taskId={taskId} />
      <BlockedBy workspace={workspace} teamId={teamId} taskId={taskId} />
    </li>
  );
}
