import { useEffect, useMemo } from "react";
import {
  definitionKey,
  teamLoadKeys,
  useExportedModelRequirements,
  usePackageExportStatus,
  useTeamActions,
  useTeamDefinitionField,
  useTeamLoadError,
  useTeamLoadStatus,
  type ImmutableDefinitionRef,
} from "@/platform/entities";
import { useActionGate } from "./use-readiness";

export interface DefinitionRefs {
  /** The package that carries this definition. */
  package: ImmutableDefinitionRef;
  packageKey: string;
  /** The definition as `useExportedModelRequirements` and friends take it. */
  definition: Pick<ImmutableDefinitionRef, "id" | "version">;
}

const NO_DEFINITION: Pick<ImmutableDefinitionRef, "id" | "version"> = { id: "", version: "" };

export function useDefinitionRefs(defKey: string): DefinitionRefs | null {
  const pkg = useTeamDefinitionField(defKey, "package");
  const id = useTeamDefinitionField(defKey, "definitionId");
  const version = useTeamDefinitionField(defKey, "version");
  return useMemo(
    () => (pkg && id && version ? { package: pkg, packageKey: definitionKey(pkg), definition: { id, version } } : null),
    [pkg, id, version],
  );
}

/**
 * Coordinator role and model aliases exist only in the package export. Fetch it once per session when a
 * definition is opened (a persisted export from an earlier session is refreshed, not trusted).
 */
export function useDefinitionExportLoader(defKey: string): void {
  const actions = useTeamActions();
  const refs = useDefinitionRefs(defKey);
  const loadStatus = useTeamLoadStatus(teamLoadKeys.packageExport(refs?.packageKey ?? ""));
  const pkg = refs?.package;
  useEffect(() => {
    if (!pkg || loadStatus !== "idle") return;
    void actions.loadPackageExport(pkg).catch(() => undefined); // Recorded in the load state; the sections explain it.
  }, [actions, pkg, loadStatus]);
}

/** Why a binding cannot be created for this definition right now, or null. Readiness first, then the package. */
export function useBindGate(workspace: string, defKey: string): string | null {
  const readiness = useActionGate(workspace, "bind");
  const refs = useDefinitionRefs(defKey);
  const packageKey = refs?.packageKey ?? "";
  const exportStatus = usePackageExportStatus(packageKey);
  const loadError = useTeamLoadError(teamLoadKeys.packageExport(packageKey));
  const requirements = useExportedModelRequirements(packageKey, refs?.definition ?? NO_DEFINITION);

  if (readiness) return readiness;
  if (!refs) return "The definition is not loaded.";
  if (loadError) return `The package export failed, so its model requirements are unknown: ${loadError.message}`;
  if (exportStatus === "refused") return "The runtime refused to export this package, so its model requirements are unknown.";
  if (exportStatus === undefined) return "Loading the package's model requirements.";
  if (requirements.length === 0) return "This package declares no agent model requirement, so the console cannot build a binding for it.";
  const unbindable = requirements.find((requirement) => requirement.preferredAliases.length === 0);
  if (unbindable) return `The model for role "${unbindable.role}" lists no preferred alias, so it cannot be bound here.`;
  return null;
}
