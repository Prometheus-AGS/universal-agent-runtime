import { useEffect, useRef } from "react";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import {
  definitionKey,
  teamLoadKeys,
  useTeamActions,
  useTeamBudgetLimit,
  useTeamBudgetRemaining,
  useTeamDefinitionField,
  useTeamField,
  useTeamLoadError,
  useTeamLoadStatus,
  useTeamRunSummaryField,
  useTeamTaskSlotsLeft,
} from "@/platform/entities";
import { elideMiddle, formatCount, formatDateTime, formatMicrocost, formatSeconds } from "../model/format";
import { TeamActivity } from "./team-activity";
// Mount point for the run view (writer B owns ./run/**).
import { TeamRunView } from "./run/team-run-view";

type Dimension = "tokens" | "costMicrounits" | "elapsedSeconds";

function useTeamHeaderLoader(workspace: string, teamId: string): void {
  const actions = useTeamActions();
  useEffect(() => {
    // One atomic read of the team and its execution summary; the run view's watcher takes over refreshing.
    void actions.refreshTeamRun(workspace, teamId).catch(() => undefined); // Recorded in the load state.
  }, [actions, workspace, teamId]);
}

interface TitleProps {
  workspace: string;
  teamId: string;
  focusOnMount: boolean;
  onFocused: () => void;
}

function Title({ workspace, teamId, focusOnMount, onFocused }: TitleProps) {
  const definition = useTeamField(workspace, teamId, "definition");
  const title = useTeamDefinitionField(definition ? definitionKey(definition) : "", "title");
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    if (!focusOnMount) return undefined;
    // Two frames, so the closing sheet's own focus handling has finished before focus lands here.
    let inner = 0;
    const outer = requestAnimationFrame(() => {
      inner = requestAnimationFrame(() => {
        heading.current?.focus();
        onFocused();
      });
    });
    return () => {
      cancelAnimationFrame(outer);
      cancelAnimationFrame(inner);
    };
  }, [focusOnMount, onFocused]);
  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1">
      <h2 ref={heading} id="teams-detail-heading" tabIndex={-1} className="min-w-0 break-words text-lg font-semibold focus-visible:outline-2 focus-visible:outline-ring">
        {title ?? definition?.id ?? teamId}
      </h2>
      <TeamActivity workspace={workspace} teamId={teamId} variant="detail" />
    </div>
  );
}

interface Fact {
  term: string;
  value: string;
  /** The untruncated value, shown on hover when `value` is shortened. */
  title?: string;
  mono?: boolean;
}

function Facts({ workspace, teamId }: { workspace: string; teamId: string }) {
  const revision = useTeamField(workspace, teamId, "revision");
  const binding = useTeamField(workspace, teamId, "binding");
  const createdAt = useTeamField(workspace, teamId, "createdAt");
  const items: readonly Fact[] = [
    { term: "Team id", value: teamId, mono: true },
    { term: "Revision", value: String(revision ?? "") },
    {
      term: "Binding",
      value: binding ? `${elideMiddle(binding.id)} (revision ${binding.revision})` : "",
      title: binding ? `${binding.id} (revision ${binding.revision})` : undefined,
      mono: true,
    },
    { term: "Created", value: formatDateTime(createdAt) },
  ];
  return (
    <dl className="grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2 xl:grid-cols-3">
      {items.map((item) => (
        <div key={item.term} className="min-w-0">
          <dt className="text-xs text-muted-foreground">{item.term}</dt>
          <dd title={item.title} className={item.mono ? "font-mono text-xs break-all" : "break-words"}>{item.value}</dd>
        </div>
      ))}
    </dl>
  );
}

function BudgetLine({ workspace, teamId, dimension, label, currency }: {
  workspace: string;
  teamId: string;
  dimension: Dimension;
  label: string;
  currency: string | undefined;
}) {
  const limit = useTeamBudgetLimit(workspace, teamId, dimension);
  const remaining = useTeamBudgetRemaining(workspace, teamId, dimension);
  if (limit === null || remaining === null) return null;
  const show = (value: number) =>
    dimension === "costMicrounits" ? formatMicrocost(value, currency) : dimension === "elapsedSeconds" ? formatSeconds(value) : formatCount(value);
  return (
    <div className="min-w-0">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className="break-words">{show(limit - remaining)} used of {show(limit)} ({show(remaining)} left)</dd>
    </div>
  );
}

