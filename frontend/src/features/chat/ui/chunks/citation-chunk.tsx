import { BookOpenTextIcon } from "lucide-react";
import type { CitationChunk, RagCitationsChunk } from "@/features/chat/model/chunk";
import { ChunkMeta, ChunkSurface } from "./chunk-surface";
import { safeContentUrl } from "./content-url";

export function CitationChunkView({ chunk }: { chunk: CitationChunk | RagCitationsChunk }) {
  if (chunk.kind === "rag-citations") {
    return (
      <ChunkSurface label="Retrieval sources">
        <details>
          <summary className="flex min-h-11 cursor-pointer list-none items-center gap-2 rounded-lg focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ember"><BookOpenTextIcon size={15} className="text-ember" aria-hidden="true" /><span className="font-mono text-xs">Sources · {chunk.citations.length}</span></summary>
          <ol className="mt-2 space-y-2">{chunk.citations.map((citation) => <li key={`${citation.marker}:${citation.chunkId}`} className="rounded-lg bg-card px-3 py-2"><p className="text-sm font-medium">[{citation.marker}] {citation.documentName}</p><p className="mt-1 text-sm text-fg-sub">{citation.snippet}</p><ChunkMeta>relevance {citation.relevanceScore.toFixed(2)}</ChunkMeta></li>)}</ol>
        </details>
      </ChunkSurface>
    );
  }
  const sourceUrl = safeContentUrl(chunk.url, "download");
  return (
    <ChunkSurface label="Citation">
      <details>
        <summary className="flex min-h-11 cursor-pointer list-none items-center gap-2 rounded-lg focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ember"><BookOpenTextIcon size={15} className="text-ember" aria-hidden="true" /><span className="font-mono text-xs">Source · {chunk.source}</span></summary>
        <blockquote className="mt-2 rounded-lg bg-card px-3 py-2 text-sm text-fg-sub">{chunk.content}</blockquote>
        {sourceUrl ? <a className="mt-2 inline-flex min-h-11 items-center text-sm text-ember underline" href={sourceUrl} target="_blank" rel="noreferrer">Open source</a> : null}
      </details>
    </ChunkSurface>
  );
}
