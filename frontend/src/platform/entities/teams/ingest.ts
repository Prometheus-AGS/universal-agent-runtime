import type { GraphBatch } from "@prometheus-ags/entity-graph-core";
import type {
  DeploymentBinding,
  PackageExportOutcome,
  PeerMessageRecord,
  TeamArtifact,
  TeamDefinitionSummary,
  TeamExecutionSummary,
  TeamInstance,
  TeamPackageRecord,
} from "./contracts";
import {
  TEAM_ARTIFACT_ENTITY,
  TEAM_ATTEMPT_ENTITY,
  TEAM_BINDING_ENTITY,
  TEAM_DEFINITION_ENTITY,
  TEAM_INSTANCE_ENTITY,
  TEAM_MEMBER_ENTITY,
  TEAM_PACKAGE_ENTITY,
  TEAM_PACKAGE_EXPORT_ENTITY,
  TEAM_PEER_MESSAGE_ENTITY,
  TEAM_RUN_SUMMARY_ENTITY,
  TEAM_TASK_ENTITY,
  TEAM_WAIT_ENTITY,
  definitionKey,
  type TeamArtifactEntity,
  type TeamAttemptEntity,
  type TeamBindingEntity,
  type TeamDefinitionEntity,
  type TeamInstanceEntity,
  type TeamMemberEntity,
  type TeamPackageEntity,
  type TeamPackageExportEntity,
  type TeamPeerMessageEntity,
  type TeamRunSummaryEntity,
  type TeamTaskEntity,
  type TeamWaitEntity,
} from "./entities";
import { parsePackageExport, type PackageExportModel } from "./export-model";
import { bindingGid, childGid, definitionGid, lists, packageGid, teamGid } from "./keys";

/**
 * Batch writers. Each load builds one `batchGraphWrites` call from these, so a
 * fetched response publishes to subscribers once. Server records replace the
 * stored copy (never merge), and each list is replaced whole.
 */

const byCreated = <T extends { createdAt: string }>(items: readonly T[], key: (item: T) => string) =>
  [...items].sort((a, b) => (a.createdAt < b.createdAt ? -1 : a.createdAt > b.createdAt ? 1 : key(a) < key(b) ? -1 : 1));

function group<T>(items: readonly T[], key: (item: T) => string): Map<string, T[]> {
  const groups = new Map<string, T[]>();
  for (const item of items) groups.set(key(item), [...(groups.get(key(item)) ?? []), item]);
  return groups;
}

const put = <T extends { id: string }>(batch: GraphBatch, type: string, entity: T) =>
  batch.replaceEntity(type, entity.id, entity as unknown as Record<string, unknown>);

export function writeDefinitions(batch: GraphBatch, owner: string, definitions: readonly TeamDefinitionSummary[]): void {
  const ids = definitions.map((definition) => {
    const key = definitionKey(definition);
    const entity: TeamDefinitionEntity = {
      id: definitionGid(owner, key), ownerId: owner, definitionKey: key, definitionId: definition.id,
      version: definition.version, digest: definition.digest, title: definition.title, purpose: definition.purpose,
      package: definition.package, members: definition.members, budget: definition.budget ?? {},
      limits: definition.limits ?? {}, instructions: definition.instructions ?? null,
    };
    put(batch, TEAM_DEFINITION_ENTITY, entity);
    return entity.id;
  });
  batch.setListIds(lists.definitions(owner), ids, TEAM_DEFINITION_ENTITY);
}

export function writePackages(batch: GraphBatch, owner: string, packages: readonly TeamPackageRecord[]): void {
  const ids = packages.map((record) => {
    const key = definitionKey(record.identity);
    const entity: TeamPackageEntity = {
      id: packageGid(owner, key), ownerId: owner, packageKey: key, packageId: record.identity.id,
      version: record.identity.version, digest: record.identity.digest, manifest: record.manifest,
      definitionKeys: record.definitionKeys, installedAt: record.installedAt,
    };
    put(batch, TEAM_PACKAGE_ENTITY, entity);
    return entity.id;
  });
  batch.setListIds(lists.packages(owner), ids, TEAM_PACKAGE_ENTITY);
}

