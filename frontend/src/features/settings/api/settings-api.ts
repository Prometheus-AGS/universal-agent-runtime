import type { SettingWithMeta } from "@/types";

const BASE = "/api/uar/settings";
export const GOVERNANCE_REQUEST_TIMEOUT_MS = 10_000;

export type GovernanceRuntimePhase = "initializing" | "on" | "off";
export type GovernanceEffectiveState =
  | "unknown"
  | "required"
  | "on"
  | "off";
export type GovernanceStatusReason =
  | "initialization_incomplete"
  | "configured_host_not_allowed"
  | "authentication_unverified"
  | "jwt_required"
  | "ingress_inventory_unsealed"
  | "ingress_proof_missing"
  | "bound_ingress_not_loopback"
  | "persistence_unavailable";

export interface GovernanceRuntimeStatus {
  boot_instance_id: string;
  revision: number;
  phase: GovernanceRuntimePhase;
  effective_state: GovernanceEffectiveState;
  effective_enabled: boolean;
  may_disable: boolean;
  mutation_available: boolean;
  configured_host: string;
  bound_addresses: string[];
  jwt_required: boolean | null;
  reasons: GovernanceStatusReason[];
}

export type GovernanceMutationStatus =
  | "updated"
  | "validation_rejected"
  | "dependency_failed"
  | "skipped";

export interface GovernanceMutationResult {
  key: string;
  status: GovernanceMutationStatus;
  error?: string;
}

export interface GovernanceAppliedStatusToken {
  boot_instance_id: string;
  revision: number;
}

export type GovernanceSaveOutcome =
  | "confirmed"
  | "partial"
  | "changed_elsewhere"
  | "rejected"
  | "unknown";

function settingsAdminHeaders(): Record<string, string> {
  const adminKey = import.meta.env.VITE_UAR_ADMIN_KEY ?? "";
  return adminKey ? { "X-UAR-Admin-Key": adminKey } : {};
}

/** Convert a namespace key (e.g. "context_management") to its URL slug (e.g. "context-management") */
export function namespaceToSlug(ns: string): string {
  const overrides: Record<string, string> = {
    provider: "providers",
    file_processing: "file-processing",
    knowledge_bases: "knowledge-bases",
    intent_classifier: "intent-classifier",
    context_management: "context-management",
    agent_config: "agent-config",
    skill_config: "skill-config",
    mistral_ocr: "mistral-ocr",
  };
  return overrides[ns] ?? ns.replace(/_/g, "-");
}

export async function fetchSettingsNamespace(
  namespace: string,
): Promise<SettingWithMeta[]> {
  const slug = namespaceToSlug(namespace);
  const res = await fetch(`${BASE}/${slug}`, {
    headers: settingsAdminHeaders(),
  });
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json() as Promise<SettingWithMeta[]>;
}

