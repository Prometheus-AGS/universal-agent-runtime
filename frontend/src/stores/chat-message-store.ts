import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { getDbInstance } from "@/platform/pglite/client";
import type {
  ContextUpdateContentBlock,
  MessageUsage,
  RagCitationMarker,
  RichMessage,
  StreamingState,
  ToolCallContentBlock,
} from "@/types/chat-content";
import type { Chunk, SkillActivationChunk } from "@/features/chat/model/chunk";
import { toChunks } from "@/features/chat/model/to-chunks";

/** Per-message metadata surfaced in the action bar (agent that answered, model, usage). */
export interface MessageMeta {
  agentId?: string;
  model?: string;
  usage?: MessageUsage;
}

interface ChatMessageState {
  messagesByThread: Record<string, RichMessage[]>;
  streamingByThread: Record<string, StreamingState>;
}

interface ChatMessageActions {
  /** Load persisted messages for a thread from PGlite into Zustand. */
  loadMessagesFromDb(threadId: string): Promise<void>;
  initThread(threadId: string, messages: RichMessage[]): void;
  beginStream(threadId: string, runId: string): void;
  setAwaitingRetry(threadId: string, runId: string, attempt: number, maxAttempts: number, delayMs: number): void;
  markStreamStarted(threadId: string, runId: string): void;
  appendTextDelta(threadId: string, runId: string, text: string): void;
  appendThinkingDelta(threadId: string, runId: string, text: string, kind?: "thinking" | "reasoning"): void;
  addToolCall(threadId: string, runId: string, toolCall: ToolCallContentBlock): void;
  addRuntimeChunk(threadId: string, runId: string, chunk: Chunk): void;
  updateToolCall(threadId: string, toolCallId: string, update: Partial<Omit<ToolCallContentBlock, "type">>): void;
  addCitation(threadId: string, runId: string, citation: { source: string; content: string; url?: string }): void;
  /** Attach the full numbered citation set ([1], [2], ...) for a RAG-augmented
   * run to the streaming message, so the hover-to-source panel can resolve
   * `[n]` markers appearing in the response text. */
  addRagCitations(threadId: string, runId: string, citations: RagCitationMarker[]): void;
  addSkillActivation(threadId: string, runId: string, skill: { skillId: string; skillName: string; selectionMethod?: string; status: "active" | "complete" }): void;
  addContextUpdate(threadId: string, runId: string, update: Omit<ContextUpdateContentBlock, "type">): void;
  /** Attach agent/model/usage metadata to the in-flight streaming message. Must run before finishStream so it is persisted. */
  setMessageMeta(threadId: string, runId: string, meta: MessageMeta): void;
  finishStream(threadId: string): void;
  setStreamError(threadId: string, error: string): void;
  clearThread(threadId: string): void;
}

type ChatMessageStore = ChatMessageState & ChatMessageActions;

const defaultStreamingState: StreamingState = {
  isStreaming: false,
  runId: null,
  streamingMessageId: null,
  awaitingFirstToken: false,
  retryAttempt: 0,
  retryMaxAttempts: 0,
  retryDelayMs: 0,
};

function ensureThread(state: ChatMessageState, threadId: string): RichMessage[] {
  if (!state.messagesByThread[threadId]) state.messagesByThread[threadId] = [];
  return state.messagesByThread[threadId];
}

function ensureStreaming(state: ChatMessageState, threadId: string): StreamingState {
  if (!state.streamingByThread[threadId]) state.streamingByThread[threadId] = { ...defaultStreamingState };
  return state.streamingByThread[threadId];
}

function getOrCreateStreamingMessage(state: ChatMessageState, threadId: string, runId: string): RichMessage {
  const messages = state.messagesByThread[threadId];
  const streaming = state.streamingByThread[threadId];
  if (streaming?.streamingMessageId) {
    const existing = messages?.find((m) => m.id === streaming.streamingMessageId);
    if (existing) return existing;
  }
  const msgId = `stream-${runId}-${Date.now()}`;
  const newMsg: RichMessage = { id: msgId, role: "assistant", content: [], chunks: [], createdAt: new Date(), status: "in_progress" };
  if (!state.messagesByThread[threadId]) state.messagesByThread[threadId] = [];
  state.messagesByThread[threadId].push(newMsg);
  state.streamingByThread[threadId] = {
    isStreaming: true,
    runId,
    streamingMessageId: msgId,
    awaitingFirstToken: false,
    retryAttempt: 0,
    retryMaxAttempts: 0,
    retryDelayMs: 0,
  };
  return newMsg;
}

