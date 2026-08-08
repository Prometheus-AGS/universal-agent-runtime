import type { RunPhase } from "../../platform/agui/agui-normalizer";

export type ChunkKind =
  | "text" | "markdown"
  | "reasoning" | "thinking"
  | "tool-call" | "tool-approval" | "tool-denied"
  | "skill-activation"
  | "memory-recall" | "memory-mutation" | "memory-update"
  | "citation" | "rag-citations"
  | "context-update"
  | "a2ui-display" | "a2ui-input"
  | "artifact"
  | "image" | "video" | "file" | "divider"
  | "state-snapshot" | "state-delta"
  | "step" | "usage" | "error" | "raw";

export interface ChunkBase {
  id: string;
  kind: ChunkKind;
  at: string;
  runId?: string;
  seq: number;
}

export interface TextChunk extends ChunkBase { kind: "text"; text: string }
export interface MarkdownChunk extends ChunkBase { kind: "markdown"; source: string }
export interface ReasoningChunk extends ChunkBase { kind: "reasoning"; text: string; signature?: string; redacted?: boolean }
export interface ThinkingChunk extends ChunkBase { kind: "thinking"; text: string; budgetTokens?: number; usedTokens?: number }
export interface ToolCallChunk extends ChunkBase {
  kind: "tool-call";
  toolCallId: string;
  toolName: string;
  server?: string;
  transport?: "stdio" | "sse" | "http";
  args: Record<string, unknown>;
  argsPartial?: string;
  result?: string;
  resultMime?: string;
  status: "pending" | "running" | "complete" | "failed" | "cancelled";
  durationMs?: number;
  riskClass?: "read" | "write" | "destructive" | "sandbox";
}
export interface ToolApprovalChunk extends ChunkBase {
  kind: "tool-approval";
  toolCallId: string;
  toolName: string;
  args: Record<string, unknown>;
  reason?: string;
  decision?: "approved" | "denied";
  decidedAt?: string;
}
export interface ToolDeniedChunk extends ChunkBase { kind: "tool-denied"; toolCallId: string; toolName: string; reason: string; policy?: string }
export interface SkillActivationChunk extends ChunkBase {
  kind: "skill-activation";
  skillId: string;
  skillName: string;
  selectionMethod?: "keyword" | "embedding" | "hybrid" | "llm" | "explicit";
  score?: number;
  threshold?: number;
  status: "active" | "complete";
}
export interface MemoryRecallChunk extends ChunkBase {
  kind: "memory-recall";
  items: Array<{ id: string; content: string; type: "episodic" | "semantic" | "procedural" | "preference"; importance?: number; score?: number; pinned?: boolean }>;
}
export interface MemoryMutationChunk extends ChunkBase {
  kind: "memory-mutation";
  operation: "create" | "update" | "delete" | "pin" | "decay";
  memoryId: string;
  content?: string;
  memoryType?: string;
  importance?: number;
}
export interface MemoryUpdateChunk extends ChunkBase { kind: "memory-update"; scope: "session" | "agent" | "global"; summary: string; itemCount: number }
export interface CitationChunk extends ChunkBase { kind: "citation"; source: string; content: string; url?: string }
export interface RagCitationsChunk extends ChunkBase {
  kind: "rag-citations";
  citations: Array<{ marker: number; chunkId: string; documentId?: string; documentName: string; knowledgeBase?: string; relevanceScore: number; snippet: string; page?: number; span?: [number, number] }>;
}
export interface ContextUpdateChunk extends ChunkBase {
  kind: "context-update";
  strategy: string;
  messagesRemoved: number;
  tokensSaved: number;
  wasApplied: boolean;
  summaryGenerated: boolean;
  windowBefore?: number;
  windowAfter?: number;
  summarizerModel?: string;
}
export interface A2uiDisplayChunk extends ChunkBase { kind: "a2ui-display"; toolCallId?: string; profile: string; component: string; version?: string; payload: unknown; validation: "valid" | "invalid" | "unknown-component"; validationError?: string }
export interface A2uiInputChunk extends ChunkBase { kind: "a2ui-input"; toolCallId?: string; profile: string; component: string; requestId: string; payload: unknown; status: "awaiting" | "submitted" | "expired" | "cancelled"; response?: unknown }
export interface ArtifactChunk extends ChunkBase { kind: "artifact"; artifactId: string; title?: string; mime: string; content?: string; url?: string; bytes?: number }
export interface ImageChunk extends ChunkBase { kind: "image"; url: string; alt?: string; width?: number; height?: number; sourcePath?: string }
export interface VideoChunk extends ChunkBase { kind: "video"; url: string; poster?: string; durationMs?: number }
export interface FileChunk extends ChunkBase { kind: "file"; name: string; mime: string; bytes: number; url?: string }
export interface DividerChunk extends ChunkBase { kind: "divider" }
export interface StateSnapshotChunk extends ChunkBase { kind: "state-snapshot"; state: unknown }
export interface StateDeltaChunk extends ChunkBase { kind: "state-delta"; delta: unknown }
export interface StepChunk extends ChunkBase { kind: "step"; name: string; status: "started" | "finished"; durationMs?: number }
export interface UsageChunk extends ChunkBase { kind: "usage"; inputTokens: number; outputTokens: number; totalTokens: number; costUsd?: number; model?: string }
export interface ErrorChunk extends ChunkBase { kind: "error"; message: string; code?: string; retryable?: boolean; attempt?: number; maxAttempts?: number }
export interface RawChunk extends ChunkBase { kind: "raw"; type: string; payload: unknown }