function Capacity({ workspace, teamId }: { workspace: string; teamId: string }) {
  const loadedAt = useTeamRunSummaryField(workspace, teamId, "loadedAt");
  const budget = useTeamRunSummaryField(workspace, teamId, "budget");
  const slotsLeft = useTeamTaskSlotsLeft(workspace, teamId);
  if (loadedAt === undefined) {
    return <p role="status" className="text-sm text-muted-foreground">Budget and task capacity appear once the execution summary has loaded.</p>;
  }
  return (
    <section aria-labelledby="team-capacity-heading" className="space-y-2">
      <h3 id="team-capacity-heading" className="text-sm font-semibold">Budget and capacity</h3>
      <dl className="grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2">
        <BudgetLine workspace={workspace} teamId={teamId} dimension="tokens" label="Tokens" currency={budget?.currency} />
        <BudgetLine workspace={workspace} teamId={teamId} dimension="costMicrounits" label="Cost" currency={budget?.currency} />
        <BudgetLine workspace={workspace} teamId={teamId} dimension="elapsedSeconds" label="Elapsed time" currency={budget?.currency} />
        {slotsLeft !== null && (
          <div className="min-w-0">
            <dt className="text-xs text-muted-foreground">Task slots left</dt>
            <dd>{formatCount(slotsLeft)} (every task ever created counts and none can be deleted)</dd>
          </div>
        )}
      </dl>
    </section>
  );
}

/** The load status flips on every poll; only this fallback subscribes, so the loaded shell does not rerender with it. */
function NotLoaded({ workspace, teamId }: { workspace: string; teamId: string }) {
  const status = useTeamLoadStatus(teamLoadKeys.run(workspace, teamId));
  const error = useTeamLoadError(teamLoadKeys.run(workspace, teamId));
  if (status === "error" && error) {
    return (
      <div className="space-y-2">
        <h2 id="teams-detail-heading" tabIndex={-1} className="text-lg font-semibold">This team could not be loaded</h2>
        <p role="alert" className="max-w-prose text-sm break-words text-destructive">
          {error.message} <span className="font-mono text-xs">({error.code})</span>
        </p>
        <p className="max-w-prose text-sm text-muted-foreground">
          The team id <span className="font-mono break-all">{teamId}</span> is looked up in workspace "{workspace}". Check the workspace.
        </p>
      </div>
    );
  }
  return (
    <div role="status" aria-label="Loading team" className="space-y-3">
      <Skeleton className="h-6 w-1/2" />
      <Skeleton className="h-4 w-3/4" />
    </div>
  );
}

interface TeamDetailProps {
  workspace: string;
  teamId: string;
  onBack: () => void;
  /** Move focus to the heading once it renders (set after a launch). */
  focusHeading?: boolean;
  onHeadingFocused?: () => void;
}

const NOOP = () => undefined;

export function TeamDetail({ workspace, teamId, onBack, focusHeading = false, onHeadingFocused = NOOP }: TeamDetailProps) {
  useTeamHeaderLoader(workspace, teamId);
  const revision = useTeamField(workspace, teamId, "revision");
  const notLoaded = revision === undefined;
  return (
    <ScrollArea className="min-h-0 flex-1">
      <div className="space-y-6 p-4 sm:p-6">
        <Button type="button" variant="ghost" className="min-h-11 md:hidden" onClick={onBack}>
          <ArrowLeft aria-hidden="true" />
          Back to list
        </Button>
        {notLoaded ? (
          <NotLoaded workspace={workspace} teamId={teamId} />
        ) : (
          <>
            <Title workspace={workspace} teamId={teamId} focusOnMount={focusHeading} onFocused={onHeadingFocused} />
            <Facts workspace={workspace} teamId={teamId} />
            <Capacity workspace={workspace} teamId={teamId} />
            <TeamRunView workspace={workspace} teamId={teamId} />
          </>
        )}
      </div>
    </ScrollArea>
  );
}
