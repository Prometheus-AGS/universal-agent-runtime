import { z } from "zod";
import type { ImmutableDefinitionRef, PackageExportOutcome } from "./contracts";

/**
 * A package version carries no definition documents. `POST /packages:export`
 * is the only place the team's `coordinatorRole` and each agent's
 * `models[].preferredAliases` appear, as JSON strings keyed by manifest path.
 * This module parses them into the small model the binding flow needs.
 */

const refSchema = z.object({ id: z.string(), version: z.string(), digest: z.string() });
const teamDocumentSchema = z.object({
  kind: z.literal("TeamDefinition"),
  id: z.string(),
  version: z.string(),
  contentDigest: z.string(),
  title: z.string().optional(),
  coordinatorRole: z.string().optional(),
  members: z.array(z.object({ role: z.string(), kind: z.string(), definition: refSchema })).default([]),
});
const agentDocumentSchema = z.object({
  kind: z.literal("AgentDefinition"),
  id: z.string(),
  version: z.string(),
  contentDigest: z.string(),
  title: z.string().optional(),
  models: z
    .array(
      z.object({
        role: z.string(),
        capabilities: z.array(z.string()).default([]),
        preferredAliases: z.array(z.string()).default([]),
      }),
    )
    .default([]),
});

/**
 * One model the binding must satisfy. The runtime resolves each agent `models[]`
 * entry through ANY of its `preferredAliases`, so a binding needs one
 * `modelBindings` entry per requirement, whose `requestedAlias` is one of them.
 */
export interface ModelRequirement {
  /** Sorted preferred aliases joined with "|"; the de-duplication key. */
  requirementKey: string;
  agentDefinitionId: string;
  role: string;
  capabilities: string[];
  preferredAliases: string[];
  /** Other agent definitions whose model entry has the same alias set. */
  alsoUsedBy: string[];
}

export interface ExportedTeamMember {
  role: string;
  kind: string;
  definition: ImmutableDefinitionRef;
}
export interface ExportedTeam {
  definition: ImmutableDefinitionRef;
  title: string | null;
  coordinatorRole: string | null;
  members: ExportedTeamMember[];
  /** Requirements of the agents this team's members use, de-duplicated by alias set. */
  modelRequirements: ModelRequirement[];
}

export interface PackageExportModel {
  status: "exported" | "refused";
  teams: ExportedTeam[];
  /** Requirements across every agent in the package, de-duplicated by alias set. */
  modelRequirements: ModelRequirement[];
  /** Paths whose JSON could not be read or did not match a known definition shape. */
  unreadableFiles: string[];
  /** Messages of the refusal, when the runtime refused the export (role and aliases must then be entered by hand). */
  refusalMessages: string[];
}

interface ParsedAgent {
  ref: ImmutableDefinitionRef;
  requirements: Array<Omit<ModelRequirement, "alsoUsedBy" | "requirementKey">>;
}

function sameRef(left: ImmutableDefinitionRef, right: ImmutableDefinitionRef): boolean {
  return left.id === right.id && left.version === right.version && left.digest === right.digest;
}

export function mergeRequirements(
  entries: ReadonlyArray<Omit<ModelRequirement, "alsoUsedBy" | "requirementKey">>,
): ModelRequirement[] {
  const merged = new Map<string, ModelRequirement>();
  for (const entry of entries) {
    const requirementKey = [...entry.preferredAliases].sort().join("|");
    const existing = merged.get(requirementKey);
    if (!existing) {
      merged.set(requirementKey, { ...entry, requirementKey, alsoUsedBy: [] });
    } else if (existing.agentDefinitionId !== entry.agentDefinitionId && !existing.alsoUsedBy.includes(entry.agentDefinitionId)) {
      merged.set(requirementKey, { ...existing, alsoUsedBy: [...existing.alsoUsedBy, entry.agentDefinitionId] });
    }
  }
  return [...merged.values()];
}

/** Parse an export outcome. Never throws on file content: unreadable files are listed instead. */
export function parsePackageExport(outcome: PackageExportOutcome): PackageExportModel {
  if (outcome.status !== "exported") {
    return {
      status: "refused",
      teams: [],
      modelRequirements: [],
      unreadableFiles: [],
      refusalMessages: outcome.reports.flatMap((report) => report.diagnostics.map((diagnostic) => diagnostic.message)),
    };
  }
  const unreadableFiles: string[] = [];
  const agents: ParsedAgent[] = [];
  const teamDocuments: Array<z.infer<typeof teamDocumentSchema>> = [];
  for (const [path, source] of Object.entries(outcome.export.files)) {
    let document: unknown;
    try {
      document = JSON.parse(source);
    } catch {
      unreadableFiles.push(path);
      continue;
    }
    const kind = document && typeof document === "object" ? (document as { kind?: unknown }).kind : undefined;
    if (kind === "TeamDefinition") {
      const team = teamDocumentSchema.safeParse(document);
      if (team.success) teamDocuments.push(team.data);
      else unreadableFiles.push(path);
    } else if (kind === "AgentDefinition") {
      const agent = agentDocumentSchema.safeParse(document);
      if (!agent.success) {
        unreadableFiles.push(path);
        continue;
      }
      const { id, version, contentDigest, models } = agent.data;
      agents.push({
        ref: { id, version, digest: contentDigest },
        requirements: models.map((model) => ({
          agentDefinitionId: id,
          role: model.role,
          capabilities: model.capabilities,
          preferredAliases: model.preferredAliases,
        })),
      });
    }
  }
  const teams = teamDocuments.map((team): ExportedTeam => {
    const definition = { id: team.id, version: team.version, digest: team.contentDigest };
    const used = agents.filter((agent) => team.members.some((member) => sameRef(member.definition, agent.ref)));
    return {
      definition,
      title: team.title ?? null,
      coordinatorRole: team.coordinatorRole ?? null,
      members: team.members,
      modelRequirements: mergeRequirements(used.flatMap((agent) => agent.requirements)),
    };
  });
  return {
    status: "exported",
    teams,
    modelRequirements: mergeRequirements(agents.flatMap((agent) => agent.requirements)),
    unreadableFiles,
    refusalMessages: [],
  };
}
