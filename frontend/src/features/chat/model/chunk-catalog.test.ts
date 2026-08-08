import { describe, expect, it } from "vitest";
import type { ContentBlock } from "@/shared/content";
import {
  CHUNK_BUBBLE_VISIBLE,
  CHUNK_PHASE,
  CHUNK_RENDERER,
  CHUNK_TRACE,
  type Chunk,
  type ChunkKind,
} from "./chunk";
import { toChunks } from "./to-chunks";

const BLOCKS: ContentBlock[] = [
  { type: "text", text: "# Hello" },
  { type: "thinking", text: "Inspecting" },
  { type: "code", language: "ts", code: "const safe = true;" },
  { type: "citation", source: "Spec", quote: "Evidence" },
  { type: "memory", operation: "recall", key: "m1", value: "Remembered" },
  { type: "memory", operation: "update", key: "m2", value: "Changed" },
  { type: "toolUse", id: "tool-1", name: "search", inputJson: "{\"q\":\"uar\"}" },
  { type: "toolResult", toolUseId: "tool-1", outputJson: "{\"ok\":true}", isError: false },
  { type: "skill", name: "research", status: "active" },
  { type: "artifact", id: "a1", kind: "application/json", content: "{}" },
  { type: "image", url: null, dataBase64: "aW1hZ2U=", mime: "image/png", path: "owned/i.png", alt: "A result", width: 320, height: 180 },
  { type: "divider" },
];

const ALL_KINDS = [
  "text", "markdown", "reasoning", "thinking", "tool-call", "tool-approval", "tool-denied",
  "skill-activation", "memory-recall", "memory-mutation", "memory-update", "citation", "rag-citations",
  "context-update", "a2ui-display", "a2ui-input", "artifact", "image", "video", "file", "divider",
  "state-snapshot", "state-delta", "step", "usage", "error", "raw",
] as const satisfies readonly ChunkKind[];

const base = { id: "fixture", at: "2026-08-08T00:00:00.000Z", seq: 0 };
const CHUNK_FIXTURES = {
  text: { ...base, kind: "text", text: "streaming" },
  markdown: { ...base, kind: "markdown", source: "final" },
  reasoning: { ...base, kind: "reasoning", text: "reasoning" },
  thinking: { ...base, kind: "thinking", text: "thinking" },
  "tool-call": { ...base, kind: "tool-call", toolCallId: "tool", toolName: "search", args: {}, status: "running" },
  "tool-approval": { ...base, kind: "tool-approval", toolCallId: "tool", toolName: "write", args: {} },
  "tool-denied": { ...base, kind: "tool-denied", toolCallId: "tool", toolName: "write", reason: "denied" },
  "skill-activation": { ...base, kind: "skill-activation", skillId: "skill", skillName: "Research", status: "active" },
  "memory-recall": { ...base, kind: "memory-recall", items: [] },
  "memory-mutation": { ...base, kind: "memory-mutation", operation: "update", memoryId: "memory" },
  "memory-update": { ...base, kind: "memory-update", scope: "session", summary: "updated", itemCount: 1 },
  citation: { ...base, kind: "citation", source: "Spec", content: "Evidence" },
  "rag-citations": { ...base, kind: "rag-citations", citations: [] },
  "context-update": { ...base, kind: "context-update", strategy: "summarize", messagesRemoved: 1, tokensSaved: 10, wasApplied: true, summaryGenerated: true },
  "a2ui-display": { ...base, kind: "a2ui-display", profile: "a2ui/v0.9", component: "Card", payload: {}, validation: "valid" },
  "a2ui-input": { ...base, kind: "a2ui-input", profile: "a2ui/v0.9", component: "confirm", requestId: "request", payload: {}, status: "awaiting" },
  artifact: { ...base, kind: "artifact", artifactId: "artifact", mime: "application/json" },
  image: { ...base, kind: "image", url: "image.png", alt: "Image" },
  video: { ...base, kind: "video", url: "video.mp4" },
  file: { ...base, kind: "file", name: "report.json", mime: "application/json", bytes: 1 },
  divider: { ...base, kind: "divider" },
  "state-snapshot": { ...base, kind: "state-snapshot", state: {} },
  "state-delta": { ...base, kind: "state-delta", delta: [] },
  step: { ...base, kind: "step", name: "execute", status: "started" },
  usage: { ...base, kind: "usage", inputTokens: 1, outputTokens: 2, totalTokens: 3 },
  error: { ...base, kind: "error", message: "failed" },
  raw: { ...base, kind: "raw", type: "provider.experimental", payload: {} },
} satisfies Record<ChunkKind, Chunk>;

describe("chunk catalog", () => {
  it("projects every portable block and joins tool results", () => {
    const chunks = toChunks(BLOCKS, { messageId: "m", at: "2026-08-08T00:00:00.000Z", finalized: true });
    expect(chunks).toHaveLength(BLOCKS.length - 1);
    expect(chunks[0]).toMatchObject({ kind: "markdown", source: "# Hello" });
    expect(chunks.find((chunk) => chunk.kind === "tool-call")).toMatchObject({ status: "complete", result: "{\"ok\":true}" });
    expect(chunks.find((chunk) => chunk.kind === "image")).toMatchObject({ url: "data:image/png;base64,aW1hZ2U=", alt: "A result", sourcePath: "owned/i.png" });
    expect(chunks.at(-1)?.kind).toBe("divider");
  });

  it("keeps streaming text distinct from finalized markdown", () => {
    expect(toChunks([{ type: "text", text: "partial" }], { messageId: "m", at: "now", finalized: false })[0]).toMatchObject({ kind: "text", text: "partial" });
  });

  it("defines exhaustive catalog dispositions", () => {
    expect(Object.keys(CHUNK_PHASE).sort()).toEqual([...ALL_KINDS].sort());
    expect(Object.keys(CHUNK_BUBBLE_VISIBLE).sort()).toEqual([...ALL_KINDS].sort());
    expect(Object.keys(CHUNK_RENDERER).sort()).toEqual([...ALL_KINDS].sort());
    expect(Object.keys(CHUNK_TRACE).sort()).toEqual([...ALL_KINDS].sort());
    expect(CHUNK_BUBBLE_VISIBLE.raw).toBe(false);
    expect(CHUNK_TRACE["state-delta"]).toBe("inspector");
    expect(Object.keys(CHUNK_FIXTURES).sort()).toEqual([...ALL_KINDS].sort());
  });
});
