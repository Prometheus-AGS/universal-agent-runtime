import { useGraphStore } from "@/platform/entities";
import type { EntityType } from "@/platform/entities";
import { ingestAgUiEvent, ingestRuntimeEvent } from "./runtime-ingest";
import { UarAguiAdapter } from "@/platform/agui/agui-adapter";

export interface RuntimeReplayEvent {
  type: string;
  id?: string;
  run_id?: string;
  sequence?: number;
  data?: Record<string, unknown>;
  payload?: Record<string, unknown>;
}

export const RUNTIME_REPLAY_RUN_ID = "replay-run-001";
export const RUNTIME_REPLAY_STEP_ID = "replay-step-001";
export const RUNTIME_REPLAY_TOOL_ID = "replay-tool-001";
export const RUNTIME_REPLAY_APPROVAL_ID = "replay-approval-001";
export const RUNTIME_REPLAY_ARTIFACT_ID = "replay-artifact-001";
export const RUNTIME_REPLAY_MEMORY_ID = "replay-memory-001";
export const RUNTIME_REPLAY_WORKFLOW_MIRROR_MEMORY_ID = "replay-memory-workflow-mirror-001";
export const RUNTIME_REPLAY_ROUTE_ID = "replay-route-001";
export const RUNTIME_REPLAY_PROVIDER_ID = "openai";
export const RUNTIME_REPLAY_A2UI_ID = "replay-a2ui-001";
export const RUNTIME_REPLAY_AGUI_ID = "replay-agui-001";

const timestamp = "2026-04-26T07:00:00.000Z";

export const runtimeReplayEvents: RuntimeReplayEvent[] = [
  {
    type: "run_started",
    id: RUNTIME_REPLAY_RUN_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 1,
    payload: {
      title: "Live Replay Run",
      thread_id: "replay-thread-001",
      agent_id: "runtime-console-agent",
      model: "gpt-5.4",
      provider_id: RUNTIME_REPLAY_PROVIDER_ID,
      status: "running",
      started_at: timestamp,
      updated_at: timestamp,
    },
  },
  {
    type: "run_updated",
    id: RUNTIME_REPLAY_RUN_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 2,
    payload: {
      status: "waiting",
      title: "Live Replay Run",
    },
  },
  {
    type: "step_started",
    id: RUNTIME_REPLAY_STEP_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 3,
    data: {
      kind: "tool",
      status: "running",
      title: "Replay tool execution",
      summary: "Calling provider health probe",
    },
  },
  {
    type: "tool_call_started",
    id: RUNTIME_REPLAY_TOOL_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 4,
    payload: {
      step_id: RUNTIME_REPLAY_STEP_ID,
      tool_name: "provider.health.check",
      namespace: "runtime",
      status: "running",
      input: { provider_id: RUNTIME_REPLAY_PROVIDER_ID },
    },
  },
  {
    type: "approval_requested",
    id: RUNTIME_REPLAY_APPROVAL_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 5,
    payload: {
      tool_call_id: RUNTIME_REPLAY_TOOL_ID,
      status: "pending",
      reason: "Replay requires approval for provider diagnostics",
    },
  },
  {
    type: "artifact_created",
    id: RUNTIME_REPLAY_ARTIFACT_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 6,
    payload: {
      kind: "json",
      title: "Replay Diagnostics Artifact",
      uri: "memory://runtime/replay-artifact-001",
      mime_type: "application/json",
    },
  },
  {
    type: "memory_recalled",
    id: RUNTIME_REPLAY_MEMORY_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 7,
    payload: {
      memory_id: "workflow-memory-001",
      action: "recall",
      summary: "Loaded runtime-console-validation-hardening waypoint",
      source_tool: "codex",
    },
  },
  {
    type: "memory_updated",
    id: RUNTIME_REPLAY_WORKFLOW_MIRROR_MEMORY_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 8,
    payload: {
      memory_id: "workflow-mirror-memory-001",
      action: "workflow_mirror",
      summary: "Mirrored KBD waypoint for surreal-memory-workflow-mirror-tests",
      source_tool: "codex",
      workflow_kind: "waypoint",
      workflow_id: "current-waypoint",
      phase: "runtime-console-validation-hardening",
      change: "surreal-memory-workflow-mirror-tests",
      source_path: ".kbd-orchestrator/current-waypoint.json",
      updated_at: timestamp,
    },
  },
  {
    type: "provider_health",
    id: "provider-health-openai",
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 9,
    payload: {
      provider_id: RUNTIME_REPLAY_PROVIDER_ID,
      status: "healthy",
      latency_ms: 42,
    },
  },
  {
    type: "model_route_decision",
    id: RUNTIME_REPLAY_ROUTE_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 10,
    payload: {
      selected_model: "gpt-5.4",
      selected_provider: RUNTIME_REPLAY_PROVIDER_ID,
      needs_tools: true,
      needs_vision: false,
      min_context: 128000,
      reason: "Replay route selected for tool-capable runtime validation",
    },
  },
  {
    type: "a2ui_surface",
    id: RUNTIME_REPLAY_A2UI_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 11,
    payload: {
      schema_id: "a2ui/display",
      title: "Replay A2UI Surface",
      status: "rendered",
      payload: {
        body: "Replay payload rendered from entity graph",
      },
    },
  },
  {
    type: "tool_call_finished",
    id: RUNTIME_REPLAY_TOOL_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 12,
    payload: {
      status: "completed",
      output: { ok: true },
    },
  },
  {
    type: "step_finished",
    id: RUNTIME_REPLAY_STEP_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 13,
    payload: {
      status: "completed",
      summary: "Provider diagnostics completed",
    },
  },
];

