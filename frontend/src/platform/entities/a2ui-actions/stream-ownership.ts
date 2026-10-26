/** Document-local workflow ownership, never persisted execution authority. */
let owner: symbol | null = null;
export interface ChatStreamLease { readonly token: symbol; release: () => void }
export function acquireChatStream(): ChatStreamLease {
  if (owner) throw new Error("Another response is active. Wait for it to finish.");
  const token = Symbol("chat-stream");
  owner = token;
  return { token, release: () => { if (owner === token) owner = null; } };
}
export function ownsChatStream(lease: ChatStreamLease): boolean { return owner === lease.token; }
