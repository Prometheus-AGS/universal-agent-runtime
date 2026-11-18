import { preconditionError } from "./api/teams-errors";
import {
  BINDING_CAPABILITY,
  COLLABORATION_PROFILE,
  type DeploymentBindingDocument,
  type ImmutableDefinitionRef,
  type ModelBindingDocument,
  type TeamBudget,
  type TeamLimits,
} from "./contracts";
import { withContentDigest } from "./digest";

export interface BuildBindingInput {
  package: ImmutableDefinitionRef;
  /** `bindingOwnerId` from the runtime capabilities. */
  ownerId: string;
  workspaceId: string;
  /** `instance.id` from the runtime capabilities; the runtime refuses an empty value. */
  runtimeInstanceId: string;
  /** Capability names the runtime reports; every `requiredCapabilities` entry must be among them. */
  runtimeCapabilities: readonly string[];
  /** One entry per agent model requirement, `requestedAlias` being one of its preferred aliases. */
  modelBindings: readonly ModelBindingDocument[];
  effectiveLimits: TeamLimits;
  effectiveBudget: Omit<TeamBudget, "currency">;
  storage: { backend: string; connectionRef: string };
  policyRevision: string;
  /** Reuse an id to publish its next revision; omit for a new binding. */
  bindingId?: string;
  /** Revision of the existing binding with this id, or null/absent for a new one. */
  existingRevision?: number | null;
  version?: string;
  requiredCapabilities?: readonly string[];
  provenance?: { source: string; authors: string[] };
}

export interface BuiltBinding {
  document: DeploymentBindingDocument;
  /** Send as `expectedRevision`: the existing revision, 0 for a new binding. */
  expectedRevision: number;
}

function ensure(condition: boolean, code: string, message: string): void {
  if (!condition) throw preconditionError(code, message);
}

/**
 * Assemble the complete `DeploymentBinding` document and its `contentDigest`.
 * The console generates every field the operator is not asked about; the
 * operator-owned references (`credentialRef`, `storage.connectionRef`,
 * `policyRevision`) are required inputs and are never invented here.
 * `effectiveBudget.currency` is forced to "USD": admission refuses any other.
 */
export async function buildDeploymentBinding(input: BuildBindingInput): Promise<BuiltBinding> {
  ensure(input.runtimeInstanceId.trim().length > 0, "binding_instance_missing", "The runtime has no instance id, so a binding cannot be created.");
  ensure(input.ownerId.trim().length > 0, "binding_owner_missing", "The binding owner is not known yet.");
  ensure(input.workspaceId.trim().length > 0, "binding_workspace_missing", "Choose a workspace for the binding.");
  ensure(input.modelBindings.length > 0, "binding_models_missing", "Bind a provider and model for every requested model alias.");
  for (const model of input.modelBindings) {
    ensure(
      Boolean(model.requestedAlias && model.providerId && model.modelId && model.credentialRef),
      "binding_model_incomplete",
      `Complete the provider, model and credential reference for alias "${model.requestedAlias}".`,
    );
  }
  ensure(input.storage.backend.trim().length > 0 && input.storage.connectionRef.trim().length > 0, "binding_storage_incomplete", "Storage backend and connection reference are required.");
  ensure(input.policyRevision.trim().length > 0, "binding_policy_missing", "A policy revision is required.");

  const requiredCapabilities = [...(input.requiredCapabilities ?? [BINDING_CAPABILITY])];
  const unsupported = requiredCapabilities.filter((name) => !input.runtimeCapabilities.includes(name));
  ensure(unsupported.length === 0, "binding_capability_unsupported", `The runtime does not report: ${unsupported.join(", ")}.`);

  const existingRevision = input.existingRevision ?? null;
  const unsigned = {
    profile: COLLABORATION_PROFILE,
    kind: "DeploymentBinding",
    id: input.bindingId ?? `urn:uar:console:binding:${crypto.randomUUID()}`,
    version: input.version ?? "1.0.0",
    provenance: input.provenance ?? { source: "Created in the UAR operations console", authors: ["UAR operator"] },
    requiredCapabilities,
    extensions: {},
    exportClass: "private-installed-state",
    package: input.package,
    ownerId: input.ownerId,
    workspaceId: input.workspaceId,
    runtimeInstanceId: input.runtimeInstanceId,
    revision: (existingRevision ?? 0) + 1,
    modelBindings: input.modelBindings.map((model) => ({ ...model })),
    skillBindings: [],
    storage: { backend: input.storage.backend, connectionRef: input.storage.connectionRef, durableTransactions: true },
    policyRevision: input.policyRevision,
    effectiveLimits: { ...input.effectiveLimits },
    effectiveBudget: { ...input.effectiveBudget, currency: "USD" },
    contextGrants: [],
    representationGrantRefs: [],
    status: "ready",
    effectiveBindingReceiptRef: null,
  } as const;
  const document = (await withContentDigest(unsigned)) as unknown as DeploymentBindingDocument;
  return { document, expectedRevision: existingRevision ?? 0 };
}
