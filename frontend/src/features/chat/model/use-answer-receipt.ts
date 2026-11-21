import { useMemo } from "react";

import { useChatMessageStore } from "@/stores/chat-message-store";
import { useThreadRegistryStore } from "@/stores/thread-registry-store";
import { parseReceiptSource, receiptSourceOf, type ModelReceipt } from "./model-receipt";

/**
 * The model receipt of one assistant message, or null while the runtime has not sent one (a turn
 * still running, refused, failed or cancelled). The store subscription returns a string, so the
 * component rerenders only when the receipt itself changes, not on every streamed token.
 */
export function useAnswerReceipt(messageId: string): ModelReceipt | null {
  const threadId = useThreadRegistryStore((state) => state.activeThreadId);
  const source = useChatMessageStore((state) => {
    if (!threadId) return null;
    const message = state.messagesByThread[threadId]?.find((row) => row.id === messageId);
    return message ? receiptSourceOf(message.content) : null;
  });
  return useMemo(() => parseReceiptSource(source), [source]);
}
