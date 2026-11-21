import { graphStore } from "@prometheus-ags/prometheus-entity-management";

import type { CatalogModelBenchmark } from "@/types";
import {
  PROVIDER_ENTITY,
  ModelCatalogIngestionError,
  catalogModelId,
  type CatalogModel,
} from "./contracts";

/** Provider display data. A REST catalog entry carries it; a realtime snapshot record does not. */
export interface CatalogProviderInfo {
  display_name?: string | null;
  configured?: boolean | null;
}

export interface ProjectCatalogModelOptions {
  /** Identity for a REST entry, whose provider and model ids are the keys of the payload. */
  providerId?: string;
  modelId?: string;
  /**
   * Provider display data. When omitted the projection reads the graph's `Provider` entity and
   * falls back to the provider id (name) and `false` (configured).
   */
  provider?: CatalogProviderInfo | null;
}

type RawRecord = Record<string, unknown>;

function asRecord(value: unknown): RawRecord | null {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as RawRecord)
    : null;
}

function nonEmptyString(value: unknown): string | null {
  return typeof value === "string" && value.length > 0 ? value : null;
}

function finiteNumber(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function stringList(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
}

function graphProvider(providerId: string): CatalogProviderInfo | null {
  const entity = graphStore.getState().entities[PROVIDER_ENTITY]?.[providerId];
  if (!entity) return null;
  return {
    display_name: typeof entity.display_name === "string" ? entity.display_name : null,
    configured: entity.configured === true,
  };
}

/**
 * The one projection for `Model` records. Accepts a REST catalog entry (nested `limit`, `cost`,
 * `modalities`; identity from `options`) or a realtime snapshot record (the same fields plus
 * `provider_id` and `model_id`) and returns the same flat record.
 *
 * Capability flags come from the entry: `vision` is true when `modalities.input` lists `image`.
 * A record without `provider_id` or `model_id` is rejected; the id is never guessed.
 *
 * @throws ModelCatalogIngestionError when the record is not an object or lacks its identity.
 */
export function projectCatalogModel(
  record: unknown,
  options: ProjectCatalogModelOptions = {},
): CatalogModel {
  const raw = asRecord(record);
  if (!raw) {
    throw new ModelCatalogIngestionError("invalid_record", "Catalog model record must be an object");
  }
  const providerId = nonEmptyString(raw.provider_id) ?? nonEmptyString(options.providerId);
  if (!providerId) {
    throw new ModelCatalogIngestionError("missing_provider_id", "Catalog model record has no provider_id");
  }
  const modelId = nonEmptyString(raw.model_id) ?? nonEmptyString(options.modelId);
  if (!modelId) {
    throw new ModelCatalogIngestionError(
      "missing_model_id",
      `Catalog model record for provider '${providerId}' has no model_id`,
    );
  }

  const limit = asRecord(raw.limit);
  const cost = asRecord(raw.cost);
  const modalities = asRecord(raw.modalities);
  const inputModalities = stringList(modalities?.input);
  const provider = options.provider ?? graphProvider(providerId);

  return {
    id: catalogModelId(providerId, modelId),
    provider_id: providerId,
    model_id: modelId,
    name: nonEmptyString(raw.name) ?? modelId,
    provider_name: nonEmptyString(provider?.display_name) ?? providerId,
    provider_configured: provider?.configured === true,
    context: finiteNumber(limit?.context, 0),
    tool_call: raw.tool_call === true,
    reasoning: raw.reasoning === true,
    vision: inputModalities.includes("image"),
    open_weights: raw.open_weights === true,
    cost_input: finiteNumber(cost?.input, 0),
    cost_output: finiteNumber(cost?.output, 0),
    modalities_input: inputModalities,
    modalities_output: stringList(modalities?.output),
    family: nonEmptyString(raw.family),
    benchmarks: Array.isArray(raw.benchmarks) ? (raw.benchmarks as CatalogModelBenchmark[]) : [],
  };
}

/**
 * Project a full `/api/models` payload (`provider_id -> { display_name, configured, models }`).
 * Rows are in payload order. A malformed provider block or record rejects the whole payload.
 */
export function projectCatalogPayload(payload: unknown): CatalogModel[] {
  const providers = asRecord(payload);
  if (!providers) {
    throw new ModelCatalogIngestionError("invalid_payload", "Models catalog payload must be an object");
  }
  const rows: CatalogModel[] = [];
  const seen = new Set<string>();
  for (const [providerId, block] of Object.entries(providers)) {
    const providerData = asRecord(block);
    const models = asRecord(providerData?.models);
    if (!providerData || !models) {
      throw new ModelCatalogIngestionError(
        "invalid_payload",
        `Models catalog entry for provider '${providerId}' has no models object`,
      );
    }
    const provider: CatalogProviderInfo = {
      display_name: typeof providerData.display_name === "string" ? providerData.display_name : null,
      configured: providerData.configured === true,
    };
    for (const [modelId, entry] of Object.entries(models)) {
      const row = projectCatalogModel(entry, { providerId, modelId, provider });
      if (seen.has(row.id)) continue;
      seen.add(row.id);
      rows.push(row);
    }
  }
  return rows;
}
