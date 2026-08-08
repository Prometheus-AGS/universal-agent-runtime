import type { Chunk, SkillActivationChunk } from "@/shared/content/chunk";

import type { UarAguiEvent } from "./agui-schema";

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function text(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

function number(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function selectionMethod(value: unknown): SkillActivationChunk["selectionMethod"] {
  if (value === "keyword" || value === "embedding" || value === "hybrid" || value === "llm" || value === "explicit") return value;
  return undefined;
}

/** Projects one validated AG-UI event into the shared runtime chunk catalog. */
export function toRuntimeChunk(event: UarAguiEvent, receivedAt: number): Chunk | undefined {
  const base = {
    id: event.eventId,
    at: new Date(receivedAt).toISOString(),
    runId: text(event.runId) || undefined,
    seq: event.sequence,
  };
  switch (event.type) {
    case "TEXT_MESSAGE_CONTENT":
      return { ...base, kind: "text", text: text(event.delta) };
    case "REASONING_MESSAGE_CONTENT":
      return { ...base, kind: "reasoning", text: text(event.delta) };
    case "TOOL_CALL_END":
      return {
        ...base,
        kind: "tool-call",
        toolCallId: text(event.toolCallId, event.eventId),
        toolName: text(event.toolCallName, "Unknown tool"),
        args: record(event.arguments),
        status: "running",
      };
    case "TOOL_CALL_RESULT":
      return {
        ...base,
        kind: "tool-call",
        toolCallId: text(event.toolCallId, event.eventId),
        toolName: text(event.toolCallName, "Tool result"),
        args: {},
        result: typeof event.content === "string" ? event.content : JSON.stringify(event.content ?? null),
        status: "complete",
      };
    case "STATE_SNAPSHOT":
      return { ...base, kind: "state-snapshot", state: event.snapshot };
    case "STATE_DELTA":
      return { ...base, kind: "state-delta", delta: event.delta };
    case "STEP_STARTED":
    case "STEP_FINISHED":
      return { ...base, kind: "step", name: text(event.stepName, text(event.name, "Step")), status: event.type === "STEP_STARTED" ? "started" : "finished", durationMs: typeof event.durationMs === "number" ? event.durationMs : undefined };
    case "RUN_FINISHED": {
      const usage = record(record(event.result).usage);
      return {
        ...base,
        kind: "usage",
        inputTokens: number(usage.inputTokens),
        outputTokens: number(usage.outputTokens),
        totalTokens: number(usage.totalTokens),
        costUsd: typeof usage.costUsdEstimate === "number" ? usage.costUsdEstimate : undefined,
        model: typeof usage.model === "string" ? usage.model : undefined,
      };
    }
    case "RUN_ERROR":
      return { ...base, kind: "error", message: text(event.message, "Run failed"), code: typeof event.code === "string" ? event.code : undefined };
    case "RAW":
    case "MESSAGES_SNAPSHOT":
      return { ...base, kind: "raw", type: event.type, payload: event };
    case "CUSTOM":
      return customChunk(event, base);
    case "RUN_STARTED":
    case "TEXT_MESSAGE_START":
    case "TEXT_MESSAGE_END":
    case "REASONING_START":
    case "REASONING_MESSAGE_START":
    case "REASONING_MESSAGE_END":
    case "REASONING_END":
    case "TOOL_CALL_START":
    case "TOOL_CALL_ARGS":
      return undefined;
    default:
      return { ...base, kind: "raw", type: event.type, payload: event };
  }
}

function customChunk(
  event: UarAguiEvent,
  base: { id: string; at: string; runId?: string; seq: number },
): Chunk {
  const name = text(event.name, "CUSTOM");
  const value = record(event.value);
  switch (name) {
    case "uar.citation.added": {
      const citation = record(value.citation);
      return { ...base, kind: "citation", source: text(citation.title, text(citation.url, "Source")), content: text(citation.snippet), url: typeof citation.url === "string" ? citation.url : undefined };
    }
    case "uar.rag_citations": {
      const citations = Array.isArray(value.citations) ? value.citations : [];
      return {
        ...base,
        kind: "rag-citations",
        citations: citations.map((item, index) => {
          const citation = record(item);
          return {
            marker: number(citation.marker, index + 1),
            chunkId: text(citation.chunk_id, text(citation.chunkId, `${event.eventId}:${index}`)),
            documentId: text(citation.document_id, text(citation.documentId)) || undefined,
            documentName: text(citation.document_name, text(citation.documentName, "Document")),
            knowledgeBase: text(citation.knowledge_base, text(citation.knowledgeBase)) || undefined,
            relevanceScore: number(citation.relevance_score, number(citation.relevanceScore)),
            snippet: text(citation.snippet),
          };
        }),
      };
    }
    case "uar.memory.recall": {
      const items = Array.isArray(value.items) ? value.items : [];
      return {
        ...base,
        kind: "memory-recall",
        items: items.map((item, index) => {
          const memory = record(item);
          const rawType = text(memory.type, text(memory.memory_type, "semantic"));
          const type = rawType === "episodic" || rawType === "procedural" || rawType === "preference" ? rawType : "semantic";
          return { id: text(memory.id, text(memory.key, `${event.eventId}:${index}`)), content: text(memory.content, text(memory.value)), type, importance: typeof memory.importance === "number" ? memory.importance : undefined, score: typeof memory.score === "number" ? memory.score : undefined, pinned: typeof memory.pinned === "boolean" ? memory.pinned : undefined };
        }),
      };
    }
    case "uar.memory.mutation": {
      const rawOperation = text(value.operation, "update");
      const operation = rawOperation === "create" || rawOperation === "delete" || rawOperation === "pin" || rawOperation === "decay" ? rawOperation : "update";
      return { ...base, kind: "memory-mutation", operation, memoryId: text(value.memory_id, text(value.memoryId)), content: text(value.content) || undefined, memoryType: text(value.memory_type, text(value.memoryType)) || undefined, importance: typeof value.importance === "number" ? value.importance : undefined };
    }
    case "uar.skill.activated": {
      const skill = record(value.skill);
      return { ...base, kind: "skill-activation", skillId: text(skill.id, text(value.skill_id, name)), skillName: text(skill.title, text(value.skill_name, "Skill")), selectionMethod: selectionMethod(value.selection_method ?? value.selectionMethod), score: typeof value.score === "number" ? value.score : undefined, threshold: typeof value.threshold === "number" ? value.threshold : undefined, status: value.status === "complete" ? "complete" : "active" };
    }
    case "uar.context.updated":
      return { ...base, kind: "context-update", strategy: text(value.strategy, "unknown"), messagesRemoved: number(value.messages_removed, number(value.messagesRemoved)), tokensSaved: number(value.tokens_saved, number(value.tokensSaved)), wasApplied: value.was_applied === true || value.wasApplied === true, summaryGenerated: value.summary_generated === true || value.summaryGenerated === true };
    case "uar.tool.approval_required":
      return { ...base, kind: "tool-approval", toolCallId: text(value.tool_call_id, text(value.toolCallId, event.eventId)), toolName: text(value.tool_name, text(value.toolName, "Tool")), args: record(value.args), reason: text(value.reason) || undefined };
    case "uar.tool.denied":
      return { ...base, kind: "tool-denied", toolCallId: text(value.tool_call_id, text(value.toolCallId, event.eventId)), toolName: text(value.tool_name, text(value.toolName, "Tool")), reason: text(value.reason, "Denied by policy"), policy: text(value.policy) || undefined };
    case "uar.artifact.available": {
      const profile = text(value.profile);
      if (profile.startsWith("a2ui")) {
        return { ...base, kind: "a2ui-display", profile, component: text(value.component, text(value.artifact_type, "surface")), version: text(value.version) || undefined, payload: value.payload ?? value, validation: value.validation === "invalid" || value.validation === "unknown-component" ? value.validation : "valid", validationError: text(value.validation_error, text(value.validationError)) || undefined };
      }
      return { ...base, kind: "artifact", artifactId: text(value.artifact_id, text(value.artifactId, event.eventId)), title: text(value.title) || undefined, mime: text(value.mime, text(value.artifact_type, "application/octet-stream")), content: typeof value.content === "string" ? value.content : JSON.stringify(value.content ?? value), url: text(value.url) || undefined, bytes: typeof value.bytes === "number" ? value.bytes : undefined };
    }
    case "uar.artifact.input_required":
      return { ...base, kind: "a2ui-input", profile: text(value.profile, "a2ui/v0.9"), component: text(value.component, text(value.artifact_type, "input")), requestId: text(value.request_id, text(value.requestId, event.eventId)), payload: value.payload ?? value, status: "awaiting" };
    default:
      return { ...base, kind: "raw", type: name, payload: event.value };
  }
}
