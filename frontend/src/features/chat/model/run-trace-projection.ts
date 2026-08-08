import { A2UI_PROFILE, A2UI_VERSION, validateA2uiMessage } from "@/features/a2ui/a2ui-protocol";
import {
  EMPTY_A2UI_PROCESSOR_STATE,
  reduceA2uiMessage,
} from "@/features/a2ui/a2ui-surface-processor";
import type {
  PersistedRunEvent,
  PersistedRunEventKind,
} from "@/platform/pglite/run-event-repository";
import { phaseOfAguiEvent } from "@/platform/agui/agui-normalizer";
import type { UarAguiEvent } from "@/platform/agui/agui-schema";
import {
  RUN_TRACE_PHASES,
  type RunEventTiming,
  type RunReplayPatchOperation,
  type RunReplayProjection,
  type RunTraceEventNode,
  type RunTraceFilters,
  type RunTracePhase,
  type RunTracePhaseNode,
  type RunTraceProjection,
  type RunTraceProjectionInput,
  type RunTraceRootNode,
  type RunTraceSegment,
  type RunTraceTreeNode,
  type VisibleTraceRow,
} from "@/features/chat/model/run-trace-types";

const EVENT_KINDS: readonly PersistedRunEventKind[] = [
  "lifecycle",
  "message",
  "reasoning",
  "tool",
  "state",
  "custom",
  "raw",
];

const SURFACE_PATH = /^\/a2ui\/surfaces\/([^/]+)(?:\/(components|dataModel))?$/;

export const DEFAULT_RUN_TRACE_FILTERS: RunTraceFilters = {
  lifecycle: true,
  message: true,
  reasoning: true,
  tool: true,
  state: true,
  custom: true,
  raw: true,
};

interface SpanStart {
  eventId: string;
  atMs: number;
}

function phaseOfEvent(event: PersistedRunEvent): RunTracePhase {
  const payload = {
    ...event.payload,
    type: event.type,
    eventId: event.eventId,
    sequence: event.wireSequence,
    profile: "uar.agui/1",
  } as UarAguiEvent;
  return phaseOfAguiEvent(payload) ?? "lifecycle";
}

function eventLabel(event: PersistedRunEvent): string {
  const summary = event.payload.name ?? event.payload.messageId ?? event.payload.toolCallId;
  return typeof summary === "string" && summary.length > 0
    ? `${event.type} · ${summary}`
    : event.type;
}

function messageIdentity(event: PersistedRunEvent): string | null {
  const value = event.payload.messageId ?? event.payload.message_id;
  return typeof value === "string" && value.length > 0 ? value : null;
}

function correlation(event: PersistedRunEvent): { action: "start" | "end"; key: string } | null {
  const type = event.type;
  const payload = event.payload;
  const id = (...keys: string[]): string => {
    for (const key of keys) {
      const value = payload[key];
      if (typeof value === "string" && value.length > 0) return value;
    }
    return event.runId;
  };

  if (type === "RUN_STARTED") return { action: "start", key: `run:${event.runId}` };
  if (type === "RUN_FINISHED" || type === "RUN_ERROR") {
    return { action: "end", key: `run:${event.runId}` };
  }
  if (type === "STEP_STARTED") return { action: "start", key: `step:${id("stepId", "stepName", "name")}` };
  if (type === "STEP_FINISHED") return { action: "end", key: `step:${id("stepId", "stepName", "name")}` };
  if (type === "TEXT_MESSAGE_START") return { action: "start", key: `message:${id("messageId", "message_id")}` };
  if (type === "TEXT_MESSAGE_END") return { action: "end", key: `message:${id("messageId", "message_id")}` };
  if (type === "REASONING_START") return { action: "start", key: `reasoning:${event.runId}` };
  if (type === "REASONING_END") return { action: "end", key: `reasoning:${event.runId}` };
  if (type === "REASONING_MESSAGE_START") return { action: "start", key: `reasoning-message:${id("messageId", "message_id")}` };
  if (type === "REASONING_MESSAGE_END") return { action: "end", key: `reasoning-message:${id("messageId", "message_id")}` };
  if (type === "TOOL_CALL_START") return { action: "start", key: `tool:${id("toolCallId", "tool_call_id")}` };
  if (type === "TOOL_CALL_END" || type === "TOOL_CALL_RESULT") {
    return { action: "end", key: `tool:${id("toolCallId", "tool_call_id")}` };
  }
  return null;
}

