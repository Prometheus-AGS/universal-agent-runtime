import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import type { TeamsErrorInfo } from "./api/teams-errors";
import type { ExecutionClaim, ImmutableDefinitionRef } from "./contracts";
import {
  TEAMS_SESSION_ID,
  TEAM_BINDING_ENTITY,
  TEAM_DEFINITION_ENTITY,
  TEAM_LOAD_STATE_ENTITY,
  TEAM_PACKAGE_ENTITY,
  TEAM_PACKAGE_EXPORT_ENTITY,
  type LoadStatus,
  type TeamBindingEntity,
  type TeamDefinitionEntity,
  type TeamLoadState,
  type TeamPackageEntity,
  type TeamPackageExportEntity,
  type TeamRuntimeState,
} from "./entities";
import type { ExportedTeam, ModelRequirement } from "./export-model";
import { bindingGid, definitionGid, lists, packageGid } from "./keys";
import { teamActions } from "./domain";
import { entityOf, idsOf, ownerOf, runtimeOf, useScopedField, useScopedIds } from "./selectors";
import { loadStateId } from "./store";

/**
 * Catalog, readiness and binding hooks. Components import these from
 * `@/platform/entities` and never touch the graph, stores or transports.
 */

export function useTeamActions() {
  return teamActions;
}

// --- Readiness ---------------------------------------------------------------

/** The owner admitted this session (capabilities `bindingOwnerId`), or null before it loads. */
export function useTeamsOwnerId(): string | null {
  return useGraphStore((state) => runtimeOf(state)?.ownerId ?? null);
}

type RuntimeField = "instanceId" | "catalogBackend" | "teamExecutionAvailable" | "ownsExecution" | "capabilitiesLoadedAt" | "executionOwnerLoadedAt";
/** `null` means not loaded this session. An empty `instanceId` means the host has not configured one. */
export function useTeamsRuntimeField<K extends RuntimeField>(field: K): TeamRuntimeState[K] | null {
  return useGraphStore((state) => runtimeOf(state)?.[field] ?? null) as TeamRuntimeState[K] | null;
}

export function useTeamsHasCapability(name: string): boolean {
  return useGraphStore((state) => runtimeOf(state)?.capabilityNames?.includes(name) ?? false);
}

/** The holder's claim, read-only: epoch, acquisition time and holder identity. */
export function useTeamExecutionClaimField<K extends keyof ExecutionClaim>(field: K): ExecutionClaim[K] | null {
  return useGraphStore((state) => runtimeOf(state)?.claim?.[field] ?? null) as ExecutionClaim[K] | null;
}

/** `idle` until a load starts in this session. */
export function useTeamLoadStatus(key: string): LoadStatus {
  return useGraphStore((state) => {
    const record = entityOf<TeamLoadState>(state, TEAM_LOAD_STATE_ENTITY, loadStateId(key));
    return record?.session === TEAMS_SESSION_ID ? record.status : "idle";
  });
}

export function useTeamLoadError(key: string): TeamsErrorInfo | null {
  return useGraphStore((state) => {
    const record = entityOf<TeamLoadState>(state, TEAM_LOAD_STATE_ENTITY, loadStateId(key));
    return record?.session === TEAMS_SESSION_ID ? record.error : null;
  });
}

// --- Definitions and packages ------------------------------------------------

/** Keys (`id@version`) of installed team definitions. */
export function useTeamDefinitionKeys(): string[] {
  return useScopedIds((owner) => lists.definitions(owner));
}

export function useTeamDefinitionField<K extends keyof TeamDefinitionEntity>(definitionKey: string, field: K): TeamDefinitionEntity[K] | undefined {
  return useScopedField<TeamDefinitionEntity, K>(TEAM_DEFINITION_ENTITY, (owner) => definitionGid(owner, definitionKey), field);
}

export function usePackageField<K extends keyof TeamPackageEntity>(packageKey: string, field: K): TeamPackageEntity[K] | undefined {
  return useScopedField<TeamPackageEntity, K>(TEAM_PACKAGE_ENTITY, (owner) => packageGid(owner, packageKey), field);
}

/** `undefined` until the export has been loaded in this graph; `refused` means role and aliases must be entered by hand. */
export function usePackageExportStatus(packageKey: string): "exported" | "refused" | undefined {
  return useGraphStore((state) => {
    const owner = runtimeOf(state)?.ownerId;
    return owner ? entityOf<TeamPackageExportEntity>(state, TEAM_PACKAGE_EXPORT_ENTITY, packageGid(owner, packageKey))?.model.status : undefined;
  });
}

export function usePackageExportRefusal(packageKey: string): string[] | undefined {
  return useGraphStore((state) => {
    const owner = runtimeOf(state)?.ownerId;
    return owner ? entityOf<TeamPackageExportEntity>(state, TEAM_PACKAGE_EXPORT_ENTITY, packageGid(owner, packageKey))?.model.refusalMessages : undefined;
  });
}

function exportedTeam(state: Parameters<typeof runtimeOf>[0], packageKey: string, definition: Pick<ImmutableDefinitionRef, "id" | "version">): ExportedTeam | undefined {
  const owner = runtimeOf(state)?.ownerId;
  if (!owner) return undefined;
  return entityOf<TeamPackageExportEntity>(state, TEAM_PACKAGE_EXPORT_ENTITY, packageGid(owner, packageKey))
    ?.model.teams.find((team) => team.definition.id === definition.id && team.definition.version === definition.version);
}

/** The team definition's coordinator role from the package export, or null. */
export function useExportedCoordinatorRole(packageKey: string, definition: Pick<ImmutableDefinitionRef, "id" | "version">): string | null {
  return useGraphStore((state) => exportedTeam(state, packageKey, definition)?.coordinatorRole ?? null);
}

const NO_REQUIREMENTS: ModelRequirement[] = [];
/** One entry per agent model needing a `modelBindings` row, de-duplicated by alias set. */
export function useExportedModelRequirements(packageKey: string, definition: Pick<ImmutableDefinitionRef, "id" | "version">): ModelRequirement[] {
  return useGraphStore((state) => exportedTeam(state, packageKey, definition)?.modelRequirements ?? NO_REQUIREMENTS);
}

// --- Bindings -----------------------------------------------------------------

/** Binding ids of the workspace that target one package. */
export function useBindingIdsForPackage(workspace: string, packageKey: string): string[] {
  return useScopedIds((owner) => lists.bindingsByPackage(owner, workspace, packageKey));
}

/** The binding a launch should start from: the first one the runtime can activate, else the first, else null. */
export function usePreferredBindingId(workspace: string, packageKey: string): string | null {
  return useGraphStore((state) => {
    const owner = ownerOf(state);
    if (!owner) return null;
    const graphIds = idsOf(state, lists.bindingsByPackage(owner, workspace, packageKey));
    const binding = (graphId: string) => entityOf<TeamBindingEntity>(state, TEAM_BINDING_ENTITY, graphId);
    const chosen = graphIds.find((graphId) => binding(graphId)?.activationSupported === true) ?? graphIds[0];
    return chosen ? (binding(chosen)?.bindingId ?? null) : null;
  });
}

export function useBindingField<K extends keyof TeamBindingEntity>(workspace: string, bindingId: string, field: K): TeamBindingEntity[K] | undefined {
  return useScopedField<TeamBindingEntity, K>(TEAM_BINDING_ENTITY, (owner) => bindingGid(owner, workspace, bindingId), field);
}
