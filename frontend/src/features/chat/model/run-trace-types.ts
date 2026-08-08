import type {
  PersistedRun,
  PersistedRunEvent,
  PersistedRunEventKind,
  PersistedRunSnapshot,
} from "@/platform/pglite/run-event-repository";
import type { A2uiProcessorState } from "@/features/a2ui/a2ui-surface-processor";

export const RUN_TRACE_PHASES = [
  "context",
  "skill",
  "memory",
  "retrieval",
  "reasoning",
  "tool",
  "generate",
] as const;

export type RunTraceCanonicalPhase = typeof RUN_TRACE_PHASES[number];
export type RunTracePhase = RunTraceCanonicalPhase | "lifecycle";

export interface RunTraceContext {
  runId: string;
  threadId: string | null;
  messageId: string | null;
  agentId: string | null;
  sessionId: string | null;
}

export interface RunCheckpoint {
  id: string;
  run_id: string;
  thread_id: string;
  node_id: string;
  iteration: number;
  state: unknown;
  messages: unknown[];
  created_at: string;
}

export interface RunCheckpointListResponse {
  run_id: string;
  checkpoints: RunCheckpoint[];
}

export interface RuntimeAgentArtifact {
  version: string;
  kind: string;
  id: string;
  metadata: Record<string, unknown>;
  runtime: Record<string, unknown>;
  policy: Record<string, unknown>;
  schemas: Record<string, unknown>;
  prompt: Record<string, unknown>;
  memory: Record<string, unknown>;
  tools: Record<string, unknown>;
  ui: Record<string, unknown>;
  extensions: Record<string, unknown>;
}

export interface ResumeRunInput {
  artifact: RuntimeAgentArtifact;
  input?: string;
  session_id?: string;
}

export interface ResumeRunResponse {
  resumed_from_run_id: string;
  run_id: string;
  stream_url: string;
}

export interface RunReplayPatchOperation {
  op: string;
  path: string;
  value?: unknown;
}

interface RunTraceNodeBase {
  id: string;
  parentId: string | null;
  label: string;
}

export interface RunTraceRootNode extends RunTraceNodeBase {
  type: "run";
  run: PersistedRun;
  childIds: string[];
}

export interface RunTracePhaseNode extends RunTraceNodeBase {
  type: "phase";
  phase: RunTracePhase;
  childIds: string[];
}

export interface RunTraceEventNode extends RunTraceNodeBase {
  type: "event";
  phase: RunTracePhase;
  event: PersistedRunEvent;
}

export type RunTraceTreeNode =
  | RunTraceRootNode
  | RunTracePhaseNode
  | RunTraceEventNode;

export interface VisibleTraceRow {
  id: string;
  node: RunTraceTreeNode;
  depth: number;
  parentId: string | null;
  positionInSet: number;
  setSize: number;
  expandable: boolean;
  expanded: boolean;
}

export interface RunEventTiming {
  start: string;
  gapMs: number;
  durationMs: number | null;
  durationSource: "explicit" | "correlated" | "instant";
}

export type RunTraceFilters = Record<PersistedRunEventKind, boolean>;

export interface RunTraceSegment {
  phase: RunTraceCanonicalPhase;
  durationMs: number;
  exactPercentage: number;
  visualWeight: number;
}

export type RunTraceActionStatus = "idle" | "loading" | "success" | "error";

export interface RunTraceActionState {
  status: RunTraceActionStatus;
  error: string | null;
}

export interface RunTraceNetworkState {
  snapshot: RunTraceActionState;
  checkpoints: RunTraceActionState;
  replay: RunTraceActionState;
  agent: RunTraceActionState;
  resume: RunTraceActionState;
}

export interface RunTraceProjectionInput extends PersistedRunSnapshot {
  filters: RunTraceFilters;
  expandedNodeIds: ReadonlySet<string>;
  selectedNodeId: string | null;
}

export interface RunTraceProjection {
  rootId: string | null;
  nodesById: Map<string, RunTraceTreeNode>;
  visibleRows: VisibleTraceRow[];
  countsByKind: Record<PersistedRunEventKind, number>;
  timingsByEventId: Map<string, RunEventTiming>;
  eventsById: Map<string, PersistedRunEvent>;
  messageNodeIds: Map<string, string[]>;
  phaseNodeIds: Map<RunTracePhase, string>;
  segments: RunTraceSegment[];
  selectedNodeId: string | null;
}

export interface RunReplayProjection {
  state: A2uiProcessorState;
  appliedOperations: number;
}
