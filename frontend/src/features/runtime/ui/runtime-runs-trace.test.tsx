import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, useLocation } from "react-router";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { useThreadRegistryStore } from "@/stores/thread-registry-store";
import { RuntimeRunsPage } from "./runtime-console-page";

vi.mock("@/entities/hooks/use-graph-entities", () => ({
  useGraphEntities: (type: string) => {
    if (type === "RuntimeRun") return [{
      id: "run-1",
      thread_id: "thread-1",
      message_id: "message-0",
      agent_id: "agent-1",
      session_id: "session-1",
      status: "completed",
      title: "Observed run",
      updated_at: "2026-08-07T20:00:00.000Z",
    }];
    if (type === "RuntimeArtifact") return [{
      id: "artifact-1",
      run_id: "run-1",
      kind: "json",
      title: "Result",
      updated_at: "2026-08-07T20:00:00.000Z",
    }];
    if (type === "RuntimeToolCall") return [{
      id: "tool-1",
      run_id: "run-1",
      tool_name: "search",
      status: "completed",
      updated_at: "2026-08-07T20:00:00.000Z",
    }];
    return [];
  },
}));

vi.mock("@/components/ui/scroll-area", () => ({
  ScrollArea: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
}));

vi.mock("@/features/chat/ui/run-trace-panel", () => ({
  RunTracePanel: ({ context, onOpenConversation, onRunHandoff, supplemental }: {
    context: { runId: string; threadId: string; messageId: string };
    onOpenConversation: (threadId: string, messageId: string) => void;
    onRunHandoff: (runId: string) => void;
    supplemental: React.ReactNode;
  }) => (
    <section aria-label={`Mock trace ${context.runId}`}>
      <button type="button" onClick={() => onOpenConversation(context.threadId, context.messageId)}>
        Open persisted message
      </button>
      <button type="button" onClick={() => onRunHandoff("run-2")}>Resume into returned run</button>
      {supplemental}
    </section>
  ),
}));

function LocationProbe() {
  const location = useLocation();
  return <span data-testid="location">{location.pathname}{location.search}</span>;
}

describe("RuntimeRunsPage trace integration", () => {
  beforeEach(() => {
    useThreadRegistryStore.setState({ activeThreadId: null });
    window.requestAnimationFrame = (callback) => {
      callback(0);
      return 1;
    };
    HTMLElement.prototype.scrollIntoView = vi.fn();
  });

  test("preserves run context and opens the stable chat message anchor", async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter initialEntries={["/admin/runs?run=run-1"]}>
        <RuntimeRunsPage />
        <div id="message-message-0" tabIndex={-1}>Persisted conversation message</div>
        <LocationProbe />
      </MemoryRouter>,
    );

    expect(screen.getByRole("region", { name: "Mock trace run-1" })).toBeInTheDocument();
    expect(screen.getByText("Artifacts · 1")).toBeInTheDocument();
    expect(screen.getByText("Tool calls · 1")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Open persisted message" }));
    expect(useThreadRegistryStore.getState().activeThreadId).toBe("thread-1");
    expect(screen.getByTestId("location")).toHaveTextContent("/threads");
    expect(screen.getByText("Persisted conversation message")).toHaveFocus();
  });

  test("persists a resumed run handoff in the route query", async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter initialEntries={["/admin/runs?run=run-1"]}>
        <RuntimeRunsPage />
        <LocationProbe />
      </MemoryRouter>,
    );

    await user.click(screen.getByRole("button", { name: "Resume into returned run" }));
    expect(screen.getByTestId("location")).toHaveTextContent("/admin/runs?run=run-2");
    expect(screen.getByText("Waiting for the selected run to appear")).toBeInTheDocument();
  });
});
