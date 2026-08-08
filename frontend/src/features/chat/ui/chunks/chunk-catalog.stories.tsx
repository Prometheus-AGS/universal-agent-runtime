import type { Meta, StoryObj } from "@storybook/react-vite";
import type { Chunk } from "@/features/chat/model/chunk";
import { ChunkRenderer } from "./chunk-renderer";

const at = "2026-08-08T00:00:00.000Z";
const bubbleChunks: Chunk[] = [
  { id: "text", at, seq: 0, kind: "text", text: "Streaming response text" },
  { id: "markdown", at, seq: 1, kind: "markdown", source: "## Final response\nRendered through the shared Markdown pipeline." },
  { id: "reasoning", at, seq: 2, kind: "reasoning", text: "Verified the available evidence." },
  { id: "thinking", at, seq: 3, kind: "thinking", text: "Comparing alternatives.", usedTokens: 48, budgetTokens: 128 },
  { id: "tool", at, seq: 4, kind: "tool-call", toolCallId: "call-1", toolName: "search", args: { query: "UAR" }, result: "3 matches", status: "complete", durationMs: 42 },
  { id: "approval", at, seq: 5, kind: "tool-approval", toolCallId: "call-2", toolName: "write_file", args: { path: "report.md" }, reason: "Writes workspace data" },
  { id: "denied", at, seq: 6, kind: "tool-denied", toolCallId: "call-3", toolName: "shell", reason: "Denied by execution policy", policy: "read-only" },
  { id: "skill", at, seq: 7, kind: "skill-activation", skillId: "research", skillName: "Research", selectionMethod: "hybrid", status: "complete" },
  { id: "memory-recall", at, seq: 8, kind: "memory-recall", items: [{ id: "decision-1", content: "Keep Base UI", type: "semantic", pinned: true }] },
  { id: "memory-mutation", at, seq: 9, kind: "memory-mutation", operation: "update", memoryId: "decision-1", content: "Base UI remains authoritative" },
  { id: "memory-update", at, seq: 10, kind: "memory-update", scope: "session", summary: "Recorded one decision", itemCount: 1 },
  { id: "citation", at, seq: 11, kind: "citation", source: "UAR specification", content: "The catalog is exhaustive." },
  { id: "rag", at, seq: 12, kind: "rag-citations", citations: [{ marker: 1, chunkId: "rag-1", documentName: "Migration plan", relevanceScore: 0.94, snippet: "Render every visible chunk." }] },
  { id: "context", at, seq: 13, kind: "context-update", strategy: "summarize", messagesRemoved: 4, tokensSaved: 1200, wasApplied: true, summaryGenerated: true },
  { id: "a2ui-display", at, seq: 14, kind: "a2ui-display", profile: "a2ui/v0.9", component: "Card", payload: { title: "Generated surface", content: "Policy-gated content" }, validation: "valid" },
  { id: "a2ui-input", at, seq: 15, kind: "a2ui-input", profile: "a2ui/v0.9", component: "confirm", requestId: "request-1", payload: { title: "Confirmation", content: "{\"message\":\"Continue?\"}" }, status: "expired" },
  { id: "artifact", at, seq: 16, kind: "artifact", artifactId: "artifact-1", title: "Result JSON", mime: "application/json", content: "{\"ok\":true}" },
  { id: "image", at, seq: 17, kind: "image", url: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='320' height='120'%3E%3Crect width='320' height='120' fill='%231e293b'/%3E%3C/svg%3E", alt: "Generated dark rectangle", width: 320, height: 120 },
  { id: "video", at, seq: 18, kind: "video", url: "" },
  { id: "file", at, seq: 19, kind: "file", name: "report.json", mime: "application/json", bytes: 842 },
  { id: "divider", at, seq: 20, kind: "divider" },
  { id: "usage", at, seq: 21, kind: "usage", inputTokens: 120, outputTokens: 80, totalTokens: 200, model: "openai/gpt-5" },
  { id: "error", at, seq: 22, kind: "error", message: "Provider unavailable", code: "provider_error", retryable: true },
];

const meta = {
  title: "Chat/Chunk Catalog",
  parameters: { layout: "fullscreen" },
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const CompleteCatalog: Story = {
  render: () => (
    <main className="min-h-screen bg-background p-6 text-foreground">
      <div className="mx-auto max-w-3xl space-y-4">
        <header className="rounded-xl bg-surface px-4 py-4">
          <h1 className="font-display text-xl font-semibold">Chunk catalog</h1>
          <p className="mt-1 text-sm text-fg-sub">Every bubble-visible discriminant is represented below.</p>
        </header>
        {bubbleChunks.map((chunk) => <ChunkRenderer key={chunk.id} chunk={chunk} />)}
        <section aria-label="Trace-only chunk dispositions" className="rounded-xl bg-surface px-4 py-4">
          <h2 className="font-display text-sm font-semibold">Trace-only</h2>
          <p className="mt-1 text-sm text-fg-sub">state-snapshot and state-delta → inspector; step → timeline tick; raw → trace row. These intentionally render no chat bubble.</p>
        </section>
      </div>
    </main>
  ),
};
