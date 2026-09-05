import {
  presentationCatalogSchema,
  presentationSchema,
  type Presentation,
  type PresentationCatalogResponse,
  type PresentationContent,
} from "../contracts";

export class PresentationApiError extends Error {
  constructor(message: string, public readonly status: number, public readonly uncertain = false) {
    super(message);
    this.name = "PresentationApiError";
  }
}

function authorizationHeaders(): Record<string, string> {
  const credential = import.meta.env.VITE_UAR_API_KEY ?? "";
  if (!credential) return {};
  return credential.startsWith("ey")
    ? { Authorization: `Bearer ${credential}` }
    : { "x-api-key": credential };
}

async function request(path: string, method: string, body?: unknown): Promise<Response> {
  const writing = method !== "GET";
  let response: Response;
  try {
    response = await fetch(`/api/uar/presentations${path}`, {
      method,
      headers: { "Content-Type": "application/json", ...authorizationHeaders() },
      body: body === undefined ? undefined : JSON.stringify(body),
      cache: "no-store",
    });
  } catch {
    throw new PresentationApiError(writing
      ? "The connection ended before the result was confirmed. Your draft is safe. Reload the catalog and check for the change before saving again."
      : "Could not load Presentations. Check your connection and reload.", 0, writing);
  }
  if (!response.ok) {
    const messages: Record<number, string> = {
      401: "Sign in with an authenticated runtime credential, then reload Presentations.",
      403: "This credential cannot access the Presentation catalog.",
      404: "This Presentation is no longer available. Your draft is safe.",
      409: "The saved version changed. Your draft is safe. Reload the saved version to resolve the conflict.",
      422: "The template was not accepted. Review the source and supported components.",
    };
    let message = messages[response.status] ?? (writing
      ? "The server did not confirm the change. Your draft is safe. Reload the catalog before trying again."
      : "The catalog is unavailable. Try reloading.");
    if (response.status === 422) {
      const detail: unknown = await response.json().catch(() => null);
      if (detail && typeof detail === "object" && "error" in detail && typeof detail.error === "string") {
        message = detail.error;
      }
    }
    throw new PresentationApiError(message, response.status, writing && response.status >= 500);
  }
  return response;
}

export async function fetchPresentations(): Promise<PresentationCatalogResponse> {
  const response = await request("", "GET");
  const catalog = presentationCatalogSchema.parse(await response.json());
  if (catalog.presentations.some((record) => record.owner_id !== catalog.owner_id)) {
    throw new PresentationApiError("The catalog owner could not be verified. No records were loaded.", 403);
  }
  return catalog;
}

export async function savePresentation(id: string | null, revision: number | null, content: PresentationContent): Promise<Presentation> {
  const response = await request(id ? `/${encodeURIComponent(id)}` : "", id ? "PUT" : "POST",
    id ? { expected_revision: revision, content } : content);
  try {
    return presentationSchema.parse(await response.json());
  } catch {
    throw new PresentationApiError("The server returned an unreadable save result. Reload the catalog before saving again.", response.status, true);
  }
}

export async function deletePresentation(id: string, revision: number): Promise<void> {
  await request(`/${encodeURIComponent(id)}?expected_revision=${revision}`, "DELETE");
}
