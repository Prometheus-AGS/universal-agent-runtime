import { useChatMessageStore, selectIsStreaming } from "@/stores/chat-message-store";
import type { RichMessage } from "@/types/chat-content";

const EMPTY_MESSAGES: RichMessage[] = [];

export function useChatMessages(threadId: string | null) {
  const messages = useChatMessageStore((state) => state.messagesByThread[threadId ?? "__none__"] ?? EMPTY_MESSAGES);
  const isStreaming = useChatMessageStore(selectIsStreaming(threadId ?? "__none__"));
  const localIsEmpty = messages === EMPTY_MESSAGES || messages.length === 0;

  return { messages, isStreaming, isLoading: localIsEmpty && !isStreaming };
}
