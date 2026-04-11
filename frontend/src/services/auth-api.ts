import type { UarApiKey } from "@/types";

export async function fetchAuthKeys(): Promise<UarApiKey[]> {
  const res = await fetch("/api/auth/keys");
  if (!res.ok) throw new Error(`${res.status}`);
  const data = await res.json() as { keys?: UarApiKey[]; data?: { keys?: UarApiKey[] } } | UarApiKey[];
  return Array.isArray(data) ? data : (data.data?.keys ?? (data as { keys?: UarApiKey[] }).keys ?? []);
}

export async function createAuthKey(name: string): Promise<{ key?: string; raw_key?: string }> {
  const res = await fetch("/api/auth/keys", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name }),
  });
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json() as Promise<{ key?: string; raw_key?: string }>;
}

export async function deleteAuthKey(id: string): Promise<void> {
  const res = await fetch(`/api/auth/keys/${id}`, { method: "DELETE" });
  if (!res.ok) throw new Error(`${res.status}`);
}
