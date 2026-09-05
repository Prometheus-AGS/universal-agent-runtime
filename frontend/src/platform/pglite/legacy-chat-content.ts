import type { Chunk } from "@/shared/content/chunk";
import type { ContentBlock } from "@/shared/content/content-block";
import { toChunks } from "@/shared/content/to-chunks";

interface DecodeContext {
  messageId: string;
  at: string;
  finalized: boolean;
}

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function text(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

function number(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function canonicalBlock(value: unknown): ContentBlock | null {
  const block = record(value);
  switch (block.type) {
    case "text": return { type: "text", text: text(block.text) };
    case "thinking": return { type: "thinking", text: text(block.text) };
    case "code": return { type: "code", language: text(block.language), code: text(block.code) };
    case "citation": return { type: "citation", source: text(block.source, "Source"), quote: text(block.quote, text(block.content)) };
    case "memory": return { type: "memory", operation: text(block.operation, "update"), key: text(block.key), value: block.value === null ? null : text(block.value) };
    case "toolUse": return { type: "toolUse", id: text(block.id), name: text(block.name), inputJson: text(block.inputJson, "{}") };
    case "toolResult": return { type: "toolResult", toolUseId: text(block.toolUseId), outputJson: text(block.outputJson), isError: block.isError === true };
    case "skill": return { type: "skill", name: text(block.name), status: text(block.status, "complete") };
    case "artifact": return { type: "artifact", id: text(block.id), kind: text(block.kind, "application/octet-stream"), content: text(block.content), title: typeof block.title === "string" ? block.title : undefined };
    case "image": return { type: "image", url: typeof block.url === "string" ? block.url : null, dataBase64: typeof block.dataBase64 === "string" ? block.dataBase64 : null, mime: text(block.mime, "image/unknown"), path: typeof block.path === "string" ? block.path : undefined, alt: typeof block.alt === "string" ? block.alt : undefined, width: typeof block.width === "number" ? block.width : undefined, height: typeof block.height === "number" ? block.height : undefined };
    case "divider": return { type: "divider" };
    default: return null;
  }
}

/** Decodes the pre-C12 partial union without widening the canonical contract. */
export function decodePersistedChatContent(raw: unknown, context: DecodeContext): { content: ContentBlock[]; chunks: Chunk[] } {
  const values = Array.isArray(raw) ? raw : [];
  const content: ContentBlock[] = [];
  const runtimeChunks: Chunk[] = [];
  const replacedPortableChunkIds = new Set<string>();

  values.forEach((value, index) => {
    const canonical = canonicalBlock(value);
    const block = record(value);
    if (canonical) {
      content.push(canonical);
      return;
    }
    const base = { id: `${context.messageId}:legacy:${index}`, at: context.at, seq: index };
    switch (block.type) {
      case "reasoning":
        content.push({ type: "thinking", text: text(block.text) });
        break;
      case "tool-call": {
        const portableIndex = content.length;
        const toolCallId = text(block.toolCallId, base.id);
        const toolName = text(block.toolName, "Unknown tool");
        const args = record(block.args);
        content.push({ type: "toolUse", id: toolCallId, name: toolName, inputJson: JSON.stringify(args) });
        if (block.result !== undefined) content.push({ type: "toolResult", toolUseId: toolCallId, outputJson: text(block.result, JSON.stringify(block.result)), isError: block.status === "failed" });
        replacedPortableChunkIds.add(`${context.messageId}:toolUse:${portableIndex}`);
        runtimeChunks.push({ ...base, kind: "tool-call", toolCallId, toolName, args, result: typeof block.result === "string" ? block.result : undefined, status: block.status === "failed" ? "failed" : block.status === "complete" ? "complete" : "running" });
        break;
      }
      case "rag-citations": {
        const citations = Array.isArray(block.citations) ? block.citations.map((item, citationIndex) => {
          const citation = record(item);
          return { marker: number(citation.marker, citationIndex + 1), chunkId: text(citation.chunkId, `${base.id}:${citationIndex}`), documentId: text(citation.documentId) || undefined, documentName: text(citation.documentName, "Document"), relevanceScore: number(citation.relevanceScore), snippet: text(citation.snippet) };
        }) : [];
        const portableIndex = content.length;
        content.push({ type: "artifact", id: base.id, kind: "application/vnd.uar.rag-citations+json", content: JSON.stringify({ citations }) });
        replacedPortableChunkIds.add(`${context.messageId}:artifact:${portableIndex}`);
        runtimeChunks.push({ ...base, kind: "rag-citations", citations });
        break;
      }
      case "skill-activation": {
        const portableIndex = content.length;
        content.push({ type: "skill", name: text(block.skillName, text(block.skillId)), status: text(block.status, "complete") });
        replacedPortableChunkIds.add(`${context.messageId}:skill:${portableIndex}`);
        runtimeChunks.push({ ...base, kind: "skill-activation", skillId: text(block.skillId, base.id), skillName: text(block.skillName, "Skill"), selectionMethod: typeof block.selectionMethod === "string" && ["keyword", "embedding", "hybrid", "llm", "explicit"].includes(block.selectionMethod) ? block.selectionMethod as "keyword" | "embedding" | "hybrid" | "llm" | "explicit" : undefined, status: block.status === "active" ? "active" : "complete" });
        break;
      }
      case "context-update": {
        const portableIndex = content.length;
        const chunk = { ...base, kind: "context-update" as const, strategy: text(block.strategy, "unknown"), messagesRemoved: number(block.messagesRemoved), tokensSaved: number(block.tokensSaved), wasApplied: block.wasApplied === true, summaryGenerated: block.summaryGenerated === true };
        content.push({ type: "artifact", id: base.id, kind: "application/vnd.uar.context-update+json", content: JSON.stringify(chunk) });
        replacedPortableChunkIds.add(`${context.messageId}:artifact:${portableIndex}`);
        runtimeChunks.push(chunk);
        break;
      }
      case "error": {
        const portableIndex = content.length;
        const message = text(block.message, "An error occurred");
        content.push({ type: "artifact", id: base.id, kind: "application/vnd.uar.error+json", content: JSON.stringify({ message, code: block.code }) });
        replacedPortableChunkIds.add(`${context.messageId}:artifact:${portableIndex}`);
        runtimeChunks.push({ ...base, kind: "error", message, code: text(block.code) || undefined });
        break;
      }
    }
  });

  return {
    content,
    chunks: [
      ...toChunks(content, context).filter((chunk) => !replacedPortableChunkIds.has(chunk.id)),
      ...runtimeChunks,
    ].sort((left, right) => left.seq - right.seq),
  };
}
