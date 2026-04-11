import { useCallback, useEffect, useMemo, useRef } from "react";
import { useExternalStoreRuntime, type AppendMessage, type ThreadMessageLike } from "@assistant-ui/react";
import { useChatMessageStore, selectIsStreaming } from "@/stores/chat-message-store";
import { useChatIntentStore } from "@/stores/chat-intent-store";
import { useThreadRegistryStore } from "@/stores/thread-registry-store";
import { useDb } from "@/lib/db-context";
import { useChatMessages } from "./use-chat-messages";
import { useMessageStream } from "./use-message-stream";
import { useThreadTitleStore } from "@/stores/thread-title-store";
import type { RichMessage, ContentBlock } from "@/types/chat-content";
import { useAttachmentManager } from "./use-attachment-manager";
import type { AttachmentManager } from "./use-attachment-manager";
import { useMemoryContext } from "./memory-context";

export type { AttachmentManager };

function richMessageToThreadMessageLike(msg: RichMessage): ThreadMessageLike {
  const baseMetadata = { custom: {} };
  const toDate = (d: Date | string): Date => (d instanceof Date ? d : new Date(d));

  if (msg.role === "user") {
    const text = msg.content.filter((b): b is Extract<ContentBlock, { type: "text" }> => b.type === "text").map((b) => b.text).join("");
    return { role: "user", id: msg.id, content: [{ type: "text", text }], createdAt: toDate(msg.createdAt), attachments: [], metadata: baseMetadata };
  }
  if (msg.role === "system") {
    const text = msg.content.filter((b): b is Extract<ContentBlock, { type: "text" }> => b.type === "text").map((b) => b.text).join("");
    return { role: "system", id: msg.id, content: [{ type: "text", text }], createdAt: toDate(msg.createdAt), metadata: baseMetadata };
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const parts: any[] = [];
  for (const block of msg.content) {
    switch (block.type) {
      case "text": parts.push({ type: "text", text: block.text }); break;
      case "reasoning": parts.push({ type: "reasoning", text: block.text }); break;
      case "tool-call": parts.push({ type: "tool-call", toolCallId: block.toolCallId, toolName: block.toolName, args: block.args, result: block.result, isError: block.status === "failed" }); break;
      case "skill-activation": parts.push({ type: "tool-call", toolCallId: `skill-${block.skillId}`, toolName: "__skill__", args: { skillId: block.skillId, skillName: block.skillName, selectionMethod: block.selectionMethod, status: block.status }, result: undefined, isError: false }); break;
      case "context-update": parts.push({ type: "tool-call", toolCallId: `ctx-${block.strategy}-${block.messagesRemoved}`, toolName: "__context__", args: { strategy: block.strategy, messagesRemoved: block.messagesRemoved, tokensSaved: block.tokensSaved, wasApplied: block.wasApplied, summaryGenerated: block.summaryGenerated }, result: undefined, isError: false }); break;
      default: break;
    }
  }
  if (parts.length === 0) parts.push({ type: "text", text: "" });

  const errorText = msg.status === "failed"
    ? (msg.content.find((b): b is Extract<ContentBlock, { type: "error" }> => b.type === "error") as { type: "error"; message: string } | undefined)?.message ?? "An error occurred."
    : undefined;

  return {
    role: "assistant", id: msg.id, content: parts, createdAt: toDate(msg.createdAt),
    status: msg.status === "in_progress" ? { type: "running" } : msg.status === "failed" ? { type: "incomplete", reason: "error" } : { type: "complete", reason: "stop" as const },
    metadata: { ...baseMetadata, unstable_state: null, unstable_annotations: [], unstable_data: [], steps: [], custom: errorText ? { errorText } : {} },
  };
}

function extractText(msg: RichMessage): string {
  return msg.content.filter((b): b is Extract<ContentBlock, { type: "text" }> => b.type === "text").map((b) => b.text).join("");
}

export function useChatRuntime(threadId: string): {
  runtime: ReturnType<typeof useExternalStoreRuntime>;
  attachmentManager: AttachmentManager;
} {
  const db = useDb();
  const consumePendingPrompt = useChatIntentStore((s) => s.consumePendingPrompt);
  const { startStream, cancelStream } = useMessageStream();
  const { messages, isStreaming } = useChatMessages(threadId);
  const initialMessageSent = useRef(false);
  const titleGeneratedRef = useRef(false);
  const messagesLoadedRef = useRef(false);
  const attachmentManager = useAttachmentManager(threadId);

  const setActive = useThreadRegistryStore((s) => s.setActive);
  const registerThread = useThreadRegistryStore((s) => s.registerThread);
  const markPersisted = useThreadRegistryStore((s) => s.markPersisted);
  const setTitle = useThreadRegistryStore((s) => s.setTitle);
  const touch = useThreadRegistryStore((s) => s.touch);
  const loadMessagesFromDb = useChatMessageStore((s) => s.loadMessagesFromDb);
  const { memoryEnabled } = useMemoryContext();

  // Register the thread and set it active; load its messages from PGlite on mount.
  useEffect(() => {
    setActive(threadId);
    const existing = useThreadRegistryStore.getState().threads[threadId];
    if (!existing) registerThread(threadId);

    if (!messagesLoadedRef.current) {
      messagesLoadedRef.current = true;
      void loadMessagesFromDb(threadId);
    }
  }, [threadId, setActive, registerThread, loadMessagesFromDb]);

  const afterStreamComplete = useCallback(
    async (userMsgText: string) => {
      markPersisted(threadId);
      touch(threadId);
      if (titleGeneratedRef.current) return;
      const currentThread = useThreadRegistryStore.getState().threads[threadId];
      if (currentThread && currentThread.title !== "New conversation") return;
      titleGeneratedRef.current = true;
      const storeMessages = useChatMessageStore.getState().messagesByThread[threadId] ?? [];
      const firstAssistant = storeMessages.find((m) => m.role === "assistant" && m.status === "complete");
      const assistantText = firstAssistant ? extractText(firstAssistant) : "";
      if (!assistantText.trim()) return;
      const title = await useThreadTitleStore.getState().generateTitle(userMsgText, assistantText);
      setTitle(threadId, title);
    },
    [threadId, markPersisted, touch, setTitle],
  );

  const onNew = useCallback(
    async (msg: AppendMessage) => {
      const textPart = msg.content.find((p): p is Extract<(typeof msg.content)[number], { type: "text" }> => p.type === "text");
      if (!textPart) return;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const userText = (textPart as any).text as string;

      // Capture (and clear) pending attachments before starting the stream.
      const attachments = attachmentManager.toPayload();
      attachmentManager.clear();

      // Persist the user message to PGlite before starting the stream
      const userMsg: RichMessage = {
        id: crypto.randomUUID(),
        role: "user",
        content: [{ type: "text", text: userText }],
        createdAt: new Date(),
        status: "complete",
      };
      // Ensure thread exists in DB before inserting message
      const existing = useThreadRegistryStore.getState().threads[threadId];
      if (existing) {
        await db.insertMessage(threadId, userMsg);
      }

      await startStream(
        threadId,
        { message: userText, attachments: attachments.length ? attachments : undefined, memory_enabled: memoryEnabled },
        { onComplete: () => { void afterStreamComplete(userText); } },
      );
    },
    [threadId, db, startStream, afterStreamComplete, attachmentManager, memoryEnabled],
  );

  const onCancel = useCallback(async () => {
    cancelStream();
    useChatMessageStore.getState().finishStream(threadId);
  }, [threadId, cancelStream]);

  useEffect(() => {
    const storeState = useChatMessageStore.getState();
    const currentMessages = storeState.messagesByThread[threadId] ?? [];
    const currentIsStreaming = selectIsStreaming(threadId)(storeState);
    if (initialMessageSent.current || currentMessages.length > 0 || currentIsStreaming) return;
    const pending = consumePendingPrompt();
    if (!pending) return;
    initialMessageSent.current = true;
    void startStream(threadId, { message: pending, memory_enabled: memoryEnabled }, { onComplete: () => { void afterStreamComplete(pending); } });
  }, [threadId, consumePendingPrompt, startStream, afterStreamComplete, memoryEnabled]);

  const threadMessageLikes = useMemo(() => messages.map(richMessageToThreadMessageLike), [messages]);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const runtime = useExternalStoreRuntime({ messages: threadMessageLikes, isRunning: isStreaming, onNew, onCancel } as any);
  return { runtime, attachmentManager };
}
