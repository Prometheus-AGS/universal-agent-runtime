import { useMemo } from "react";
import type { RagCitationMarker } from "@/types/chat-content";
import { selectMessageById, useChatMessageSelector } from "@/hooks/use-thread-ui";

const EMPTY_CITATIONS: RagCitationMarker[] = [];

/**
 * Numbered RAG citation markers ([1], [2], ...) attached to one message, for
 * rendering the hover-to-source panel. Subscribes to the chat message store
 * via `selectMessageById` — components never touch the store directly.
 *
 * Returns a stable empty array reference when the thread/message isn't
 * resolved yet or carries no `rag-citations` content block, so consumers can
 * safely check `.length` without an extra null-guard.
 */
export function useMessageCitations(threadId: string | null, messageId: string): RagCitationMarker[] {
  const selector = useMemo(
    () => (threadId ? selectMessageById(threadId, messageId) : () => null),
    [threadId, messageId],
  );
  const message = useChatMessageSelector(selector);

  return useMemo(() => {
    if (!message) return EMPTY_CITATIONS;
    const block = message.content.find((b) => b.type === "rag-citations");
    return block && block.type === "rag-citations" ? block.citations : EMPTY_CITATIONS;
  }, [message]);
}
