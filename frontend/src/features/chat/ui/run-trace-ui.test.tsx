import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type {
  PersistedRun,
  PersistedRunEvent,
} from "@/platform/pglite/run-event-repository";
import {
  DEFAULT_RUN_TRACE_FILTERS,
  projectRunTrace,
} from "@/features/chat/model/run-trace-projection";
import type {
  RunTraceContext,
  RunTraceNetworkState,
} from "@/features/chat/model/run-trace-types";
import { RunInspector } from "@/features/chat/ui/run-inspector";
import { RunTraceBar } from "@/features/chat/ui/run-trace-bar";
import { RunTraceTimeline } from "@/features/chat/ui/run-trace-timeline";

const run: PersistedRun = {
  id: "run-1",
  threadId: "thread-1",
  messageId: "message-1",
  status: "finished",
  startedAt: "2026-08-07T20:00:00.000Z",
  finishedAt: "2026-08-07T20:00:01.000Z",
  model: "openai/gpt-5",
  usage: null,
  costUsd: null,
  phaseTimings: { context: 10, skill: 0, memory: 0, retrieval: 0, reasoning: 30, tool: 0, generate: 60 },
};

const context: RunTraceContext = {
  runId: "run-1",
  threadId: "thread-1",
  messageId: "message-1",
  agentId: "agent-1",
  sessionId: "session-1",
};

const network: RunTraceNetworkState = {
  snapshot: { status: "success", error: null },
  checkpoints: { status: "success", error: null },
  replay: { status: "success", error: null },
  agent: { status: "success", error: null },
  resume: { status: "idle", error: null },
};

function event(index: number, type = "TEXT_MESSAGE_END"): PersistedRunEvent {
  return {
    runId: "run-1",
    seq: index,
    eventId: `event-${index}`,
    wireSequence: index,
    type,
    kind: type.startsWith("TEXT") ? "message" : "lifecycle",
    at: new Date(Date.parse(run.startedAt) + index).toISOString(),
    payload: {
      messageId: `message-${index}`,
      content: index === 0 ? "<script>window.bad = true</script>" : `event ${index}`,
    },
  };
}

function projection(count: number, selectedNodeId: string | null = null) {
  return projectRunTrace({
    run,
    events: Array.from({ length: count }, (_, index) => event(index)),
    filters: { ...DEFAULT_RUN_TRACE_FILTERS },
    expandedNodeIds: new Set(["run:run-1", "phase:generate"]),
    selectedNodeId,
  });
}

beforeEach(() => {
  class TestResizeObserver {
    constructor(private readonly callback: ResizeObserverCallback) {}
    observe(target: Element) {
      this.callback([{
        target,
        contentRect: {
          x: 0, y: 0, top: 0, left: 0, bottom: 440, right: 800,
          width: 800, height: 440, toJSON: () => ({}),
        },
      } as ResizeObserverEntry], this as unknown as ResizeObserver);
    }
    unobserve() {}
    disconnect() {}
  }
  vi.stubGlobal("ResizeObserver", TestResizeObserver);
  Object.defineProperty(HTMLElement.prototype, "clientHeight", { configurable: true, value: 440 });
  HTMLElement.prototype.scrollIntoView = vi.fn();
  HTMLElement.prototype.getBoundingClientRect = vi.fn(() => ({
    x: 0,
    y: 0,
    top: 0,
    left: 0,
    bottom: 440,
    right: 800,
    width: 800,
    height: 44,
    toJSON: () => ({}),
  }));
});

