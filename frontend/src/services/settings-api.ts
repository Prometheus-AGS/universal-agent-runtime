import type { SettingWithMeta } from "@/types";

const BASE = "/api/uar/settings";

/** Convert a namespace key (e.g. "context_management") to its URL slug (e.g. "context-management") */
export function namespaceToSlug(ns: string): string {
  const overrides: Record<string, string> = {
    provider: "providers",
    file_processing: "file-processing",
    knowledge_bases: "knowledge-bases",
    intent_classifier: "intent-classifier",
    context_management: "context-management",
    agent_config: "agent-config",
    skill_config: "skill-config",
    mistral_ocr: "mistral-ocr",
  };
  return overrides[ns] ?? ns.replace(/_/g, "-");
}

export async function fetchSettingsNamespace(slug: string): Promise<SettingWithMeta[]> {
  const res = await fetch(`${BASE}/${slug}`);
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json() as Promise<SettingWithMeta[]>;
}

export async function putSettingValue(key: string, value: unknown): Promise<void> {
  const res = await fetch(`${BASE}/${encodeURIComponent(key)}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ value }),
  });
  if (!res.ok) throw new Error(`Save ${key} failed: ${res.status}`);
}

export async function fetchSettingTypes(): Promise<unknown> {
  const res = await fetch(`${BASE}/types`);
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json();
}

export async function fetchResilienceSettings(): Promise<unknown> {
  const res = await fetch(`${BASE}/resilience`);
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json();
}
