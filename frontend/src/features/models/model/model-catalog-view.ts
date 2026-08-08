import type { CatalogModelBenchmark } from "@/types";

export interface ModelRowShape {
  key: string;
  provider_id: string;
  provider_name: string;
  provider_configured: boolean;
  model_id: string;
  name: string;
  context: number;
  tool_call: boolean;
  reasoning: boolean;
  vision: boolean;
  open_weights?: boolean;
  cost_input: number;
  cost_output: number;
  benchmarks: CatalogModelBenchmark[];
}

export interface ModelCapabilityFilters {
  tools: boolean;
  reasoning: boolean;
  vision: boolean;
}

export function projectModelRows(
  rows: ReadonlyArray<Record<string, unknown>>,
): ModelRowShape[] {
  return rows.map((row) => {
    const providerId = typeof row.provider_id === "string" ? row.provider_id : "unknown";
    const modelId = typeof row.model_id === "string" ? row.model_id : "";
    const key = typeof row.id === "string" ? row.id : `${providerId}/${modelId}`;
    return {
      key,
      provider_id: providerId,
      provider_name:
        typeof row.provider_name === "string" ? row.provider_name : providerId,
      provider_configured: row.provider_configured === true,
      model_id: modelId,
      name: typeof row.name === "string" ? row.name : modelId || key,
      context: typeof row.context === "number" ? row.context : 0,
      tool_call: row.tool_call === true,
      reasoning: row.reasoning === true,
      vision: row.vision === true,
      open_weights: row.open_weights === true,
      cost_input: typeof row.cost_input === "number" ? row.cost_input : 0,
      cost_output: typeof row.cost_output === "number" ? row.cost_output : 0,
      benchmarks: Array.isArray(row.benchmarks)
        ? (row.benchmarks as CatalogModelBenchmark[])
        : [],
    };
  });
}

export function filterModelRows(
  rows: ModelRowShape[],
  providerId: string,
  capabilities: ModelCapabilityFilters,
  query: string,
): ModelRowShape[] {
  const normalizedQuery = query.trim().toLowerCase();
  return rows.filter((model) => {
    if (providerId !== "all" && model.provider_id !== providerId) return false;
    if (capabilities.tools && !model.tool_call) return false;
    if (capabilities.reasoning && !model.reasoning) return false;
    if (capabilities.vision && !model.vision) return false;
    if (
      normalizedQuery &&
      !model.key.toLowerCase().includes(normalizedQuery) &&
      !model.name.toLowerCase().includes(normalizedQuery)
    ) {
      return false;
    }
    return true;
  });
}

export function selectComparisonModels(
  keys: string[],
  rows: ModelRowShape[],
): ModelRowShape[] {
  return keys
    .map((key) => rows.find((model) => model.key === key))
    .filter((model): model is ModelRowShape => model !== undefined);
}
