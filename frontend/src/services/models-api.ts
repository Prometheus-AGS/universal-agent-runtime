import type { CatalogModelsResponse } from "@/types";

export async function fetchModelsCatalog(): Promise<CatalogModelsResponse> {
  const res = await fetch("/api/models");
  if (!res.ok) throw new Error(`Models fetch failed: ${res.status}`);
  return res.json() as Promise<CatalogModelsResponse>;
}

export interface ProviderModelRow {
  id: string;
  name: string;
  tool_call: boolean;
  reasoning: boolean;
  context: number;
}

/** Map /api/models payload to a flat list for one provider id. */
export function modelsRowsForProvider(
  data: CatalogModelsResponse,
  providerId: string,
): ProviderModelRow[] {
  const providerData = data[providerId];
  if (!providerData?.models) return [];
  return Object.entries(providerData.models).map(([id, m]) => ({
    id,
    name: m.name,
    tool_call: m.tool_call,
    reasoning: m.reasoning,
    context: m.limit.context,
  }));
}
