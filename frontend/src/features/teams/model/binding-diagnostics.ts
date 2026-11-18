import type { ConversionDiagnostic, ConversionDisposition } from "@/platform/entities";

/**
 * Routing preflight diagnostics to the place they belong. `field` is an RFC 6901 pointer.
 * Model diagnostics (`/models/N`) point into the AGENT definition, not the binding, and N counts per agent,
 * so they cannot be matched to one binding field: they form a group shown above the model step.
 * `/modelBindings/N/...` indexes the binding's own list, which the console builds in requirement order.
 */

export interface DiagnosticBuckets {
  models: ConversionDiagnostic[];
  byRequirement: Record<number, ConversionDiagnostic[]>;
  credential: ConversionDiagnostic[];
  connection: ConversionDiagnostic[];
  policy: ConversionDiagnostic[];
  limits: ConversionDiagnostic[];
  general: ConversionDiagnostic[];
}

const ORDER: Record<ConversionDisposition, number> = {
  "required-unsupported": 0,
  "optional-unsupported": 1,
  translated: 2,
  exact: 3,
};

export const isBlocking = (diagnostic: ConversionDiagnostic) => diagnostic.disposition === "required-unsupported";

export function sortBlockingFirst(diagnostics: readonly ConversionDiagnostic[]): ConversionDiagnostic[] {
  return [...diagnostics].sort((a, b) => ORDER[a.disposition] - ORDER[b.disposition]);
}

export function emptyBuckets(): DiagnosticBuckets {
  return { models: [], byRequirement: {}, credential: [], connection: [], policy: [], limits: [], general: [] };
}

function segments(pointer: string): string[] {
  return pointer.split("/").slice(1).map((part) => part.replaceAll("~1", "/").replaceAll("~0", "~"));
}

export function bucketDiagnostics(diagnostics: readonly ConversionDiagnostic[]): DiagnosticBuckets {
  const buckets = emptyBuckets();
  for (const diagnostic of sortBlockingFirst(diagnostics)) {
    const [head, second, third] = segments(diagnostic.field);
    if (head === "models") buckets.models.push(diagnostic);
    else if (head === "modelBindings" && third === "credentialRef") buckets.credential.push(diagnostic);
    else if (head === "modelBindings" && second !== undefined && /^\d+$/.test(second)) {
      const index = Number(second);
      (buckets.byRequirement[index] ??= []).push(diagnostic);
    } else if (head === "storage" && second === "connectionRef") buckets.connection.push(diagnostic);
    else if (head === "policyRevision") buckets.policy.push(diagnostic);
    else if (head === "effectiveLimits" || head === "effectiveBudget") buckets.limits.push(diagnostic);
    else buckets.general.push(diagnostic);
  }
  return buckets;
}

export function hasBlocking(diagnostics: readonly ConversionDiagnostic[]): boolean {
  return diagnostics.some(isBlocking);
}

export const DISPOSITION_LABEL: Record<ConversionDisposition, string> = {
  "required-unsupported": "Blocks activation",
  "optional-unsupported": "Optional, not supported",
  translated: "Translated",
  exact: "Exact",
};
