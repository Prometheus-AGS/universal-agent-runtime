import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { CHUNK_BUBBLE_VISIBLE, type Chunk } from "@/features/chat/model/chunk";
import { ChunkRenderer } from "./chunk-renderer";
import { parseChartModel } from "./chart-model";
import { RichDataRenderers } from "./rich-data-renderers";

const assistantDataNames = vi.hoisted(() => [] as string[]);

vi.mock("@assistant-ui/react", async (importOriginal) => {
  const original = await importOriginal<typeof import("@assistant-ui/react")>();
  return {
    ...original,
    useAssistantDataUI: ({ name }: { name: string }) => {
      assistantDataNames.push(name);
    },
  };
});

vi.mock("@/features/chat/components/a2ui-artifact-block", () => ({
  A2uiDisplayBlock: ({ artifactType }: { artifactType: string }) => <div data-testid="policy-a2ui-display">{artifactType}</div>,
  A2uiInputBlock: ({ artifactType }: { artifactType: string }) => <div data-testid="policy-a2ui-input">{artifactType}</div>,
}));

const base = { id: "chunk-1", at: "2026-08-08T00:00:00.000Z", seq: 1 };

describe("ChunkRenderer", () => {
  beforeEach(() => assistantDataNames.splice(0));

  it("renders protocol dividers as spacer separators, never horizontal rules", () => {
    const { container } = render(<ChunkRenderer chunk={{ ...base, kind: "divider" }} />);
    expect(screen.getByRole("separator")).toBeInTheDocument();
    expect(container.querySelector("hr")).toBeNull();
  });

  it("starts reasoning, tool, and citation detail collapsed with textual state", () => {
    const chunks: Chunk[] = [
      { ...base, id: "reasoning", kind: "reasoning", text: "Private chain" },
      { ...base, id: "tool", kind: "tool-call", toolCallId: "call", toolName: "search", args: {}, status: "running" },
      { ...base, id: "citation", kind: "citation", source: "Spec", content: "Evidence" },
      { ...base, id: "denied", kind: "tool-denied", toolCallId: "call-2", toolName: "write", reason: "Policy denied it" },
    ];
    const { container } = render(<>{chunks.map((chunk) => <ChunkRenderer key={chunk.id} chunk={chunk} />)}</>);

    expect([...container.querySelectorAll("details")].every((details) => !details.open)).toBe(true);
    expect(screen.getByText("running")).toBeInTheDocument();
    expect(screen.getByText("Denied · write")).toBeInTheDocument();
    expect(screen.getByText("Policy denied it")).toBeInTheDocument();
  });

  it("shows a safe media fallback when an image lacks a source or description", () => {
    render(<ChunkRenderer chunk={{ ...base, kind: "image", url: "", alt: "" }} />);
    expect(screen.getByLabelText("Image unavailable")).toHaveTextContent("a source and description are required");
  });

  it("rejects executable provider URLs at citation, artifact, and media boundaries", () => {
    const chunks: Chunk[] = [
      { ...base, id: "citation-url", kind: "citation", source: "Unsafe", content: "Evidence", url: "javascript:alert(1)" },
      { ...base, id: "artifact-url", kind: "artifact", artifactId: "artifact", mime: "application/octet-stream", url: "data:text/html,<script>alert(2)</script>" },
      { ...base, id: "image-url", kind: "image", url: "javascript:alert(3)", alt: "Unsafe image" },
      { ...base, id: "file-url", kind: "file", name: "unsafe.html", mime: "text/html", bytes: 1, url: "javascript:alert(4)" },
    ];
    const { container } = render(<>{chunks.map((chunk) => <ChunkRenderer key={chunk.id} chunk={chunk} />)}</>);

    expect(container.querySelectorAll('a[href^="javascript:"]')).toHaveLength(0);
    expect(container.querySelectorAll('a[href^="data:text/html"]')).toHaveLength(0);
    expect(container.querySelectorAll('img[src^="javascript:"]')).toHaveLength(0);
    expect(screen.getByLabelText("Image unavailable")).toBeInTheDocument();
  });

  it("sandboxes HTML and discloses its source", () => {
    render(<ChunkRenderer chunk={{ ...base, kind: "artifact", artifactId: "html", mime: "text/html", content: "<script>top.alert(1)</script>" }} />);
    const frame = screen.getByTitle("Sandboxed HTML artifact");
    expect(frame).toHaveAttribute("sandbox", "");
    expect(frame).toHaveAttribute("srcdoc", "<script>top.alert(1)</script>");
    expect(screen.getByText("Show source")).toBeInTheDocument();
  });

  it("sanitizes SVG and keeps JSON as escaped text", () => {
    const { container, rerender } = render(<ChunkRenderer chunk={{
      ...base,
      kind: "artifact",
      artifactId: "svg",
      mime: "image/svg+xml",
      content: '<svg viewBox="0 0 10 10"><script>alert(2)</script><path d="M0 0L10 10" onload="alert(1)" /></svg>',
    }} />);
    expect(container.querySelector("script")).toBeNull();
    expect(container.querySelector("[onload]")).toBeNull();
    expect(screen.queryByRole("img", { name: "Generated SVG" }) ?? screen.getByLabelText("Invalid SVG artifact")).toBeInTheDocument();

    rerender(<ChunkRenderer chunk={{ ...base, kind: "artifact", artifactId: "json", mime: "application/json", content: '{"markup":"<script>alert(3)</script>"}' }} />);
    expect(screen.getByLabelText("JSON")).toHaveTextContent("<script>alert(3)</script>");
    expect(document.querySelectorAll("script")).toHaveLength(0);
  });

  it("accepts only the closed finite chart model and falls back for malformed data", () => {
    expect(parseChartModel('{"kind":"bar","title":"Latency","labels":["p50"],"series":[{"name":"ms","values":[12]}]}')).not.toBeNull();
    expect(parseChartModel('{"kind":"pie","title":"No","labels":["x"],"series":[{"name":"v","values":[1]}]}')).toBeNull();
    expect(parseChartModel('{"kind":"bar","title":"No","labels":["x"],"series":[{"name":"v","values":[1]}],"style":{"background":"url(javascript:1)"}}')).toBeNull();
    render(<ChunkRenderer chunk={{ ...base, kind: "artifact", artifactId: "chart", mime: "application/vnd.uar.chart+json", content: '{"kind":"pie"}' }} />);
    expect(screen.getByLabelText("Invalid chart artifact")).toHaveTextContent("Chart preview unavailable");
  });

  it("routes A2UI chunks through the maintained policy component", () => {
    render(<ChunkRenderer chunk={{ ...base, kind: "a2ui-display", profile: "a2ui/v0.9", component: "Card", payload: {}, validation: "valid" }} />);
    expect(screen.getByTestId("policy-a2ui-display")).toHaveTextContent("Card");
  });

  it("registers every visible chunk family and no trace-only kind", () => {
    render(<RichDataRenderers />);
    const expected = Object.entries(CHUNK_BUBBLE_VISIBLE).filter(([, visible]) => visible).map(([kind]) => kind);
    expect(assistantDataNames.sort()).toEqual(expected.sort());
    expect(assistantDataNames).not.toContain("raw");
    expect(assistantDataNames).not.toContain("state-delta");
  });
});
