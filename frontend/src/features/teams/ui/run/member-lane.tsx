import { memo, useId } from "react";
import { AlertTriangle, Circle, ShieldAlert } from "lucide-react";
import { Link } from "react-router";
import {
  useAttemptField,
  useMemberAttemptIds,
  useMemberField,
  useMemberOpenAttemptId,
  useMemberTaskIds,
} from "@/platform/entities";
import { cn } from "@/lib/utils";
import { formatMicrocost } from "../../model/format";
import { AttemptCard } from "./attempt-card";
import { Disclosure } from "./disclosure";
import { MemberArtifacts, MemberMessages } from "./member-records";
import { OwedTasks } from "./routing-progress";
import { isLiveCategory, laneLabel, unreportedChargeSummary, type LaneCategory } from "./run-model";
import { useTeamCurrency, useUnreportedCharge, type RunContext, type UnreportedCharge } from "./run-hooks";
import { StatusLabel } from "./status-label";
import { BlockedBy, TaskRow, TaskTitle } from "./task-parts";

interface LaneProps extends RunContext {
  memberId: string;
  category: LaneCategory;
}

function LaneStatus({ category }: { category: LaneCategory }) {
  if (category === "idle") {
    return (
      <span className="inline-flex items-center gap-1.5 text-sm font-medium text-muted-foreground">
        <Circle aria-hidden="true" className="size-4 shrink-0" />
        No work yet
      </span>
    );
  }
  return <StatusLabel view={category} />;
}

/**
 * The lane's one live region. It is rendered with the lane and keeps its place when the lane re-sorts, so a change
 * from "No work yet" to "Running" is a text change inside an existing region and is announced; a new attempt card
 * mounting is not what announces it. The subject names whose status it is.
 */
function AnnouncedLaneStatus({ category, subject }: { category: LaneCategory; subject: string }) {
  return (
    <span role="status">
      <span className="sr-only">{subject}: </span>
      <LaneStatus category={category} />
    </span>
  );
}

/** The member id's last six characters, so two members of one role can be told apart; the full id is the title. */
function MemberIdTag({ memberId }: { memberId: string }) {
  return (
    <span title={memberId} className="ml-2 font-mono text-xs font-normal text-muted-foreground">
      {memberId.slice(-6)}
    </span>
  );
}

/** One line in the lane header when the lane's attempt settled with no usage report: the whole reservation is charged. */
function UnreportedChargeLine({ workspace, teamId, category, charge }: RunContext & { category: LaneCategory; charge: UnreportedCharge }) {
  const currency = useTeamCurrency(workspace, teamId);
  const charged = charge.costMicrounits === null ? null : formatMicrocost(charge.costMicrounits, currency);
  return (
    <p className="flex items-start gap-1.5 text-sm break-words">
      <AlertTriangle aria-hidden="true" className="mt-0.5 size-4 shrink-0" />
      <span>{unreportedChargeSummary(laneLabel(category), charged)}</span>
    </p>
  );
}

interface LaneHeaderProps extends RunContext {
  memberId: string;
  category: LaneCategory;
  headingId: string;
  roleName: string;
  kind: string | undefined;
}

/**
 * Role, member id, kind and the lane's one live status. When the lane's attempt settled without a usage report the
 * header takes the warning treatment (bar, tint, icon and a one-line summary), because the whole reservation is
 * charged against the team budget and a neutral "Cancelled" hides that.
 */
function LaneHeader({ workspace, teamId, memberId, category, headingId, roleName, kind }: LaneHeaderProps) {
  const charge = useUnreportedCharge(workspace, teamId, memberId);
  return (
    <div className={cn("space-y-1", charge !== null && "rounded-md bg-warning/15 px-3 py-2")}>
      <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
        <h5
          id={headingId}
          tabIndex={-1}
          data-lane-heading=""
          className="min-w-0 text-base font-semibold break-words outline-none focus-visible:outline-2 focus-visible:outline-ring"
        >
          {roleName}
          <MemberIdTag memberId={memberId} />
          {kind ? <span className="ml-2 text-xs font-normal text-muted-foreground">{kind}</span> : null}
        </h5>
        <AnnouncedLaneStatus category={category} subject={roleName} />
      </div>
      {charge !== null ? <UnreportedChargeLine workspace={workspace} teamId={teamId} category={category} charge={charge} /> : null}
    </div>
  );
}

function CurrentTask({ workspace, teamId, memberId }: RunContext & { memberId: string }) {
  const openId = useMemberOpenAttemptId(workspace, teamId, memberId);
  const taskId = useAttemptField(workspace, teamId, openId ?? "", "taskId");
  // Said only when there is one: "none" on an idle or finished member is noise.
  if (openId === null || taskId === undefined) return null;
  return (
    <p className="min-w-0 text-xs break-words text-muted-foreground">
      Current task: <span className="text-foreground"><TaskTitle workspace={workspace} teamId={teamId} taskId={taskId} /></span>
    </p>
  );
}