export async function putSettingValue(
  key: string,
  value: unknown,
): Promise<void> {
  const res = await fetch(`${BASE}/${encodeURIComponent(key)}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      ...settingsAdminHeaders(),
    },
    body: JSON.stringify({ value }),
  });
  if (!res.ok) throw new Error(`Save ${key} failed: ${res.status}`);
}

export interface BulkSettingsUpdateResponse {
  status: "updated" | "partial";
  updated?: SettingWithMeta[];
  errors?: Array<{ key: string; error: string }>;
  results?: GovernanceMutationResult[];
  applied_status?: GovernanceAppliedStatusToken;
  governance_status?: GovernanceRuntimeStatus;
  governance_outcome?: GovernanceSaveOutcome;
  observed_governance_status?: GovernanceRuntimeStatus;
  retained_draft_keys?: string[];
}

const GOVERNANCE_PHASES = new Set<GovernanceRuntimePhase>([
  "initializing",
  "on",
  "off",
]);
const GOVERNANCE_EFFECTIVE_STATES = new Set<GovernanceEffectiveState>([
  "unknown",
  "required",
  "on",
  "off",
]);
const GOVERNANCE_REASON_CODES = new Set<GovernanceStatusReason>([
  "initialization_incomplete",
  "configured_host_not_allowed",
  "authentication_unverified",
  "jwt_required",
  "ingress_inventory_unsealed",
  "ingress_proof_missing",
  "bound_ingress_not_loopback",
  "persistence_unavailable",
]);
const GOVERNANCE_MANDATORY_REASON_CODES = new Set<GovernanceStatusReason>([
  "configured_host_not_allowed",
  "authentication_unverified",
  "jwt_required",
  "ingress_inventory_unsealed",
  "ingress_proof_missing",
  "bound_ingress_not_loopback",
]);
const GOVERNANCE_MUTATION_STATUSES = new Set<GovernanceMutationStatus>([
  "updated",
  "validation_rejected",
  "dependency_failed",
  "skipped",
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function parseGovernanceRuntimeStatus(
  value: unknown,
): GovernanceRuntimeStatus {
  if (!isRecord(value)) throw new Error("Malformed governance status projection");
  const phase = value.phase;
  const effectiveState = value.effective_state;
  const reasons = value.reasons;
  const boundAddresses = value.bound_addresses;
  if (
    typeof value.boot_instance_id !== "string" ||
    typeof value.revision !== "number" ||
    !Number.isSafeInteger(value.revision) ||
    typeof phase !== "string" ||
    !GOVERNANCE_PHASES.has(phase as GovernanceRuntimePhase) ||
    typeof effectiveState !== "string" ||
    !GOVERNANCE_EFFECTIVE_STATES.has(
      effectiveState as GovernanceEffectiveState,
    ) ||
    typeof value.effective_enabled !== "boolean" ||
    typeof value.may_disable !== "boolean" ||
    typeof value.mutation_available !== "boolean" ||
    typeof value.configured_host !== "string" ||
    !Array.isArray(boundAddresses) ||
    !boundAddresses.every((address) => typeof address === "string") ||
    !(
      typeof value.jwt_required === "boolean" || value.jwt_required === null
    ) ||
    !Array.isArray(reasons) ||
    !reasons.every(
      (reason) =>
        typeof reason === "string" &&
        GOVERNANCE_REASON_CODES.has(reason as GovernanceStatusReason),
    )
  ) {
    throw new Error("Malformed governance status projection");
  }

  const status = value as unknown as GovernanceRuntimeStatus;
  const persistenceUnavailable = status.reasons.includes(
    "persistence_unavailable",
  );
  const hasMandatoryReason = status.reasons.some((reason) =>
    GOVERNANCE_MANDATORY_REASON_CODES.has(reason),
  );
  const coherent =
    (effectiveState === "unknown" &&
      phase === "initializing" &&
      status.effective_enabled &&
      !status.mutation_available) ||
    (effectiveState === "required" &&
      phase === "on" &&
      status.effective_enabled &&
      !status.may_disable &&
      hasMandatoryReason &&
      !status.reasons.includes("initialization_incomplete") &&
      status.mutation_available !== persistenceUnavailable) ||
    (effectiveState === "on" &&
      phase === "on" &&
      status.effective_enabled &&
      status.may_disable &&
      ((status.mutation_available && status.reasons.length === 0) ||
        (!status.mutation_available &&
          status.reasons.length === 1 &&
          status.reasons[0] === "persistence_unavailable"))) ||
    (effectiveState === "off" &&
      phase === "off" &&
      !status.effective_enabled &&
      status.may_disable &&
      status.mutation_available &&
      status.reasons.length === 0);
  if (!coherent) throw new Error("Malformed governance status projection");
  return status;
}

function parseGovernanceMutationResponse(
  value: unknown,
  submittedFields: string[],
): BulkSettingsUpdateResponse {
  if (!isRecord(value)) {
    throw new Error("Malformed governance mutation response");
  }
  const results = value.results;
  const token = value.applied_status;
  if (
    (value.status !== "updated" && value.status !== "partial") ||
    !Array.isArray(results) ||
    !results.every(
      (result) =>
        isRecord(result) &&
        typeof result.key === "string" &&
        typeof result.status === "string" &&
        GOVERNANCE_MUTATION_STATUSES.has(
          result.status as GovernanceMutationStatus,
        ) &&
        (result.error === undefined || typeof result.error === "string"),
    ) ||
    !isRecord(token) ||
    typeof token.boot_instance_id !== "string" ||
    typeof token.revision !== "number" ||
    !Number.isSafeInteger(token.revision)
  ) {
    throw new Error("Malformed governance mutation response");
  }

  const governanceStatus = parseGovernanceRuntimeStatus(
    value.governance_status,
  );
  const expectedKeys = new Set(
    submittedFields.map((field) =>
      field.startsWith("governance.") ? field : `governance.${field}`,
    ),
  );
  const returnedKeys = results.map((result) => result.key as string);
  const allUpdated = results.every((result) => result.status === "updated");
  if (
    token.boot_instance_id !== governanceStatus.boot_instance_id ||
    token.revision !== governanceStatus.revision ||
    returnedKeys.length !== expectedKeys.size ||
    new Set(returnedKeys).size !== returnedKeys.length ||
    returnedKeys.some((key) => !expectedKeys.has(key)) ||
    (value.status === "updated") !== allUpdated
  ) {
    throw new Error("Malformed governance mutation response");
  }

  return {
    status: value.status,
    results: results as GovernanceMutationResult[],
    applied_status: token as unknown as GovernanceAppliedStatusToken,
    governance_status: governanceStatus,
  };
}

async function fetchWithGovernanceDeadline(
  input: RequestInfo | URL,
  init?: RequestInit,
): Promise<Response> {
  const controller = new AbortController();
  const timeout = window.setTimeout(
    () => controller.abort(),
    GOVERNANCE_REQUEST_TIMEOUT_MS,
  );
  try {
    return await fetch(input, { ...init, signal: controller.signal });
  } catch (error) {
    if (controller.signal.aborted) {
      throw new Error("Governance request timed out after 10 seconds", {
        cause: error,
      });
    }
    throw error;
  } finally {
    window.clearTimeout(timeout);
  }
}

export async function fetchGovernanceStatus(): Promise<GovernanceRuntimeStatus> {
  const res = await fetchWithGovernanceDeadline(`${BASE}/governance/status`, {
    headers: settingsAdminHeaders(),
  });
  if (!res.ok) throw new Error(`Governance status unavailable: ${res.status}`);
  return parseGovernanceRuntimeStatus(await res.json());
}

export async function putSettingsNamespace(
  namespace: string,
  data: Record<string, unknown>,
): Promise<BulkSettingsUpdateResponse> {
  const slug = namespaceToSlug(namespace);
  const request =
    namespace === "governance" ? fetchWithGovernanceDeadline : fetch;
  const res = await request(`${BASE}/${slug}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      ...settingsAdminHeaders(),
    },
    body: JSON.stringify({ data }),
  });
  if (!res.ok) throw new Error(`Save ${namespace} failed: ${res.status}`);
  const response: unknown = await res.json();
  return namespace === "governance"
    ? parseGovernanceMutationResponse(response, Object.keys(data))
    : (response as BulkSettingsUpdateResponse);
}

export async function fetchSettingTypes(): Promise<unknown> {
  const res = await fetch(`${BASE}/types`, { headers: settingsAdminHeaders() });
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json();
}

export async function fetchResilienceSettings(): Promise<unknown> {
  const res = await fetch(`${BASE}/resilience`, {
    headers: settingsAdminHeaders(),
  });
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json();
}