describe("run trace UI", () => {
  test("navigates the labelled phase listbox with arrow, Home, and End keys", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(
      <RunTraceBar
        segments={projection(1).segments}
        selectedPhase="context"
        onSelectPhase={onSelect}
      />,
    );

    const listbox = screen.getByRole("listbox", { name: "Run phases" });
    const options = within(listbox).getAllByRole("option");
    expect(options).toHaveLength(3);
    options[0]!.focus();
    await user.keyboard("{ArrowRight}");
    expect(onSelect).toHaveBeenLastCalledWith("reasoning");
    await user.keyboard("{End}");
    expect(onSelect).toHaveBeenLastCalledWith("generate");
    await user.keyboard("{Home}");
    expect(onSelect).toHaveBeenLastCalledWith("context");
  });

  test("exposes counting filters and complete keyboard tree metadata", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const onToggle = vi.fn();
    const onFilter = vi.fn();
    const trace = projection(3, "run:run-1");
    render(
      <RunTraceTimeline
        projection={trace}
        filters={DEFAULT_RUN_TRACE_FILTERS}
        onToggleFilter={onFilter}
        onToggleExpanded={onToggle}
        onSelectNode={onSelect}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Messages 3" }));
    expect(onFilter).toHaveBeenCalledWith("message");
    const tree = screen.getByRole("tree", { name: "Run event tree" });
    const rows = within(tree).getAllByRole("treeitem");
    expect(rows[0]).toHaveAttribute("aria-level", "1");
    expect(rows[1]).toHaveAttribute("aria-level", "2");
    expect(rows[2]).toHaveAttribute("aria-posinset", "1");
    expect(rows[2]).toHaveAttribute("aria-setsize", "3");

    rows[0]!.focus();
    await user.keyboard("{ArrowDown}");
    expect(onSelect).toHaveBeenCalledWith("phase:generate");
    fireEvent.keyDown(rows[0]!, { key: "ArrowLeft" });
    expect(onToggle).toHaveBeenCalledWith("run:run-1");
    fireEvent.keyDown(rows.at(-1)!, { key: "Home" });
    expect(onSelect).toHaveBeenLastCalledWith("run:run-1");
  });

  test("moves the roving tree focus with consecutive arrow keys", async () => {
    const user = userEvent.setup();
    function ControlledTimeline() {
      const [selectedNodeId, setSelectedNodeId] = useState("run:run-1");
      return (
        <RunTraceTimeline
          projection={projection(3, selectedNodeId)}
          filters={DEFAULT_RUN_TRACE_FILTERS}
          onToggleFilter={vi.fn()}
          onToggleExpanded={vi.fn()}
          onSelectNode={setSelectedNodeId}
        />
      );
    }
    render(<ControlledTimeline />);

    screen.getByRole("treeitem", { name: /run run-1/i }).focus();
    await user.keyboard("{ArrowDown}");
    await waitFor(() => expect(screen.getByRole("treeitem", { name: /generate/i })).toHaveFocus());
    await user.keyboard("{ArrowDown}");
    await waitFor(() => expect(screen.getAllByRole("treeitem")[2]).toHaveFocus());
  });

  test("moves focus after an End jump mounts a distant virtual row", async () => {
    const user = userEvent.setup();
    function ControlledVirtualTimeline() {
      const [selectedNodeId, setSelectedNodeId] = useState("run:run-1");
      return (
        <RunTraceTimeline
          projection={projection(500, selectedNodeId)}
          filters={DEFAULT_RUN_TRACE_FILTERS}
          onToggleFilter={vi.fn()}
          onToggleExpanded={vi.fn()}
          onSelectNode={setSelectedNodeId}
        />
      );
    }
    render(<ControlledVirtualTimeline />);

    screen.getByRole("treeitem", { name: /run run-1/i }).focus();
    await user.keyboard("{End}");
    await waitFor(() => expect(screen.getByText("#499").closest('[role="treeitem"]')).toHaveFocus());
  });

  test("virtualizes a 500-event tree within the mount budget", async () => {
    const trace = projection(500, "event:event-0");
    const startedAt = performance.now();
    render(
      <RunTraceTimeline
        projection={trace}
        filters={DEFAULT_RUN_TRACE_FILTERS}
        onToggleFilter={vi.fn()}
        onToggleExpanded={vi.fn()}
        onSelectNode={vi.fn()}
      />,
    );
    const duration = performance.now() - startedAt;

    await waitFor(() => {
      const mountedRows = screen.getAllByRole("treeitem");
      expect(mountedRows.length).toBeGreaterThan(0);
      expect(mountedRows.length).toBeLessThan(40);
      expect(mountedRows.some((row) => row.getAttribute("aria-setsize") === "500")).toBe(true);
    });
    expect(duration).toBeLessThan(100);
  });

  test("does not recenter a stable selection when live events append", async () => {
    const scrollIntoView = vi.mocked(HTMLElement.prototype.scrollIntoView);
    const view = render(
      <RunTraceTimeline
        projection={projection(1, "event:event-0")}
        filters={DEFAULT_RUN_TRACE_FILTERS}
        onToggleFilter={vi.fn()}
        onToggleExpanded={vi.fn()}
        onSelectNode={vi.fn()}
      />,
    );
    await waitFor(() => expect(scrollIntoView).toHaveBeenCalledOnce());
    scrollIntoView.mockClear();

    view.rerender(
      <RunTraceTimeline
        projection={projection(2, "event:event-0")}
        filters={DEFAULT_RUN_TRACE_FILTERS}
        onToggleFilter={vi.fn()}
        onToggleExpanded={vi.fn()}
        onSelectNode={vi.fn()}
      />,
    );
    expect(scrollIntoView).not.toHaveBeenCalled();
  });

  test("keeps payload and raw AG-UI inert while announcing explicit copy", async () => {
    const user = userEvent.setup();
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });
    const trace = projection(1, "event:event-0");
    const selectedNode = trace.nodesById.get("event:event-0");
    const view = render(
      <RunInspector
        context={context}
        selectedNode={selectedNode}
        timing={trace.timingsByEventId.get("event-0")}
        checkpoints={[]}
        selectedCheckpointId={null}
        replay={null}
        network={network}
        canResume={false}
        onSelectCheckpoint={vi.fn()}
        onRefreshReplay={vi.fn()}
        onResume={vi.fn()}
      />,
    );

    expect(view.container.querySelector("script")).toBeNull();
    expect(screen.getByText(/<script>window.bad = true<\/script>/)).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Raw AG-UI" }));
    await user.click(screen.getByRole("button", { name: "Copy raw event" }));
    expect(writeText).toHaveBeenCalledWith(expect.stringContaining("TEXT_MESSAGE_END"));
    expect(screen.getByText("Copied inspector JSON (1)")).toHaveAttribute("aria-live", "polite");
  });

  test("reports replay, resume, checkpoint, and conversation actions without rendering replay content", async () => {
    const user = userEvent.setup();
    const onCheckpoint = vi.fn();
    const onResume = vi.fn();
    const onConversation = vi.fn();
    const onReplay = vi.fn();
    const trace = projection(1, "event:event-0");
    render(
      <RunInspector
        context={context}
        selectedNode={trace.nodesById.get("event:event-0")}
        timing={trace.timingsByEventId.get("event-0")}
        checkpoints={[{
          id: "checkpoint-1",
          run_id: "run-1",
          thread_id: "thread-1",
          node_id: "node-1",
          iteration: 4,
          state: { private: "checkpoint-state" },
          messages: [],
          created_at: "2026-08-07T20:00:00.000Z",
        }]}
        selectedCheckpointId={"checkpoint-1"}
        replay={{ state: { surfaces: {}, error: null }, appliedOperations: 2 }}
        network={network}
        canResume
        onSelectCheckpoint={onCheckpoint}
        onRefreshReplay={onReplay}
        onResume={onResume}
        onOpenConversation={onConversation}
      />,
    );

    await user.click(screen.getByRole("button", { name: /node-1 · iteration 4/ }));
    expect(onCheckpoint).toHaveBeenCalledWith("checkpoint-1");
    await user.click(screen.getByRole("button", { name: "Resume latest checkpoint" }));
    expect(onResume).toHaveBeenCalledOnce();
    await user.click(screen.getByRole("button", { name: "Refresh A2UI replay" }));
    expect(onReplay).toHaveBeenCalledOnce();
    await user.click(screen.getByRole("button", { name: "Open in conversation" }));
    expect(onConversation).toHaveBeenCalledWith("thread-1", "message-0");
    expect(screen.getByText("2 operations · 0 surfaces")).toBeInTheDocument();
    expect(screen.getByText("Validated inert metadata only")).toBeInTheDocument();
    expect(screen.getByText(/"private": "checkpoint-state"/)).toBeInTheDocument();
  });

  test("reports loading states without claiming replay validation", () => {
    const trace = projection(1, "event:event-0");
    render(
      <RunInspector
        context={context}
        selectedNode={trace.nodesById.get("event:event-0")}
        timing={trace.timingsByEventId.get("event-0")}
        checkpoints={[]}
        selectedCheckpointId={null}
        replay={null}
        network={{
          snapshot: { status: "loading", error: null },
          checkpoints: { status: "loading", error: null },
          replay: { status: "loading", error: null },
          agent: { status: "loading", error: null },
          resume: { status: "idle", error: null },
        }}
        canResume={false}
        onSelectCheckpoint={vi.fn()}
        onRefreshReplay={vi.fn()}
        onResume={vi.fn()}
      />,
    );

    expect(screen.getByText("Loading checkpoints…")).toHaveAttribute("role", "status");
    expect(screen.getByText("Resolving runtime agent…")).toHaveAttribute("role", "status");
    expect(screen.getByText("Validating replay metadata…")).toBeInTheDocument();
    expect(screen.queryByText("Validated inert metadata only")).not.toBeInTheDocument();
  });
});
