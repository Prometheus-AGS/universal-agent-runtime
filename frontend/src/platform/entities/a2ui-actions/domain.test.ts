import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { registerA2uiActionEntities } from "./index";
import { bindA2uiActionPersistence } from "./durability";
import { A2UI_ACTION_ENTITY, a2uiActionId } from "./contracts";
import { reconcileA2uiAction, submitA2uiAction, latestActionReceipt } from "./domain";
import { acquireChatStream } from "./stream-ownership";

const harness = vi.hoisted(() => ({ attach: vi.fn(async () => {}) }));
vi.mock("@/stores/chat-stream-store", () => ({ useChatStreamStore: { getState: () => ({ attachContinuation: harness.attach }) } }));
let serial = 0;
const action = { surface_generation: "generation-1", interaction_id: "logical-click", name: "calculate", surfaceId: "parcel", sourceComponentId: "submit", timestamp: "2026-10-25T00:00:00Z", context: {}, a2uiClientDataModel: { parcel: { value: "137" } } };
const read = (source: string) => latestActionReceipt(source, "parcel");
function source() { return `owned-action-${++serial}`; }
function server(runId: string) {
  const request = vi.fn(async (url: RequestInfo | URL, init?: RequestInit) => new Response(JSON.stringify(init?.method === "POST"
    ? { run_id: runId, continuation_run_id: `${runId}-next`, status: "continued", interaction_id: JSON.parse(String(init.body)).interaction_id, retry_protected: true }
    : String(url).endsWith("surface-authority") ? { status: "complete", authoritative: true, recovery_hint: "", surfaces: [{ surface_id: "parcel", surface_generation: "generation-1", source_run_id: runId }] }
    : { run_id: runId, conversation_id: "owned-session", status: "done" }), { status: init?.method === "POST" ? 202 : 200 }));
  vi.stubGlobal("fetch", request);
  return request;
}
beforeEach(() => { registerA2uiActionEntities(); harness.attach.mockReset().mockResolvedValue(undefined); bindA2uiActionPersistence(async () => {}); });
afterEach(() => { vi.unstubAllGlobals(); bindA2uiActionPersistence(null); });

