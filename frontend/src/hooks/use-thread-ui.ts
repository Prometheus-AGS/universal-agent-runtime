import { useChatMessageStore, selectMessageById } from "@/stores/chat-message-store";
import { useThreadRegistryStore } from "@/stores/thread-registry-store";

export type ChatMessageStoreState = ReturnType<typeof useChatMessageStore.getState>;

export { selectMessageById };

/** Subscribe to thread registry state and submit thread mutation intents. */
export function useThreadUi() {
  return {
    threads: useThreadRegistryStore((state) => state.threads),
    activeThreadId: useThreadRegistryStore((state) => state.activeThreadId),
    registerThread: useThreadRegistryStore((state) => state.registerThread),
    setTitle: useThreadRegistryStore((state) => state.setTitle),
    setActive: useThreadRegistryStore((state) => state.setActive),
    removeThread: useThreadRegistryStore((state) => state.removeThread),
    messagesByThread: useChatMessageStore((state) => state.messagesByThread),
    clearThread: useChatMessageStore((state) => state.clearThread),
  };
}

/** Select chat message state without exposing the store to components. */
export function useChatMessageSelector<T>(selector: (state: ReturnType<typeof useChatMessageStore.getState>) => T) {
  return useChatMessageStore(selector);
}
