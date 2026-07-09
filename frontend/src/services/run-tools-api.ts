export async function postToolApproval(runId: string, approved: boolean): Promise<void> {
  await fetch(`/api/uar/runs/${runId}/tool-approval`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ approved }),
  });
}

export async function postArtifactResponse(
  runId: string,
  body: Record<string, unknown>,
): Promise<Response> {
  return fetch(`/api/uar/runs/${encodeURIComponent(runId)}/artifact-response`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export interface TestTriggerPayload {
  artifact_type: string;
  title: string;
  content: string;
  metadata?: Record<string, unknown>;
}

export async function postA2uiTestTrigger(
  runId: string,
  body: TestTriggerPayload,
): Promise<Response> {
  return fetch(`/api/uar/runs/${encodeURIComponent(runId)}/a2ui/test-trigger`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}
