import type {
  CatalogModelsResponse,
  CatalogResponse,
  ProvidersResponse,
  UarProvider,
} from "@/types";

/**
 * Transport for the model catalog and provider registry. Moved here from
 * `features/models/api/models-api.ts` and `features/providers/api/providers-api.ts`; those
 * modules re-export these functions for callers that have not migrated yet.
 */

/** A provider route answered with a non-2xx status. `message` keeps the historical text. */
export class ProviderApiError extends Error {
  readonly status: number;
  /** The route's `{ "error": ... }` text, when the body carried one. */
  readonly detail: string | null;

  constructor(message: string, status: number, detail: string | null) {
    super(message);
    this.name = "ProviderApiError";
    this.status = status;
    this.detail = detail;
  }
}

async function readErrorDetail(res: Response): Promise<string | null> {
  const text = await res.text().catch(() => "");
  if (!text) return null;
  try {
    const body = JSON.parse(text) as unknown;
    if (body && typeof body === "object") {
      const error = (body as { error?: unknown }).error;
      if (typeof error === "string") return error;
      if (error && typeof error === "object") {
        const detail = (error as { detail?: unknown; message?: unknown }).detail
          ?? (error as { message?: unknown }).message;
        if (typeof detail === "string") return detail;
      }
    }
  } catch {
    // Not JSON: fall through to the raw text.
  }
  return text;
}

async function failure(res: Response, message: string): Promise<ProviderApiError> {
  return new ProviderApiError(message, res.status, await readErrorDetail(res));
}

export async function fetchModelsCatalog(): Promise<CatalogModelsResponse> {
  const res = await fetch("/api/models");
  if (!res.ok) throw await failure(res, `Models fetch failed: ${res.status}`);
  return res.json() as Promise<CatalogModelsResponse>;
}

export async function fetchCatalog(): Promise<CatalogResponse> {
  const res = await fetch("/api/catalog");
  if (!res.ok) throw await failure(res, `Catalog fetch failed: ${res.status}`);
  return res.json() as Promise<CatalogResponse>;
}

export async function fetchConfiguredProviders(): Promise<ProvidersResponse> {
  const res = await fetch("/api/uar/providers");
  if (!res.ok) throw await failure(res, `Providers fetch failed: ${res.status}`);
  return res.json() as Promise<ProvidersResponse>;
}

export interface ProviderHealthEntry {
  healthy: boolean;
  consecutive_errors: number;
  cooldown_remaining_secs: number | null;
}

export interface ProviderHealthResponse {
  providers: Record<string, ProviderHealthEntry>;
}

export async function fetchProviderHealth(): Promise<ProviderHealthResponse> {
  const res = await fetch("/api/uar/providers/health");
  if (!res.ok) throw await failure(res, `Provider health fetch failed: ${res.status}`);
  return res.json() as Promise<ProviderHealthResponse>;
}

export interface CreateProviderBody {
  id: string;
  display_name: string;
  base_url: string;
  api_key?: string;
  protocol: string;
  enabled: boolean;
}

/** POST /api/uar/providers — create a provider that is not already configured. */
export async function createProvider(body: CreateProviderBody): Promise<Response> {
  return fetch("/api/uar/providers", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

/**
 * PUT /api/uar/providers/{id} — replace a provider's full configuration.
 *
 * The backend overwrites `id` from the path and re-enriches/persists the
 * config, returning the stored `ProviderConfig`. This is the single write
 * path used to enable/remove a model (mutate `models[]`) or change the
 * provider's `default_model` — there is no dedicated models write endpoint.
 * A stale execution-profile revision answers 409 `TEAM_REVISION_CONFLICT`.
 */
export async function updateProvider(
  providerId: string,
  body: UarProvider,
): Promise<UarProvider> {
  const res = await fetch(`/api/uar/providers/${providerId}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw await failure(res, `Update provider failed: ${res.status}`);
  return res.json() as Promise<UarProvider>;
}

/**
 * POST /api/uar/providers/{id}/default — 404 for an unknown provider and 409
 * `Provider '<id>' is disabled` for a disabled one.
 */
export async function setDefaultProvider(providerId: string): Promise<void> {
  const res = await fetch(`/api/uar/providers/${providerId}/default`, { method: "POST" });
  if (!res.ok) throw await failure(res, `Set default failed: ${res.status}`);
}

export async function deleteProvider(providerId: string): Promise<void> {
  const res = await fetch(`/api/uar/providers/${providerId}`, { method: "DELETE" });
  if (!res.ok) throw await failure(res, `Delete failed: ${res.status}`);
}
