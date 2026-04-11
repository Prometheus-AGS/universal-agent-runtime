import type { CatalogResponse, ProvidersResponse } from "@/types";

export async function fetchCatalog(): Promise<CatalogResponse> {
  const res = await fetch("/api/catalog");
  if (!res.ok) throw new Error(`Catalog fetch failed: ${res.status}`);
  return res.json() as Promise<CatalogResponse>;
}

export async function fetchConfiguredProviders(): Promise<ProvidersResponse> {
  const res = await fetch("/api/uar/providers");
  if (!res.ok) throw new Error(`Providers fetch failed: ${res.status}`);
  return res.json() as Promise<ProvidersResponse>;
}

export interface CreateProviderBody {
  id: string;
  display_name: string;
  base_url: string;
  api_key?: string;
  protocol: string;
  enabled: boolean;
}

/** POST /api/uar/providers — 200/201 success; 409 treated as success by caller. */
export async function createProvider(body: CreateProviderBody): Promise<Response> {
  return fetch("/api/uar/providers", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function setDefaultProvider(providerId: string): Promise<void> {
  const res = await fetch(`/api/uar/providers/${providerId}/default`, { method: "POST" });
  if (!res.ok) throw new Error(`Set default failed: ${res.status}`);
}

export async function deleteProvider(providerId: string): Promise<void> {
  const res = await fetch(`/api/uar/providers/${providerId}`, { method: "DELETE" });
  if (!res.ok) throw new Error(`Delete failed: ${res.status}`);
}
