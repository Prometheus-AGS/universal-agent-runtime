import { batchGraphWrites } from "@prometheus-ags/entity-graph-core";

import type { ProvidersResponse, UarProvider } from "@/types";
import {
  ProviderApiError,
  fetchConfiguredProviders,
  setDefaultProvider,
  updateProvider,
} from "./api/model-catalog-api";
import {
  PROVIDER_META_ENTITY,
  PROVIDER_META_ID,
  SystemDefaultModelError,
  type ProviderMeta,
  type SystemDefaultModelErrorInit,
  type SystemDefaultModelErrorKind,
  type SystemDefaultModelResult,
  type SystemDefaultModelStage,
} from "./contracts";
import { invalidateInFlightRegistryLoads, loadProviderRegistry } from "./domain";

/**
 * The one write path that changes the system default model.
 *
 * Order is model first, then provider: the default route accepts only a provider id, so the pair
 * is `PUT /api/uar/providers/{id}` (`default_model`, only when it differs) followed by
 * `POST /api/uar/providers/{id}/default`. A failure after the first write leaves the live default
 * provider unchanged. No endpoint is added and no setting row is written.
 *
 * The graph is written once, after both calls succeed: `ProviderMeta` receives `default_id` and
 * `default_model` together. Nothing is published optimistically, so there is nothing to roll back
 * and the displayed pair is never half-populated.
 */

type Target = { kind: "model"; modelId: string } | { kind: "provider-default" };

let tail: Promise<unknown> = Promise.resolve();

/** Changes run one at a time so a second request reads the provider the first one left behind. */
function serialized<T>(operation: () => Promise<T>): Promise<T> {
  const run = tail.then(operation, operation);
  tail = run.then(
    () => undefined,
    () => undefined,
  );
  return run;
}

function statusOf(error: unknown): number | null {
  return error instanceof ProviderApiError ? error.status : null;
}

function detailOf(error: unknown): string | null {
  if (error instanceof ProviderApiError) return error.detail ?? error.message;
  return error instanceof Error ? error.message : null;
}

function describe(
  kind: SystemDefaultModelErrorKind,
  init: SystemDefaultModelErrorInit,
): string {
  const { providerId, detail, modelSaved } = init;
  const outcome = modelSaved
    ? "Model saved, system default not changed."
    : "System default not changed.";
  switch (kind) {
    case "invalid_selection":
      return "Choose a provider and a model.";
    case "provider_fetch_failed":
      return `${outcome} Could not read provider '${providerId}'${detail ? `: ${detail}` : "."}`;
    case "provider_not_found":
      return `${outcome} Provider '${providerId}' no longer exists.`;
    case "provider_disabled":
      return `${outcome} Provider '${providerId}' is disabled. Enable it before making it the system default.`;
    case "revision_conflict":
      return `${outcome} Provider '${providerId}' was changed by someone else${
        init.refetched ? "; its latest settings were reloaded. Review the selection and try again." : "."
      }`;
    case "model_save_failed":
      return `${outcome} Could not save the model for '${providerId}'${detail ? `: ${detail}` : "."}`;
    case "default_route_failed":
      return `${outcome} ${detail ?? `The default route failed for '${providerId}'.`}`;
  }
}

function fail(
  kind: SystemDefaultModelErrorKind,
  stage: SystemDefaultModelStage,
  base: Pick<SystemDefaultModelErrorInit, "providerId" | "modelId">,
  extra: Partial<Pick<SystemDefaultModelErrorInit, "status" | "detail" | "modelSaved" | "refetched">> = {},
): SystemDefaultModelError {
  const init: SystemDefaultModelErrorInit = {
    kind,
    stage,
    providerId: base.providerId,
    modelId: base.modelId,
    status: extra.status ?? null,
    detail: extra.detail ?? null,
    modelSaved: extra.modelSaved ?? false,
    refetched: extra.refetched ?? false,
  };
  return new SystemDefaultModelError(init, describe(kind, init));
}

/** Best-effort reload of the registry; reports whether the graph now matches the server. */
async function refreshRegistry(): Promise<boolean> {
  try {
    await loadProviderRegistry();
    return true;
  } catch {
    return false;
  }
}

