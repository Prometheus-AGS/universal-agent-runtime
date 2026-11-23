import { graphStore, useGraphStore } from "@prometheus-ags/prometheus-entity-management";

/**
 * Which model a conversation would actually be served by, and why.
 *
 * Source: `GET /api/uar/sessions/{id}/effective-config` (the runtime resolves the route the same way
 * chat admission does). The record is flat so each control subscribes to one primitive. The graph is
 * written once per successful load; a failed load or a refused save leaves the previous record, and
 * a failed refresh after a save removes it so the sheet never presents a value it could not confirm.
 */

export const SESSION_EFFECTIVE_MODEL_ENTITY = "SessionEffectiveModel" as const;

/** Where the serving model came from. Wire values of `model_origin`. */
export type ModelOrigin = "system_default" | "conversation" | "agent" | "turn" | "configured_fallback";

export const MODEL_ORIGINS: readonly ModelOrigin[] = [
  "system_default",
  "conversation",
  "agent",
  "turn",
  "configured_fallback",
];

export function isModelOrigin(value: unknown): value is ModelOrigin {
  return typeof value === "string" && (MODEL_ORIGINS as readonly string[]).includes(value);
}

export interface SessionEffectiveModel extends Record<string, unknown> {
  id: string;
  session_id: string;
  /** False when the conversation has no stored settings (`requested_policy` is null); the values are still the runtime's. */
  stored: boolean;
  /** `provider/model` the next turn would be admitted on; null when none could serve. */
  model: string | null;
  origin: ModelOrigin | null;
  /** RFC 3339, seconds. Present only for `system_default`. */
  since: string | null;
  /** Why admission would refuse the turn; set only while `model` is null. */
  unavailable_reason: string | null;
  /** The same four, if the conversation's own model were cleared. */
  inherited_model: string | null;
  inherited_origin: ModelOrigin | null;
  inherited_since: string | null;
  inherited_unavailable_reason: string | null;
}

export type SessionEffectiveModelField = Exclude<keyof SessionEffectiveModel, "id" | "session_id">;

function authorizationHeaders(): HeadersInit {
  const apiKey =
    (import.meta as unknown as { env: Record<string, string> }).env.VITE_UAR_API_KEY ?? "";
  return apiKey.startsWith("ey")
    ? { Authorization: `Bearer ${apiKey}` }
    : apiKey
      ? { "x-api-key": apiKey }
      : {};
}

function record(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

function text(value: unknown): string | null {
  return typeof value === "string" && value.trim() !== "" ? value : null;
}

function route(value: unknown): string | null {
  const source = record(value);
  const provider = text(source?.provider_id);
  const model = text(source?.model_id);
  return provider && model ? `${provider}/${model}` : null;
}

function origin(value: unknown): ModelOrigin | null {
  return isModelOrigin(value) ? value : null;
}

/** A record with nothing stated; the starting point the decoder fills. */
export function emptySessionEffectiveModel(sessionId: string): SessionEffectiveModel {
  return {
    id: sessionId,
    session_id: sessionId,
    stored: false,
    model: null,
    origin: null,
    since: null,
    unavailable_reason: null,
    inherited_model: null,
    inherited_origin: null,
    inherited_since: null,
    inherited_unavailable_reason: null,
  };
}

/** Decodes the effective-config body; throws on a body that is not an object. */
export function decodeSessionEffectiveModel(sessionId: string, body: unknown): SessionEffectiveModel {
  const root = record(body);
  if (!root) throw new Error("Effective model response must be an object");
  const effective = record(root.effective_policy);
  const inherited = record(root.inherited_model);
  const model = route(effective?.model);
  return {
    id: sessionId,
    session_id: sessionId,
    stored: root.requested_policy != null,
    model,
    origin: model ? origin(effective?.model_origin) : null,
    since: model ? text(effective?.model_origin_since) : null,
    unavailable_reason: model ? null : text(root.model_unavailable_reason),
    inherited_model: inherited ? route(inherited.model) : null,
    inherited_origin: inherited && route(inherited.model) ? origin(inherited.model_origin) : null,
    inherited_since: inherited && route(inherited.model) ? text(inherited.model_origin_since) : null,
    inherited_unavailable_reason: inherited && !route(inherited.model) ? text(inherited.unavailable_reason) : null,
  };
}

/**
 * The runtime answers for every conversation, including one with no stored settings
 * (`requested_policy: null`). A failed read, 503 included, is an error: the console never fills a
 * model in from anywhere else.
 */
export async function fetchSessionEffectiveModel(
  sessionId: string,
  signal?: AbortSignal,
): Promise<SessionEffectiveModel> {
  const response = await fetch(`/api/uar/sessions/${encodeURIComponent(sessionId)}/effective-config`, {
    headers: authorizationHeaders(),
    signal,
    cache: "no-store",
  });
  if (!response.ok) throw new Error(`Effective model load failed: ${response.status}`);
  return decodeSessionEffectiveModel(sessionId, await response.json());
}

/**
 * Loads the effective model for one conversation and publishes it as one graph write.
 * Rejects (and writes nothing) when the runtime cannot answer.
 */
export async function loadSessionEffectiveModel(
  sessionId: string,
  signal?: AbortSignal,
): Promise<SessionEffectiveModel> {
  const effective = await fetchSessionEffectiveModel(sessionId, signal);
  if (signal?.aborted) throw new DOMException("Request aborted", "AbortError");
  graphStore.getState().replaceEntity(SESSION_EFFECTIVE_MODEL_ENTITY, sessionId, effective);
  return effective;
}

/** Drops the record so no value is shown that the runtime has not just confirmed. */
export function clearSessionEffectiveModel(sessionId: string): void {
  graphStore.getState().removeEntity(SESSION_EFFECTIVE_MODEL_ENTITY, sessionId);
}

/** One field of a conversation's effective model; undefined until a load has completed. */
export function useSessionEffectiveModelField<K extends SessionEffectiveModelField>(
  sessionId: string,
  field: K,
): SessionEffectiveModel[K] | undefined {
  return useGraphStore((state) => {
    const row = state.entities[SESSION_EFFECTIVE_MODEL_ENTITY]?.[sessionId] as
      | SessionEffectiveModel
      | undefined;
    return row?.[field];
  });
}
