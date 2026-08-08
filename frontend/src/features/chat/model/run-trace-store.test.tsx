import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import * as runTraceApi from "@/features/chat/api/run-trace-api";
import { useRunTraceStore } from "@/features/chat/model/run-trace-store";
import { useRunTrace } from "@/features/chat/model/use-run-trace";
import type {
  RunTraceContext,
  RuntimeAgentArtifact,
} from "@/features/chat/model/run-trace-types";
import type {
  PersistedRun,
  PersistedRunEvent,
  PersistedRunSnapshot,
} from "@/platform/pglite/run-event-repository";

vi.mock("@/features/chat/api/run-trace-api", () => ({
  fetchRunCheckpoints: vi.fn(),
  fetchRunSurfaceReplay: vi.fn(),
  resolveRuntimeAgentArtifact: vi.fn(),
  resumeRunFromLatestCheckpoint: vi.fn(),
  subscribeRunTraceSnapshot: vi.fn(),
}));

const context: RunTraceContext = {
  runId: "run-1",
  threadId: "thread-1",
  messageId: "message-1",
  agentId: "agent-1",
  sessionId: "session-1",
};

const artifact: RuntimeAgentArtifact = {
  version: "1",
  kind: "agent",
  id: "agent-1",
  metadata: {},
  runtime: {},
  policy: {},
  schemas: {},
  prompt: {},
  memory: {},
  tools: {},
  ui: {},
  extensions: {},
};

function run(id = "run-1"): PersistedRun {
  return {
    id,
    threadId: "thread-1",
    messageId: "message-1",
    status: "running",
    startedAt: "2026-08-07T20:00:00.000Z",
    finishedAt: null,
    model: null,
    usage: null,
    costUsd: null,
    phaseTimings: {},
  };
}

function event(seq: number): PersistedRunEvent {
  return {
    runId: "run-1",
    seq,
    eventId: `event-${seq}`,
    wireSequence: seq,
    type: "TEXT_MESSAGE_END",
    kind: "message",
    at: new Date(Date.parse("2026-08-07T20:00:00.000Z") + seq).toISOString(),
    payload: { messageId: "message-1", content: `message ${seq}` },
  };
}

function snapshot(id = "run-1", events = [event(0)]): PersistedRunSnapshot {
  return { run: run(id), events };
}

function subscription(initialSnapshot = snapshot()) {
  return { initialSnapshot, unsubscribe: vi.fn().mockResolvedValue(undefined) };
}