export const runtimeReplayUpdateEvents: RuntimeReplayEvent[] = [
  {
    type: "approval_updated",
    id: RUNTIME_REPLAY_APPROVAL_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 13,
    payload: {
      status: "approved",
      reason: "Replay approval accepted",
    },
  },
  {
    type: "a2ui_surface",
    id: RUNTIME_REPLAY_A2UI_ID,
    run_id: RUNTIME_REPLAY_RUN_ID,
    sequence: 14,
    payload: {
      title: "Replay A2UI Surface",
      status: "rendered",
      payload: {
        body: "Replay payload updated from chunk",
      },
    },
  },
];

export const agUiReplayEvents: RuntimeReplayEvent[] = [
  {
    type: "RUN_STARTED",
    id: RUNTIME_REPLAY_AGUI_ID,
    sequence: 1,
    payload: {
      profile: "uar.agui/1",
      runId: RUNTIME_REPLAY_RUN_ID,
      input: { agentId: "replay-agent" },
    },
  },
  {
    type: "TOOL_CALL_ARGS",
    id: "replay-agui-002",
    sequence: 2,
    payload: {
      profile: "uar.agui/1",
      runId: RUNTIME_REPLAY_RUN_ID,
      toolCallId: RUNTIME_REPLAY_TOOL_ID,
      delta: "provider health chunk",
    },
  },
];

export const runtimeReplayEntityTypes: EntityType[] = [
  "RuntimeRun",
  "RuntimeRunStep",
  "RuntimeToolCall",
  "RuntimeApproval",
  "RuntimeArtifact",
  "RuntimeMemoryEvent",
  "RuntimeAgUiEvent",
  "RuntimeA2uiSurface",
  "RuntimeModelRouteDecision",
  "RuntimeProviderHealth",
];

export function resetRuntimeReplayGraph() {
  useGraphStore.setState({
    entities: {},
    patches: {},
    entityStates: {},
    syncMetadata: {},
    lists: {},
  });
}

export function replayRuntimeEvents(events = runtimeReplayEvents) {
  for (const event of events) {
    ingestRuntimeEvent(event);
  }
}

export function replayAgUiEvents(events = agUiReplayEvents) {
  const adapter = new UarAguiAdapter();
  for (const event of events) {
    const adapted = adapter.ingest(
      event.type,
      JSON.stringify({
        ...event.payload,
        type: event.type,
        eventId: event.id,
        sequence: event.sequence,
      }),
      event.sequence,
    );
    if (adapted?.eventRow) ingestAgUiEvent(RUNTIME_REPLAY_RUN_ID, adapted.eventRow);
  }
}

export function replayAllRuntimeFixtures() {
  replayRuntimeEvents();
  replayAgUiEvents();
}

export function replayRuntimeUpdates() {
  replayRuntimeEvents(runtimeReplayUpdateEvents);
}
