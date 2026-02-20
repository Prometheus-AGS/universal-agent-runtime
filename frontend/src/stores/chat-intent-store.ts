import { create } from "zustand";

interface ChatIntentState {
  pendingPrompt: string | null;
}
interface ChatIntentActions {
  setPendingPrompt: (prompt: string) => void;
  consumePendingPrompt: () => string | null;
}
export type ChatIntentStore = ChatIntentState & ChatIntentActions;

export const useChatIntentStore = create<ChatIntentStore>((set, get) => ({
  pendingPrompt: null,
  setPendingPrompt: (prompt) => set({ pendingPrompt: prompt }),
  consumePendingPrompt: () => {
    const prompt = get().pendingPrompt;
    if (prompt !== null) set({ pendingPrompt: null });
    return prompt;
  },
}));
