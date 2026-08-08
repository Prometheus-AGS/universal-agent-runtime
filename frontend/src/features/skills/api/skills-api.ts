import type { UarSkill } from "@/types";

export type SkillsResponse =
  | { skills?: UarSkill[]; data?: { skills?: UarSkill[] } }
  | UarSkill[];

export function parseSkillsList(data: SkillsResponse): UarSkill[] {
  if (Array.isArray(data)) return data;
  return data.data?.skills ?? data.skills ?? [];
}

export async function fetchSkillsList(): Promise<UarSkill[]> {
  const res = await fetch("/api/skills");
  if (!res.ok) throw new Error(`${res.status}`);
  const data = (await res.json()) as SkillsResponse;
  return parseSkillsList(data);
}

export async function toggleSkillApi(skillId: string, enabled: boolean): Promise<void> {
  const res = await fetch(`/api/skills/${encodeURIComponent(skillId)}/toggle`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ enabled }),
  });
  if (!res.ok) throw new Error(`${res.status}`);
}

export async function deleteSkillApi(skillId: string): Promise<void> {
  const res = await fetch(`/api/skills/${encodeURIComponent(skillId)}`, {
    method: "DELETE",
  });
  if (!res.ok && res.status !== 404) throw new Error(`${res.status}`);
}

export async function updateSkillApi(skillId: string, body: unknown): Promise<void> {
  const res = await fetch(`/api/skills/${encodeURIComponent(skillId)}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`${res.status}`);
}

export interface ImportedSkillParsed {
  name: string;
  title: string;
  description: string;
  version: string;
  triggers: { keywords: string[] };
  prompt_overlay: string;
  source: string;
  source_path: string;
  references: string[];
  detected_format: string;
}

export interface ImportedSkillValidation {
  valid: boolean;
  warnings: string[];
}

export interface ImportSkillResult {
  parsed: ImportedSkillParsed;
  validation: ImportedSkillValidation;
}

export async function importSkillFromDisk(path: string): Promise<ImportSkillResult> {
  const res = await fetch("/api/uar/skills/import", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ path }),
  });
  if (!res.ok) throw new Error(`Import failed: ${res.status}`);
  return res.json() as Promise<ImportSkillResult>;
}

export async function createSkillApi(body: unknown): Promise<void> {
  const res = await fetch("/api/skills", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`${res.status}`);
}