export function writePackageExport(batch: GraphBatch, owner: string, packageKey: string, outcome: PackageExportOutcome, now: number): PackageExportModel {
  const model = parsePackageExport(outcome);
  const entity: TeamPackageExportEntity = { id: packageGid(owner, packageKey), ownerId: owner, packageKey, model, loadedAt: now };
  put(batch, TEAM_PACKAGE_EXPORT_ENTITY, entity);
  return model;
}

export function writeBindings(batch: GraphBatch, owner: string, workspace: string, bindings: readonly DeploymentBinding[]): void {
  const entities = bindings.map((binding): TeamBindingEntity => ({
    id: bindingGid(owner, workspace, binding.id), ownerId: owner, workspace, bindingId: binding.id,
    revision: binding.revision, package: binding.package, packageKey: definitionKey(binding.package),
    document: binding.document, activationSupported: binding.activationSupported,
    preflightDiagnostics: binding.preflightDiagnostics ?? [], effectiveBindingReceipt: binding.effectiveBindingReceipt ?? null,
    updatedAt: binding.updatedAt,
  }));
  for (const entity of entities) put(batch, TEAM_BINDING_ENTITY, entity);
  batch.setListIds(lists.bindings(owner, workspace), entities.map((entity) => entity.id), TEAM_BINDING_ENTITY);
  for (const [packageKey, items] of group(entities, (entity) => entity.packageKey)) {
    batch.setListIds(lists.bindingsByPackage(owner, workspace, packageKey), items.map((item) => item.id), TEAM_BINDING_ENTITY);
  }
}

/** One team with its members and tasks, and the lists derived from them. Does not touch the teams list. */
export function writeTeam(batch: GraphBatch, owner: string, workspace: string, team: TeamInstance): string {
  const gid = teamGid(owner, workspace, team.id);
  const instance: TeamInstanceEntity = {
    id: gid, ownerId: owner, workspace, teamId: team.id, revision: team.revision, status: team.status,
    definition: team.definition, package: team.package, binding: team.binding, input: team.input,
    createdAt: team.createdAt, updatedAt: team.updatedAt,
  };
  put(batch, TEAM_INSTANCE_ENTITY, instance);
  // `ordinal` counts within a role, so order by role first or the roles interleave.
  const members = [...team.members].sort((a, b) => a.role.localeCompare(b.role) || a.ordinal - b.ordinal);
  for (const member of members) {
    const entity: TeamMemberEntity = {
      id: childGid(owner, workspace, team.id, member.id), ownerId: owner, workspace, teamId: team.id,
      memberId: member.id, role: member.role, kind: member.kind, ordinal: member.ordinal,
      definition: member.definition, revision: member.revision, status: member.status,
    };
    put(batch, TEAM_MEMBER_ENTITY, entity);
  }
  batch.setListIds(lists.members(owner, workspace, team.id), members.map((m) => childGid(owner, workspace, team.id, m.id)), TEAM_MEMBER_ENTITY);
  const tasks = byCreated(team.tasks, (task) => task.id);
  for (const task of tasks) {
    const entity: TeamTaskEntity = {
      id: childGid(owner, workspace, team.id, task.id), ownerId: owner, workspace, teamId: team.id, taskId: task.id,
      title: task.title, role: task.role, input: task.input, outputContract: task.outputContract, output: task.output ?? null,
      dependsOn: task.dependsOn ?? [], status: task.status, revision: task.revision,
      assigneeMemberId: task.assigneeMemberId ?? null, reviewerMemberId: task.reviewerMemberId ?? null,
      stateReason: task.stateReason ?? null, createdAt: task.createdAt, updatedAt: task.updatedAt,
    };
    put(batch, TEAM_TASK_ENTITY, entity);
  }
  const taskGid = (taskId: string) => childGid(owner, workspace, team.id, taskId);
  batch.setListIds(lists.tasks(owner, workspace, team.id), tasks.map((task) => taskGid(task.id)), TEAM_TASK_ENTITY);
  batch.setListIds(
    lists.unassignedTasks(owner, workspace, team.id),
    tasks.filter((task) => !task.assigneeMemberId).map((task) => taskGid(task.id)),
    TEAM_TASK_ENTITY,
  );
  for (const member of members) {
    batch.setListIds(
      lists.tasksByMember(owner, workspace, team.id, member.id),
      tasks.filter((task) => task.assigneeMemberId === member.id).map((task) => taskGid(task.id)),
      TEAM_TASK_ENTITY,
    );
  }
  return gid;
}

