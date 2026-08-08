import { BrainIcon } from "lucide-react";
import type { ReasoningChunk, ThinkingChunk } from "@/features/chat/model/chunk";
import { ChunkMeta, ChunkSurface } from "./chunk-surface";

export function ReasoningChunkView({ chunk }: { chunk: ReasoningChunk | ThinkingChunk }) {
  const tokenSummary = chunk.kind === "thinking" && chunk.usedTokens !== undefined
    ? `${chunk.usedTokens}${chunk.budgetTokens !== undefined ? ` / ${chunk.budgetTokens}` : ""} tokens`
    : null;
  return (
    <ChunkSurface className="bg-cyan-soft" label="Model reasoning">
      <details>
        <summary className="flex min-h-11 cursor-pointer list-none items-center gap-2 rounded-lg px-1 text-cyan focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ember motion-reduce:transition-none">
          <BrainIcon size={16} aria-hidden="true" />
          <span className="font-mono text-xs">// thinking</span>
          {tokenSummary ? <span className="ml-auto"><ChunkMeta>{tokenSummary}</ChunkMeta></span> : null}
        </summary>
        <p className="mt-2 whitespace-pre-wrap ps-6 font-body text-sm leading-relaxed text-fg-sub">{chunk.text}</p>
      </details>
    </ChunkSurface>
  );
}
