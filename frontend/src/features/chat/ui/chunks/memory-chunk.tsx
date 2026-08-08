import { BrainIcon, DatabaseIcon, Edit3Icon } from "lucide-react";
import type { MemoryMutationChunk, MemoryRecallChunk, MemoryUpdateChunk } from "@/features/chat/model/chunk";
import { ChunkMeta, ChunkSurface } from "./chunk-surface";

export function MemoryChunkView({ chunk }: { chunk: MemoryRecallChunk | MemoryMutationChunk | MemoryUpdateChunk }) {
  if (chunk.kind === "memory-recall") {
    return (
      <ChunkSurface className="bg-cyan-soft" label="Memory recall">
        <div className="flex items-center gap-2 text-cyan"><BrainIcon size={15} aria-hidden="true" /><span className="font-mono text-xs font-medium">Memory · read</span><span className="ml-auto text-xs">{chunk.items.length} recalled</span></div>
        <div className="mt-2 space-y-2">{chunk.items.map((item) => <div key={item.id} className="rounded-lg bg-card px-3 py-2"><p className="text-sm font-medium">{item.id}</p><p className="mt-1 text-sm text-fg-sub">{item.content}</p><ChunkMeta>{item.type}{item.pinned ? " · pinned" : ""}</ChunkMeta></div>)}</div>
      </ChunkSurface>
    );
  }
  if (chunk.kind === "memory-update") {
    return <ChunkSurface label="Memory update"><div className="flex items-center gap-2"><DatabaseIcon size={15} className="text-cyan" aria-hidden="true" /><span className="font-mono text-xs">Memory · updated · {chunk.scope}</span><span className="ml-auto text-xs">{chunk.itemCount} items</span></div><p className="mt-1 text-sm text-fg-sub">{chunk.summary}</p></ChunkSurface>;
  }
  return <ChunkSurface label="Memory mutation"><div className="flex items-center gap-2"><Edit3Icon size={15} className="text-cyan" aria-hidden="true" /><span className="font-mono text-xs">Memory · {chunk.operation}</span><span className="ml-auto text-xs">{chunk.memoryId}</span></div>{chunk.content ? <p className="mt-1 text-sm text-fg-sub">{chunk.content}</p> : null}</ChunkSurface>;
}