/** Teams list: newest first by creation time. */
export function writeTeams(batch: GraphBatch, owner: string, workspace: string, teams: readonly TeamInstance[]): void {
  const ordered = [...teams].sort((a, b) => (a.createdAt < b.createdAt ? 1 : a.createdAt > b.createdAt ? -1 : 0));
  const ids = ordered.map((team) => writeTeam(batch, owner, workspace, team));
  batch.setListIds(lists.teams(owner, workspace), ids, TEAM_INSTANCE_ENTITY);
}

/** Adds one team to the front of the teams list (after a create) without refetching the list. */
export function prependTeam(batch: GraphBatch, owner: string, workspace: string, team: TeamInstance): void {
  const gid = writeTeam(batch, owner, workspace, team);
  batch.insertIdInList(lists.teams(owner, workspace), gid, "start");
}

export function writeExecution(
  batch: GraphBatch, owner: string, workspace: string, teamId: string, summary: TeamExecutionSummary,
  members: readonly string[], now: number,
): void {
  const attempts = byCreated(summary.attempts, (attempt) => attempt.id);
  const attemptGid = (id: string) => childGid(owner, workspace, teamId, id);
  for (const attempt of attempts) {
    const entity: TeamAttemptEntity = {
      id: attemptGid(attempt.id), ownerId: owner, workspace, teamId, attemptId: attempt.id, runId: attempt.runId,
      taskId: attempt.taskId, memberId: attempt.memberId, status: attempt.status,
      executionOutcome: attempt.executionOutcome ?? null, effectDisposition: attempt.effectDisposition,
      accountingState: attempt.accountingState, reservation: attempt.reservation, usage: attempt.usage ?? null,
      output: attempt.output ?? null, stateReason: attempt.stateReason ?? null, diagnostic: attempt.diagnostic ?? null,
      effectiveModels: attempt.effectiveModels ?? [], continuationOfWaitId: attempt.continuationOfWaitId ?? null,
      contextArtifactIds: attempt.contextArtifactIds ?? [], queueSequence: attempt.queueSequence ?? 0,
      createdAt: attempt.createdAt, updatedAt: attempt.updatedAt,
    };
    put(batch, TEAM_ATTEMPT_ENTITY, entity);
  }
  batch.setListIds(lists.attempts(owner, workspace, teamId), attempts.map((a) => attemptGid(a.id)), TEAM_ATTEMPT_ENTITY);
  const memberIds = new Set([...members, ...attempts.map((attempt) => attempt.memberId)]);
  for (const memberId of memberIds) {
    batch.setListIds(
      lists.attemptsByMember(owner, workspace, teamId, memberId),
      attempts.filter((a) => a.memberId === memberId).map((a) => attemptGid(a.id)),
      TEAM_ATTEMPT_ENTITY,
    );
  }
  for (const [taskId, items] of group(attempts, (attempt) => attempt.taskId)) {
    batch.setListIds(lists.attemptsByTask(owner, workspace, teamId, taskId), items.map((a) => attemptGid(a.id)), TEAM_ATTEMPT_ENTITY);
  }
  const waits = summary.waits ?? [];
  const waitGid = (id: string) => childGid(owner, workspace, teamId, id);
  for (const wait of waits) {
    const entity: TeamWaitEntity = {
      id: waitGid(wait.waitId), ownerId: owner, workspace, teamId, waitId: wait.waitId, attemptId: wait.authority.attemptId,
      taskId: wait.authority.taskId, memberId: wait.authority.memberId, targetTaskIds: wait.targetTaskIds,
      predicate: wait.predicate, state: wait.state, continuationAttemptId: wait.continuationAttemptId ?? null,
      reasonCode: wait.reasonCode ?? null, wakeOutcomes: wait.wakeOutcomes ?? [], continuationInput: wait.continuationInput,
    };
    put(batch, TEAM_WAIT_ENTITY, entity);
  }
  batch.setListIds(lists.waits(owner, workspace, teamId), waits.map((w) => waitGid(w.waitId)), TEAM_WAIT_ENTITY);
  for (const memberId of memberIds) {
    batch.setListIds(
      lists.waitsByMember(owner, workspace, teamId, memberId),
      waits.filter((w) => w.authority.memberId === memberId).map((w) => waitGid(w.waitId)),
      TEAM_WAIT_ENTITY,
    );
  }
  const run: TeamRunSummaryEntity = {
    id: teamGid(owner, workspace, teamId), ownerId: owner, workspace, teamId, committed: summary.committed,
    reserved: summary.reserved, uncertainAttemptIds: summary.uncertainAttempts ?? [], budget: summary.budget ?? {},
    limits: summary.limits ?? {},
    delegations: (summary.commandReceipts ?? []).filter((receipt) => receipt.operation === "team_delegate"),
    continuations: summary.continuations ?? [], loadedAt: now,
  };
  put(batch, TEAM_RUN_SUMMARY_ENTITY, run);
}

