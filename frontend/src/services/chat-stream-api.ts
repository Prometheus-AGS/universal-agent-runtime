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
 * GET /api/uar/runs/{runId}/stream — resume an in-flight run's SSE stream from
 * `lastEventId`. Used to recover after a mid-stream drop without re-POSTing
 * (which would start a duplicate run). The server replays buffered events after
 * `lastEventId` and continues live.
 */
export async function resumeRunStream(
  runId: string,
  lastEventId: number,
  signal: AbortSignal,
): Promise<Response> {
  return fetch(
    `/api/uar/runs/${encodeURIComponent(runId)}/stream?last_event_id=${lastEventId}`,
    {
      method: "GET",
      headers: {
        Accept: "text/event-stream",
        "Last-Event-ID": String(lastEventId),
      },
      signal,
    },
  );
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
