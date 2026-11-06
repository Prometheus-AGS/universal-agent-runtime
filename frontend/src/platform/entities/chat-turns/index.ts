import { graphStore, registerSchema } from "@prometheus-ags/prometheus-entity-management";
import type { RichMessage } from "@/types/chat-content";
import { persistA2uiActionReceipt } from "../a2ui-actions/durability";
import { fetchChatTurnReceipt } from "@/services/chat-stream-api";

const ENTITY = "ChatTurnReceipt";
export interface ChatTurnReceipt extends Record<string, unknown> {
  id: string;
  session_id: string;
  user_message_id: string;
  message: RichMessage;
  state: "prepared" | "sending" | "uncertain" | "admitted" | "settled";
  run_id: string | null;
}
let registered = false;
export function registerChatTurnEntities(): void {
  if (registered) return;
  registerSchema({ type: ENTITY });
  registered = true;
}
export function readChatTurn(id: string): ChatTurnReceipt | undefined {
  return graphStore.getState().readEntity<ChatTurnReceipt>(ENTITY, id) ?? undefined;
}
export async function persistChatTurn(receipt: ChatTurnReceipt): Promise<void> {
  graphStore.getState().replaceEntity(ENTITY, receipt.id, receipt);
  await persistA2uiActionReceipt();
}
export async function reconcileChatTurn(id: string) {
  const local = readChatTurn(id);
  if (!local) return null;
  const remote = await fetchChatTurnReceipt(local.session_id, id);
  if (remote.run_id) await persistChatTurn({ ...local, run_id: remote.run_id,
    state: ["complete", "error", "cancelled", "interrupted"].includes(remote.state) ? "settled" : "admitted" });
  return remote;
}
export function pendingChatTurns(sessionId: string): ChatTurnReceipt[] {
  return (Object.values(graphStore.getState().entities[ENTITY] ?? {}) as ChatTurnReceipt[])
    .filter((row) => Boolean(row) && row.session_id === sessionId && row.state !== "settled");
}
