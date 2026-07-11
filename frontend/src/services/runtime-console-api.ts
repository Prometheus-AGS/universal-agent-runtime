import type { ProviderHealthResponse } from "@/services/providers-api";

async function getJson<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url} fetch failed: ${response.status}`);
  return response.json() as Promise<T>;
}

export function fetchRuntimeProviderHealth(): Promise<ProviderHealthResponse> {
  return getJson("/api/uar/providers/health");
}

export function fetchRuntimeA2uiSchemas(): Promise<Array<Record<string, unknown>>> {
  return getJson("/api/uar/a2ui/schemas");
}

export interface ResolvedRuntimeModel {
  ok?: boolean;
  provider_id?: string;
  model_id?: string;
}

export function fetchRuntimeModelRoute(): Promise<ResolvedRuntimeModel> {
  return getJson("/api/uar/resolve-model");
}

export async function resolveRuntimeApproval(runId: string, approved: boolean): Promise<void> {
  const response = await fetch(`/api/uar/runs/${encodeURIComponent(runId)}/approval`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ approved }),
  });
  if (!response.ok) throw new Error(`Approval resolution failed: ${response.status}`);
}
