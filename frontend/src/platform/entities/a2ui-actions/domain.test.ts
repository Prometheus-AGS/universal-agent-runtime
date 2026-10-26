import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { registerA2uiActionEntities } from "./index";
import { bindA2uiActionPersistence } from "./durability";
import { A2UI_ACTION_ENTITY, a2uiActionId, type A2uiActionReceipt } from "./contracts";
import { reconcileA2uiAction, submitA2uiAction } from "./domain";
import { acquireChatStream } from "./stream-ownership";

const harness = vi.hoisted(() => ({ attach: vi.fn(async () => {}) }));
vi.mock("@/stores/chat-stream-store", () => ({ useChatStreamStore: { getState: () => ({ attachContinuation: harness.attach }) } }));
let serial = 0;
const action = { name: "calculate", surfaceId: "parcel", sourceComponentId: "submit", timestamp: "2026-10-25T00:00:00Z", context: {}, a2uiClientDataModel: { parcel: { value: "137" } } };
const read = (source: string) => graphStore.getState().readEntity<A2uiActionReceipt>(A2UI_ACTION_ENTITY, a2uiActionId(source, "parcel"));
function source() { return `owned-action-${++serial}`; }
function server(runId: string) {
  const request = vi.fn(async (_url: RequestInfo | URL, init?: RequestInit) => new Response(JSON.stringify(init?.method === "POST"
    ? { run_id: runId, continuation_run_id: `${runId}-next`, status: "continued" }
    : { run_id: runId, conversation_id: "owned-session", status: "done" }), { status: init?.method === "POST" ? 202 : 200 }));
  vi.stubGlobal("fetch", request);
  return request;
}
beforeEach(() => { registerA2uiActionEntities(); harness.attach.mockReset().mockResolvedValue(undefined); bindA2uiActionPersistence(async () => {}); });
afterEach(() => { vi.unstubAllGlobals(); bindA2uiActionPersistence(null); });

describe("one admitted standard action per source surface", () => {
  test("writes pending before any I/O, rejects duplicate pending clicks and retains live ownership on remount", async () => {
    const runId = source();
    const request = server(runId);
    let resume!: () => void;
    let first = true;
    bindA2uiActionPersistence(() => first ? (first = false, new Promise<void>((resolve) => { resume = resolve; })) : Promise.resolve());
    const task = submitA2uiAction(runId, action);
    expect(read(runId)?.status).toBe("pending");
    expect(request).not.toHaveBeenCalled();
    reconcileA2uiAction(runId, "parcel");
    expect(read(runId)?.status).toBe("pending");
    await expect(submitA2uiAction(runId, action)).rejects.toThrow("already been submitted");
    expect(() => acquireChatStream()).toThrow("Another response is active");
    resume();
    await task;
    expect(harness.attach).toHaveBeenCalledWith("owned-session", `${runId}-next`, expect.any(Object));
    expect(read(runId)?.status).toBe("completed");
    await expect(submitA2uiAction(runId, action)).rejects.toThrow("already been submitted");
    expect(request.mock.calls.filter((call) => call[1]?.method === "POST")).toHaveLength(1);
  });
  test("failed pending persistence sends no request and releases its own slot", async () => {
    const runId = source();
    const request = server(runId);
    bindA2uiActionPersistence(async () => { throw new Error("storage unavailable"); });
    await expect(submitA2uiAction(runId, action)).rejects.toThrow("rejected");
    expect(request).not.toHaveBeenCalled();
    const lease = acquireChatStream(); lease.release();
  });
  test("an accepted run whose receipt cannot be persisted stays consumed with its actual identity", async () => {
    const runId = source();
    const request = server(runId);
    let writes = 0;
    bindA2uiActionPersistence(async () => { if (++writes > 1) throw new Error("storage ended after admission"); });
    await expect(submitA2uiAction(runId, action)).rejects.toThrow("interrupted");
    expect(read(runId)).toMatchObject({ status: "interrupted", continuation_run_id: `${runId}-next` });
    expect(harness.attach).not.toHaveBeenCalled();
    await expect(submitA2uiAction(runId, action)).rejects.toThrow("already been submitted");
    expect(request.mock.calls.filter((call) => call[1]?.method === "POST")).toHaveLength(1);
    const lease = acquireChatStream(); lease.release();
  });
  test("restored pending without a document task becomes unconfirmed without an orphan stream lock", () => {
    const runId = source();
    graphStore.getState().replaceEntity(A2UI_ACTION_ENTITY, a2uiActionId(runId, "parcel"), {
      id: a2uiActionId(runId, "parcel"), source_run_id: runId, surface_id: "parcel", continuation_run_id: null, status: "pending",
    });
    reconcileA2uiAction(runId, "parcel");
    expect(read(runId)?.status).toBe("uncertain");
    const lease = acquireChatStream(); lease.release();
  });
  test("attachment cancellation/error never marks the admitted response complete", async () => {
    const runId = source(); server(runId);
    harness.attach.mockRejectedValue(new Error("cancelled or missing terminal"));
    await expect(submitA2uiAction(runId, action)).rejects.toThrow("interrupted");
    expect(read(runId)).toMatchObject({ status: "interrupted", continuation_run_id: `${runId}-next` });
  });
});
