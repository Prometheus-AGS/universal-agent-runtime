import type { ContentBlock, RichMessage } from "@/types/chat-content";
import type { ServerSessionMessage } from "@/services/chat-sessions-api";

function contentOf(content: ServerSessionMessage["content"]): ContentBlock[] {
  if (typeof content === "string") return [{ type: "text", text: content }];
  return content.flatMap((block): ContentBlock[] => {
    // Canonical model content uses image_url; portable UI history uses image.
    const wire = block as unknown as { type: string; image_url?: { url: string }; text?: string };
    if (wire.type === "image_url" && typeof wire.image_url?.url === "string") {
      return [{ type: "image", url: wire.image_url.url, dataBase64: null, mime: "image/*" }];
    }
    return [block];
  });
}
function textOf(content: ContentBlock[]): string {
  return content.flatMap((block) => block.type === "text" ? [block.text] : []).join("");
}
/** A divergent local user sequence belongs to another history snapshot. */
export function matchesServerHistory(local: RichMessage[], server: ServerSessionMessage[]): boolean {
  const userLocal = local.filter((message) => message.role === "user");
  const userRemote = server.filter((message) => message.role === "user");
  return userLocal.length <= userRemote.length && userLocal.every((message, index) => {
    const remote = userRemote[index];
    if (message.id === remote.message_id) return true;
    // Only legacy text aliases lack authoritative identity. A caption cannot
    // identify media, and conflicting modern IDs cannot identify the same turn.
    if (message.clientTurnId || remote.client_turn_id || message.runId || remote.run_id) return false;
    if (remote.message_id && !remote.message_id.startsWith("legacy:") && !message.id.startsWith("legacy:")) return false;
    const remoteContent = contentOf(remote.content);
    if (!message.content.every(block => block.type === "text") || !remoteContent.every(block => block.type === "text")) return false;
    const text = textOf(message.content).trim();
    return text.length > 0 && text === textOf(remoteContent).trim();
  });
}
/** Stable identity wins. Legacy text matching is confined to matching user turns. */
export function missingServerMessages(local: RichMessage[], server: ServerSessionMessage[]): RichMessage[] {
  if (!matchesServerHistory(local, server)) return [];
  const ids = new Set(local.map((message) => message.id));
  const userLocal = local.filter((message) => message.role === "user");
  const metadataRuns = new Set<string>();
  let userIndex = -1;
  return server.flatMap((message, index): RichMessage[] => {
    if (message.role === "user") userIndex += 1;
    const id = message.message_id ?? `legacy:${index}:${message.role}`;
    // Run evidence belongs to one canonical assistant, even when that row is cached.
    const ownsRunMetadata = message.role === "assistant" && Boolean(message.run_id) && !metadataRuns.has(message.run_id!);
    if (ownsRunMetadata) metadataRuns.add(message.run_id!);
    if (ids.has(id)) return [];
    if (message.role === "user" && userIndex < userLocal.length) return [];
    if (message.role === "assistant" && userIndex < userLocal.length) {
      const start = local.indexOf(userLocal[userIndex]);
      const next = local.slice(start + 1).find((row) => row.role === "user" || row.role === "assistant");
      if (next?.role === "assistant" && !message.run_id) return [];
    }
    const content = contentOf(message.content);
    const metadata = { provenance: message.provenance ?? "unavailable",
      sources_availability: "unavailable", usage_availability: "unavailable", policy_availability: "unavailable", approvals_availability: "unavailable", approvals: [],
      ...(message.run_id && !ownsRunMetadata ? undefined : message.run_metadata),
      created_at_availability: message.created_at ? "available" : "unavailable" };
    content.push({ type: "artifact", id: `${id}:history`, kind: "application/vnd.uar.history-provenance+json", title: "History evidence", content: JSON.stringify(metadata) });
    return [{ id, role: message.role, content, createdAt: new Date(message.created_at ?? 0), createdAtKnown: Boolean(message.created_at), historyPosition: index,
      status: message.status === "complete" ? "complete" : message.status === "error" ? "failed"
        : message.status === "cancelled" ? "cancelled" : "interrupted",
      runId: message.run_id ?? undefined, clientTurnId: message.client_turn_id ?? undefined }];
  });
}