function explicitDuration(event: PersistedRunEvent): number | null {
  const value = event.payload.durationMs ?? event.payload.duration_ms;
  return typeof value === "number" && Number.isFinite(value) && value >= 0 ? value : null;
}

function buildTimings(events: PersistedRunEvent[]): Map<string, RunEventTiming> {
  const timings = new Map<string, RunEventTiming>();
  const openSpans = new Map<string, SpanStart>();
  let precedingAtMs: number | null = null;

  for (const event of events) {
    const atMs = Date.parse(event.at);
    const explicit = explicitDuration(event);
    timings.set(event.eventId, {
      start: event.at,
      gapMs: precedingAtMs === null || !Number.isFinite(atMs)
        ? 0
        : Math.max(0, atMs - precedingAtMs),
      durationMs: explicit,
      durationSource: explicit === null ? "instant" : "explicit",
    });
    if (Number.isFinite(atMs)) precedingAtMs = atMs;

    const span = correlation(event);
    if (!span || !Number.isFinite(atMs)) continue;
    if (span.action === "start") {
      openSpans.set(span.key, { eventId: event.eventId, atMs });
      continue;
    }
    const start = openSpans.get(span.key);
    if (!start) continue;
    openSpans.delete(span.key);
    const startTiming = timings.get(start.eventId);
    if (startTiming?.durationSource === "instant") {
      timings.set(start.eventId, {
        ...startTiming,
        durationMs: Math.max(0, atMs - start.atMs),
        durationSource: "correlated",
      });
    }
  }
  return timings;
}

export function buildRunTraceSegments(
  timings: Record<string, number>,
): RunTraceSegment[] {
  const present = RUN_TRACE_PHASES
    .map((phase) => ({ phase, durationMs: Math.max(0, Number(timings[phase] ?? 0)) }))
    .filter((entry) => entry.durationMs > 0);
  const total = present.reduce((sum, entry) => sum + entry.durationMs, 0);
  return present.map((entry) => {
    const exactPercentage = total === 0 ? 0 : (entry.durationMs / total) * 100;
    return {
      ...entry,
      exactPercentage,
      visualWeight: Math.max(3, exactPercentage),
    };
  });
}

function flattenVisibleRows(
  root: RunTraceRootNode,
  nodesById: Map<string, RunTraceTreeNode>,
  expandedNodeIds: ReadonlySet<string>,
): VisibleTraceRow[] {
  const rows: VisibleTraceRow[] = [];

  const appendChildren = (parent: RunTraceRootNode | RunTracePhaseNode, depth: number): void => {
    const setSize = parent.childIds.length;
    for (const [index, childId] of parent.childIds.entries()) {
      const node = nodesById.get(childId);
      if (!node) continue;
      const expandable = node.type !== "event" && node.childIds.length > 0;
      const expanded = expandable && expandedNodeIds.has(node.id);
      rows.push({
        id: node.id,
        node,
        depth,
        parentId: parent.id,
        positionInSet: index + 1,
        setSize,
        expandable,
        expanded,
      });
      if (node.type === "phase" && expanded) appendChildren(node, depth + 1);
    }
  };

  rows.push({
    id: root.id,
    node: root,
    depth: 1,
    parentId: null,
    positionInSet: 1,
    setSize: 1,
    expandable: root.childIds.length > 0,
    expanded: expandedNodeIds.has(root.id),
  });
  if (expandedNodeIds.has(root.id)) appendChildren(root, 2);
  return rows;
}

