import type { ResumeRunInput } from "@/features/chat/model/run-trace-types";

async function readJson(response: Response, label: string): Promise<unknown> {
  if (!response.ok) throw new Error(`${label} failed: ${response.status}`);
  return response.json();
}

export function requestRunCheckpoints(runId: string): Promise<unknown> {
  return fetch(`/api/uar/runs/${encodeURIComponent(runId)}/checkpoints`)
    .then((response) => readJson(response, "Checkpoint request"));
}

export function requestRunSurfaceReplay(runId: string): Promise<unknown> {
  return fetch(`/api/uar/runs/${encodeURIComponent(runId)}/a2ui/surface-replay`)
    .then((response) => readJson(response, "Surface replay request"));
}

export function requestLatestCheckpointResume(
  runId: string,
  input: ResumeRunInput,
): Promise<unknown> {
  return fetch(`/api/uar/runs/${encodeURIComponent(runId)}/resume`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  }).then((response) => readJson(response, "Run resume request"));
}
