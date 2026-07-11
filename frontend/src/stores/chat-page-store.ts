import { create } from "zustand";
import { fetchResolveModel } from "@/services/models-api";

interface ChatPageState {
  modelCheck: { loading: boolean; ok: boolean; error?: string };
  checkModel: () => Promise<void>;
}

export const useChatPageStore = create<ChatPageState>((set) => ({
  modelCheck: { loading: true, ok: true },
  checkModel: async () => {
    try {
      const result = await fetchResolveModel();
      set({ modelCheck: { loading: false, ok: result.ok } });
    } catch {
      set({ modelCheck: { loading: false, ok: false, error: "Could not verify model configuration." } });
    }
  },
}));
