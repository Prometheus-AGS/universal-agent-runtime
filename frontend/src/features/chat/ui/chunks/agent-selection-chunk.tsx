import { SparklesIcon } from "lucide-react";
import type { AgentSelectionChunk } from "@/features/chat/model/chunk";
import { useSelectionAnnouncement } from "@/features/chat/use-selection-announcement";
import { ChunkSurface } from "./chunk-surface";

export function AgentSelectionChunkView({ chunk }: { chunk: AgentSelectionChunk }) {
  const announcement = useSelectionAnnouncement(chunk.id, chunk.announce === true, chunk.agentName);
  return (
    <ChunkSurface label="Agent selection">
      <div className="flex min-w-0 items-start gap-2" data-agent-selection={chunk.agentId}>
        <SparklesIcon size={15} className="mt-0.5 shrink-0 text-fg-faint" aria-hidden="true" />
        <div className="min-w-0 [overflow-wrap:anywhere]">
          <p className="text-sm">Selected agent: <span className="font-medium">{chunk.agentName}</span></p>
          <p className="mt-1 text-xs text-fg-faint">{chunk.selectionMethod === "intent" ? "Intent-based routing" : chunk.selectionMethod}</p>
        </div>
      </div>
      <span className="sr-only" role="status" aria-live="polite" aria-atomic="true">{announcement}</span>
    </ChunkSurface>
  );
}
