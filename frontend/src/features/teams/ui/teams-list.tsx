import { useLayoutEffect, useRef, useState, type RefObject } from "react";
import { Loader2, PackagePlus, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";
import { AdminEmptyInline, AdminSidebarSkeleton } from "@/shared/ui/configuration/configuration-states";
import {
  definitionKey,
  teamLoadKeys,
  useTeamDefinitionField,
  useTeamDefinitionKeys,
  useTeamField,
  useTeamIds,
  useTeamIsLive,
  useTeamLoadError,
  useTeamLoadStatus,
} from "@/platform/entities";
import { lastSegment, shortId } from "../model/format";
import { useActionGate } from "../model/use-readiness";
import { useTeamActivityPrefetch } from "../model/use-teams-bootstrap";
import type { TeamsSelection } from "../model/teams-location";
import { GatedAction } from "./gated-action";
import { RelativeTime } from "./relative-time";
import { TeamActivity } from "./team-activity";

/**
 * The selected row carries a filled marker in the primary colour as well as the tint, so it never relies on tint
 * alone. The marker is a fill, not a border: the console's flat style separates by surface fill.
 */
const ROW_CLASS =
  "relative flex min-h-11 w-full flex-col items-start gap-0.5 rounded-lg px-3 py-2 text-left transition-colors before:absolute before:inset-y-2 before:left-0 before:w-[3px] before:rounded-full hover:bg-muted focus-visible:outline-2 focus-visible:outline-ring";
const ROW_SELECTED = "bg-accent before:bg-primary hover:bg-accent";
const ROW_IDLE = "before:bg-transparent";

interface TeamRowProps {
  workspace: string;
  teamId: string;
  selected: boolean;
  onSelect: (teamId: string) => void;
  /** Remembers which row held focus when it moved between groups, so a background refresh never drops focus. */
  focusMemoryRef: RefObject<string | null>;
}

function TeamRowBody({ workspace, teamId, selected, onSelect, focusMemoryRef }: TeamRowProps) {
  const buttonRef = useRef<HTMLButtonElement>(null);
  useLayoutEffect(() => {
    const button = buttonRef.current;
    if (focusMemoryRef.current === teamId) {
      focusMemoryRef.current = null;
      button?.focus({ preventScroll: true });
    }
    return () => {
      if (button && document.activeElement === button) focusMemoryRef.current = teamId;
    };
  }, [focusMemoryRef, teamId]);
  const definition = useTeamField(workspace, teamId, "definition");
  const binding = useTeamField(workspace, teamId, "binding");
  const createdAt = useTeamField(workspace, teamId, "createdAt");
  const title = useTeamDefinitionField(definition ? definitionKey(definition) : "", "title");
  return (
    <li>
      <button
        ref={buttonRef}
        type="button"
        aria-current={selected ? "true" : undefined}
        onClick={() => onSelect(teamId)}
        className={cn(ROW_CLASS, selected ? ROW_SELECTED : ROW_IDLE)}
      >
        <span className="flex w-full min-w-0 items-baseline justify-between gap-2">
          <span className="min-w-0 truncate text-sm font-medium">{title ?? definition?.id ?? teamId}</span>
          <TeamActivity workspace={workspace} teamId={teamId} variant="row" />
        </span>
        <span className="flex w-full min-w-0 items-baseline gap-x-2 text-xs text-muted-foreground">
          <span className="min-w-0 truncate font-mono" title={binding?.id}>{binding ? lastSegment(binding.id) : "no binding"}</span>
          <span className="shrink-0 font-mono" title={teamId}>#{shortId(teamId)}</span>
          <RelativeTime value={createdAt} className="shrink-0" />
        </span>
      </button>
    </li>
  );
}

/**
 * Teams that are working come first, then the rest newest first. The domain has no ordered-by-activity list,
 * so each id is offered to both groups and only the matching one renders (DOM order equals visual order).
 */
function TeamRow({ mode, ...row }: TeamRowProps & { mode: "live" | "rest" }) {
  const live = useTeamIsLive(row.workspace, row.teamId);
  if (live !== (mode === "live")) return null;
  return <TeamRowBody {...row} />;
}

function TeamsGroup({ workspace, selectedTeamId, onSelect }: { workspace: string; selectedTeamId: string | null; onSelect: (teamId: string) => void }) {
  const ids = useTeamIds(workspace);
  const status = useTeamLoadStatus(teamLoadKeys.teams(workspace));
  const error = useTeamLoadError(teamLoadKeys.teams(workspace));
  useTeamActivityPrefetch(workspace, ids);
  const focusMemoryRef = useRef<string | null>(null);
  return (
    <section aria-labelledby="teams-group-heading" className="space-y-1">
      <h2 id="teams-group-heading" className="sr-only">Teams in workspace "{workspace}" ({ids.length})</h2>
      {ids.length === 0 && status === "ready" && (
        <AdminEmptyInline>
          No teams in workspace "{workspace}". Open the Definitions tab, choose a definition, then Launch team.
        </AdminEmptyInline>
      )}
      {ids.length === 0 && status !== "ready" && status !== "error" && <AdminSidebarSkeleton rows={2} />}
      {error && error.kind !== "unauthorized" && ids.length === 0 && (
        <p role="alert" className="px-3 text-xs break-words text-destructive">{error.message}</p>
      )}
      <ul aria-labelledby="teams-group-heading" className="space-y-0.5">
        {(["live", "rest"] as const).flatMap((mode) =>
          ids.map((id) => (
            <TeamRow key={`${mode}:${id}`} workspace={workspace} teamId={id} mode={mode} selected={id === selectedTeamId} onSelect={onSelect} focusMemoryRef={focusMemoryRef} />
          )),
        )}
      </ul>
    </section>
  );
}

function DefinitionRow({ defKey, selected, onSelect }: { defKey: string; selected: boolean; onSelect: (key: string) => void }) {
  const title = useTeamDefinitionField(defKey, "title");
  const version = useTeamDefinitionField(defKey, "version");
  const members = useTeamDefinitionField(defKey, "members");
  return (
    <li>
      <button
        type="button"
        aria-current={selected ? "true" : undefined}
        onClick={() => onSelect(defKey)}
        className={cn(ROW_CLASS, selected ? ROW_SELECTED : ROW_IDLE)}
      >
        <span className="w-full truncate text-sm font-medium">{title ?? defKey}</span>
        <span className="text-xs text-muted-foreground">
          Version {version ?? "unknown"}, {members?.length ?? 0} member {members?.length === 1 ? "role" : "roles"}
        </span>
      </button>
    </li>
  );
}

function DefinitionsGroup({ selectedKey, onSelect }: { selectedKey: string | null; onSelect: (key: string) => void }) {
  const keys = useTeamDefinitionKeys();
  const status = useTeamLoadStatus(teamLoadKeys.definitions);
  const error = useTeamLoadError(teamLoadKeys.definitions);
  return (
    <section aria-labelledby="definitions-group-heading" className="space-y-1">
      <h2 id="definitions-group-heading" className="sr-only">Definitions ({keys.length})</h2>
      {keys.length === 0 && status === "ready" && <AdminEmptyInline>No team package is installed. Use Install package above.</AdminEmptyInline>}
      {keys.length === 0 && status !== "ready" && status !== "error" && <AdminSidebarSkeleton rows={2} />}
      {error && error.kind !== "unauthorized" && keys.length === 0 && (
        <p role="alert" className="px-3 text-xs break-words text-destructive">{error.message}</p>
      )}
      <ul aria-labelledby="definitions-group-heading" className="space-y-0.5">
        {keys.map((key) => (
          <DefinitionRow key={key} defKey={key} selected={key === selectedKey} onSelect={onSelect} />
        ))}
      </ul>
    </section>
  );
}

function RefreshButton({ workspace, onRefresh }: { workspace: string; onRefresh: () => void }) {
  const loading = useTeamLoadStatus(teamLoadKeys.definitions) === "loading";
  const blocked = useActionGate(workspace, "install");
  return (
    <Button
      type="button"
      variant="ghost"
      size="icon"
      className="min-h-11 min-w-11 aria-disabled:cursor-not-allowed aria-disabled:opacity-50"
      aria-label={blocked ? `Refresh is unavailable. ${blocked}` : "Refresh teams and definitions"}
      aria-disabled={blocked ? true : undefined}
      onClick={() => { if (!blocked) onRefresh(); }}
    >
      {loading ? <Loader2 aria-hidden="true" className="size-4 motion-safe:animate-spin" /> : <RefreshCw aria-hidden="true" className="size-4" />}
    </Button>
  );
}

type ListTab = "teams" | "definitions";

/**
 * Teams and Definitions are two tabs of one list, so neither sits below the other. The tab follows what is
 * selected (a definition opens Definitions, a team opens Teams) until the operator picks one by hand, and an
 * empty workspace that has definitions opens on Definitions.
 */
function ListBody({ workspace, selection, onSelectTeam, onSelectDefinition }: Pick<TeamsListProps, "workspace" | "selection" | "onSelectTeam" | "onSelectDefinition">) {
  const teamCount = useTeamIds(workspace).length;
  const definitionCount = useTeamDefinitionKeys().length;
  // A hand-picked tab applies only to the kind of selection it was picked under; a new kind of selection takes over.
  const [picked, setPicked] = useState<{ forKind: TeamsSelection["kind"]; tab: ListTab } | null>(null);
  const followed: ListTab =
    selection.kind === "definition" ? "definitions"
      : selection.kind === "team" ? "teams"
        : teamCount === 0 && definitionCount > 0 ? "definitions" : "teams";
  const tab = picked !== null && picked.forKind === selection.kind ? picked.tab : followed;
  return (
    <Tabs
      value={tab}
      onValueChange={(value) => setPicked({ forKind: selection.kind, tab: value === "definitions" ? "definitions" : "teams" })}
      // The shared Tabs keys its column layout on a `data-horizontal` attribute that is never set, so callers pass `flex-col`.
      className="min-h-0 flex-col md:flex-1"
    >
      <TabsList className="mx-3 h-11 w-[calc(100%-1.5rem)]">
        <TabsTrigger value="teams">Teams ({teamCount})</TabsTrigger>
        <TabsTrigger value="definitions">Definitions ({definitionCount})</TabsTrigger>
      </TabsList>
      <ScrollArea className="min-h-0 md:flex-1">
        <TabsContent value="teams" className="pb-4">
          <TeamsGroup workspace={workspace} selectedTeamId={selection.kind === "team" ? selection.teamId : null} onSelect={onSelectTeam} />
        </TabsContent>
        <TabsContent value="definitions" className="pb-4">
          <DefinitionsGroup selectedKey={selection.kind === "definition" ? selection.key : null} onSelect={onSelectDefinition} />
        </TabsContent>
      </ScrollArea>
    </Tabs>
  );
}

interface TeamsListProps {
  workspace: string;
  selection: TeamsSelection;
  onSelectTeam: (teamId: string) => void;
  onSelectDefinition: (key: string) => void;
  onInstall: () => void;
  onRefresh: () => void;
  className?: string;
}

export function TeamsList({ workspace, selection, onSelectTeam, onSelectDefinition, onInstall, onRefresh, className }: TeamsListProps) {
  const installReason = useActionGate(workspace, "install");
  return (
    <nav aria-label="Teams and definitions" className={cn("flex shrink-0 flex-col bg-muted/30 md:min-h-0 md:w-80", className)}>
      <div className="flex items-start gap-2 px-3 py-3">
        <div className="min-w-0 flex-1">
          <GatedAction reason={installReason} variant="secondary" className="w-full" onClick={onInstall}>
            <PackagePlus aria-hidden="true" />
            Install package
          </GatedAction>
        </div>
        <RefreshButton workspace={workspace} onRefresh={onRefresh} />
      </div>
      <ListBody workspace={workspace} selection={selection} onSelectTeam={onSelectTeam} onSelectDefinition={onSelectDefinition} />
    </nav>
  );
}
