import { create } from "zustand";

import { generateThreadTitle as generateThreadTitleApi } from "@/services/chat-titles-api";

interface ThreadTitleActions {
  generateTitle: (userMsg: string, assistantMsg: string) => Promise<string>;
}

export const useThreadTitleStore = create<ThreadTitleActions>(() => ({
  generateTitle: (userMsg, assistantMsg) => generateThreadTitleApi(userMsg, assistantMsg),
}));
