import { buildDeploymentBinding } from "./binding-builder";
import {
  cancelAttempt, createTeam, dismissRoutingTest, installTeamBinding, installTeamPackage,
  preflightTeamBinding, preflightTeamPackage, resumeRoutingTest, startRoutingTest,
} from "./commands";
import {
  ensureOwner, loadBindings, loadCapabilities, loadExecutionOwner, loadPackageExport, loadPackages,
  loadTeamArtifacts, loadTeamDefinitions, loadTeamExecutions, loadTeamPeerMessages, loadTeams, refreshTeamRun,
} from "./loads";
import { refreshTeamRunNow, watchTeamRun } from "./watch";

/**
 * The teams domain actions. Loads fetch then publish with one atomic graph write;
 * commands re-read revisions, send once and publish the returned record. Workspace
 * is an argument of every workspace-scoped action. Components reach this through
 * `useTeamActions()`.
 */
export const teamActions = {
  // runtime readiness
  loadCapabilities,
  loadExecutionOwner,
  ensureOwner,
  // catalog
  loadTeamDefinitions,
  loadPackages,
  loadPackageExport,
  preflightTeamPackage,
  installTeamPackage,
  // bindings
  buildDeploymentBinding,
  loadBindings,
  preflightTeamBinding,
  installTeamBinding,
  // teams and tasks
  loadTeams,
  createTeam,
  cancelAttempt,
  // run view
  refreshTeamRun,
  loadTeamArtifacts,
  loadTeamPeerMessages,
  loadTeamExecutions,
  watchTeamRun,
  refreshTeamRunNow,
  // routing test
  startRoutingTest,
  resumeRoutingTest,
  dismissRoutingTest,
} as const;

export type TeamActions = typeof teamActions;