export type Chunk =
  | TextChunk | MarkdownChunk | ReasoningChunk | ThinkingChunk
  | ToolCallChunk | ToolApprovalChunk | ToolDeniedChunk
  | SkillActivationChunk | MemoryRecallChunk | MemoryMutationChunk | MemoryUpdateChunk
  | CitationChunk | RagCitationsChunk | ContextUpdateChunk
  | A2uiDisplayChunk | A2uiInputChunk | ArtifactChunk
  | ImageChunk | VideoChunk | FileChunk | DividerChunk
  | StateSnapshotChunk | StateDeltaChunk | StepChunk | UsageChunk | ErrorChunk | RawChunk;

export type ChunkRendererName =
  | "text" | "reasoning" | "tool" | "approval" | "denied" | "skill" | "memory"
  | "citation" | "context" | "a2ui" | "artifact" | "media" | "divider"
  | "usage" | "error" | "hidden";

export type ChunkTraceDisposition = "segment" | "row" | "tick" | "inspector" | "footer" | "none";

export const CHUNK_PHASE = {
  text: "generate", markdown: "generate", reasoning: "reasoning", thinking: "reasoning",
  "tool-call": "tool", "tool-approval": "tool", "tool-denied": "tool",
  "skill-activation": "skill", "memory-recall": "memory", "memory-mutation": "memory", "memory-update": "memory",
  citation: "retrieval", "rag-citations": "retrieval", "context-update": "context",
  "a2ui-display": null, "a2ui-input": null, artifact: null, image: null, video: null, file: null, divider: null,
  "state-snapshot": "context", "state-delta": "context", step: null, usage: null, error: null, raw: null,
} satisfies Record<ChunkKind, RunPhase | null>;

export const CHUNK_BUBBLE_VISIBLE = {
  text: true, markdown: true, reasoning: true, thinking: true,
  "tool-call": true, "tool-approval": true, "tool-denied": true,
  "skill-activation": true, "memory-recall": true, "memory-mutation": true, "memory-update": true,
  citation: true, "rag-citations": true, "context-update": true,
  "a2ui-display": true, "a2ui-input": true, artifact: true, image: true, video: true, file: true, divider: true,
  "state-snapshot": false, "state-delta": false, step: false, usage: true, error: true, raw: false,
} satisfies Record<ChunkKind, boolean>;

export const CHUNK_RENDERER = {
  text: "text", markdown: "text", reasoning: "reasoning", thinking: "reasoning",
  "tool-call": "tool", "tool-approval": "approval", "tool-denied": "denied", "skill-activation": "skill",
  "memory-recall": "memory", "memory-mutation": "memory", "memory-update": "memory",
  citation: "citation", "rag-citations": "citation", "context-update": "context",
  "a2ui-display": "a2ui", "a2ui-input": "a2ui", artifact: "artifact",
  image: "media", video: "media", file: "media", divider: "divider",
  "state-snapshot": "hidden", "state-delta": "hidden", step: "hidden", usage: "usage", error: "error", raw: "hidden",
} satisfies Record<ChunkKind, ChunkRendererName>;

export const CHUNK_TRACE = {
  text: "segment", markdown: "segment", reasoning: "segment", thinking: "segment",
  "tool-call": "row", "tool-approval": "row", "tool-denied": "row", "skill-activation": "segment",
  "memory-recall": "segment", "memory-mutation": "segment", "memory-update": "segment",
  citation: "segment", "rag-citations": "segment", "context-update": "segment",
  "a2ui-display": "row", "a2ui-input": "row", artifact: "row",
  image: "none", video: "none", file: "none", divider: "none",
  "state-snapshot": "inspector", "state-delta": "inspector", step: "tick", usage: "footer", error: "segment", raw: "row",
} satisfies Record<ChunkKind, ChunkTraceDisposition>;
