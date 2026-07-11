/** Masked credential metadata returned by the runtime. */
export interface CredentialView {
  provider_id: string;
  /** Last four characters of the stored key, for display only. */
  api_key_hint: string;
  created_at: string;
  updated_at: string;
}

/** Lifecycle state of per-user credential storage. */
export type CredentialServiceState = "ok" | "unauthorized" | "disabled";

/** Result of listing the caller's provider credentials. */
export interface CredentialListResult {
  state: CredentialServiceState;
  credentials: CredentialView[];
}
