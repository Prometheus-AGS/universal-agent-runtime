import { z } from "zod";

import { fetchAgentsList } from "@/features/agents/api";
import {
  requestLatestCheckpointResume,
  requestRunCheckpoints,
  requestRunSurfaceReplay,
} from "@/services/run-trace-api";
import { getDbInstance } from "@/platform/pglite/client";
import type {
  PersistedRunSnapshot,
  PersistedRunSnapshotSubscription,
} from "@/platform/pglite/run-event-repository";
import type {
  ResumeRunInput,
  ResumeRunResponse,
  RunCheckpointListResponse,
  RunReplayPatchOperation,
  RuntimeAgentArtifact,
} from "@/features/chat/model/run-trace-types";

const jsonObject = z.record(z.string(), z.unknown());

const checkpointSchema = z.object({
  id: z.string(),
  run_id: z.string(),
  thread_id: z.string(),
  node_id: z.string(),
  iteration: z.number().int().nonnegative(),
  state: z.unknown(),
  messages: z.array(z.unknown()),
  created_at: z.string(),
}).strict();

const checkpointListSchema = z.object({
  run_id: z.string(),
  checkpoints: z.array(checkpointSchema),
}).strict();

const resumeResponseSchema = z.object({
  resumed_from_run_id: z.string(),
  run_id: z.string(),
  stream_url: z.string(),
}).strict();

const replayPatchSchema = z.object({
  op: z.string(),
  path: z.string(),
  value: z.unknown().optional(),
}).strict();

const runtimeAgentArtifactSchema = z.object({
  version: z.string(),
  kind: z.string(),
  id: z.string(),
  metadata: jsonObject,
  runtime: jsonObject,
  policy: jsonObject,
  schemas: jsonObject,
  prompt: jsonObject,
  memory: jsonObject,
  tools: jsonObject,
  ui: jsonObject,
  extensions: jsonObject,
});

function parseResponse<T>(schema: z.ZodType<T>, value: unknown, label: string): T {
  const parsed = schema.safeParse(value);
  if (!parsed.success) {
    throw new Error(`${label} returned an invalid response`);
  }
  return parsed.data;
}

export function subscribeRunTraceSnapshot(
  runId: string,
  onSnapshot: (snapshot: PersistedRunSnapshot) => void,
): Promise<PersistedRunSnapshotSubscription> {
  return getDbInstance().subscribeRunSnapshot(runId, onSnapshot);
}

export async function fetchRunCheckpoints(runId: string): Promise<RunCheckpointListResponse> {
  const value = await requestRunCheckpoints(runId);
  return parseResponse(checkpointListSchema, value, "Checkpoint request");
}

export async function fetchRunSurfaceReplay(runId: string): Promise<RunReplayPatchOperation[]> {
  const value = await requestRunSurfaceReplay(runId);
  return parseResponse(z.array(replayPatchSchema), value, "Surface replay request");
}

export async function resolveRuntimeAgentArtifact(
  agentId: string,
): Promise<RuntimeAgentArtifact> {
  const agents = await fetchAgentsList();
  const agent = agents.find((candidate) => candidate.id === agentId);
  if (!agent) throw new Error(`Runtime agent ${agentId} was not found`);
  return parseResponse(runtimeAgentArtifactSchema, agent, "Agent catalog");
}

export async function resumeRunFromLatestCheckpoint(
  runId: string,
  input: ResumeRunInput,
): Promise<ResumeRunResponse> {
  const value = await requestLatestCheckpointResume(runId, input);
  return parseResponse(resumeResponseSchema, value, "Run resume request");
}
