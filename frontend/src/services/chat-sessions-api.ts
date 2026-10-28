/** A conversational message as stored in the server session. */
export interface ServerSessionMessage {
  role: "user" | "assistant";
  content: string;
}

function conversationalMessages(payload: unknown): ServerSessionMessage[] {
  if (!payload || typeof payload !== "object") throw new Error("Session response is not an object");
  const messages = (payload as { messages?: unknown }).messages;
  if (!Array.isArray(messages)) throw new Error("Session response has no messages");
  return messages.flatMap((message): ServerSessionMessage[] => {
    if (!message || typeof message !== "object") return [];
    const { role, content } = message as { role?: unknown; content?: unknown };
    if ((role !== "user" && role !== "assistant") || typeof content !== "string" || !content.trim()) return [];
    return [{ role, content }];
  });
}

/**
 * Reads the user and assistant messages of one owned server session.
 * Returns null when the server holds no session with this id.
 */
export async function fetchSessionMessages(sessionId: string): Promise<ServerSessionMessage[] | null> {
  const response = await fetch(`/api/uar/sessions/${encodeURIComponent(sessionId)}`);
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`Session request failed: ${response.status}`);
  return conversationalMessages(await response.json());
}
