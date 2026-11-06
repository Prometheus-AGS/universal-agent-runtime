import type { ContentBlock } from "@/types/chat-content";

export interface ServerSessionMessage {
  role: "user" | "assistant";
  content: string | ContentBlock[];
  message_id?: string;
  run_id?: string | null;
  client_turn_id?: string | null;
  created_at?: string | null;
  status?: string;
  provenance?: string;
  run_metadata?: Record<string, unknown>;
}
function conversationalMessages(payload: unknown, sessionId: string): ServerSessionMessage[] {
  if (!payload || typeof payload !== "object") throw new Error("Session response is not an object");
  const body = payload as { history_version?: unknown; history_runs_availability?: unknown; history?: unknown; messages?: unknown; history_runs?: Record<string, Record<string, unknown>> };
  const messages = body.history_version === 1 ? body.history : body.messages;
  if (!Array.isArray(messages)) throw new Error("Session response has no messages");
  return messages.flatMap((message, index): ServerSessionMessage[] => {
    if (!message || typeof message !== "object") return [];
    const row = message as ServerSessionMessage;
    if ((row.role !== "user" && row.role !== "assistant") || (typeof row.content !== "string" && !Array.isArray(row.content))) return [];
    return [{ ...row, message_id: row.message_id ?? `legacy:${sessionId}:${index}:${row.role}`,
      status: row.status ?? "unknown", provenance: row.provenance ?? "unavailable",
      run_metadata: { history_runs_availability: body.history_runs_availability ?? "unavailable", ...(row.run_id ? body.history_runs?.[row.run_id] : undefined) } }];
  });
}
export async function fetchSessionMessages(sessionId: string): Promise<ServerSessionMessage[] | null> {
  const response = await fetch(`/api/uar/sessions/${encodeURIComponent(sessionId)}`, { cache: "no-store" });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`Session request failed: ${response.status}`);
  return conversationalMessages(await response.json(), sessionId);
}
