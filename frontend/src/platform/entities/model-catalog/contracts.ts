import { serializeKey } from "@prometheus-ags/prometheus-entity-management";

import type { CatalogModelBenchmark, UarProvider } from "@/types";

/** Graph entity types owned by the model-catalog domain. */
export const PROVIDER_ENTITY = "Provider" as const;
export const PROVIDER_META_ENTITY = "ProviderMeta" as const;
export const CATALOG_MODEL_ENTITY = "Model" as const;
export const MODEL_CATALOG_LOAD_STATE_ENTITY = "ModelCatalogLoadState" as const;
/** Owned by the runtime console; the domain writes it only as one batch (see `ingestProviderHealth`). */
export const RUNTIME_PROVIDER_HEALTH_ENTITY = "RuntimeProviderHealth" as const;

/** The `ProviderMeta` singleton id. */
export const PROVIDER_META_ID = "current" as const;

/** Load-state singleton ids. */
export const MODEL_CATALOG_LOAD_ID = "models" as const;
export const PROVIDER_REGISTRY_LOAD_ID = "providers" as const;
export type ModelCatalogLoadId = typeof MODEL_CATALOG_LOAD_ID | typeof PROVIDER_REGISTRY_LOAD_ID;

/**
 * The one identity of a catalog model: `provider/model`.
 *
 * The id is an opaque key. Model ids may themselves contain `/`, so an entity id must never be
 * split to recover its parts; read `provider_id` and `model_id` from the record instead.
 */
export function catalogModelId(providerId: string, modelId: string): string {
  return `${providerId}/${modelId}`;
}

/**
 * One `Model` graph record. Every writer (REST hydration and realtime snapshot) produces exactly
 * this shape through `projectCatalogModel`; capability flags are derived from the catalog entry.
 */
export interface CatalogModel extends Record<string, unknown> {
  /** `catalogModelId(provider_id, model_id)`. */
  id: string;
  provider_id: string;
  model_id: string;
  name: string;
  /** Provider display name; the provider id when the provider is not known. */
  provider_name: string;
  provider_configured: boolean;
  context: number;
  tool_call: boolean;
  reasoning: boolean;
  /** True when the entry lists `image` among its input modalities. */
  vision: boolean;
  open_weights: boolean;
  cost_input: number;
  cost_output: number;
  modalities_input: string[];
  modalities_output: string[];
  family: string | null;
  benchmarks: CatalogModelBenchmark[];
}

/** One `Provider` graph record (catalog provider merged with the configured registry). */
export interface Provider extends Record<string, unknown> {
  id: string;
  display_name: string;
  base_url?: string;
  configured: boolean;
  auth_env_var?: string;
  endpoints: string[];
  model_count: number;
  /** Mirrors `CatalogProviderSummary.status` so the Provider page can render
   *  credential-blocked badges without a separate fetch. */
  status?: "configured" | "credential-blocked" | "available";
  status_detail?: string;
}

/**
 * Singleton holding the system default pair: the default provider id and that provider's
 * `default_model`. Always published as one write, so the pair is never half-populated.
 */
export interface ProviderMeta extends Record<string, unknown> {
  id: typeof PROVIDER_META_ID;
  default_id: string | null;
  default_model: string | null;
}

export type ProviderMetaField = "default_id" | "default_model";

/** Outcome of the last load of the models catalog or the provider registry. */
export interface ModelCatalogLoadState extends Record<string, unknown> {
  id: ModelCatalogLoadId;
  /** Message of the last failed load; null after a successful load. A failed load keeps the previous entities. */
  error: string | null;
  loaded_at: string | null;
}

export type { ProviderHealthEntry, ProviderHealthResponse } from "./api/model-catalog-api";

/** Graph list key holding the ordered provider ids (the key `useProviders` reads). */
export const PROVIDER_LIST_KEY = serializeKey(["providers", "", "[]"]);
/** Graph list key holding every catalog model id. */
export const CATALOG_MODELS_LIST_KEY = serializeKey(["model-catalog", "models"]);
/** Graph list key holding the model ids of one provider (the key `Provider.models` cascades on). */
export function providerModelsListKey(providerId: string): string {
  return serializeKey([CATALOG_MODEL_ENTITY, { provider_id: providerId }]);
}

export type ModelCatalogIngestionErrorCode =
  | "invalid_payload"
  | "invalid_record"
  | "missing_provider_id"
  | "missing_model_id";

/** A catalog record the projection rejects. Ingestion stops and the previous entities stay. */
export class ModelCatalogIngestionError extends Error {
  readonly code: ModelCatalogIngestionErrorCode;

  constructor(code: ModelCatalogIngestionErrorCode, message: string) {
    super(message);
    this.name = "ModelCatalogIngestionError";
    this.code = code;
  }
}

export type SystemDefaultModelErrorKind =
  /** Provider id or model id is empty. Nothing was sent. */
  | "invalid_selection"
  /** The provider could not be read. Nothing was written. */
  | "provider_fetch_failed"
  /** 404: the provider no longer exists. */
  | "provider_not_found"
  /** The provider is disabled (route 409, or detected before any write). */
  | "provider_disabled"
  /** 409 `TEAM_REVISION_CONFLICT` on the model write: the provider changed under the console. */
  | "revision_conflict"
  /** The model write failed for another reason. No default call was issued. */
  | "model_save_failed"
  /** The default route failed for another reason. */
  | "default_route_failed";

export type SystemDefaultModelStage = "fetch_provider" | "save_model" | "set_default";

export interface SystemDefaultModelErrorInit {
  kind: SystemDefaultModelErrorKind;
  stage: SystemDefaultModelStage;
  providerId: string;
  modelId: string | null;
  /** HTTP status of the failing call; null when none was received. */
  status: number | null;
  /** The server's `error` text, verbatim. */
  detail: string | null;
  /** The provider's `default_model` write succeeded before the failure. */
  modelSaved: boolean;
  /** On `revision_conflict`: the provider registry was re-read so the console shows the newer revision. */
  refetched: boolean;
}

/**
 * The only error `setSystemDefaultModel` and `setSystemDefaultProvider` throw. The system default
 * is never changed when this is thrown. `message` is operator-facing text the picker can show.
 */
export class SystemDefaultModelError extends Error implements SystemDefaultModelErrorInit {
  readonly kind: SystemDefaultModelErrorKind;
  readonly stage: SystemDefaultModelStage;
  readonly providerId: string;
  readonly modelId: string | null;
  readonly status: number | null;
  readonly detail: string | null;
  readonly modelSaved: boolean;
  readonly refetched: boolean;

  constructor(init: SystemDefaultModelErrorInit, message: string) {
    super(message);
    this.name = "SystemDefaultModelError";
    this.kind = init.kind;
    this.stage = init.stage;
    this.providerId = init.providerId;
    this.modelId = init.modelId;
    this.status = init.status;
    this.detail = init.detail;
    this.modelSaved = init.modelSaved;
    this.refetched = init.refetched;
  }
}

export interface SystemDefaultModelResult {
  providerId: string;
  /** The model now used by the system default; null when the provider has no default model. */
  modelId: string | null;
  /** The provider configuration as the server confirmed it. */
  provider: UarProvider;
  /** The provider's `default_model` was written by this call. */
  modelSaved: boolean;
}
