import { useMemo } from "react";
import { useMemberIds, useUnassignedTaskIds } from "@/platform/entities";
import { Collect } from "./collect";
import { MemberLane } from "./member-lane";
import { OwedTasks } from "./routing-progress";
import {
  addLane,
  addWaitFact,
  addWaitIds,
  NO_IDS,
  NO_LANES,
  NO_WAITS,
  useMemberCategory,
  useMemberWaits,
  useWaitFact,
  type RunContext,
  type WaitFact,
} from "./run-hooks";
import { countLanes, laneLabel, LANE_SUMMARY_THRESHOLD, sortLanes, type LaneEntry } from "./run-model";
import { TaskRow } from "./task-parts";

function LaneSummary({ entries }: { entries: readonly LaneEntry[] }) {
  const counts = useMemo(() => countLanes(entries), [entries]);
  return (
    <p className="flex flex-wrap gap-x-4 gap-y-1 text-sm" aria-label="Members by state">
      {counts.map(({ category, count }) => (
        <span key={category}>
          <span className="font-medium">{count}</span> <span className="text-muted-foreground">{laneLabel(category).toLowerCase()}</span>
        </span>
      ))}
    </p>
  );
}

function LaneList({ workspace, teamId, entries }: RunContext & { entries: readonly LaneEntry[] }) {
  const ordered = useMemo(() => sortLanes(entries), [entries]);
  return (
    <div className="space-y-3">
      {entries.length > LANE_SUMMARY_THRESHOLD && <LaneSummary entries={entries} />}
      <ul className="space-y-3">
        {ordered.map((entry) => (
          <MemberLane key={entry.id} workspace={workspace} teamId={teamId} memberId={entry.id} category={entry.category} />
        ))}
      </ul>
    </div>
  );
}

/**
 * Tasks no member holds yet: their dependency state, and (for a routing test that has not been claimed) the step
 * still owed. The whole list is rendered, because truncating it could hide a branch blocked by a failed task.
 */
function UnassignedTasks({ workspace, teamId }: RunContext) {
  const taskIds = useUnassignedTaskIds(workspace, teamId);
  if (taskIds.length === 0) return <OwedTasks workspace={workspace} teamId={teamId} memberId={null} />;
  return (
    <section aria-labelledby="run-unassigned-heading" className="space-y-2">
      <h4 id="run-unassigned-heading" className="text-sm font-semibold">Tasks without a member ({taskIds.length})</h4>
      <OwedTasks workspace={workspace} teamId={teamId} memberId={null} />
      <ul className="space-y-2">
        {taskIds.map((taskId) => (
          <TaskRow key={taskId} workspace={workspace} teamId={teamId} taskId={taskId} />
        ))}
      </ul>
    </section>
  );
}

/** The lane categories, read with every wait of the team so a yielded attempt is told by its wait. */
function CategorisedLanes({ workspace, teamId, memberIds, waits }: RunContext & { memberIds: readonly string[]; waits: readonly WaitFact[] }) {
  const context = useMemo(() => ({ workspace, teamId, waits }), [workspace, teamId, waits]);
  return (
    <Collect ids={memberIds} context={context} useValue={useMemberCategory} reduce={addLane} initial={NO_LANES}>
      {(entries) => <LaneList workspace={workspace} teamId={teamId} entries={entries} />}
    </Collect>
  );
}

/** Every member as a stacked row, attention first, with state counts above when there are more than six. */
export function MemberLanes({ workspace, teamId }: RunContext) {
  const memberIds = useMemberIds(workspace, teamId);
  const context = useMemo(() => ({ workspace, teamId }), [workspace, teamId]);
  return (
    <section aria-labelledby="run-members-heading" className="space-y-3">
      <h4 id="run-members-heading" className="text-sm font-semibold">Members ({memberIds.length})</h4>
      {memberIds.length === 0 ? (
        <p role="status" className="text-sm text-muted-foreground">
          This team has no members yet. They appear once the team record has loaded.
        </p>
      ) : (
        <Collect ids={memberIds} context={context} useValue={useMemberWaits} reduce={addWaitIds} initial={NO_IDS}>
          {(waitIds) => (
            <Collect ids={waitIds} context={context} useValue={useWaitFact} reduce={addWaitFact} initial={NO_WAITS}>
              {(waits) => <CategorisedLanes workspace={workspace} teamId={teamId} memberIds={memberIds} waits={waits} />}
            </Collect>
          )}
        </Collect>
      )}
      <UnassignedTasks workspace={workspace} teamId={teamId} />
    </section>
  );
}
