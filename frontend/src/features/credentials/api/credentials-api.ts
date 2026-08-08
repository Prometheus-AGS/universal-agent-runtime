// Per-user provider credential API client.
//
// Talks to the JWT-gated `/api/uar/credentials` endpoints (same-origin fetch;
// auth rides the session cookie, like the other admin services). Raw keys are
// only ever sent on write — reads return masked metadata, never plaintext.

import type { CredentialListResult, CredentialView } from "@/entities/credential-types";

export type {
  CredentialListResult,
  CredentialServiceState,
  CredentialView,
} from "@/entities/credential-types";

/**
 * List the caller's stored provider credentials.
 *
 * Maps the auth/lifecycle status codes to an explicit state so the page can
 * distinguish "you have no keys" (ok + empty) from "sign in" (401) and
 * "multi-tenant credentials are disabled on this server" (503).
 */
export async function listCredentials(): Promise<CredentialListResult> {
  const res = await fetch("/api/uar/credentials");
  if (res.status === 401) return { state: "unauthorized", credentials: [] };
  if (res.status === 503) return { state: "disabled", credentials: [] };
  if (!res.ok) throw new Error(`Credentials fetch failed: ${res.status}`);
  const credentials = (await res.json()) as CredentialView[];
  return { state: "ok", credentials };
}

/**
 * Store or rotate the caller's key for `providerId`.
 * The raw key is sent once and never returned by the server.
 */
export async function putCredential(
  providerId: string,
  apiKey: string,
): Promise<CredentialView> {
  const res = await fetch(`/api/uar/credentials/${encodeURIComponent(providerId)}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ api_key: apiKey }),
  });
  if (!res.ok) throw new Error(`Save credential failed: ${res.status}`);
  return (await res.json()) as CredentialView;
}

/** Delete the caller's key for `providerId`. */
export async function deleteCredential(providerId: string): Promise<void> {
  const res = await fetch(`/api/uar/credentials/${encodeURIComponent(providerId)}`, {
    method: "DELETE",
  });
  // 404 == already gone; treat as success for idempotent delete UX.
  if (!res.ok && res.status !== 404) {
    throw new Error(`Delete credential failed: ${res.status}`);
  }
}
