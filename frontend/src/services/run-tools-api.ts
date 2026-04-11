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
