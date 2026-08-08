import { useLayoutEffect, useMemo, useRef, useState } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, waitFor, within } from "storybook/test";

import type {
  PersistedRun,
  PersistedRunEvent,
} from "@/platform/pglite/run-event-repository";
import {
  DEFAULT_RUN_TRACE_FILTERS,
  projectRunTrace,
} from "@/features/chat/model/run-trace-projection";
import { RunTraceTimeline } from "@/features/chat/ui/run-trace-timeline";
import { assertPerformanceBudget } from "@/test/performance-budget";

const run: PersistedRun = {
  id: "run-browser-500",
  threadId: "thread-browser",
  messageId: "message-browser",
  status: "finished",
  startedAt: "2026-08-07T20:00:00.000Z",
  finishedAt: "2026-08-07T20:00:01.000Z",
  model: "openai/gpt-5",
  usage: null,
  costUsd: null,
  phaseTimings: {},
};

function fixtureEvent(index: number): PersistedRunEvent {
  return {
    runId: run.id,
    seq: index,
    eventId: `browser-event-${index}`,
    wireSequence: index,
    type: "TEXT_MESSAGE_END",
    kind: "message",
    at: new Date(Date.parse(run.startedAt) + index).toISOString(),
    payload: { messageId: `message-${index}`, content: `Event ${index}` },
  };
}

function FiveHundredEventFixture() {
  const root = useRef<HTMLDivElement>(null);
  const startedAt = useRef(0);
  const [mounted, setMounted] = useState(false);
  const projection = useMemo(() => projectRunTrace({
    run,
    events: Array.from({ length: 500 }, (_, index) => fixtureEvent(index)),
    filters: DEFAULT_RUN_TRACE_FILTERS,
    expandedNodeIds: new Set([`run:${run.id}`, "phase:generate"]),
    selectedNodeId: "event:browser-event-450",
  }), []);

  useLayoutEffect(() => {
    if (!mounted || !root.current) return;
    const current = root.current;
    const recordWhenReady = () => {
      const tree = current.querySelector<HTMLElement>('[role="tree"][aria-label="Run event tree"]');
      const rows = [...current.querySelectorAll<HTMLElement>('[role="treeitem"]')];
      const selected = current.querySelector<HTMLElement>('[role="treeitem"][aria-selected="true"]');
      const ready = tree?.dataset.virtualized === "true"
        && Number(tree.dataset.mountedCount) < 40
        && rows.length > 0
        && rows.length < 40
        && rows.some((row) => row.getAttribute("aria-setsize") === "500")
        && selected?.textContent?.includes("#450");
      if (ready) current.dataset.mountMs = String(performance.now() - startedAt.current);
      return ready;
    };
    if (recordWhenReady()) return;
    const observer = new MutationObserver(() => {
      if (recordWhenReady()) observer.disconnect();
    });
    observer.observe(current, { attributes: true, childList: true, subtree: true });
    return () => observer.disconnect();
  }, [mounted]);

  if (!mounted) {
    return (
      <button
        type="button"
        onClick={() => {
          startedAt.current = performance.now();
          setMounted(true);
        }}
      >
        Mount 500-event trace
      </button>
    );
  }

  return (
    <div
      ref={root}
      data-testid="trace-performance-root"
      className="bg-background p-4"
      style={{ display: "flex", height: 720, width: 1000 }}
    >
      <RunTraceTimeline
        projection={projection}
        filters={DEFAULT_RUN_TRACE_FILTERS}
        onToggleFilter={() => {}}
        onToggleExpanded={() => {}}
        onSelectNode={() => {}}
      />
    </div>
  );
}

const meta = {
  title: "Runtime/Run Trace Timeline",
  parameters: { layout: "fullscreen" },
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const FiveHundredEvents: Story = {
  render: () => <FiveHundredEventFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(canvas.getByRole("button", { name: "Mount 500-event trace" }));
    const root = await canvas.findByTestId("trace-performance-root");
    const tree = canvas.getByRole("tree", { name: "Run event tree" });
    await waitFor(() => {
      expect(canvas.getAllByRole("treeitem").length).toBeGreaterThan(0);
      expect(canvas.getByRole("treeitem", { selected: true })).toHaveTextContent("#450");
      expect(root.dataset.mountMs).toBeDefined();
    });
    const mountedRows = canvas.getAllByRole("treeitem");
    expect(tree).toHaveAttribute("data-virtualized", "true");
    expect(tree.getBoundingClientRect().height).toBeLessThan(1000);
    expect(tree.clientHeight).toBeLessThan(1000);
    expect(Number(tree.dataset.mountedCount)).toBeLessThan(40);
    expect(mountedRows.length).toBeLessThan(40);
    expect(mountedRows.some((row) => row.getAttribute("aria-setsize") === "500")).toBe(true);
    const result = assertPerformanceBudget(
      "fiveHundredEventTraceLane",
      Number(root.dataset.mountMs),
    );
    root.dataset.performanceResult = JSON.stringify(result);
    console.info("[performance-budget]", JSON.stringify(result));
    expect(result.verdict).toBe("pass");
  },
};
