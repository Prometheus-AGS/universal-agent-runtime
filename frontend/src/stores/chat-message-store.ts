import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { getDbInstance } from "@/lib/db";
import type {
  ContentBlock,
  ContextUpdateContentBlock,
  RichMessage,
  SkillActivationContentBlock,
  StreamingState,
  ToolCallContentBlock,
} from "@/types/chat-content";

interface ChatMessageState {
  messagesByThread: Record<string, RichMessage[]>;
  streamingByThread: Record<string, StreamingState>;
}

interface ChatMessageActions {
  /** Load persisted messages for a thread from PGlite into Zustand. */
  loadMessagesFromDb(threadId: string): Promise<void>;
  initThread(threadId: string, messages: RichMessage[]): void;
  appendTextDelta(threadId: string, runId: string, text: string): void;
  appendThinkingDelta(threadId: string, runId: string, text: string): void;
  addToolCall(threadId: string, toolCall: ToolCallContentBlock): void;
  updateToolCall(threadId: string, toolCallId: string, update: Partial<Omit<ToolCallContentBlock, "type">>): void;
  addCitation(threadId: string, citation: { source: string; content: string; url?: string }): void;
  addSkillActivation(threadId: string, skill: { skillId: string; skillName: string; selectionMethod?: string; status: "active" | "complete" }): void;
  addContextUpdate(threadId: string, update: Omit<ContextUpdateContentBlock, "type">): void;
  finishStream(threadId: string): void;
  setStreamError(threadId: string, error: string): void;
  clearThread(threadId: string): void;
}

type ChatMessageStore = ChatMessageState & ChatMessageActions;

const defaultStreamingState: StreamingState = { isStreaming: false, runId: null, streamingMessageId: null };

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
  state.streamingByThread[threadId] = { isStreaming: true, runId, streamingMessageId: msgId };
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

    appendTextDelta: (threadId, runId, text) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = ensureStreaming(state, threadId);
        if (!streaming.isStreaming || streaming.runId !== runId) {
          state.streamingByThread[threadId] = { isStreaming: true, runId, streamingMessageId: null };
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
        ensureStreaming(state, threadId);
        const msg = getOrCreateStreamingMessage(state, threadId, runId);
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === msg.id);
        if (idx === -1) return;
        const last = messages[idx].content[messages[idx].content.length - 1];
        if (last?.type === "reasoning") (last as { type: "reasoning"; text: string }).text += text;
        else messages[idx].content.push({ type: "reasoning", text });
      }),

    addToolCall: (threadId, toolCall) =>
      set((state) => {
        ensureThread(state, threadId);
        const streaming = state.streamingByThread[threadId];
        if (!streaming?.streamingMessageId) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages.findIndex((m) => m.id === streaming.streamingMessageId);
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

    addCitation: (threadId, citation) =>
      set((state) => {
        const streaming = state.streamingByThread[threadId];
        if (!streaming?.streamingMessageId) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages?.findIndex((m) => m.id === streaming.streamingMessageId);
        if (idx === undefined || idx === -1) return;
        messages[idx].content.push({ type: "citation", ...citation });
      }),

    addSkillActivation: (threadId, skill) =>
      set((state) => {
        const streaming = state.streamingByThread[threadId];
        if (!streaming?.streamingMessageId) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages?.findIndex((m) => m.id === streaming.streamingMessageId);
        if (idx === undefined || idx === -1) return;
        messages[idx].content.push({ type: "skill-activation", skillId: skill.skillId, skillName: skill.skillName, selectionMethod: skill.selectionMethod, status: skill.status });
      }),

    addContextUpdate: (threadId, update) =>
      set((state) => {
        const streaming = state.streamingByThread[threadId];
        if (!streaming?.streamingMessageId) return;
        const messages = state.messagesByThread[threadId];
        const idx = messages?.findIndex((m) => m.id === streaming.streamingMessageId);
        if (idx === undefined || idx === -1) return;
        messages[idx].content.push({ type: "context-update", ...update });
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
            // Take a plain snapshot; never leak Immer draft proxies outside set().
            finalMsgSnapshot = {
              ...messages[idx],
              content: structuredClone(messages[idx].content),
            };
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

    setStreamError: (threadId, error) =>
      set((state) => {
        const streaming = state.streamingByThread[threadId];
        if (!streaming) return;
        const messages = state.messagesByThread[threadId];
        if (messages && streaming.streamingMessageId) {
          const idx = messages.findIndex((m) => m.id === streaming.streamingMessageId);
          if (idx !== -1) {
            messages[idx].status = "failed";
            messages[idx].content.push({ type: "error", message: error });
          }
        }
        state.streamingByThread[threadId] = { ...defaultStreamingState };
      }),

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

export const selectIsStreaming = (threadId: string) => (state: ChatMessageStore) =>
  state.streamingByThread[threadId]?.isStreaming ?? false;