function messageAt(message: RichMessage): string {
  return message.createdAt instanceof Date ? message.createdAt.toISOString() : String(message.createdAt);
}

function ensureMessageChunks(message: RichMessage, runId?: string): Chunk[] {
  if (!message.chunks) {
    message.chunks = toChunks(message.content, {
      messageId: message.id,
      at: messageAt(message),
      runId,
      finalized: message.status !== "in_progress",
    });
  }
  return message.chunks;
}

function chunkBase(message: RichMessage, runId: string) {
  const chunks = ensureMessageChunks(message, runId);
  const seq = chunks.length;
  return { id: `${message.id}:chunk:${seq}`, at: messageAt(message), runId, seq };
}

export const useChatMessageStore = create<ChatMessageStore>()(
  immer((set) => ({
    messagesByThread: {},
    streamingByThread: {},

    loadMessagesFromDb: async (threadId) => {
      const db = getDbInstance();
      const messages = await db.getMessages(threadId);
      set((state) => {
        state.messagesByThread[threadId] = messages;
        if (!state.streamingByThread[threadId]) state.streamingByThread[threadId] = { ...defaultStreamingState };
      });
    },

    initThread: (threadId, messages) =>
      set((state) => {
        state.messagesByThread[threadId] = messages;
        if (!state.streamingByThread[threadId]) state.streamingByThread[threadId] = { ...defaultStreamingState };
      }),

    beginStream: (threadId, runId) =>
      set((state) => {
        ensureThread(state, threadId);
        state.streamingByThread[threadId] = {
          isStreaming: true,
          runId,
          streamingMessageId: null,
          awaitingFirstToken: true,
          retryAttempt: 0,
          retryMaxAttempts: 0,
          retryDelayMs: 0,
        };
      }),

    setAwaitingRetry: (threadId, runId, attempt, maxAttempts, delayMs) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (streaming.runId !== runId) return;
        streaming.awaitingFirstToken = true;
        streaming.retryAttempt = attempt;
        streaming.retryMaxAttempts = maxAttempts;
        streaming.retryDelayMs = delayMs;
      }),

    markStreamStarted: (threadId, runId) =>
      set((state) => {
        const streaming = ensureStreaming(state, threadId);
        if (streaming.runId !== runId) return;
        streaming.awaitingFirstToken = false;
        streaming.retryAttempt = 0;
        streaming.retryMaxAttempts = 0;
        streaming.retryDelayMs = 0;
      }),

    appendTextDelta: (threadId, runId, text) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === -1) return;
        const last = messages[idx].content[messages[idx].content.length - 1];
        if (last?.type === "text") (last as { type: "text"; text: string }).text += text;
        else messages[idx].content.push({ type: "text", text });
        const chunks = ensureMessageChunks(messages[idx], runId);
        const lastChunk = chunks.at(-1);
        if (lastChunk?.kind === "text") lastChunk.text += text;
        else chunks.push({ ...chunkBase(messages[idx], runId), kind: "text", text });
      }),

    appendThinkingDelta: (threadId, runId, text, kind = "thinking") =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === -1) return;
        const last = messages[idx].content[messages[idx].content.length - 1];
        if (last?.type === "thinking") last.text += text;
        else messages[idx].content.push({ type: "thinking", text });
        const chunks = ensureMessageChunks(messages[idx], runId);
        const lastChunk = chunks.at(-1);
        if (lastChunk?.kind === kind) lastChunk.text += text;
        else chunks.push({ ...chunkBase(messages[idx], runId), kind, text });
      }),

    addToolCall: (threadId, runId, toolCall) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === -1) return;
        const message = messages[idx];
        const chunks = ensureMessageChunks(message, runId);
        const base = chunkBase(message, runId);
        if (toolCall.toolName === "__memory_recall__") {
          const items = Array.isArray(toolCall.args.items) ? toolCall.args.items.map((item, itemIndex) => {
            const memory = item && typeof item === "object" ? item as Record<string, unknown> : {};
            const id = typeof memory.id === "string" ? memory.id : typeof memory.key === "string" ? memory.key : `${base.id}:${itemIndex}`;
            const content = typeof memory.content === "string" ? memory.content : typeof memory.value === "string" ? memory.value : "";
            message.content.push({ type: "memory", operation: "recall", key: id, value: content });
            return { id, content, type: "semantic" as const, importance: typeof memory.importance === "number" ? memory.importance : undefined, score: typeof memory.score === "number" ? memory.score : undefined, pinned: typeof memory.pinned === "boolean" ? memory.pinned : undefined };
          }) : [];
          chunks.push({ ...base, kind: "memory-recall", items });
          return;
        }
        if (toolCall.toolName === "__memory_mutation__") {
          const operation = String(toolCall.args.operation ?? "update");
          const memoryId = String(toolCall.args.memoryId ?? "");
          const content = String(toolCall.args.content ?? "");
          message.content.push({ type: "memory", operation, key: memoryId, value: content || null });
          const normalizedOperation = operation === "create" || operation === "delete" || operation === "pin" || operation === "decay" ? operation : "update";
          chunks.push({ ...base, kind: "memory-mutation", operation: normalizedOperation, memoryId, content: content || undefined, memoryType: typeof toolCall.args.memoryType === "string" ? toolCall.args.memoryType : undefined });
          return;
        }
        if (toolCall.toolName === "__memory_update__") {
          const memoryKey = String(toolCall.args.key ?? "");
          const value = String(toolCall.args.value ?? "");
          message.content.push({ type: "memory", operation: String(toolCall.args.operation ?? "update"), key: memoryKey, value });
          chunks.push({ ...base, kind: "memory-update", scope: "session", summary: value, itemCount: 1 });
          return;
        }
        if (toolCall.toolName === "__a2ui_input__") {
          const inputChunk = { ...base, kind: "a2ui-input" as const, toolCallId: toolCall.toolCallId, profile: "a2ui/v0.9", component: String(toolCall.args.artifactType ?? "input"), requestId: String(toolCall.args.runId ?? toolCall.toolCallId), payload: toolCall.args, status: "awaiting" as const };
          message.content.push({ type: "artifact", id: toolCall.toolCallId, kind: "application/vnd.uar.a2ui-input+json", content: JSON.stringify({ profile: inputChunk.profile, component: inputChunk.component, requestId: inputChunk.requestId, payload: inputChunk.payload, status: inputChunk.status }) });
          chunks.push(inputChunk);
          return;
        }
        if (toolCall.toolName === "__a2ui_display__") {
          const displayChunk = { ...base, kind: "a2ui-display" as const, toolCallId: toolCall.toolCallId, profile: "a2ui/v0.9", component: String(toolCall.args.artifactType ?? "surface"), payload: { ...toolCall.args, content: toolCall.result }, validation: "valid" as const };
          message.content.push({ type: "artifact", id: toolCall.toolCallId, kind: "application/vnd.uar.a2ui+json", content: JSON.stringify({ profile: displayChunk.profile, component: displayChunk.component, payload: displayChunk.payload, validation: displayChunk.validation }) });
          chunks.push(displayChunk);
          return;
        }
        message.content.push({ type: "toolUse", id: toolCall.toolCallId, name: toolCall.toolName, inputJson: JSON.stringify(toolCall.args) });
        if (toolCall.result !== undefined) message.content.push({ type: "toolResult", toolUseId: toolCall.toolCallId, outputJson: toolCall.result, isError: toolCall.status === "failed" });
        chunks.push({ ...base, kind: "tool-call", toolCallId: toolCall.toolCallId, toolName: toolCall.toolName, args: toolCall.args, result: toolCall.result, status: toolCall.status });
      }),

    addRuntimeChunk: (threadId, runId, chunk) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        }
        const message = getOrCreateStreamingMessage(state, threadId, runId);
        ensureMessageChunks(message, runId).push(chunk);
      }),

    updateToolCall: (threadId, toolCallId, update) =>
      set((state) => {
        const messages = state.messagesByThread[threadId];
        if (!messages) return;
        for (const msg of messages) {
          const chunks = ensureMessageChunks(msg);
          const chunk = chunks.find((item) => (
            item.kind === "tool-call"
            || item.kind === "a2ui-input"
            || item.kind === "a2ui-display"
          ) && item.toolCallId === toolCallId);
          if (chunk?.kind === "tool-call") {
            Object.assign(chunk, update);
            if (update.result !== undefined || update.status === "complete" || update.status === "failed") {
              const existingResult = msg.content.find((block) => block.type === "toolResult" && block.toolUseId === toolCallId);
              const result = { type: "toolResult" as const, toolUseId: toolCallId, outputJson: update.result ?? (existingResult?.type === "toolResult" ? existingResult.outputJson : ""), isError: update.status === "failed" };
              if (existingResult) Object.assign(existingResult, result);
              else msg.content.push(result);
            }
            return;
          }
          if (chunk?.kind === "a2ui-input") {
            chunk.status = update.status === "complete" ? "submitted" : update.status === "failed" ? "cancelled" : "awaiting";
            if (update.result !== undefined) chunk.response = update.result;
            const artifact = msg.content.find((block) => block.type === "artifact" && block.id === toolCallId);
            if (artifact?.type === "artifact") artifact.content = JSON.stringify({ profile: chunk.profile, component: chunk.component, requestId: chunk.requestId, payload: chunk.payload, status: chunk.status, response: chunk.response });
            return;
          }
          if (chunk?.kind === "a2ui-display" && update.result !== undefined) {
            const payload = chunk.payload && typeof chunk.payload === "object" && !Array.isArray(chunk.payload)
              ? chunk.payload as Record<string, unknown>
              : {};
            chunk.payload = { ...payload, content: update.result };
            const artifact = msg.content.find((block) => block.type === "artifact" && block.id === toolCallId);
            if (artifact?.type === "artifact") artifact.content = JSON.stringify({ profile: chunk.profile, component: chunk.component, payload: chunk.payload, validation: chunk.validation });
            return;
          }
        }
      }),

    addCitation: (threadId, runId, citation) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === undefined || idx === -1) return;
        const message = messages[idx];
        message.content.push({ type: "citation", source: citation.source, quote: citation.content });
        ensureMessageChunks(message, runId).push({ ...chunkBase(message, runId), kind: "citation", source: citation.source, content: citation.content, url: citation.url });
      }),

    addRagCitations: (threadId, runId, citations) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === undefined || idx === -1) return;
        const message = messages[idx];
        message.content.push({ type: "artifact", id: `${message.id}:rag:${message.content.length}`, kind: "application/vnd.uar.rag-citations+json", content: JSON.stringify({ citations }) });
        ensureMessageChunks(message, runId).push({ ...chunkBase(message, runId), kind: "rag-citations", citations });
      }),

    addSkillActivation: (threadId, runId, skill) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === undefined || idx === -1) return;
        const message = messages[idx];
        message.content.push({ type: "skill", name: skill.skillName, status: skill.status });
        const allowedMethod = skill.selectionMethod === "keyword" || skill.selectionMethod === "embedding" || skill.selectionMethod === "hybrid" || skill.selectionMethod === "llm" || skill.selectionMethod === "explicit" ? skill.selectionMethod : undefined;
        ensureMessageChunks(message, runId).push({ ...chunkBase(message, runId), kind: "skill-activation", skillId: skill.skillId, skillName: skill.skillName, selectionMethod: allowedMethod, status: skill.status });
      }),

    addContextUpdate: (threadId, runId, update) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = {
            isStreaming: true,
            runId,
            streamingMessageId: null,
            awaitingFirstToken: false,
            retryAttempt: 0,
            retryMaxAttempts: 0,
            retryDelayMs: 0,
          };
        } else {
          streaming.awaitingFirstToken = false;
          streaming.retryAttempt = 0;
          streaming.retryMaxAttempts = 0;
          streaming.retryDelayMs = 0;
        }
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === undefined || idx === -1) return;
        const message = messages[idx];
        message.content.push({ type: "artifact", id: `${message.id}:context:${message.content.length}`, kind: "application/vnd.uar.context-update+json", content: JSON.stringify(update) });
        ensureMessageChunks(message, runId).push({ ...chunkBase(message, runId), kind: "context-update", ...update });
      }),

    setMessageMeta: (threadId, runId, meta) =>
      set((state) => {
        ensureThread(state, threadId);
        ensureStreaming(state, threadId);
        // Resolve (or create) the assistant message this run is producing, so
        // metadata attaches even if it arrives before the first content chunk.
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === -1) return;
        if (meta.agentId !== undefined) messages[idx].agentId = meta.agentId;
        if (meta.model !== undefined) messages[idx].model = meta.model;
        if (meta.usage !== undefined) messages[idx].usage = meta.usage;
        if (meta.usage !== undefined) {
          const chunks = ensureMessageChunks(messages[idx], runId);
          const existing = chunks.find((chunk) => chunk.kind === "usage");
          const usage = { ...meta.usage, model: meta.model ?? messages[idx].model };
          if (existing?.kind === "usage") Object.assign(existing, usage);
          else chunks.push({ ...chunkBase(messages[idx], runId), kind: "usage" as const, ...usage });
        }
      }),

    finishStream: (threadId) => {
      let finalMsgSnapshot: RichMessage | null = null;
      set((state) => {
        const streaming = state.streamingByThread[threadId];
        if (!streaming) return;
        const messages = state.messagesByThread[threadId];
        if (messages && streaming.streamingMessageId) {
          const idx = messages.findIndex((m) => m.id === streaming.streamingMessageId);
          if (idx !== -1) {
            messages[idx].status = "complete";
            for (const block of messages[idx].content) if (block.type === "skill" && block.status === "active") block.status = "complete";
            const finalizedChunks = ensureMessageChunks(messages[idx], streaming.runId ?? undefined).map((chunk) => {
              if (chunk.kind === "skill-activation" && chunk.status === "active") (chunk as SkillActivationChunk).status = "complete";
              if (chunk.kind === "text") {
                const { text, ...base } = chunk;
                return { ...base, kind: "markdown" as const, source: text };
              }
              return chunk;
            });
            messages[idx].chunks = finalizedChunks;
            // Snapshot outside the Immer proxy using JSON round-trip.
            // structuredClone() fails on Immer draft proxies; JSON parse/stringify is safe
            // for our plain JSON-serialisable content blocks.
            finalMsgSnapshot = JSON.parse(JSON.stringify({
              ...messages[idx],
              content: messages[idx].content,
            })) as RichMessage;
          }
        }
        state.streamingByThread[threadId] = { ...defaultStreamingState };
      });
      // Persist the completed assistant message to PGlite
      if (finalMsgSnapshot) {
        void getDbInstance().insertMessage(threadId, finalMsgSnapshot);
      }
      // Also update the thread's updated_at timestamp
      void getDbInstance().touchThread(threadId);
    },

    setStreamError: (threadId, error) => {
      set((state) => {
        if (!state.messagesByThread[threadId]) state.messagesByThread[threadId] = [];
        const messages = state.messagesByThread[threadId];
        const streaming = state.streamingByThread[threadId];

        if (streaming?.streamingMessageId) {
          // In-flight message exists — mark it failed and append error block
          const idx = messages.findIndex((m) => m.id === streaming.streamingMessageId);
          if (idx !== -1) {
            messages[idx].status = "failed";
            const message = messages[idx];
            message.content.push({ type: "artifact", id: `${message.id}:error`, kind: "application/vnd.uar.error+json", content: JSON.stringify({ message: error }) });
            ensureMessageChunks(message, streaming.runId ?? undefined).push({ ...chunkBase(message, streaming.runId ?? "error"), kind: "error", message: error });
          }
        } else {
          // No in-flight message (error hit before stream started) — inject a fresh error bubble
          const errMsg: RichMessage = {
            id: `error-${Date.now()}`,
            role: "assistant",
            content: [{ type: "artifact", id: `error-${Date.now()}`, kind: "application/vnd.uar.error+json", content: JSON.stringify({ message: error }) }],
            chunks: [{ id: `error-${Date.now()}`, kind: "error", at: new Date().toISOString(), seq: 0, message: error }],
            createdAt: new Date(),
            status: "failed",
          };
          messages.push(errMsg);
        }
        state.streamingByThread[threadId] = { ...defaultStreamingState };
      });
    },

    clearThread: (threadId) =>
      set((state) => {
        delete state.messagesByThread[threadId];
        delete state.streamingByThread[threadId];
      }),

    // expose getter for callers needing to read outside React
    // (already available via .getState())
  })),
);

export const selectThreadMessages = (threadId: string) => (state: ChatMessageStore) =>
  state.messagesByThread[threadId] ?? [];

/** Look up a single message by id within a thread (for per-message metadata rendering). */
export const selectMessageById =
  (threadId: string, messageId: string) => (state: ChatMessageStore) =>
    state.messagesByThread[threadId]?.find((m) => m.id === messageId) ?? null;

export const selectIsStreaming = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.isStreaming ?? false;

export const selectIsAwaitingFirstToken = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.awaitingFirstToken ?? false;

export const selectRetryAttempt = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.retryAttempt ?? 0;

export const selectRetryMaxAttempts = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.retryMaxAttempts ?? 0;

export const selectRetryDelayMs = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.retryDelayMs ?? 0;
