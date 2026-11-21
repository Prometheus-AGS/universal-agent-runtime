import { modelCatalogTransport } from "@/platform/entities";
import type { CatalogModelsResponse, ProviderModelRow } from "@/types";

/**
 * The catalog transport moved to the model-catalog platform domain; this re-export remains for
 * callers that have not migrated to `loadModelCatalog` from `@/platform/entities`.
 */
export const fetchModelsCatalog = modelCatalogTransport.fetchModelsCatalog;

export interface ResolveModelResult {
  ok: boolean;
  provider_id?: string;
  model_id?: string;
  error?: string;
}

/** Check whether a default model is configured and resolvable. */
export async function fetchResolveModel(): Promise<ResolveModelResult> {
  const res = await fetch("/api/uar/resolve-model");
  const body = await res.json() as ResolveModelResult;
  return { ...body, ok: res.ok };
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
