import { assertNever, type ContentBlock } from "./content-block";
import type { Chunk, MemoryMutationChunk, ToolCallChunk } from "./chunk";

export interface ChunkProjectionContext {
  messageId: string;
  at: string;
  runId?: string;
  finalized?: boolean;
  sequenceStart?: number;
}

function parseRecord(source: string): Record<string, unknown> {
  try {
    const value = JSON.parse(source) as unknown;
    return value && typeof value === "object" && !Array.isArray(value)
      ? value as Record<string, unknown>
      : { value };
  } catch {
    return { _raw: source };
  }
}

function mutationOperation(operation: string): MemoryMutationChunk["operation"] {
  switch (operation.toLowerCase()) {
    case "create": case "write": case "proposed": return "create";
    case "delete": case "rejected": return "delete";
    case "pin": return "pin";
    case "decay": return "decay";
    default: return "update";
  }
}

function imageUrl(block: Extract<ContentBlock, { type: "image" }>): string {
  if (block.url) return block.url;
  if (block.dataBase64 && /^image\/(?:avif|gif|jpeg|png|webp)$/.test(block.mime)) {
    return `data:${block.mime};base64,${block.dataBase64}`;
  }
  return "";
}

export function toChunks(blocks: readonly ContentBlock[], context: ChunkProjectionContext): Chunk[] {
  const chunks: Chunk[] = [];
  const toolIndices = new Map<string, number>();
  const sequenceStart = context.sequenceStart ?? 0;

  blocks.forEach((block, index) => {
    const base = {
      id: `${context.messageId}:${block.type}:${index}`,
      at: context.at,
      runId: context.runId,
      seq: sequenceStart + index,
    };
    switch (block.type) {
      case "text":
        chunks.push(context.finalized === false
          ? { ...base, kind: "text", text: block.text }
          : { ...base, kind: "markdown", source: block.text });
        return;
      case "thinking":
        chunks.push({ ...base, kind: "thinking", text: block.text });
        return;
      case "code":
        chunks.push({ ...base, kind: "artifact", artifactId: base.id, title: block.language || "Code", mime: `text/x-code;language=${block.language || "text"}`, content: block.code });
        return;
      case "citation":
        chunks.push({ ...base, kind: "citation", source: block.source, content: block.quote });
        return;
      case "memory":
        if (["read", "recall", "retrieved"].includes(block.operation.toLowerCase())) {
          chunks.push({ ...base, kind: "memory-recall", items: [{ id: block.key, content: block.value ?? "", type: "semantic" }] });
        } else {
          chunks.push({ ...base, kind: "memory-mutation", operation: mutationOperation(block.operation), memoryId: block.key, content: block.value ?? undefined });
        }
        return;
      case "toolUse": {
        const chunk: ToolCallChunk = { ...base, kind: "tool-call", toolCallId: block.id, toolName: block.name, args: parseRecord(block.inputJson), status: "running" };
        toolIndices.set(block.id, chunks.length);
        chunks.push(chunk);
        return;
      }
      case "toolResult": {
        const toolIndex = toolIndices.get(block.toolUseId);
        if (toolIndex !== undefined) {
          const tool = chunks[toolIndex];
          if (tool.kind === "tool-call") {
            chunks[toolIndex] = { ...tool, result: block.outputJson, status: block.isError ? "failed" : "complete" };
          }
        } else {
          chunks.push({ ...base, kind: "tool-call", toolCallId: block.toolUseId, toolName: "Unknown tool", args: {}, result: block.outputJson, status: block.isError ? "failed" : "complete" });
        }
        return;
      }
      case "skill":
        chunks.push({ ...base, kind: "skill-activation", skillId: block.name, skillName: block.name, status: block.status === "active" ? "active" : "complete" });
        return;
      case "artifact":
        chunks.push({ ...base, kind: "artifact", artifactId: block.id, mime: block.kind, content: block.content });
        return;
      case "image":
        chunks.push({ ...base, kind: "image", url: imageUrl(block), alt: block.alt, width: block.width, height: block.height, sourcePath: block.path });
        return;
      case "divider":
        chunks.push({ ...base, kind: "divider" });
        return;
      default:
        assertNever(block);
    }
  });

  return chunks;
}
