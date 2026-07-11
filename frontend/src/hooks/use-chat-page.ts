import { useEffect } from "react";
import { useChatPageStore } from "@/stores/chat-page-store";

/** Load and expose the chat-page model readiness contract. */
export function useChatPage() {
  const checkModel = useChatPageStore((state) => state.checkModel);
  useEffect(() => { void checkModel(); }, [checkModel]);
  return { modelCheck: useChatPageStore((state) => state.modelCheck) };
}
