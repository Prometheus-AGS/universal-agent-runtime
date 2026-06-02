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

/**
 * POST /api/uar/runs/{runId}/cancel — request server-side cancellation of an
 * in-flight run. Idempotent; safe to call for unknown/finished runs. Best-effort
 * (fire-and-forget): aborting the local stream also triggers server cancellation
 * via the last-subscriber-drop guard, so failures here are non-fatal.
 */
export async function cancelRun(runId: string): Promise<void> {
  try {
    await fetch(`/api/uar/runs/${encodeURIComponent(runId)}/cancel`, {
      method: "POST",
    });
  } catch {
    // Non-fatal: the disconnect guard cancels the run when the stream drops.
  }
}