describe("durable logical standard actions", () => {
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
    bindA2uiActionPersistence(async () => { if (++writes > 2) throw new Error("storage ended after admission"); });
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


describe("logical identity recovery", () => {
  test("a distinct intentional click uses a new identity after successful settlement", async () => {
    const runId = source(); const request = server(runId);
    await submitA2uiAction(runId, { ...action, interaction_id: "first" });
    await submitA2uiAction(runId, { ...action, interaction_id: "second" });
    const posts = request.mock.calls.filter((call) => call[1]?.method === "POST");
    expect(posts).toHaveLength(2);
    expect(posts.map((call) => JSON.parse(String(call[1]?.body)).interaction_id)).toEqual(["first", "second"]);
    expect(posts.every((call) => JSON.parse(String(call[1]?.body)).surface_generation === "generation-1")).toBe(true);
  });
  test("lost response reconciles original logical ID without another POST", async () => {
    const runId = source();
    graphStore.getState().replaceEntity(A2UI_ACTION_ENTITY, "lost-receipt", { id: "lost-receipt", interaction_id: "lost-id",
      source_run_id: runId, surface_id: "parcel", continuation_run_id: null, status: "pending" });
    const request = vi.fn<typeof fetch>(async (url) => new Response(JSON.stringify(String(url).includes("/a2ui/actions/")
      ? { run_id: runId, interaction_id: "lost-id", retry_protected: true, continuation_run_id: "original-child", status: "continued", outcome: "completed" }
      : { run_id: runId, conversation_id: "owned-session", status: "done" }), { status: String(url).includes("/a2ui/actions/") ? 202 : 200 }));
    vi.stubGlobal("fetch", request);
    await reconcileA2uiAction(runId, "parcel");
    expect(read(runId)).toMatchObject({ interaction_id: "lost-id", continuation_run_id: "original-child", status: "completed" });
    expect(request).toHaveBeenCalledTimes(2);
    expect(request.mock.calls.every((call) => call[1]?.method !== "POST")).toBe(true);
    expect(String(request.mock.calls[0][0])).toContain("/a2ui/actions/lost-id");
  });
  test("incomplete authority disables admission before effects", async () => {
    const runId = source();
    const request = vi.fn<typeof fetch>(async () => new Response(JSON.stringify({ status: "truncated", authoritative: false,
      recovery_hint: "Start a new conversation for a fresh surface.", surfaces: [] })));
    vi.stubGlobal("fetch", request);
    await expect(submitA2uiAction(runId, action)).rejects.toThrow("rejected");
    expect(request.mock.calls.every((call) => call[1]?.method !== "POST")).toBe(true);
  });
});

describe("rendered publication admission", () => {
  test("G1 intent is refused against G2 authority even when surface, button and action identities survive", async () => {
    const runId = source();
    const request = server(runId);
    await expect(submitA2uiAction(runId, { ...action, surface_generation: "generation-0" })).rejects.toThrow("rejected");
    expect(request.mock.calls.filter((call) => call[1]?.method === "POST")).toHaveLength(0);
    expect(read(runId)).toMatchObject({ surface_generation: "generation-0", interaction_id: action.interaction_id,
      input: { ...action, surface_generation: "generation-0" }, status: "rejected" });
  });
  test("missing rendered generation is persisted as missing and never substituted from authority", async () => {
    const runId = source(); const request = server(runId);
    await expect(submitA2uiAction(runId, { ...action, surface_generation: undefined })).rejects.toThrow("rejected");
    expect(request).not.toHaveBeenCalled();
    expect(read(runId)?.surface_generation).toBeUndefined();
    expect(read(runId)?.input?.surface_generation).toBeUndefined();
  });
  test.each(["foreign-source", "duplicate", "deleted"])("%s authority cannot qualify an exact-generation intent", async (kind) => {
    const runId = source();
    const row = { surface_id: "parcel", surface_generation: action.surface_generation, source_run_id: runId };
    const request = vi.fn<typeof fetch>(async () => new Response(JSON.stringify({ status: "complete", authoritative: true,
      recovery_hint: "", surfaces: kind === "deleted" ? [] : kind === "duplicate" ? [row, row] : [{ ...row, source_run_id: "foreign-run" }] })));
    vi.stubGlobal("fetch", request);
    await expect(submitA2uiAction(runId, action)).rejects.toThrow("rejected");
    expect(request.mock.calls.every((call) => call[1]?.method !== "POST")).toBe(true);
  });
  test("matching generation, original context and generated interaction ID are durable before I/O and immutable across awaits", async () => {
    const runId = source(); const request = server(runId);
    const mutable = { ...action, interaction_id: undefined, context: { choice: "original" },
      a2uiClientDataModel: { parcel: { value: "137" } } };
    let resume!: () => void;
    let first = true;
    bindA2uiActionPersistence(() => first ? (first = false, new Promise<void>((resolve) => { resume = resolve; })) : Promise.resolve());
    const task = submitA2uiAction(runId, mutable);
    const original = structuredClone(read(runId));
    expect(original?.interaction_id).toBeTruthy();
    expect(original?.input?.interaction_id).toBe(original?.interaction_id);
    expect(original?.surface_generation).toBe("generation-1");
    expect(request).not.toHaveBeenCalled();
    mutable.surface_generation = "later-generation";
    mutable.context.choice = "later";
    mutable.a2uiClientDataModel.parcel.value = "999";
    resume(); await task;
    const post = request.mock.calls.find((call) => call[1]?.method === "POST");
    expect(JSON.parse(String(post?.[1]?.body))).toEqual(original?.input);
    expect(read(runId)?.input).toEqual(original?.input);
  });
  test("uncertain matched intent retains its original generation and identity during read-only recovery", async () => {
    const runId = source(); const request = server(runId);
    request.mockImplementation(async (url, init) => {
      if (init?.method === "POST") throw new Error("lost acknowledgment");
      if (String(url).includes(`/a2ui/actions/${action.interaction_id}`)) return new Response(JSON.stringify({ run_id: runId,
        interaction_id: action.interaction_id, retry_protected: true, continuation_run_id: "original-child", outcome: "uncertain" }));
      return new Response(JSON.stringify(String(url).endsWith("surface-authority") ? { status: "complete", authoritative: true,
        recovery_hint: "", surfaces: [{ surface_id: "parcel", surface_generation: "generation-1", source_run_id: runId }] }
        : { run_id: runId, conversation_id: "owned-session", status: "done" }));
    });
    await expect(submitA2uiAction(runId, action)).rejects.toThrow("unconfirmed");
    await reconcileA2uiAction(runId, "parcel");
    expect(read(runId)).toMatchObject({ interaction_id: action.interaction_id, surface_generation: "generation-1", input: action });
    await expect(submitA2uiAction(runId, { ...action, surface_generation: "generation-2" })).rejects.toThrow("already been submitted");
    expect(request.mock.calls.filter((call) => call[1]?.method === "POST")).toHaveLength(1);
  });
});
