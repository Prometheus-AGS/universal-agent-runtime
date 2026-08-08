import { DatabaseZapIcon } from "lucide-react";
import type { ContextUpdateChunk } from "@/features/chat/model/chunk";
import { ChunkSurface } from "./chunk-surface";

export function ContextChunkView({ chunk }: { chunk: ContextUpdateChunk }) {
  return <ChunkSurface label="Context update"><div className="flex items-center gap-2"><DatabaseZapIcon size={14} className="text-cyan" aria-hidden="true" /><span className="text-sm">Compacted {chunk.messagesRemoved} messages, saved {chunk.tokensSaved.toLocaleString()} tokens</span><span className="ml-auto font-mono text-[10px] text-fg-faint">{chunk.wasApplied ? "applied" : "not applied"}</span></div></ChunkSurface>;
}
