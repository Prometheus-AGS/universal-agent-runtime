export interface UarUserSettings {
  user_id?: string;
  prompt_caching_enabled: boolean | null;
  /** @deprecated Retained for wire compatibility. It no longer controls precedence. */
  preferred_scope: "session" | "user" | "agent";
  updated_at: string;
}

export interface UarUserSettingsUpdate {
  prompt_caching_enabled?: boolean | null;
  /** @deprecated Retained for wire compatibility. It no longer controls precedence. */
  preferred_scope?: "session" | "user" | "agent";
}

export async function fetchUserSettings(
  headers: HeadersInit,
): Promise<UarUserSettings> {
  const res = await fetch("/api/uar/user/settings", {
    headers: { "Content-Type": "application/json", ...headers },
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<UarUserSettings>;
}

export async function putUserSettings(
  headers: HeadersInit,
  body: UarUserSettingsUpdate,
): Promise<UarUserSettings> {
  const res = await fetch("/api/uar/user/settings", {
    method: "PUT",
    headers: { "Content-Type": "application/json", ...headers },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<UarUserSettings>;
}