export function projectRunTrace(input: RunTraceProjectionInput): RunTraceProjection {
  const nodesById = new Map<string, RunTraceTreeNode>();
  const eventsById = new Map<string, PersistedRunEvent>();
  const messageNodeIds = new Map<string, string[]>();
  const phaseNodeIds = new Map<RunTracePhase, string>();
  const phaseNodes = new Map<RunTracePhase, RunTracePhaseNode>();
  const countsByKind = Object.fromEntries(EVENT_KINDS.map((kind) => [kind, 0])) as Record<
    PersistedRunEventKind,
    number
  >;
  const timingsByEventId = buildTimings(input.events);

  if (!input.run) {
    return {
      rootId: null,
      nodesById,
      visibleRows: [],
      countsByKind,
      timingsByEventId,
      eventsById,
      messageNodeIds,
      phaseNodeIds,
      segments: [],
      selectedNodeId: null,
    };
  }

  const rootId = `run:${input.run.id}`;
  const root: RunTraceRootNode = {
    id: rootId,
    parentId: null,
    type: "run",
    label: `Run ${input.run.id}`,
    run: input.run,
    childIds: [],
  };
  nodesById.set(rootId, root);

  for (const event of input.events) {
    countsByKind[event.kind] += 1;
    eventsById.set(event.eventId, event);
    if (!input.filters[event.kind]) continue;

    const phase = phaseOfEvent(event);
    let phaseNode = phaseNodes.get(phase);
    if (!phaseNode) {
      const phaseId = `phase:${phase}`;
      phaseNode = {
        id: phaseId,
        parentId: rootId,
        type: "phase",
        phase,
        label: phase,
        childIds: [],
      };
      phaseNodes.set(phase, phaseNode);
      phaseNodeIds.set(phase, phaseId);
      root.childIds.push(phaseId);
      nodesById.set(phaseId, phaseNode);
    }

    const eventNodeId = `event:${event.eventId}`;
    const eventNode: RunTraceEventNode = {
      id: eventNodeId,
      parentId: phaseNode.id,
      type: "event",
      phase,
      label: eventLabel(event),
      event,
    };
    phaseNode.childIds.push(eventNodeId);
    nodesById.set(eventNodeId, eventNode);

    const messageId = messageIdentity(event);
    if (messageId) {
      const current = messageNodeIds.get(messageId) ?? [];
      current.push(eventNodeId);
      messageNodeIds.set(messageId, current);
    }
  }

  const visibleRows = flattenVisibleRows(root, nodesById, input.expandedNodeIds);
  const selectedNodeId = input.selectedNodeId && visibleRows.some((row) => row.id === input.selectedNodeId)
    ? input.selectedNodeId
    : visibleRows.find((row) => row.node.type === "event")?.id ?? visibleRows[0]?.id ?? null;

  return {
    rootId,
    nodesById,
    visibleRows,
    countsByKind,
    timingsByEventId,
    eventsById,
    messageNodeIds,
    phaseNodeIds,
    segments: buildRunTraceSegments(input.run.phaseTimings),
    selectedNodeId,
  };
}

function replayEnvelope(operation: RunReplayPatchOperation): unknown {
  const match = SURFACE_PATH.exec(operation.path);
  if (!match) throw new Error(`Invalid A2UI replay path: ${operation.path}`);
  const surfaceId = match[1];
  const suffix = match[2];
  const base = { version: A2UI_VERSION, profile: A2UI_PROFILE };

  if (operation.op === "add" && suffix === undefined) {
    return {
      ...base,
      createSurface: { surfaceId, ...(operation.value as Record<string, unknown>) },
    };
  }
  if (operation.op === "replace" && suffix === "components") {
    return {
      ...base,
      updateComponents: { surfaceId, ...(operation.value as Record<string, unknown>) },
    };
  }
  if (operation.op === "replace" && suffix === "dataModel") {
    return {
      ...base,
      updateDataModel: { surfaceId, ...(operation.value as Record<string, unknown>) },
    };
  }
  if (operation.op === "remove" && suffix === undefined) {
    return { ...base, deleteSurface: { surfaceId } };
  }
  throw new Error(`Unsupported A2UI replay operation: ${operation.op} ${operation.path}`);
}

export function projectA2uiReplay(
  operations: RunReplayPatchOperation[],
): RunReplayProjection {
  let state = EMPTY_A2UI_PROCESSOR_STATE;
  for (const [index, operation] of operations.entries()) {
    const envelope = replayEnvelope(operation);
    const validation = validateA2uiMessage(envelope);
    if (!validation.success) {
      throw new Error(`Invalid A2UI replay operation ${index}: ${validation.error}`);
    }
    state = reduceA2uiMessage(state, validation.data);
    if (state.error) throw new Error(`Invalid A2UI replay operation ${index}: ${state.error}`);
  }
  return { state, appliedOperations: operations.length };
}
