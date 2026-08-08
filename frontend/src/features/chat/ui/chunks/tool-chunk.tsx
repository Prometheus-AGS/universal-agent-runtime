import { WrenchIcon } from "lucide-react";
import type { ToolCallChunk } from "@/features/chat/model/chunk";
import { ChunkMeta, ChunkSurface, JsonSource } from "./chunk-surface";

export function ToolChunkView({ chunk }: { chunk: ToolCallChunk }) {
  return (
    <ChunkSurface label={`Tool ${chunk.toolName}`} live={chunk.status === "running"}>
      <details>
        <summary className="flex min-h-11 cursor-pointer list-none items-center gap-2 rounded-lg px-1 focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ember">
          <WrenchIcon size={15} className="text-ember" aria-hidden="true" />
          <span className="font-mono text-xs font-medium">{chunk.server ? `${chunk.server}__` : ""}{chunk.toolName}</span>
          <span className="ml-auto rounded-full bg-card px-2 py-1 font-mono text-[10px]">{chunk.status}</span>
          {chunk.durationMs !== undefined ? <ChunkMeta>{chunk.durationMs} ms</ChunkMeta> : null}
        </summary>
        <div className="mt-2 space-y-2 ps-6">
          <JsonSource value={chunk.args} label="Tool arguments" />
          {chunk.result !== undefined ? <JsonSource value={chunk.result} label="Tool result" /> : <p className="text-xs text-fg-faint">Waiting for result</p>}
        </div>
      </details>
    </ChunkSurface>
  );
}
