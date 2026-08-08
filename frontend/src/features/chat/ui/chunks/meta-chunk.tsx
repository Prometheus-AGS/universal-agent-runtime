import { AlertTriangleIcon } from "lucide-react";
import type { DividerChunk, ErrorChunk, UsageChunk } from "@/features/chat/model/chunk";
import { ChunkSurface } from "./chunk-surface";

export function DividerChunkView({ chunk }: { chunk: DividerChunk }) {
  return <div className="my-5 h-3" role="separator" data-chunk-id={chunk.id} />;
}

export function UsageChunkView({ chunk }: { chunk: UsageChunk }) {
  return <p className="my-2 font-mono text-[10px] text-fg-faint" aria-label="Run usage">{chunk.inputTokens.toLocaleString()} input · {chunk.outputTokens.toLocaleString()} output · {chunk.totalTokens.toLocaleString()} total{chunk.model ? ` · ${chunk.model}` : ""}{chunk.costUsd !== undefined ? ` · $${chunk.costUsd.toFixed(4)}` : ""}</p>;
}

export function ErrorChunkView({ chunk }: { chunk: ErrorChunk }) {
  return <ChunkSurface className="bg-destructive/10" label="Run error" live><div className="flex items-center gap-2 text-destructive"><AlertTriangleIcon size={16} aria-hidden="true" /><span className="font-mono text-xs font-medium">Failed{chunk.code ? ` · ${chunk.code}` : ""}</span></div><p className="mt-2 whitespace-pre-wrap text-sm">{chunk.message}</p>{chunk.retryable ? <p className="mt-2 text-xs">Retry is available from the message actions.</p> : null}</ChunkSurface>;
}
