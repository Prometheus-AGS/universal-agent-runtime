import type { UarAguiEvent } from "@/platform/agui/agui-schema";
import type { Chunk } from "@/shared/content/chunk";

import { toRuntimeChunk } from "./runtime-chunk-projection";

export const RUN_PHASES = [
  "context",
  "skill",
  "memory",
  "retrieval",
  "reasoning",
  "tool",
  "generate",
] as const;

export type RunPhase = (typeof RUN_PHASES)[number];
export type RunPhaseTimings = Record<RunPhase, number>;

export interface AguiMessageChunk {
  kind: "text" | "reasoning";
  messageId: string;
  delta: string;
}

export interface AguiEventRow {
  id: string;
  sequence: number;
  type: string;
  runId?: string;
  phase: RunPhase | null;
  receivedAt: number;
  payload: UarAguiEvent;
}

export interface AguiConsumerProjections {
  messageChunk?: AguiMessageChunk;
  runtimeChunk?: Chunk;
  eventRow: AguiEventRow;
  phaseTimings?: RunPhaseTimings;
}

interface PhaseSpan {
  first: number;
  last: number;
}

type Clock = () => number;

function customPhase(name: unknown): RunPhase | null {
  if (typeof name !== "string") return null;
  if (name === "uar.context.updated") return "context";
  if (name === "uar.skill.activated") return "skill";
  if (name.startsWith("uar.memory.")) return "memory";
  if (name === "uar.citation.added" || name === "uar.rag_citations") return "retrieval";
  if (name.startsWith("uar.tool.")) return "tool";
  return null;
}

export function phaseOfAguiEvent(event: UarAguiEvent): RunPhase | null {
  if (event.type === "CUSTOM") return customPhase(event.name);
  if (event.type.startsWith("TEXT_MESSAGE_")) return "generate";
  if (event.type.startsWith("REASONING_")) return "reasoning";
  if (event.type.startsWith("TOOL_CALL_")) return "tool";
  if (event.type === "STATE_SNAPSHOT" || event.type === "STATE_DELTA") return "context";
  return null;
}

function messageChunk(event: UarAguiEvent): AguiMessageChunk | undefined {
  if (
    event.type !== "TEXT_MESSAGE_CONTENT" &&
    event.type !== "REASONING_MESSAGE_CONTENT"
  ) return undefined;
  if (typeof event.delta !== "string" || event.delta.length === 0) return undefined;
  return {
    kind: event.type === "TEXT_MESSAGE_CONTENT" ? "text" : "reasoning",
    messageId: typeof event.messageId === "string" ? event.messageId : "",
    delta: event.delta,
  };
}

function emptyTimings(): RunPhaseTimings {
  return {
    context: 0,
    skill: 0,
    memory: 0,
    retrieval: 0,
    reasoning: 0,
    tool: 0,
    generate: 0,
  };
}

/** Stateful, per-stream projection of validated UAR AG-UI profile frames. */
export class UarAguiNormalizer {
  private readonly spans = new Map<RunPhase, PhaseSpan>();
  private runStartedAt: number | undefined;
  private firstObservedAt: number | undefined;
  private terminalTimingsEmitted = false;

  constructor(private readonly clock: Clock = Date.now) {}

  normalize(event: UarAguiEvent): AguiConsumerProjections {
    const receivedAt = this.clock();
    this.firstObservedAt ??= receivedAt;
    if (event.type === "RUN_STARTED") this.runStartedAt = receivedAt;

    const phase = phaseOfAguiEvent(event);
    if (phase) {
      const span = this.spans.get(phase);
      if (span) span.last = receivedAt;
      else this.spans.set(phase, { first: receivedAt, last: receivedAt });
    }

    const projection: AguiConsumerProjections = {
      eventRow: {
        id: event.eventId,
        sequence: event.sequence,
        type: event.type,
        runId: typeof event.runId === "string" ? event.runId : undefined,
        phase,
        receivedAt,
        payload: event,
      },
    };
    const chunk = messageChunk(event);
    if (chunk) projection.messageChunk = chunk;
    const runtimeChunk = toRuntimeChunk(event, receivedAt);
    if (runtimeChunk) projection.runtimeChunk = runtimeChunk;

    if (
      !this.terminalTimingsEmitted &&
      (event.type === "RUN_FINISHED" || event.type === "RUN_ERROR")
    ) {
      projection.phaseTimings = this.completeTimings(receivedAt);
      this.terminalTimingsEmitted = true;
    }
    return projection;
  }

  private completeTimings(finishedAt: number): RunPhaseTimings {
    const startedAt = Math.min(this.runStartedAt ?? this.firstObservedAt ?? finishedAt, finishedAt);
    const runDuration = Math.max(0, finishedAt - startedAt);
    const timings = emptyTimings();

    for (const [phase, span] of this.spans) {
      const first = Math.max(startedAt, Math.min(span.first, finishedAt));
      const last = Math.max(first, Math.min(span.last, finishedAt));
      timings[phase] = last - first;
    }

    const attributed = RUN_PHASES.reduce((total, phase) => total + timings[phase], 0);
    timings.generate += Math.max(0, runDuration - attributed);
    return timings;
  }
}
