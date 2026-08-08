import { FileIcon, ImageOffIcon } from "lucide-react";
import type { FileChunk, ImageChunk, VideoChunk } from "@/features/chat/model/chunk";
import { ChunkSurface } from "./chunk-surface";
import { safeContentUrl } from "./content-url";

export function MediaChunkView({ chunk }: { chunk: ImageChunk | VideoChunk | FileChunk }) {
  if (chunk.kind === "image") {
    const imageUrl = safeContentUrl(chunk.url, "image");
    if (!imageUrl || !chunk.alt?.trim()) return <ChunkSurface label="Image unavailable"><div className="flex items-center gap-2"><ImageOffIcon size={16} aria-hidden="true" /><span className="text-sm">Image preview unavailable: a source and description are required.</span></div></ChunkSurface>;
    return <figure className="my-2 overflow-hidden rounded-xl bg-surface p-2"><img className="h-auto max-w-full rounded-lg" src={imageUrl} alt={chunk.alt} width={chunk.width} height={chunk.height} /><figcaption className="mt-2 text-xs text-fg-faint">{chunk.alt}</figcaption></figure>;
  }
  if (chunk.kind === "video") {
    const videoUrl = safeContentUrl(chunk.url, "video");
    const posterUrl = safeContentUrl(chunk.poster, "image");
    return <ChunkSurface label="Video artifact">{videoUrl ? <video className="w-full rounded-lg bg-card" controls preload="metadata" poster={posterUrl || undefined} aria-label="Generated video"><source src={videoUrl} /></video> : <p className="text-sm">Video preview unavailable.</p>}</ChunkSurface>;
  }
  const fileUrl = safeContentUrl(chunk.url, "download");
  return <ChunkSurface label={`File ${chunk.name}`}><div className="flex items-center gap-2"><FileIcon size={16} aria-hidden="true" /><span className="text-sm">{chunk.name}</span><span className="ml-auto font-mono text-[10px] text-fg-faint">{chunk.mime} · {chunk.bytes.toLocaleString()} bytes</span></div>{fileUrl ? <a className="mt-2 inline-flex min-h-11 items-center text-sm text-ember underline" href={fileUrl} download={chunk.name}>Download file</a> : null}</ChunkSurface>;
}
