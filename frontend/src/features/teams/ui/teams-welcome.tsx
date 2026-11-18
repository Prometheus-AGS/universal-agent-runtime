import { PackagePlus, Users } from "lucide-react";
import { teamLoadKeys, useTeamDefinitionKeys, useTeamIds, useTeamLoadStatus } from "@/platform/entities";
import { useActionGate } from "../model/use-readiness";
import { GatedAction } from "./gated-action";

/** Shown when nothing is selected. Names the workspace and the single next step. */
export function TeamsWelcome({ workspace, onInstall }: { workspace: string; onInstall: () => void }) {
  const definitions = useTeamDefinitionKeys().length;
  const teams = useTeamIds(workspace).length;
  const definitionsStatus = useTeamLoadStatus(teamLoadKeys.definitions);
  const teamsStatus = useTeamLoadStatus(teamLoadKeys.teams(workspace));
  const installReason = useActionGate(workspace, "install");
  const nothing = definitions === 0 && teams === 0 && definitionsStatus === "ready" && teamsStatus === "ready";
  return (
    <div className="flex flex-1 items-center justify-center p-6">
      <div className="flex max-w-md flex-col items-center gap-3 text-center">
        <div className="flex size-12 items-center justify-center rounded-xl bg-primary/10">
          {nothing ? <PackagePlus aria-hidden="true" size={20} className="text-primary" /> : <Users aria-hidden="true" size={20} className="text-primary" />}
        </div>
        {nothing ? (
          <>
            <h2 className="text-sm font-semibold">No team package is installed</h2>
            <p className="text-xs text-muted-foreground">
              Nothing is installed and workspace "{workspace}" has no teams. Install a team package to begin.
            </p>
            <GatedAction reason={installReason} onClick={onInstall}>
              <PackagePlus aria-hidden="true" />
              Install package
            </GatedAction>
          </>
        ) : (
          <>
            <h2 className="text-sm font-semibold">Select a team or a definition</h2>
            <p className="text-xs text-muted-foreground">
              Open a definition to bind and launch it in workspace "{workspace}", or open a team to follow what each member is doing.
            </p>
          </>
        )}
      </div>
    </div>
  );
}
