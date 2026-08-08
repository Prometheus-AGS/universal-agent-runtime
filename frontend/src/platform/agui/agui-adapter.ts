import { isUarAguiEvent, type UarAguiEvent } from "@/platform/agui/agui-schema";
import {
  UarAguiNormalizer,
  type AguiConsumerProjections,
} from "@/platform/agui/agui-normalizer";

export interface AdaptedAguiEvent extends Partial<AguiConsumerProjections> {
  event: string;
  payload: Record<string, unknown>;
  eventId: string;
  sequence: number;
}

const LEGACY_HIGH_FREQUENCY = new Set([
  "agui.message.delta",
  "agui.thinking.delta",
  "agui.reasoning.delta",
  "agui.tool_call.delta",
]);

export function isHighFrequencyAguiEvent(event: string): boolean {
  return LEGACY_HIGH_FREQUENCY.has(event);
}

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function cloneJson(value: unknown): unknown {
  return value === undefined ? undefined : JSON.parse(JSON.stringify(value));
}

function pointerSegments(path: string): string[] | null {
  if (path === "") return [];
  if (!path.startsWith("/")) return null;
  return path.slice(1).split("/").map((part) => part.replaceAll("~1", "/").replaceAll("~0", "~"));
}

function applyJsonPatch(snapshot: unknown, patch: unknown[]): unknown | undefined {
  let next = cloneJson(snapshot);
  for (const rawOperation of patch) {
    const operation = record(rawOperation);
    const segments = typeof operation.path === "string" ? pointerSegments(operation.path) : null;
    if (!segments || !["add", "replace", "remove"].includes(String(operation.op))) return undefined;
    if (segments.length === 0) {
      if (operation.op === "remove") return undefined;
      next = cloneJson(operation.value);
      continue;
    }
    let parent: unknown = next;
    for (const segment of segments.slice(0, -1)) {
      if (!parent || typeof parent !== "object") return undefined;
      parent = (parent as Record<string, unknown>)[segment];
    }
    if (!parent || typeof parent !== "object") return undefined;
    const key = segments.at(-1)!;
    if (Array.isArray(parent)) {
      const index = key === "-" ? parent.length : Number(key);
      if (!Number.isInteger(index) || index < 0 || index > parent.length) return undefined;
      if (operation.op === "add") parent.splice(index, 0, cloneJson(operation.value));
      else if (operation.op === "replace" && index < parent.length) parent[index] = cloneJson(operation.value);
      else if (operation.op === "remove" && index < parent.length) parent.splice(index, 1);
      else return undefined;
    } else {
      const object = parent as Record<string, unknown>;
      if (operation.op === "remove") {
        if (!(key in object)) return undefined;
        delete object[key];
      } else {
        if (operation.op === "replace" && !(key in object)) return undefined;
        object[key] = cloneJson(operation.value);
      }
    }
  }
  return next;
}

function legacyEvent(event: string, payload: Record<string, unknown>, fallbackId: string): AdaptedAguiEvent {
  const sequence = typeof payload.sequence === "number" ? payload.sequence : Number.MAX_SAFE_INTEGER;
  const eventId = typeof payload.eventId === "string" ? payload.eventId : fallbackId;
  return { event, payload, eventId, sequence };
}

function customToLegacy(wire: UarAguiEvent): [string, Record<string, unknown>] {
  const value = record(wire.value);
  const request_id = String(wire.runId ?? "");
  switch (wire.name) {
    case "uar.citation.added":
      return ["agui.citation.added", { kind: "citation", phase: "added", request_id, citation: value.citation }];
    case "uar.rag_citations":
      return ["agui.rag_citations", { kind: "rag_citations", phase: "added", request_id, citations: value.citations }];
    case "uar.memory.recall":
      return ["agui.memory.recall", { kind: "memory", phase: "recall", request_id, ...value }];
    case "uar.memory.mutation":
      return ["agui.memory.mutation", { kind: "memory", phase: "mutation", request_id, ...value }];
    case "uar.artifact.available":
      return ["agui.artifact", { kind: "artifact", phase: "complete", request_id, ...value }];
    case "uar.artifact.input_required":
      return ["agui.artifact_input_request", { kind: "artifact_input_request", request_id, ...value }];
    case "uar.skill.activated":
      return ["agui.skill.activated", { kind: "skill", phase: "activated", request_id, ...value }];
    case "uar.context.updated":
      return ["agui.context.update", { kind: "context", phase: "update", request_id, ...value }];
    case "uar.tool.approval_required":
      return ["agui.tool_call.approval_required", { kind: "tool_call", phase: "approval_required", request_id, ...value }];
    case "uar.tool.denied":
      return ["agui.tool_call.denied", { kind: "tool_call", phase: "denied", request_id, ...value }];
    default:
      return ["agui.custom", { kind: "custom", request_id, name: wire.name, value }];
  }
}

