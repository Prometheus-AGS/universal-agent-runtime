import { useThreadTitleStore } from "@/stores/thread-title-store";

/** Generate a title through the store-owned service action. */
export function generateThreadTitle(userMsg: string, assistantMsg: string) {
  return useThreadTitleStore.getState().generateTitle(userMsg, assistantMsg);
}
