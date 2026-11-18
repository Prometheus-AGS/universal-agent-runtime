import type { ModelBindingDocument, ModelRequirement, TeamBudget, TeamLimits } from "@/platform/entities";
import { dollarsToMicrounits, microunitsToDollars } from "./format";

/** The operator's unsaved binding entries. UI-local mechanics, not a business record. */

export const EXAMPLE_CREDENTIAL_REF = "protected-credential://uar/local-primary";
export const EXAMPLE_CONNECTION_REF = "protected-connection://uar/catalog";
export const EXAMPLE_POLICY_REVISION = "console-policy-v1";

export interface ModelRoute {
  /** `provider/model`, the model selector's value. */
  value: string;
  providerId: string;
  modelId: string;
}

export interface ModelChoice {
  /** Null until the operator picks among several preferred aliases; the first alias applies meanwhile. */
  alias: string | null;
  route: ModelRoute | null;
}

export interface NumberFields {
  concurrentTurns: string;
  maxMembers: string;
  maxDepth: string;
  maxPendingTasks: string;
  maxTokens: string;
  maxCostUsd: string;
  maxElapsedSeconds: string;
}

export interface BindingDraft {
  choices: Record<string, ModelChoice>;
  numbers: NumberFields;
  credentialRef: string;
  connectionRef: string;
  policyRevision: string;
}

const text = (value: number | undefined) => (value === undefined ? "" : String(value));

/** Limits and budget are prefilled from the team definition; the three references start empty on purpose. */
export function initialDraft(limits: TeamLimits | undefined, budget: TeamBudget | undefined): BindingDraft {
  return {
    choices: {},
    numbers: {
      concurrentTurns: text(limits?.concurrentTurns),
      maxMembers: text(limits?.maxMembers),
      maxDepth: text(limits?.maxDepth),
      maxPendingTasks: text(limits?.maxPendingTasks),
      maxTokens: text(budget?.maxTokens),
      maxCostUsd: budget?.maxCostMicrounits === undefined ? "" : String(microunitsToDollars(budget.maxCostMicrounits)),
      maxElapsedSeconds: text(budget?.maxElapsedSeconds),
    },
    credentialRef: "",
    connectionRef: "",
    policyRevision: "",
  };
}

export function aliasFor(requirement: ModelRequirement, choice: ModelChoice | undefined): string | null {
  return choice?.alias ?? requirement.preferredAliases[0] ?? null;
}

const INTEGER = /^\d+$/;
const DECIMAL = /^\d+(\.\d+)?$/;

type NumberKind = "integer" | "decimal";
const KINDS: Record<keyof NumberFields, NumberKind> = {
  concurrentTurns: "integer",
  maxMembers: "integer",
  maxDepth: "integer",
  maxPendingTasks: "integer",
  maxTokens: "integer",
  maxCostUsd: "decimal",
  maxElapsedSeconds: "integer",
};

/** An empty entry means "not set" and is omitted from the binding. */
export function parseNumber(input: string, kind: NumberKind): number | null | "invalid" {
  const value = input.trim();
  if (!value) return null;
  if (!(kind === "integer" ? INTEGER : DECIMAL).test(value)) return "invalid";
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : "invalid";
}

export function numberProblems(numbers: NumberFields): Partial<Record<keyof NumberFields, string>> {
  const problems: Partial<Record<keyof NumberFields, string>> = {};
  for (const key of Object.keys(KINDS) as Array<keyof NumberFields>) {
    if (parseNumber(numbers[key], KINDS[key]) === "invalid") {
      problems[key] = KINDS[key] === "integer" ? "Enter a whole number, or leave it empty." : "Enter an amount such as 5 or 12.5, or leave it empty.";
    }
  }
  return problems;
}

function present<T extends object>(entries: Array<[keyof T, number | null | "invalid"]>): Partial<T> {
  const result: Partial<T> = {};
  for (const [key, value] of entries) if (typeof value === "number") result[key] = value as T[keyof T];
  return result;
}

export function effectiveLimitsOf(numbers: NumberFields): TeamLimits {
  return present<TeamLimits>([
    ["concurrentTurns", parseNumber(numbers.concurrentTurns, "integer")],
    ["maxMembers", parseNumber(numbers.maxMembers, "integer")],
    ["maxDepth", parseNumber(numbers.maxDepth, "integer")],
    ["maxPendingTasks", parseNumber(numbers.maxPendingTasks, "integer")],
  ]);
}

/** Currency is not set here: the binding builder forces USD, the only currency admission accepts. */
export function effectiveBudgetOf(numbers: NumberFields): Omit<TeamBudget, "currency"> {
  const cost = parseNumber(numbers.maxCostUsd, "decimal");
  return present<TeamBudget>([
    ["maxTokens", parseNumber(numbers.maxTokens, "integer")],
    ["maxCostMicrounits", typeof cost === "number" ? dollarsToMicrounits(cost) : cost],
    ["maxElapsedSeconds", parseNumber(numbers.maxElapsedSeconds, "integer")],
  ]);
}

/**
 * Aliases that two requirements resolve to with different models. The runtime resolves an alias by its
 * first match, so the later requirement would silently run on the earlier one's model.
 */
function conflictingAliases(draft: BindingDraft, requirements: readonly ModelRequirement[]): string[] {
  const routes = new Map<string, Set<string>>();
  for (const requirement of requirements) {
    const choice = draft.choices[requirement.requirementKey];
    const alias = aliasFor(requirement, choice);
    if (!alias || !choice?.route) continue;
    const seen = routes.get(alias) ?? new Set<string>();
    seen.add(choice.route.value);
    routes.set(alias, seen);
  }
  return [...routes.entries()].filter(([, seen]) => seen.size > 1).map(([alias]) => alias);
}

/** What is still needed before the binding can be checked, worded for the reason beside the action. */
export function missingItems(draft: BindingDraft, requirements: readonly ModelRequirement[]): string[] {
  const missing: string[] = [];
  for (const requirement of requirements) {
    const choice = draft.choices[requirement.requirementKey];
    if (!choice?.route) missing.push(`a provider and model for the alias "${aliasFor(requirement, choice)}" (${requirement.role} model)`);
  }
  for (const alias of conflictingAliases(draft, requirements)) {
    missing.push(`one model for the alias "${alias}" (two agents use it but have different models chosen)`);
  }
  if (!draft.credentialRef.trim()) missing.push("a credential reference");
  if (!draft.connectionRef.trim()) missing.push("a storage connection reference");
  if (!draft.policyRevision.trim()) missing.push("a policy revision");
  if (Object.keys(numberProblems(draft.numbers)).length > 0) missing.push("valid limits and budget");
  return missing;
}

export function modelBindingsOf(draft: BindingDraft, requirements: readonly ModelRequirement[]): ModelBindingDocument[] {
  const emitted = new Set<string>();
  return requirements.flatMap((requirement) => {
    const choice = draft.choices[requirement.requirementKey];
    const alias = aliasFor(requirement, choice);
    if (!alias || !choice?.route) return [];
    const identity = `${alias}\u0000${choice.route.value}`;
    if (emitted.has(identity)) return []; // Two requirements that agree on an alias and model need one entry.
    emitted.add(identity);
    return [{
      requestedAlias: alias,
      providerId: choice.route.providerId,
      modelId: choice.route.modelId,
      credentialRef: draft.credentialRef.trim(),
    }];
  });
}
