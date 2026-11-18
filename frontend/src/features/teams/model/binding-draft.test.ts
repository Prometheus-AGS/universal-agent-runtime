import { describe, expect, test } from "vitest";

import {
  effectiveBudgetOf,
  initialDraft,
  missingItems,
  modelBindingsOf,
  numberProblems,
  type BindingDraft,
} from "./binding-draft";

const REQUIREMENT = {
  requirementKey: "fast|smart",
  agentDefinitionId: "urn:agent:worker",
  role: "worker",
  capabilities: [],
  preferredAliases: ["smart", "fast"],
  alsoUsedBy: [],
};

describe("binding draft", () => {
  test("prefills limits and budget from the definition but never the references", () => {
    const draft = initialDraft({ maxMembers: 4 }, { maxCostMicrounits: 2_500_000, maxTokens: 1000 });
    expect(draft.numbers.maxMembers).toBe("4");
    expect(draft.numbers.maxCostUsd).toBe("2.5");
    expect([draft.credentialRef, draft.connectionRef, draft.policyRevision]).toEqual(["", "", ""]);
  });

  test("names every missing item and clears them when supplied", () => {
    const draft = initialDraft(undefined, undefined);
    expect(missingItems(draft, [REQUIREMENT])).toEqual([
      'a provider and model for the alias "smart" (worker model)',
      "a credential reference",
      "a storage connection reference",
      "a policy revision",
    ]);
    const full: BindingDraft = {
      ...draft,
      choices: { [REQUIREMENT.requirementKey]: { alias: null, route: { value: "p/m", providerId: "p", modelId: "m" } } },
      credentialRef: "protected-credential://x",
      connectionRef: "protected-connection://y",
      policyRevision: "rev-1",
    };
    expect(missingItems(full, [REQUIREMENT])).toEqual([]);
    expect(modelBindingsOf(full, [REQUIREMENT])).toEqual([
      { requestedAlias: "smart", providerId: "p", modelId: "m", credentialRef: "protected-credential://x" },
    ]);
  });

  test("blocks one alias chosen with two different models and merges identical choices", () => {
    const other = { ...REQUIREMENT, requirementKey: "smart|slow", agentDefinitionId: "urn:agent:reviewer", role: "reviewer", preferredAliases: ["smart", "slow"] };
    const route = (value: string) => ({ alias: null, route: { value, providerId: value.split("/")[0], modelId: value.split("/")[1] } });
    const base: BindingDraft = {
      ...initialDraft(undefined, undefined),
      credentialRef: "protected-credential://x",
      connectionRef: "protected-connection://y",
      policyRevision: "rev-1",
    };
    const differing: BindingDraft = { ...base, choices: { [REQUIREMENT.requirementKey]: route("p/m"), [other.requirementKey]: route("q/n") } };
    const items = missingItems(differing, [REQUIREMENT, other]);
    expect(items).toHaveLength(1);
    expect(items[0]).toContain('"smart"');
    const agreeing: BindingDraft = { ...base, choices: { [REQUIREMENT.requirementKey]: route("p/m"), [other.requirementKey]: route("p/m") } };
    expect(missingItems(agreeing, [REQUIREMENT, other])).toEqual([]);
    expect(modelBindingsOf(agreeing, [REQUIREMENT, other])).toHaveLength(1);
  });

  test("converts the cost entry to microunits and reports invalid numbers", () => {
    const draft = initialDraft(undefined, undefined);
    expect(effectiveBudgetOf({ ...draft.numbers, maxCostUsd: "1.25", maxTokens: "10" })).toEqual({ maxTokens: 10, maxCostMicrounits: 1_250_000 });
    expect(numberProblems({ ...draft.numbers, maxTokens: "ten" }).maxTokens).toBeDefined();
  });
});