describe("run trace store and hook", () => {
  beforeEach(async () => {
    await useRunTraceStore.getState().dispose();
    await useRunTraceStore.getState().selectContext(null);
    vi.clearAllMocks();
    vi.mocked(runTraceApi.fetchRunCheckpoints).mockResolvedValue({
      run_id: "run-1",
      checkpoints: [],
    });
    vi.mocked(runTraceApi.fetchRunSurfaceReplay).mockResolvedValue([]);
    vi.mocked(runTraceApi.resolveRuntimeAgentArtifact).mockResolvedValue(artifact);
  });

  test("unsubscribes before switching runs and ignores stale snapshots", async () => {
    const callbacks = new Map<string, (value: PersistedRunSnapshot) => void>();
    const order: string[] = [];
    const first = subscription();
    first.unsubscribe.mockImplementation(async () => { order.push("unsubscribe-run-1"); });
    const second = subscription(snapshot("run-2", []));
    vi.mocked(runTraceApi.subscribeRunTraceSnapshot).mockImplementation(async (runId, callback) => {
      callbacks.set(runId, callback);
      order.push(`subscribe-${runId}`);
      return runId === "run-1" ? first : second;
    });

    await useRunTraceStore.getState().selectContext(context);
    await useRunTraceStore.getState().selectContext({ ...context, runId: "run-2" });

    expect(order).toEqual(["subscribe-run-1", "unsubscribe-run-1", "subscribe-run-2"]);
    callbacks.get("run-1")?.(snapshot("run-1", [event(0), event(1)]));
    expect(useRunTraceStore.getState().snapshot.run?.id).toBe("run-2");
  });

  test("preserves stable selection while live events append", async () => {
    let onSnapshot: ((value: PersistedRunSnapshot) => void) | undefined;
    vi.mocked(runTraceApi.subscribeRunTraceSnapshot).mockImplementation(async (_runId, callback) => {
      onSnapshot = callback;
      return subscription(snapshot("run-1", [event(0), event(1)]));
    });
    await useRunTraceStore.getState().selectContext(context);
    useRunTraceStore.getState().selectNode("event:event-1");

    onSnapshot?.(snapshot("run-1", [event(0), event(1), event(2)]));
    expect(useRunTraceStore.getState().selectedNodeId).toBe("event:event-1");
    expect(useRunTraceStore.getState().projection.eventsById.size).toBe(3);
  });

  test("reopens the run root when selecting a phase", async () => {
    vi.mocked(runTraceApi.subscribeRunTraceSnapshot).mockResolvedValue(subscription());
    await useRunTraceStore.getState().selectContext(context);
    useRunTraceStore.getState().toggleExpanded("run:run-1");

    expect(useRunTraceStore.getState().projection.visibleRows).toHaveLength(1);
    expect(useRunTraceStore.getState().selectPhase("generate")).toBe("phase:generate");
    expect(useRunTraceStore.getState().selectedNodeId).toBe("phase:generate");
    expect(useRunTraceStore.getState().projection.visibleRows.some((row) => row.id === "phase:generate")).toBe(true);
  });

  test("loads remote endpoints concurrently and scopes their errors away from local trace", async () => {
    let rejectCheckpoints!: (error: Error) => void;
    let rejectReplay!: (error: Error) => void;
    vi.mocked(runTraceApi.subscribeRunTraceSnapshot).mockResolvedValue(subscription());
    vi.mocked(runTraceApi.fetchRunCheckpoints).mockReturnValue(new Promise((_resolve, reject) => {
      rejectCheckpoints = reject;
    }));
    vi.mocked(runTraceApi.fetchRunSurfaceReplay).mockReturnValue(new Promise((_resolve, reject) => {
      rejectReplay = reject;
    }));

    await useRunTraceStore.getState().selectContext(context);
    expect(useRunTraceStore.getState().network).toMatchObject({
      checkpoints: { status: "loading" },
      replay: { status: "loading" },
    });

    rejectCheckpoints(new Error("checkpoints offline"));
    rejectReplay(new Error("replay offline"));
    await vi.waitFor(() => {
      expect(useRunTraceStore.getState().network).toMatchObject({
        checkpoints: { status: "error", error: "checkpoints offline" },
        replay: { status: "error", error: "replay offline" },
      });
    });
    expect(useRunTraceStore.getState().projection.eventsById.size).toBe(1);
  });

  test("surfaces local subscription failure while still loading remote run data", async () => {
    vi.mocked(runTraceApi.subscribeRunTraceSnapshot).mockRejectedValue(new Error("local database unavailable"));

    await useRunTraceStore.getState().selectContext(context);

    expect(useRunTraceStore.getState().network.snapshot).toEqual({
      status: "error",
      error: "local database unavailable",
    });
    expect(runTraceApi.fetchRunCheckpoints).toHaveBeenCalledWith("run-1");
    expect(runTraceApi.fetchRunSurfaceReplay).toHaveBeenCalledWith("run-1");
    expect(runTraceApi.resolveRuntimeAgentArtifact).toHaveBeenCalledWith("agent-1");
  });

  test("preserves a selected checkpoint when refresh still contains it", async () => {
    vi.mocked(runTraceApi.subscribeRunTraceSnapshot).mockResolvedValue(subscription());
    const first = {
      id: "checkpoint-1",
      run_id: "run-1",
      thread_id: "thread-1",
      node_id: "node-1",
      iteration: 1,
      state: {},
      messages: [],
      created_at: "2026-08-07T20:00:00.000Z",
    };
    const second = { ...first, id: "checkpoint-2", node_id: "node-2", iteration: 2 };
    vi.mocked(runTraceApi.fetchRunCheckpoints)
      .mockResolvedValueOnce({ run_id: "run-1", checkpoints: [first, second] })
      .mockResolvedValueOnce({ run_id: "run-1", checkpoints: [first, second] });
    await useRunTraceStore.getState().selectContext(context);
    await vi.waitFor(() => expect(useRunTraceStore.getState().checkpoints).toHaveLength(2));
    useRunTraceStore.getState().selectCheckpoint("checkpoint-1");

    await useRunTraceStore.getState().refreshCheckpoints();
    expect(useRunTraceStore.getState().selectedCheckpointId).toBe("checkpoint-1");
  });

  test("keeps resume disabled until checkpoint and complete artifact prerequisites exist", async () => {
    vi.mocked(runTraceApi.subscribeRunTraceSnapshot).mockResolvedValue(subscription());
    await useRunTraceStore.getState().selectContext({ ...context, agentId: null });

    await expect(useRunTraceStore.getState().resumeLatestCheckpoint()).resolves.toBeNull();
    expect(runTraceApi.resumeRunFromLatestCheckpoint).not.toHaveBeenCalled();
  });

  test("hands off the returned run and preserves the local trace on resume failure", async () => {
    vi.mocked(runTraceApi.subscribeRunTraceSnapshot).mockResolvedValue(subscription());
    vi.mocked(runTraceApi.fetchRunCheckpoints).mockResolvedValue({
      run_id: "run-1",
      checkpoints: [{
        id: "checkpoint-1",
        run_id: "run-1",
        thread_id: "thread-1",
        node_id: "node-1",
        iteration: 1,
        state: {},
        messages: [],
        created_at: "2026-08-07T20:00:00.000Z",
      }],
    });
    vi.mocked(runTraceApi.resumeRunFromLatestCheckpoint).mockResolvedValueOnce({
      resumed_from_run_id: "run-1",
      run_id: "run-2",
      stream_url: "/api/uar/runs/run-2/stream",
    }).mockRejectedValueOnce(new Error("resume denied"));
    await useRunTraceStore.getState().selectContext(context);
    await vi.waitFor(() => {
      expect(useRunTraceStore.getState().agentArtifact?.id).toBe("agent-1");
      expect(useRunTraceStore.getState().checkpoints).toHaveLength(1);
    });

    await expect(useRunTraceStore.getState().resumeLatestCheckpoint("Continue")).resolves.toMatchObject({
      run_id: "run-2",
    });
    expect(runTraceApi.resumeRunFromLatestCheckpoint).toHaveBeenCalledWith("run-1", {
      artifact,
      input: "Continue",
      session_id: "session-1",
    });
    expect(useRunTraceStore.getState().resumedRunId).toBe("run-2");

    const preservedSnapshot = useRunTraceStore.getState().snapshot;
    await expect(useRunTraceStore.getState().resumeLatestCheckpoint()).resolves.toBeNull();
    expect(useRunTraceStore.getState().snapshot).toBe(preservedSnapshot);
    expect(useRunTraceStore.getState().network.resume).toEqual({
      status: "error",
      error: "resume denied",
    });
  });

  test("exposes a shallow, stable hook facade for unrelated store updates", async () => {
    const { result } = renderHook(() => useRunTrace(null));
    await act(async () => {});
    const before = result.current;

    act(() => {
      useRunTraceStore.setState((state) => ({ generation: state.generation + 1 }));
    });
    expect(result.current).toBe(before);
    expect(result.current).not.toHaveProperty("subscription");
  });
});