export function writeArtifacts(
  batch: GraphBatch, owner: string, workspace: string, teamId: string, artifacts: readonly TeamArtifact[],
  members: readonly string[],
): void {
  const ordered = byCreated(artifacts, (artifact) => artifact.id);
  const gidOf = (id: string) => childGid(owner, workspace, teamId, id);
  for (const artifact of ordered) {
    const entity: TeamArtifactEntity = {
      id: gidOf(artifact.id), ownerId: owner, workspace, teamId, artifactId: artifact.id, taskId: artifact.taskId,
      memberId: artifact.memberId, attemptId: artifact.attemptId, content: artifact.content, createdAt: artifact.createdAt,
    };
    put(batch, TEAM_ARTIFACT_ENTITY, entity);
  }
  batch.setListIds(lists.artifacts(owner, workspace, teamId), ordered.map((a) => gidOf(a.id)), TEAM_ARTIFACT_ENTITY);
  for (const memberId of new Set([...members, ...ordered.map((a) => a.memberId)])) {
    batch.setListIds(
      lists.artifactsByMember(owner, workspace, teamId, memberId),
      ordered.filter((a) => a.memberId === memberId).map((a) => gidOf(a.id)),
      TEAM_ARTIFACT_ENTITY,
    );
  }
}

export function writePeerMessages(
  batch: GraphBatch, owner: string, workspace: string, teamId: string, records: readonly PeerMessageRecord[],
): void {
  const ordered = [...records].sort((a, b) => (a.message.acceptedAt < b.message.acceptedAt ? -1 : 1));
  for (const { message, delivery } of ordered) {
    const entity: TeamPeerMessageEntity = {
      id: childGid(owner, workspace, teamId, message.messageId), ownerId: owner, workspace, teamId,
      messageId: message.messageId, senderMemberId: message.senderMemberId, senderAttemptId: message.senderAttemptId,
      recipientMemberId: message.recipientMemberId, recipientTaskId: message.recipientTaskId ?? null, mode: message.mode,
      payload: message.payload, acceptedAt: message.acceptedAt, delivery,
    };
    put(batch, TEAM_PEER_MESSAGE_ENTITY, entity);
  }
  batch.setListIds(
    lists.peerMessages(owner, workspace, teamId),
    ordered.map((record) => childGid(owner, workspace, teamId, record.message.messageId)),
    TEAM_PEER_MESSAGE_ENTITY,
  );
}
