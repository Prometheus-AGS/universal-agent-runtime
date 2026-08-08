import type { MarkdownChunk, TextChunk } from "@/features/chat/model/chunk";
import { MarkdownBubble } from "@/shared/markdown";

export function TextChunkView({ chunk }: { chunk: TextChunk | MarkdownChunk }) {
  if (chunk.kind === "markdown") return <MarkdownBubble source={chunk.source} />;
  return <p className="whitespace-pre-wrap font-body text-sm leading-relaxed">{chunk.text}</p>;
}
