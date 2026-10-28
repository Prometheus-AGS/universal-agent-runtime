import type { RichMessage } from "@/types/chat-content";
import type { ServerSessionMessage } from "@/services/chat-sessions-api";

interface Turn {
  user: string;
  reply: string | null;
}

function userText(message: RichMessage): string {
  return message.content
    .flatMap((block) => (block.type === "text" ? [block.text] : []))
    .join("");
}

function localTurns(messages: RichMessage[]): Turn[] {
  const turns: Turn[] = [];
  for (const message of messages) {
    if (message.role === "user") turns.push({ user: userText(message), reply: null });
    else if (message.role === "assistant" && turns.length > 0) turns[turns.length - 1] = { ...turns[turns.length - 1], reply: "" };
  }
  return turns;
}

function serverTurns(messages: ServerSessionMessage[]): Turn[] {
  const turns: Turn[] = [];
  for (const message of messages) {
    if (message.role === "user") turns.push({ user: message.content, reply: null });
    else if (turns.length > 0) turns[turns.length - 1] = { ...turns[turns.length - 1], reply: message.content };
  }
  return turns;
}

function textMessage(role: "user" | "assistant", text: string, createdAt: Date): RichMessage {
  return { id: crypto.randomUUID(), role, content: [{ type: "text", text }], createdAt, status: "complete" };
}

/**
 * Returns the server messages the local history is missing at its end.
 *
 * Backfills only when every local user message matches the server's in order;
 * otherwise the histories diverge and nothing is returned. A missing reply to
 * the last local turn is added, followed by any later server turns.
 */
export function missingServerMessages(local: RichMessage[], server: ServerSessionMessage[], now = Date.now()): RichMessage[] {
  const known = localTurns(local);
  const remote = serverTurns(server);
  if (known.length > remote.length) return [];
  if (known.some((turn, index) => turn.user.trim() !== remote[index].user.trim())) return [];

  const missing: RichMessage[] = [];
  const at = () => new Date(now + missing.length);
  const last = known.length - 1;
  const lostReply = last >= 0 && known[last].reply === null ? remote[last].reply : null;
  if (lostReply) missing.push(textMessage("assistant", lostReply, at()));
  for (const turn of remote.slice(known.length)) {
    missing.push(textMessage("user", turn.user, at()));
    if (turn.reply) missing.push(textMessage("assistant", turn.reply, at()));
  }
  return missing;
}