function officialToLegacy(wire: UarAguiEvent): [string, Record<string, unknown>] | null {
  const request_id = String(wire.runId ?? "");
  switch (wire.type) {
    case "RUN_STARTED":
      return ["agui.stream.start", { kind: "stream", phase: "start", request_id, agent_id: record(wire.input).agentId }];
    case "RUN_FINISHED": {
      const usage = record(record(wire.result).usage);
      return ["agui.done", {
        kind: "done", request_id,
        usage: {
          input_tokens: usage.inputTokens,
          output_tokens: usage.outputTokens,
          total_tokens: usage.totalTokens,
          cost_usd_estimate: usage.costUsdEstimate,
          model: usage.model,
        },
      }];
    }
    case "RUN_ERROR":
      return wire.code === "CANCELLED"
        ? ["agui.cancelled", { kind: "cancelled", request_id }]
        : ["agui.error", { kind: "error", request_id, code: wire.code, message: wire.message }];
    case "TEXT_MESSAGE_CONTENT":
      return ["agui.message.delta", { kind: "message", phase: "delta", request_id, delta: { text: wire.delta } }];
    case "REASONING_MESSAGE_CONTENT":
      return ["agui.reasoning.delta", { kind: "reasoning", phase: "delta", request_id, delta: { text: wire.delta } }];
    case "TOOL_CALL_ARGS":
      return ["agui.tool_call.delta", { kind: "tool_call", phase: "delta", request_id, id: wire.toolCallId, delta: { arguments: wire.delta } }];
    case "TOOL_CALL_END":
      return ["agui.tool_call.complete", {
        kind: "tool_call", phase: "complete", request_id, id: wire.toolCallId,
        name: wire.toolCallName, arguments_json: JSON.stringify(wire.arguments ?? {}),
      }];
    case "TOOL_CALL_RESULT":
      return ["agui.tool_result", {
        kind: "tool_result", request_id, id: wire.toolCallId,
        content: wire.content, success: true,
      }];
    case "STATE_DELTA":
      return ["agui.state.patch", { kind: "state", phase: "patch", request_id, patch: wire.delta }];
    case "CUSTOM":
      return customToLegacy(wire);
    case "STATE_SNAPSHOT":
      return ["agui.state.snapshot", { kind: "state", phase: "snapshot", request_id, snapshot: wire.snapshot }];
    case "MESSAGES_SNAPSHOT":
      return ["agui.messages.snapshot", { ...wire }];
    case "RAW":
      return ["agui.raw", { ...wire }];
    case "TEXT_MESSAGE_START":
    case "TEXT_MESSAGE_END":
    case "REASONING_START":
    case "REASONING_MESSAGE_START":
    case "REASONING_MESSAGE_END":
    case "REASONING_END":
    case "TOOL_CALL_START":
    case "STEP_STARTED":
    case "STEP_FINISHED":
      return ["agui.lifecycle", { ...wire }];
    default:
      return null;
  }
}

/** Per-stream canonical adapter shared by Chat reduction and Runtime Console projection. */
export class UarAguiAdapter {
  private readonly seen = new Set<string>();
  private lastSequence = -1;
  private stateSnapshot: unknown;
  private stateSynchronized = false;
  private readonly normalizer: UarAguiNormalizer;

  constructor(clock: () => number = Date.now) {
    this.normalizer = new UarAguiNormalizer(clock);
  }

  ingest(event: string, data: string, sseId: number | undefined): AdaptedAguiEvent | null {
    let payload: Record<string, unknown>;
    try {
      payload = record(JSON.parse(data));
    } catch {
      return null;
    }

    const fallbackId = `${event}:${sseId ?? this.seen.size}`;
    let adapted: AdaptedAguiEvent;
    if (event.startsWith("agui.")) {
      adapted = legacyEvent(event, payload, fallbackId);
    } else if (isUarAguiEvent(payload)) {
      const mapped = officialToLegacy(payload);
      if (!mapped) return null;
      adapted = legacyEvent(mapped[0], mapped[1], payload.eventId);
      adapted.sequence = payload.sequence;
      adapted.eventId = payload.eventId;
    } else {
      return null;
    }

    if (this.seen.has(adapted.eventId)) return null;
    this.seen.add(adapted.eventId);
    // Never allow a late replay frame to regress already-reduced UI state.
    if (adapted.sequence < this.lastSequence) return null;
    this.lastSequence = adapted.sequence;

    if (isUarAguiEvent(payload)) {
      Object.assign(adapted, this.normalizer.normalize(payload));
    }

    if (event === "STATE_SNAPSHOT") {
      this.stateSnapshot = cloneJson((payload as UarAguiEvent).snapshot);
      this.stateSynchronized = true;
      adapted.payload.snapshot = cloneJson(this.stateSnapshot);
      adapted.payload.synchronized = true;
    } else if (event === "STATE_DELTA") {
      const delta = (payload as UarAguiEvent).delta as unknown[];
      if (this.stateSynchronized) {
        const patched = applyJsonPatch(this.stateSnapshot, delta);
        this.stateSynchronized = patched !== undefined;
        if (this.stateSynchronized) this.stateSnapshot = patched;
      }
      adapted.payload.snapshot = cloneJson(this.stateSnapshot);
      adapted.payload.synchronized = this.stateSynchronized;
    }
    return adapted;
  }
}
