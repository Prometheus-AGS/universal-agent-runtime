import { beforeEach, describe, expect, test, vi } from "vitest";

import { fetchAgentsList } from "@/features/agents/api";
import { getDbInstance } from "@/platform/pglite/client";
import {
  fetchRunCheckpoints,
  fetchRunSurfaceReplay,
  resolveRuntimeAgentArtifact,
  resumeRunFromLatestCheckpoint,
  subscribeRunTraceSnapshot,
} from "@/features/chat/api/run-trace-api";
import type { RuntimeAgentArtifact } from "@/features/chat/model/run-trace-types";

vi.mock("@/features/agents/api", () => ({ fetchAgentsList: vi.fn() }));
vi.mock("@/platform/pglite/client", () => ({ getDbInstance: vi.fn() }));

const artifact: RuntimeAgentArtifact = {
  version: "1",
  kind: "agent",
  id: "agent/one",
  metadata: { title: "Agent one" },
  runtime: { entry: "main" },
  policy: { provider: {} },
  schemas: {},
  prompt: { system: "System" },
  memory: {},
  tools: {},
  ui: {},
  extensions: { preserved: true },
};

function jsonResponse(value: unknown, init?: ResponseInit): Response {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "Content-Type": "application/json" },
    ...init,
  });
}

describe("run trace API", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  test("subscribes through the platform database boundary", async () => {
    const onSnapshot = vi.fn();
    const subscription = { initialSnapshot: { run: null, events: [] }, unsubscribe: vi.fn() };
    const subscribeRunSnapshot = vi.fn().mockResolvedValue(subscription);
    vi.mocked(getDbInstance).mockReturnValue({ subscribeRunSnapshot } as never);

    await expect(subscribeRunTraceSnapshot("run-1", onSnapshot)).resolves.toBe(subscription);
    expect(subscribeRunSnapshot).toHaveBeenCalledWith("run-1", onSnapshot);
  });

  test("loads and validates checkpoints from the encoded run URL", async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({
      run_id: "run/one",
      checkpoints: [{
        id: "checkpoint-1",
        run_id: "run/one",
        thread_id: "thread-1",
        node_id: "node-1",
        iteration: 2,
        state: { count: 1 },
        messages: [],
        created_at: "2026-08-07T20:00:00.000Z",
      }],
    }));
    vi.stubGlobal("fetch", fetchMock);

    await expect(fetchRunCheckpoints("run/one")).resolves.toMatchObject({
      run_id: "run/one",
      checkpoints: [{ id: "checkpoint-1", iteration: 2 }],
    });
    expect(fetchMock).toHaveBeenCalledWith("/api/uar/runs/run%2Fone/checkpoints");
  });

  test("rejects a checkpoint that omits its required state", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(jsonResponse({
      run_id: "run-1",
      checkpoints: [{
        id: "checkpoint-1",
        run_id: "run-1",
        thread_id: "thread-1",
        node_id: "node-1",
        iteration: 2,
        messages: [],
        created_at: "2026-08-07T20:00:00.000Z",
      }],
    })));

    await expect(fetchRunCheckpoints("run-1")).rejects.toThrow("invalid response");
  });

  test("posts the complete artifact and session context to latest resume", async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({
      resumed_from_run_id: "run/one",
      run_id: "run-2",
      stream_url: "/api/uar/runs/run-2/stream",
    }));
    vi.stubGlobal("fetch", fetchMock);

    await expect(resumeRunFromLatestCheckpoint("run/one", {
      artifact,
      input: "Continue",
      session_id: "session-1",
    })).resolves.toMatchObject({ run_id: "run-2" });

    expect(fetchMock).toHaveBeenCalledWith("/api/uar/runs/run%2Fone/resume", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ artifact, input: "Continue", session_id: "session-1" }),
    });
  });

  test("resolves and validates a complete runtime artifact from the agent catalog", async () => {
    vi.mocked(fetchAgentsList).mockResolvedValue([
      { ...artifact, _type: "runtime" } as never,
    ]);

    await expect(resolveRuntimeAgentArtifact("agent/one")).resolves.toEqual(artifact);
    expect(fetchAgentsList).toHaveBeenCalledOnce();
  });

  test("rejects malformed successful responses and HTTP failures", async () => {
    vi.stubGlobal("fetch", vi.fn()
      .mockResolvedValueOnce(jsonResponse({ checkpoints: [] }))
      .mockResolvedValueOnce(jsonResponse({ error: "unavailable" }, { status: 503 }))
      .mockResolvedValueOnce(jsonResponse([{ op: 7, path: "/surface" }])));

    await expect(fetchRunCheckpoints("run-1")).rejects.toThrow("invalid response");
    await expect(resumeRunFromLatestCheckpoint("run-1", { artifact })).rejects.toThrow("503");
    await expect(fetchRunSurfaceReplay("run-1")).rejects.toThrow("invalid response");
  });

  test("keeps checkpoint and replay failures independent", async () => {
    vi.stubGlobal("fetch", vi.fn((url: string | URL | Request) => {
      const href = String(url);
      if (href.endsWith("/checkpoints")) {
        return Promise.resolve(jsonResponse({ error: "offline" }, { status: 503 }));
      }
      return Promise.resolve(jsonResponse([
        { op: "add", path: "/surfaces/surface-1", value: { surfaceId: "surface-1" } },
      ]));
    }));

    const [checkpoints, replay] = await Promise.allSettled([
      fetchRunCheckpoints("run-1"),
      fetchRunSurfaceReplay("run-1"),
    ]);
    expect(checkpoints.status).toBe("rejected");
    expect(replay).toMatchObject({ status: "fulfilled", value: [{ op: "add" }] });
  });
});
