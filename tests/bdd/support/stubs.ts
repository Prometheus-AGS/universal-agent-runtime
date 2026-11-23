import { STUB_BASE_URL, STUB_BASE_URL_B, STUB_BASE_URL_C } from './ports';

/**
 * Per-provider stub-llm instances for the S1 live-model-choice features. Each
 * synthetic provider's base URL points at its own stub, so the stub that logged
 * a request names the provider that served it (design.md "S1 value rules").
 * `openai` is the provider `playwright.config.ts` registers through
 * `UAR_LLM__BASE_URL`; `bdd-provider-off` is disabled and shares C's stub, which
 * therefore must never log a request for it.
 */
export const S1_STUBS = [
  { provider: 'openai', baseUrl: STUB_BASE_URL },
  { provider: 'bdd-provider-b', baseUrl: STUB_BASE_URL_B },
  { provider: 'bdd-provider-c', baseUrl: STUB_BASE_URL_C },
] as const;

export type S1StubProvider = (typeof S1_STUBS)[number]['provider'];

export interface S1StubRequest {
  model: string;
  messages: Array<{ role: string; content: string | null }>;
  tools?: unknown[];
  stream?: boolean;
}

export interface LoggedRequest {
  /** The provider whose stub logged the request. */
  provider: S1StubProvider;
  request: S1StubRequest;
}

/** Base URL of the stub behind a provider id (the disabled provider shares C's stub). */
export function stubBaseUrlFor(providerId: string): string {
  if (providerId === 'bdd-provider-off') return STUB_BASE_URL_C;
  const stub = S1_STUBS.find((entry) => entry.provider === providerId);
  if (!stub) throw new Error(`no stub instance is configured for provider "${providerId}"`);
  return stub.baseUrl;
}

async function readJson<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, init);
  if (!res.ok) throw new Error(`${init?.method ?? 'GET'} ${url} failed: ${res.status}`);
  return (await res.json()) as T;
}

/** Every raw request body one stub has logged, in arrival order. */
export async function stubRequests(baseUrl: string): Promise<S1StubRequest[]> {
  const body = await readJson<{ requests: S1StubRequest[] }>(`${baseUrl}/_stub/requests`);
  return body.requests;
}

/** The last `user`-role message of a request, as the stub's fingerprint reads it. */
export function lastUserMessageOf(request: S1StubRequest): string {
  const user = request.messages.filter((message) => message.role === 'user');
  return user.at(-1)?.content ?? '';
}

/** Every logged request, across every stub, whose last user message equals `message`. */
export async function requestsForMessage(message: string): Promise<LoggedRequest[]> {
  const logs = await Promise.all(
    S1_STUBS.map(async (stub) => ({ provider: stub.provider, requests: await stubRequests(stub.baseUrl) })),
  );
  return logs.flatMap(({ provider, requests }) =>
    requests
      .filter((request) => lastUserMessageOf(request) === message)
      .map((request) => ({ provider, request })),
  );
}

/** Every logged request, across every stub, that asked for the model `modelId` (bare or provider-prefixed). */
export async function requestsForModel(modelId: string): Promise<LoggedRequest[]> {
  const logs = await Promise.all(
    S1_STUBS.map(async (stub) => ({ provider: stub.provider, requests: await stubRequests(stub.baseUrl) })),
  );
  return logs.flatMap(({ provider, requests }) =>
    requests.filter((request) => wireModelMatches(request.model, modelId)).map((request) => ({ provider, request })),
  );
}

/** The wire model is either the bare id or `provider/model` depending on the resolution path. */
export function wireModelMatches(wireModel: string, modelId: string): boolean {
  return wireModel === modelId || wireModel.endsWith(`/${modelId}`);
}

export async function resetAllStubs(): Promise<void> {
  await Promise.all(
    S1_STUBS.map(async (stub) => {
      const res = await fetch(`${stub.baseUrl}/_stub/requests`, { method: 'POST' });
      if (!res.ok) throw new Error(`stub reset failed for ${stub.provider}: ${res.status}`);
    }),
  );
}

/** Held-reply count of one stub (`GET /_stub/held`). */
export async function heldCount(baseUrl: string): Promise<number> {
  const body = await readJson<{ held: number }>(`${baseUrl}/_stub/held`);
  return body.held;
}

export async function totalHeld(): Promise<number> {
  const counts = await Promise.all(S1_STUBS.map((stub) => heldCount(stub.baseUrl)));
  return counts.reduce((sum, count) => sum + count, 0);
}

/**
 * Frees every held reply on every stub (`POST /_stub/release`). Only teardown may allow
 * a missing route: a stub built before the hold kind (the baseline) has none and holds
 * nothing, and a teardown error would hide what the scenario itself reported.
 */
export async function releaseAllStubs(options: { allowMissingRoute?: boolean } = {}): Promise<void> {
  await Promise.all(
    S1_STUBS.map(async (stub) => {
      const res = await fetch(`${stub.baseUrl}/_stub/release`, { method: 'POST' });
      if (res.status === 404 && options.allowMissingRoute) return;
      if (!res.ok) throw new Error(`stub release failed for ${stub.provider}: ${res.status}`);
    }),
  );
}

/**
 * Fixture kinds one stub supports (`GET /_stub/capabilities`), or null when the
 * stub has no such route (a stub built before the hold and served-model kinds).
 */
export async function stubCapabilities(baseUrl: string): Promise<string[] | null> {
  const res = await fetch(`${baseUrl}/_stub/capabilities`);
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`GET ${baseUrl}/_stub/capabilities failed: ${res.status}`);
  const body = (await res.json()) as { kinds: string[] };
  return body.kinds;
}
