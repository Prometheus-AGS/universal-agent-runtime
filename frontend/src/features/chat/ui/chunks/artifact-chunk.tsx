import { FileJsonIcon } from "lucide-react";
import type { ArtifactChunk } from "@/features/chat/model/chunk";
import { MarkdownBubble } from "@/shared/markdown";
import { sanitizeRawSvg } from "@/shared/markdown/plugins/sanitize-raw-svg";
import { ChartChunkView } from "./chart-chunk";
import { parseChartModel } from "./chart-model";
import { ChunkSurface, JsonSource } from "./chunk-surface";
import { safeContentUrl } from "./content-url";

function languageFromMime(mime: string): string {
  return /language=([^;]+)/u.exec(mime)?.[1] ?? "text";
}

export function ArtifactChunkView({ chunk }: { chunk: ArtifactChunk }) {
  const source = chunk.content ?? "";
  const downloadUrl = safeContentUrl(chunk.url, "download");
  if (chunk.mime === "text/markdown") return <ChunkSurface label={chunk.title ?? "Markdown artifact"}><MarkdownBubble source={source} /></ChunkSurface>;
  if (chunk.mime.startsWith("text/x-code")) return <ChunkSurface label={chunk.title ?? "Code artifact"}><MarkdownBubble source={`\`\`\`${languageFromMime(chunk.mime)}\n${source}\n\`\`\``} /></ChunkSurface>;
  if (chunk.mime === "text/x-mermaid") return <ChunkSurface label={chunk.title ?? "Diagram artifact"}><MarkdownBubble source={`\`\`\`mermaid\n${source}\n\`\`\``} /></ChunkSurface>;
  if (chunk.mime === "image/svg+xml") {
    const sanitized = sanitizeRawSvg(source);
    return sanitized
      ? <ChunkSurface label={chunk.title ?? "SVG artifact"}><div role="img" aria-label={chunk.title ?? "Generated SVG"} dangerouslySetInnerHTML={{ __html: sanitized }} /></ChunkSurface>
      : <ChunkSurface label="Invalid SVG artifact"><p className="text-sm text-destructive">SVG preview unavailable</p><JsonSource value={source} label="SVG source" /></ChunkSurface>;
  }
  if (chunk.mime === "text/html") {
    return <ChunkSurface label={chunk.title ?? "HTML artifact"}><iframe className="h-72 w-full rounded-lg bg-card" sandbox="" srcDoc={source} title={chunk.title ?? "Sandboxed HTML artifact"} /><details className="mt-2"><summary className="min-h-11 cursor-pointer py-2 text-xs">Show source</summary><JsonSource value={source} label="HTML source" /></details></ChunkSurface>;
  }
  if (chunk.mime === "application/vnd.uar.chart+json") {
    const model = parseChartModel(source);
    return model ? <ChartChunkView model={model} /> : <ChunkSurface label="Invalid chart artifact"><p className="mb-2 text-sm text-destructive">Chart preview unavailable</p><JsonSource value={source} label="Chart source" /></ChunkSurface>;
  }
  if (chunk.mime === "application/json" || chunk.mime.endsWith("+json")) {
    let json: unknown = source;
    try { json = JSON.parse(source) as unknown; } catch { /* escaped source remains visible */ }
    return <ChunkSurface label={chunk.title ?? "JSON artifact"}><div className="mb-2 flex items-center gap-2"><FileJsonIcon size={15} className="text-ember" aria-hidden="true" /><span className="font-mono text-xs">{chunk.title ?? chunk.mime}</span></div><JsonSource value={json} /></ChunkSurface>;
  }
  return <ChunkSurface label={chunk.title ?? "Artifact"}><p className="font-mono text-xs">{chunk.title ?? chunk.mime}</p>{downloadUrl ? <a className="mt-2 inline-flex min-h-11 items-center text-sm text-ember underline" href={downloadUrl} download>Download artifact</a> : <JsonSource value={source} label="Artifact source" />}</ChunkSurface>;
}
