/** POST /api/chat/completion — SSE streaming chat. */
export const CHAT_COMPLETION_URL = "/api/chat/completion";

export async function postChatCompletion(
  body: string,
  sessionId: string,
  signal: AbortSignal,
): Promise<Response> {
  return fetch(CHAT_COMPLETION_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json", "X-UAR-Session-ID": sessionId },
    body,
    signal,
  });
}
