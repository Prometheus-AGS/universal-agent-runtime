import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { getDbInstance } from "@/lib/db";
import type {
  ContentBlock,
  ContextUpdateContentBlock,
  MessageUsage,
  RichMessage,
  SkillActivationContentBlock,
  StreamingState,
  ToolCallContentBlock,
} from "@/types/chat-content";

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
  appendThinkingDelta(threadId: string, runId: string, text: string): void;
  addToolCall(threadId: string, runId: string, toolCall: ToolCallContentBlock): void;
  updateToolCall(threadId: string, toolCallId: string, update: Partial<Omit<ToolCallContentBlock, "type">>): void;
  addCitation(threadId: string, runId: string, citation: { source: string; content: string; url?: string }): void;
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
  const newMsg: RichMessage = { id: msgId, role: "assistant", content: [], createdAt: new Date(), status: "in_progress" };
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
      }),

    appendThinkingDelta: (threadId, runId, text) =>
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
        if (last?.type === "reasoning") (last as { type: "reasoning"; text: string }).text += text;
        else messages[idx].content.push({ type: "reasoning", text });
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
        messages[idx].content.push(toolCall as ContentBlock);
      }),

    updateToolCall: (threadId, toolCallId, update) =>
      set((state) => {
        const messages = state.messagesByThread[threadId];
        if (!messages) return;
        for (const msg of messages) {
          const block = msg.content.find((b): b is ToolCallContentBlock => b.type === "tool-call" && b.toolCallId === toolCallId);
          if (block) { Object.assign(block, update); return; }
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
        messages[idx].content.push({ type: "citation", ...citation });
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
        messages[idx].content.push({ type: "skill-activation", skillId: skill.skillId, skillName: skill.skillName, selectionMethod: skill.selectionMethod, status: skill.status });
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
        messages[idx].content.push({ type: "context-update", ...update });
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
            for (const block of messages[idx].content) {
              if (block.type === "skill-activation" && block.status === "active") {
                (block as SkillActivationContentBlock).status = "complete";
              }
            }
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
            // Replace any empty content with the error block
            if (messages[idx].content.length === 0) {
              messages[idx].content.push({ type: "error", message: error });
            } else {
              messages[idx].content.push({ type: "error", message: error });
            }
          }
        } else {
          // No in-flight message (error hit before stream started) — inject a fresh error bubble
          const errMsg: RichMessage = {
            id: `error-${Date.now()}`,
            role: "assistant",
            content: [{ type: "error", message: error }],
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
