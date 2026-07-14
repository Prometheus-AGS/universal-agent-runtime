import type { z } from "zod";
import type {
  chatCompletionSchema,
  checkpointListSchema,
  embeddingResponseSchema,
  ingestResponseSchema,
  knowledgeBaseSchema,
  runResponseSchema,
  searchResponseSchema,
  toolResultSchema,
} from "./schemas.js";

export type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };
export type ChatCompletion = z.infer<typeof chatCompletionSchema>;
export type RunResponse = z.infer<typeof runResponseSchema>;
export type ToolResult = z.infer<typeof toolResultSchema>;
export type EmbeddingResponse = z.infer<typeof embeddingResponseSchema>;
export type KnowledgeBase = z.infer<typeof knowledgeBaseSchema>;
export type SearchResponse = z.infer<typeof searchResponseSchema>;
export type IngestResponse = z.infer<typeof ingestResponseSchema>;
export type CheckpointList = z.infer<typeof checkpointListSchema>;

export interface ClientOptions {
  apiKey?: string;
  headers?: HeadersInit;
  fetch?: typeof globalThis.fetch;
}

export interface ChatMessage { role: "system" | "user" | "assistant" | "tool"; content: string }
export interface ChatRequest {
  messages: ChatMessage[];
  model?: string;
  sessionId?: string;
  temperature?: number;
  tools?: JsonValue[];
}
export interface StreamOptions { signal?: AbortSignal; lastEventId?: string }
export interface SseEvent<T = JsonValue> { id?: string; event?: string; data: T }
export interface AgentArtifact { [key: string]: JsonValue }
export interface CreateRunRequest { artifact: AgentArtifact; input: string; sessionId?: string }
export interface ResumeRunRequest { artifact: AgentArtifact; input?: string; sessionId?: string }
export interface KnowledgeBaseInput {
  name: string;
  description?: string;
  config?: Record<string, JsonValue>;
}
export interface SearchRequest { query: string; limit?: number; minScore?: number }
export interface IngestRequest { content: string; metadata?: Record<string, JsonValue> }
