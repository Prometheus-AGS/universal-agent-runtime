import type { Chunk } from "@/shared/content/chunk";
import type { ContentBlock } from "@/shared/content";

export type { ContentBlock } from "@/shared/content";

/** Runtime input accepted by the message store; never persisted as a wire block. */
export interface ToolCallContentBlock {
  type: "tool-call";
  toolCallId: string;
  toolName: string;
  args: Record<string, unknown>;
  result?: string;
  status: "running" | "complete" | "failed";
}

export interface RagCitationMarker {
  marker: number;
  chunkId: string;
  documentId?: string;
  documentName: string;
  knowledgeBase?: string;
  relevanceScore: number;
  snippet: string;
  page?: number;
  span?: [number, number];
}

/** Runtime input accepted by the message store; persisted as an artifact block. */
export interface ContextUpdateContentBlock {
  type: "context-update";
  strategy: string;
  messagesRemoved: number;
  tokensSaved: number;
  wasApplied: boolean;
  summaryGenerated: boolean;
}

export interface MessageUsage {
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
}

export interface RichMessage {
  id: string;
  role: "user" | "assistant" | "system";
  /** Portable wire/storage blocks only. */
  content: ContentBlock[];
  /** UAR-rich view projection. Old rows derive this at the PGlite boundary. */
  chunks?: Chunk[];
  createdAt: Date;
  status?: "in_progress" | "complete" | "failed";
  agentId?: string;
  model?: string;
  usage?: MessageUsage;
}

export interface StreamingState {
  isStreaming: boolean;
  runId: string | null;
  streamingMessageId: string | null;
  awaitingFirstToken: boolean;
  retryAttempt: number;
  retryMaxAttempts: number;
  retryDelayMs: number;
}

export function getMessageText(message: RichMessage): string {
  return message.content
    .filter((block): block is Extract<ContentBlock, { type: "text" }> => block.type === "text")
    .map((block) => block.text)
    .join("");
}
