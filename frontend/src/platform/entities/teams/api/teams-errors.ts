/**
 * Typed failures of the collaboration API. The runtime wraps every error as
 * `{error:{code, messageKey, detail?, diagnostic?}}`; a 429 has no body.
 * Every domain conflict is HTTP 409, including "budget exhausted" and "member
 * busy", so `code` alone never means "the revision changed": show `detail`.
 */
export type TeamsErrorKind =
  | "unauthorized"
  | "workspace_required"
  | "conflict"
  | "rate_limited"
  | "not_found"
  | "invalid"
  | "forbidden"
  | "unavailable"
  | "runtime"
  | "network"
  | "malformed"
  | "precondition";

/** The runtime's `error.diagnostic`, present on some refusals (for example a refused admit). */
export interface TeamsErrorDiagnostic {
  code: string | null;
  retryable: boolean | null;
  /** What the runtime says to do next, for example `rebind` or `change-settings`. */
  action: string | null;
  category: string | null;
  sourceStage: string | null;
}

/** Plain-data form of an error, safe to keep in the graph. */
export interface TeamsErrorInfo {
  kind: TeamsErrorKind;
  /** HTTP status, or 0 when none was received. */
  status: number;
  /** The runtime's `error.code` (`TEAM_*` codes replace the generic one), or a local code. */
  code: string;
  /** The runtime's `error.detail`, verbatim. */
  detail: string | null;
  /** Operator-facing text. For a conflict this is the runtime's detail. */
  message: string;
  /** Set when a refused request names a missing durable capability. */
  capability: string | null;
  /** A write whose outcome is unknown: the request may have been committed. */
  uncertain: boolean;
  /** The runtime's diagnostic object, when the error body carried one. */
  diagnostic?: TeamsErrorDiagnostic | null;
}

export class TeamsApiError extends Error {
  readonly info: TeamsErrorInfo;

  constructor(info: TeamsErrorInfo) {
    super(info.message);
    this.name = "TeamsApiError";
    this.info = info;
  }

  get kind(): TeamsErrorKind {
    return this.info.kind;
  }
  get status(): number {
    return this.info.status;
  }
  get code(): string {
    return this.info.code;
  }
  get detail(): string | null {
    return this.info.detail;
  }
  get uncertain(): boolean {
    return this.info.uncertain;
  }
}

/** A precondition the console can check locally (no matching member, task not claimed). Nothing was sent. */
export function preconditionError(code: string, message: string): TeamsApiError {
  return new TeamsApiError({ kind: "precondition", status: 0, code, detail: null, message, capability: null, uncertain: false });
}

export function toErrorInfo(error: unknown): TeamsErrorInfo {
  if (error instanceof TeamsApiError) return error.info;
  return {
    kind: "runtime",
    status: 0,
    code: "unexpected",
    detail: null,
    message: error instanceof Error ? error.message : "Unexpected error.",
    capability: null,
    uncertain: false,
  };
}

interface ErrorBody {
  code: string | null;
  detail: string | null;
  capability: string | null;
  diagnostic: TeamsErrorDiagnostic | null;
}

function readDiagnostic(value: unknown): TeamsErrorDiagnostic | null {
  if (!value || typeof value !== "object") return null;
  const record = value as Record<string, unknown>;
  const text = (key: string) => (typeof record[key] === "string" ? (record[key] as string) : null);
  return {
    code: text("code"),
    retryable: typeof record.retryable === "boolean" ? record.retryable : null,
    action: text("action"),
    category: text("category"),
    sourceStage: text("sourceStage"),
  };
}

function readBody(value: unknown): ErrorBody {
  const body = value && typeof value === "object" ? (value as { error?: unknown }).error : null;
  const record = body && typeof body === "object" ? (body as Record<string, unknown>) : {};
  const text = (key: string) => (typeof record[key] === "string" ? (record[key] as string) : null);
  return { code: text("code"), detail: text("detail"), capability: text("capability"), diagnostic: readDiagnostic(record.diagnostic) };
}

/** Classify a non-2xx response. `body` is the parsed JSON (or null when absent or unreadable). */
export function classifyFailure(status: number, body: unknown, writing: boolean): TeamsApiError {
  const parsed: ErrorBody = status === 429 ? { code: null, detail: null, capability: null, diagnostic: null } : readBody(body);
  const make = (kind: TeamsErrorKind, fallbackCode: string, message: string, uncertain = false) =>
    new TeamsApiError({
      kind,
      status,
      code: parsed.code ?? fallbackCode,
      detail: parsed.detail,
      message: parsed.detail ?? message,
      capability: parsed.capability,
      uncertain,
      diagnostic: parsed.diagnostic,
    });
  if (status === 401) {
    return make("unauthorized", "authenticated_owner_required", "A verified owner is required. Open the console through the identity host.");
  }
  if (status === 400 && parsed.code === "workspace_required") {
    return make("workspace_required", "workspace_required", "A workspace is required for this request.");
  }
  if (status === 429) return make("rate_limited", "rate_limited", "The runtime is rate limiting requests. Try again shortly.");
  if (status === 409) return make("conflict", "collaboration_revision_conflict", "The runtime reported a conflict.");
  if (status === 403) return make("forbidden", "forbidden", "The runtime refused this request.");
  if (status === 404) return make("not_found", "collaboration_not_found", "The runtime has no such record.");
  if (status === 422) return make("invalid", "collaboration_invalid", "The runtime did not accept the request.");
  if (status === 503) {
    return make("unavailable", "collaboration_storage_unavailable", "The runtime cannot serve this request right now.", writing);
  }
  return make("runtime", `http_${status}`, "The runtime returned an unexpected response.", writing && status >= 500);
}
