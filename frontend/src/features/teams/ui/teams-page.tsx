import { useCallback, useState } from "react";
import { cn } from "@/lib/utils";
import { useTeamsBootstrap } from "../model/use-teams-bootstrap";
import { useTeamsLocation } from "../model/teams-location";
import { useSheetFocusReturn } from "../model/use-sheet-helpers";
import { BindingSheet } from "./binding-sheet";
import { DefinitionDetail } from "./definition-detail";
import { InstallPackageSheet } from "./install-package-sheet";
import { LaunchSheet } from "./launch-sheet";
import { ReadinessStrip } from "./readiness-strip";
import { TeamDetail } from "./team-detail";
import { TeamsList } from "./teams-list";
import { TeamsWelcome } from "./teams-welcome";
import { WorkspaceControl } from "./workspace-control";

type OpenSheet = "install" | "binding" | "launch" | null;

export function TeamsPage() {
  const location = useTeamsLocation();
  const { workspace, selection } = location;
  const { refresh } = useTeamsBootstrap(workspace);
  const focus = useSheetFocusReturn();
  const [sheet, setSheet] = useState<OpenSheet>(null);
  // The team just launched: its detail view takes focus once its heading has rendered (the launch button is gone by then).
  const [focusTeamId, setFocusTeamId] = useState<string | null>(null);
  const clearFocusTeam = useCallback(() => setFocusTeamId(null), []);
  const hasSelection = selection.kind !== "none";

  const open = (next: Exclude<OpenSheet, null>) => {
    focus.remember();
    setSheet(next);
  };
  const close = (preferredId?: string) => {
    setSheet(null);
    focus.restore(preferredId);
  };
  const sheetChanged = (kind: Exclude<OpenSheet, null>) => (next: boolean) => {
    if (next) setSheet(kind);
    else close();
  };

  // Below md the page is one scroll: the header and readiness scroll away with the list. Opening a team or
  // definition hides them there, so the detail starts with "Back to list" and the name. From md up they are one
  // compact row above two independently scrolling panes.
  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-y-auto md:overflow-hidden">
      <header className={cn("shrink-0 flex-wrap items-center gap-x-5 gap-y-2 px-4 pt-3 pb-2 sm:px-6 md:flex", hasSelection ? "hidden" : "flex")}>
        <h1 id="teams-heading" tabIndex={-1} className="text-xl font-semibold focus-visible:outline-2 focus-visible:outline-ring">Teams</h1>
        <WorkspaceControl key={workspace} workspace={workspace} onCommit={location.commitWorkspace} />
        <ReadinessStrip workspace={workspace} />
      </header>
      <div className="flex flex-col md:min-h-0 md:flex-1 md:flex-row md:overflow-hidden">
        <TeamsList
          workspace={workspace}
          selection={selection}
          onSelectTeam={location.selectTeam}
          onSelectDefinition={location.selectDefinition}
          onInstall={() => open("install")}
          onRefresh={refresh}
          className={cn(hasSelection ? "hidden md:flex" : "flex")}
        />
        <section aria-label="Team detail" className={cn("flex min-w-0 flex-col md:min-h-0 md:flex-1 md:overflow-hidden", !hasSelection && "hidden md:flex")}>
          {selection.kind === "team" && (
            <TeamDetail
              key={`${workspace}:${selection.teamId}`}
              workspace={workspace}
              teamId={selection.teamId}
              onBack={location.clearSelection}
              focusHeading={selection.teamId === focusTeamId}
              onHeadingFocused={clearFocusTeam}
            />
          )}
          {selection.kind === "definition" && (
            <DefinitionDetail
              workspace={workspace}
              defKey={selection.key}
              onBack={location.clearSelection}
              onCreateBinding={() => open("binding")}
              onLaunch={() => open("launch")}
            />
          )}
          {selection.kind === "none" && <TeamsWelcome workspace={workspace} onInstall={() => open("install")} />}
        </section>
      </div>

      <InstallPackageSheet
        open={sheet === "install"}
        onOpenChange={sheetChanged("install")}
        onInstalled={(key) => {
          if (key) location.selectDefinition(key);
          close();
        }}
      />
      {selection.kind === "definition" && (
        <>
          <BindingSheet
            open={sheet === "binding"}
            onOpenChange={sheetChanged("binding")}
            workspace={workspace}
            defKey={selection.key}
            onInstalled={(bindingId) => close(`binding-row-${bindingId}`)}
          />
          <LaunchSheet
            open={sheet === "launch"}
            onOpenChange={sheetChanged("launch")}
            workspace={workspace}
            defKey={selection.key}
            onLaunched={(teamId) => {
              setFocusTeamId(teamId);
              location.selectTeam(teamId);
              setSheet(null); // Focus goes to the new team's heading, not back to the launch button that is about to unmount.
            }}
          />
        </>
      )}
    </div>
  );
}
