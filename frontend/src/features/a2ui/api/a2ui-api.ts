import type { A2uiArtifactSchema, A2uiTestTriggerPayload } from "../a2ui-protocol";

export async function fetchA2uiSchemas(): Promise<A2uiArtifactSchema[]> {
  const res = await fetch("/api/uar/a2ui/schemas");
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const data = await res.json() as unknown;
  if (!Array.isArray(data)) throw new Error("Invalid A2UI schema response");
  return data as A2uiArtifactSchema[];
}

export async function triggerA2uiTest(runId: string, body: A2uiTestTriggerPayload): Promise<void> {
  const res = await fetch(`/api/uar/runs/${encodeURIComponent(runId)}/a2ui/test-trigger`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const detail = await res.text().catch(() => "");
    throw new Error(detail || `HTTP ${res.status}`);
  }
}
