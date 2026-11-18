import { registerSchema } from "@prometheus-ags/prometheus-entity-management";
import {
  TEAM_ARTIFACT_ENTITY,
  TEAM_ATTEMPT_ENTITY,
  TEAM_BINDING_ENTITY,
  TEAM_DEFINITION_ENTITY,
  TEAM_INSTANCE_ENTITY,
  TEAM_LOAD_STATE_ENTITY,
  TEAM_MEMBER_ENTITY,
  TEAM_PACKAGE_ENTITY,
  TEAM_PACKAGE_EXPORT_ENTITY,
  TEAM_PEER_MESSAGE_ENTITY,
  TEAM_ROUTING_PROGRESS_ENTITY,
  TEAM_RUNTIME_ENTITY,
  TEAM_RUN_SUMMARY_ENTITY,
  TEAM_RUN_WATCH_ENTITY,
  TEAM_TASK_ENTITY,
  TEAM_WAIT_ENTITY,
} from "./entities";

const TEAM_ENTITY_TYPES = [
  TEAM_RUNTIME_ENTITY, TEAM_LOAD_STATE_ENTITY, TEAM_DEFINITION_ENTITY, TEAM_PACKAGE_ENTITY, TEAM_PACKAGE_EXPORT_ENTITY,
  TEAM_BINDING_ENTITY, TEAM_INSTANCE_ENTITY, TEAM_MEMBER_ENTITY, TEAM_TASK_ENTITY, TEAM_ATTEMPT_ENTITY, TEAM_WAIT_ENTITY,
  TEAM_RUN_SUMMARY_ENTITY, TEAM_ARTIFACT_ENTITY, TEAM_PEER_MESSAGE_ENTITY, TEAM_RUN_WATCH_ENTITY, TEAM_ROUTING_PROGRESS_ENTITY,
] as const;

let registered = false;

/**
 * Register the team schemas. No entity transport is registered on purpose: every team record is
 * owner- and workspace-qualified, so the generic list/get path (which identifies records by the
 * server's own id and takes no workspace) would write unqualified, cross-principal ids into the
 * persisted graph. The transport in `./api/teams-transport` is called only by the domain actions.
 */
export function registerTeamEntities(): void {
  if (registered) return;
  for (const type of TEAM_ENTITY_TYPES) registerSchema({ type });
  registered = true;
}
