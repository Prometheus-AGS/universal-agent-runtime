export interface TextContentBlock { type: "text"; text: string }
export interface ReasoningContentBlock { type: "reasoning"; text: string }
export interface ToolCallContentBlock {
  type: "tool-call";
  toolCallId: string;
  toolName: string;
  args: Record<string, unknown>;
  result?: string;
  status: "running" | "complete" | "failed";
}
export interface CitationContentBlock {
  type: "citation";
  source: string;
  content: string;
  url?: string;
}
export interface SkillActivationContentBlock {
  type: "skill-activation";
  skillId: string;
  skillName: string;
  selectionMethod?: string;
  status: "active" | "complete";
}
export interface ContextUpdateContentBlock {
  type: "context-update";
  strategy: string;
  messagesRemoved: number;
  tokensSaved: number;
  wasApplied: boolean;
  summaryGenerated: boolean;
}
export interface ImageContentBlock { type: "image"; url: string; alt?: string }
export interface ErrorContentBlock { type: "error"; message: string; code?: string }

export type ContentBlock =
  | TextContentBlock
  | ReasoningContentBlock
  | ToolCallContentBlock
  | CitationContentBlock
  | SkillActivationContentBlock
  | ContextUpdateContentBlock
  | ImageContentBlock
  | ErrorContentBlock;

export interface RichMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: ContentBlock[];
  createdAt: Date;
  status?: "in_progress" | "complete" | "failed";
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
    .filter((b): b is TextContentBlock => b.type === "text")
    .map((b) => b.text)
    .join("");
}
