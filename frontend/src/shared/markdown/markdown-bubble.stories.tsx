import { useLayoutEffect, useRef, useState } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, waitFor, within } from "storybook/test";

import { MarkdownBubble } from "@/shared/markdown/markdown-bubble";
import { assertPerformanceBudget } from "@/test/performance-budget";

const FINAL_SENTINEL = "uar-markdown-line-2000";

const representativeMarkdown = Array.from({ length: 2_000 }, (_, index) => {
  const lineNumber = index + 1;
  if (lineNumber === 2_000) return `Final sentinel: ${FINAL_SENTINEL}`;
  if (lineNumber === 1) return "# Finalized run report";
  if (lineNumber % 200 === 0) return `## Section ${lineNumber}`;
  if (lineNumber % 100 === 0) return `- List item ${lineNumber}`;
  if (lineNumber % 250 === 0) return `Line ${lineNumber} with **strong text** and *emphasis*.`;
  if (lineNumber % 50 === 0) return `[Reference ${lineNumber}](https://example.com/reference-${lineNumber})`;
  return `Finalized output line ${lineNumber}.`;
}).join("\n");

function TwoThousandLineMarkdownFixture() {
  const root = useRef<HTMLDivElement>(null);
  const startedAt = useRef(0);
  const [source, setSource] = useState<string | null>(null);

  useLayoutEffect(() => {
    if (source === null || !root.current) return;
    const current = root.current;
    const recordWhenReady = () => {
      const hasHeading = [...current.querySelectorAll("h2")]
        .some((heading) => heading.textContent === "Section 200");
      const hasLink = [...current.querySelectorAll("a")]
        .some((link) => link.textContent === "Reference 50");
      const hasListItem = [...current.querySelectorAll("li")]
        .some((item) => item.textContent?.includes("List item 100"));
      const ready = current.textContent?.includes(FINAL_SENTINEL)
        && hasHeading
        && hasLink
        && hasListItem;
      if (ready) current.dataset.finalizeMs = String(performance.now() - startedAt.current);
      return ready;
    };
    if (recordWhenReady()) return;
    const observer = new MutationObserver(() => {
      if (recordWhenReady()) observer.disconnect();
    });
    observer.observe(current, { childList: true, subtree: true });
    return () => observer.disconnect();
  }, [source]);

  if (source === null) {
    return (
      <button
        type="button"
        onClick={() => {
          startedAt.current = performance.now();
          setSource(representativeMarkdown);
        }}
      >
        Finalize 2,000-line Markdown
      </button>
    );
  }

  return (
    <div ref={root} data-testid="markdown-performance-root" className="bg-background p-4">
      <MarkdownBubble source={source} />
    </div>
  );
}

const meta = {
  title: "Content/Markdown Bubble Performance",
  parameters: { layout: "fullscreen" },
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const TwoThousandFinalizedLines: Story = {
  render: () => <TwoThousandLineMarkdownFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(canvas.getByRole("button", { name: "Finalize 2,000-line Markdown" }));
    const root = await canvas.findByTestId("markdown-performance-root");
    await waitFor(() => {
      expect(canvas.getByRole("heading", { name: "Section 200" })).toBeVisible();
      expect(canvas.getByRole("link", { name: "Reference 50" })).toBeVisible();
      expect(canvas.getAllByRole("listitem")[0]).toHaveTextContent("List item 100");
      expect(root).toHaveTextContent(FINAL_SENTINEL);
      expect(root.dataset.finalizeMs).toBeDefined();
    });
    const result = assertPerformanceBudget(
      "twoThousandLineMarkdownFinalize",
      Number(root.dataset.finalizeMs),
    );
    root.dataset.performanceResult = JSON.stringify(result);
    console.info("[performance-budget]", JSON.stringify(result));
    expect(result.verdict).toBe("pass");
  },
};