function RecordStatus({ workspace, teamId, memberId }: RunContext & { memberId: string }) {
  const status = useMemberField(workspace, teamId, memberId, "status");
  // Team and member records are written `inactive` and never advance; only a withdrawn member is worth saying.
  if (status !== "revoked" && status !== "stopped") return null;
  return <p className="text-xs text-destructive">Member record: {status}. The runtime will not assign it new work.</p>;
}

/** Tasks assigned to the member whose dependency failed, always visible so a stalled branch is not hidden in a list. */
function BlockedTasks({ workspace, teamId, memberId }: RunContext & { memberId: string }) {
  const taskIds = useMemberTaskIds(workspace, teamId, memberId);
  if (taskIds.length === 0) return null;
  return (
    <div className="space-y-1">
      {taskIds.map((taskId) => (
        <BlockedBy key={taskId} workspace={workspace} teamId={teamId} taskId={taskId} />
      ))}
    </div>
  );
}

function MemberTasks({ workspace, teamId, memberId }: RunContext & { memberId: string }) {
  const taskIds = useMemberTaskIds(workspace, teamId, memberId);
  return (
    <Disclosure label="Assigned tasks" count={taskIds.length}>
      {taskIds.length === 0 ? (
        <p className="text-sm text-muted-foreground">No task is assigned to this member.</p>
      ) : (
        <ul className="space-y-2">
          {taskIds.map((taskId) => (
            <TaskRow key={taskId} workspace={workspace} teamId={teamId} taskId={taskId} />
          ))}
        </ul>
      )}
    </Disclosure>
  );
}

function ApprovalsLine() {
  return (
    <p className="flex items-start gap-2 text-sm">
      <ShieldAlert aria-hidden="true" className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
      <span>
        If this member delegates work or messages a peer, approval is decided under Approvals.{" "}
        <Link to="/admin/approvals" className="underline underline-offset-4 focus-visible:outline-2 focus-visible:outline-ring">
          Open approvals
        </Link>
      </span>
    </p>
  );
}

function LaneAttempts({ workspace, teamId, memberId }: RunContext & { memberId: string }) {
  const attemptIds = useMemberAttemptIds(workspace, teamId, memberId);
  const openId = useMemberOpenAttemptId(workspace, teamId, memberId);
  if (attemptIds.length === 0) return <p className="text-sm text-muted-foreground">No attempt has started for this member.</p>;
  const latestId = attemptIds[attemptIds.length - 1];
  const earlier = attemptIds.slice(0, -1).reverse();
  return (
    <div className="space-y-2">
      <ul className="space-y-2">
        <AttemptCard
          key={latestId}
          workspace={workspace}
          teamId={teamId}
          attemptId={latestId}
          memberId={memberId}
          open={latestId === openId}
          statusInLane={openId === null || latestId === openId}
        />
      </ul>
      {earlier.length > 0 && (
        <Disclosure label="Earlier attempts" count={earlier.length}>
          <ul className="space-y-2">
            {earlier.map((attemptId) => (
              <AttemptCard
                key={attemptId}
                workspace={workspace}
                teamId={teamId}
                attemptId={attemptId}
                memberId={memberId}
                open={attemptId === openId}
              />
            ))}
          </ul>
        </Disclosure>
      )}
    </div>
  );
}

/**
 * One member as a stacked row: role, kind, derived state, current task, the standing approvals line while an
 * attempt is live, owed routing steps, blocked tasks, the latest attempt open with older ones behind a count,
 * and collapsed tasks, peer messages and artifacts. The parent passes the lane category as a primitive, so `memo`
 * skips re-renders caused by the parent; the lane's own id-list subscriptions still re-render it on every poll,
 * because the graph allocates a new id list on every write.
 */
export const MemberLane = memo(function MemberLane({ workspace, teamId, memberId, category }: LaneProps) {
  const role = useMemberField(workspace, teamId, memberId, "role");
  const kind = useMemberField(workspace, teamId, memberId, "kind");
  const headingId = useId();
  const roleName = role ?? memberId;
  return (
    <li className="min-w-0" data-member-lane={`${teamId}:${memberId}`}>
      <article
        aria-labelledby={headingId}
        className="min-w-0 space-y-3 rounded-lg bg-muted/40 p-3 sm:p-4 data-[reveal=true]:ring-2 data-[reveal=true]:ring-ring motion-safe:transition-shadow"
      >
        <LaneHeader
          workspace={workspace}
          teamId={teamId}
          memberId={memberId}
          category={category}
          headingId={headingId}
          roleName={roleName}
          kind={kind}
        />
        <CurrentTask workspace={workspace} teamId={teamId} memberId={memberId} />
        <RecordStatus workspace={workspace} teamId={teamId} memberId={memberId} />
        {isLiveCategory(category) && <ApprovalsLine />}
        <OwedTasks workspace={workspace} teamId={teamId} memberId={memberId} />
        <BlockedTasks workspace={workspace} teamId={teamId} memberId={memberId} />
        <LaneAttempts workspace={workspace} teamId={teamId} memberId={memberId} />
        <MemberTasks workspace={workspace} teamId={teamId} memberId={memberId} />
        <MemberMessages workspace={workspace} teamId={teamId} memberId={memberId} />
        <MemberArtifacts workspace={workspace} teamId={teamId} memberId={memberId} />
      </article>
    </li>
  );
});