function publishSystemDefault(providerId: string, modelId: string | null): void {
  // Older in-flight registry loads would overwrite this pair with the pre-change default.
  invalidateInFlightRegistryLoads();
  const meta: ProviderMeta = { id: PROVIDER_META_ID, default_id: providerId, default_model: modelId };
  batchGraphWrites((batch) => {
    batch.replaceEntity(PROVIDER_META_ENTITY, PROVIDER_META_ID, meta);
  });
}

async function readProvider(
  providerId: string,
  requestedModelId: string | null,
): Promise<UarProvider> {
  const base = { providerId, modelId: requestedModelId };
  let configured: ProvidersResponse;
  try {
    configured = await fetchConfiguredProviders();
  } catch (error) {
    throw fail("provider_fetch_failed", "fetch_provider", base, {
      status: statusOf(error),
      detail: detailOf(error),
    });
  }
  const existing = configured.providers.find((provider) => provider.id === providerId);
  if (!existing) throw fail("provider_not_found", "fetch_provider", base, { status: 404 });
  // The default route refuses a disabled provider after the model write would already be saved.
  if (existing.enabled === false) throw fail("provider_disabled", "fetch_provider", base);
  return existing;
}

async function saveModel(
  existing: UarProvider,
  modelId: string,
): Promise<UarProvider> {
  const base = { providerId: existing.id, modelId };
  try {
    return await updateProvider(existing.id, { ...existing, default_model: modelId });
  } catch (error) {
    const status = statusOf(error);
    const detail = detailOf(error);
    if (status === 404) throw fail("provider_not_found", "save_model", base, { status, detail });
    if (status === 409) {
      // TEAM_REVISION_CONFLICT: the provider changed. Re-read it and let the operator decide.
      const refetched = await refreshRegistry();
      throw fail("revision_conflict", "save_model", base, { status, detail, refetched });
    }
    throw fail("model_save_failed", "save_model", base, { status, detail });
  }
}

async function changeSystemDefault(
  providerId: string,
  target: Target,
): Promise<SystemDefaultModelResult> {
  const requestedModelId = target.kind === "model" ? target.modelId : null;
  if (!providerId.trim() || (target.kind === "model" && !target.modelId.trim())) {
    throw fail("invalid_selection", "fetch_provider", { providerId, modelId: requestedModelId });
  }

  const existing = await readProvider(providerId, requestedModelId);
  const modelId = target.kind === "model" ? target.modelId : (existing.default_model ?? null);
  const base = { providerId, modelId };

  let provider = existing;
  let modelSaved = false;
  if (target.kind === "model" && existing.default_model !== target.modelId) {
    provider = await saveModel(existing, target.modelId);
    modelSaved = true;
  }

  try {
    await setDefaultProvider(providerId);
  } catch (error) {
    const status = statusOf(error);
    const detail = detailOf(error);
    // The saved model may already be live when this provider is the current default.
    if (modelSaved) await refreshRegistry();
    if (status === 404) throw fail("provider_not_found", "set_default", base, { status, detail, modelSaved });
    if (status === 409) throw fail("provider_disabled", "set_default", base, { status, detail, modelSaved });
    throw fail("default_route_failed", "set_default", base, { status, detail, modelSaved });
  }

  publishSystemDefault(providerId, modelId);
  return { providerId, modelId, provider, modelSaved };
}

/**
 * Make `providerId` the system default provider and `modelId` its default model.
 *
 * Reads the provider's current configuration, saves `default_model` only when it differs, then
 * calls the default route. On success `ProviderMeta` is published as one write.
 *
 * @throws SystemDefaultModelError on every failure. The system default is unchanged then; check
 * `modelSaved` for the "model saved, system default not changed" case.
 */
export function setSystemDefaultModel(
  providerId: string,
  modelId: string,
): Promise<SystemDefaultModelResult> {
  return serialized(() => changeSystemDefault(providerId, { kind: "model", modelId }));
}

/**
 * Make `providerId` the system default provider with the model it already has. Same write path
 * as `setSystemDefaultModel`; the provider's `default_model` is not rewritten. When the provider
 * has no default model the result carries `modelId: null` and the status reads "unavailable".
 *
 * @throws SystemDefaultModelError on every failure.
 */
export function setSystemDefaultProvider(providerId: string): Promise<SystemDefaultModelResult> {
  return serialized(() => changeSystemDefault(providerId, { kind: "provider-default" }));
}
