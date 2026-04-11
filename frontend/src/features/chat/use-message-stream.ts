import { useChatStreamStore } from "@/stores/chat-stream-store";

export type {
  StreamRetryPolicy,
  UarChatPayload,
  StreamCallbacks,
} from "@/stores/chat-stream-store";

export {
  DEFAULT_STREAM_RETRY_POLICY,
  isRetryableHttpStatus,
  isRetryableTransportError,
  computeRetryDelayMs,
} from "@/stores/chat-stream-store";

/** Binds chat streaming to React; I/O runs in the chat stream store. */
export function useMessageStream() {
  const startStream = useChatStreamStore((s) => s.startStream);
  const cancelStream = useChatStreamStore((s) => s.cancelStream);
  return { startStream, cancelStream };
}
