import { catalogEntityId, listKey, scopedEntityId } from "./entities";

/** Graph ids of workspace-scoped records. Always owner- and workspace-qualified. */
export const teamGid = (owner: string, workspace: string, teamId: string) => scopedEntityId(owner, workspace, teamId);
export const bindingGid = (owner: string, workspace: string, bindingId: string) => scopedEntityId(owner, workspace, bindingId);
export const childGid = (owner: string, workspace: string, teamId: string, childId: string) =>
  scopedEntityId(owner, workspace, teamId, childId);

export const routingGid = (owner: string, workspace: string, teamId: string, taskId: string) =>
  scopedEntityId(owner, workspace, teamId, taskId);
export const definitionGid = (owner: string, definitionKey: string) => catalogEntityId(owner, definitionKey);
export const packageGid = (owner: string, packageKey: string) => catalogEntityId(owner, packageKey);

/** Ordered id lists. Every list is replaced as a whole by the ingest that owns it. */
export const lists = {
  definitions: (owner: string) => listKey("definitions", owner),
  packages: (owner: string) => listKey("packages", owner),
  bindings: (owner: string, workspace: string) => listKey("bindings", owner, workspace),
  bindingsByPackage: (owner: string, workspace: string, packageKey: string) =>
    listKey("bindings-by-package", owner, workspace, packageKey),
  teams: (owner: string, workspace: string) => listKey("teams", owner, workspace),
  members: (owner: string, workspace: string, teamId: string) => listKey("members", owner, workspace, teamId),
  tasks: (owner: string, workspace: string, teamId: string) => listKey("tasks", owner, workspace, teamId),
  unassignedTasks: (owner: string, workspace: string, teamId: string) =>
    listKey("tasks-unassigned", owner, workspace, teamId),
  tasksByMember: (owner: string, workspace: string, teamId: string, memberId: string) =>
    listKey("tasks-by-member", owner, workspace, teamId, memberId),
  attempts: (owner: string, workspace: string, teamId: string) => listKey("attempts", owner, workspace, teamId),
  attemptsByMember: (owner: string, workspace: string, teamId: string, memberId: string) =>
    listKey("attempts-by-member", owner, workspace, teamId, memberId),
  attemptsByTask: (owner: string, workspace: string, teamId: string, taskId: string) =>
    listKey("attempts-by-task", owner, workspace, teamId, taskId),
  waits: (owner: string, workspace: string, teamId: string) => listKey("waits", owner, workspace, teamId),
  waitsByMember: (owner: string, workspace: string, teamId: string, memberId: string) =>
    listKey("waits-by-member", owner, workspace, teamId, memberId),
  artifacts: (owner: string, workspace: string, teamId: string) => listKey("artifacts", owner, workspace, teamId),
  artifactsByMember: (owner: string, workspace: string, teamId: string, memberId: string) =>
    listKey("artifacts-by-member", owner, workspace, teamId, memberId),
  routing: (owner: string, workspace: string, teamId: string) => listKey("routing-tests", owner, workspace, teamId),
  peerMessages: (owner: string, workspace: string, teamId: string) =>
    listKey("peer-messages", owner, workspace, teamId),
};
