import { describe, expect, it } from "vitest";
import type { ContentBlock } from "@/shared/content";
import { decodePersistedChatContent } from "./legacy-chat-content";

const context = {
  messageId: "legacy-message",
  at: "2026-08-08T00:00:00.000Z",
  finalized: true,
};

const historicalContent = [
  { type: "text", text: "Answer" },
  { type: "reasoning", text: "Checking" },
  { type: "tool-call", toolCallId: "tool-1", toolName: "search", args: { query: "UAR" }, result: "done", status: "complete" },
  { type: "citation", source: "Spec", content: "Quoted evidence", url: "https://example.test/spec" },
  { type: "rag-citations", citations: [{ marker: 1, chunkId: "chunk-1", documentName: "Guide", relevanceScore: 0.9, snippet: "Relevant" }] },
  { type: "skill-activation", skillId: "skill-1", skillName: "Research", selectionMethod: "hybrid", status: "active" },
  { type: "context-update", strategy: "summarize", messagesRemoved: 2, tokensSaved: 80, wasApplied: true, summaryGenerated: true },
  { type: "image", url: "https://example.test/image.png", alt: "Generated chart" },
  { type: "error", message: "Provider unavailable", code: "provider_error" },
];

describe("decodePersistedChatContent", () => {
  it("decodes every historical content discriminant into the exact portable contract", () => {
    const decoded = decodePersistedChatContent(historicalContent, context);
    const portableTypes = decoded.content.map(({ type }) => type);

    expect(portableTypes).toEqual([
      "text", "thinking", "toolUse", "toolResult", "citation", "artifact",
      "skill", "artifact", "image", "artifact",
    ]);
    expect(decoded.content).toSatisfy((blocks: ContentBlock[]) => blocks.every((block) => [
      "text", "thinking", "code", "citation", "memory", "toolUse", "toolResult",
      "skill", "artifact", "image", "divider",
    ].includes(block.type)));
  });

  it("preserves rich historical meaning without duplicate bubble chunks", () => {
    const decoded = decodePersistedChatContent(historicalContent, context);
    const kinds = decoded.chunks.map(({ kind }) => kind);

    expect(kinds).toEqual([
      "markdown", "thinking", "tool-call", "citation", "rag-citations",
      "skill-activation", "context-update", "image", "error",
    ]);
    expect(decoded.chunks.filter(({ kind }) => kind === "tool-call")).toHaveLength(1);
    expect(decoded.chunks.find(({ kind }) => kind === "skill-activation")).toMatchObject({
      selectionMethod: "hybrid",
      status: "active",
    });
  });

  it("accepts malformed storage values as an empty message", () => {
    expect(decodePersistedChatContent({ type: "text" }, context)).toEqual({ content: [], chunks: [] });
  });
});
