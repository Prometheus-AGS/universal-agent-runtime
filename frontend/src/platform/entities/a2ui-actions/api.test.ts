import { afterEach, describe, expect, test, vi } from "vitest";
import { A2uiActionError, postA2uiAction } from "./api";

const action = { name: "calculate", surfaceId: "parcel", sourceComponentId: "submit", timestamp: "2026-10-25T00:00:00Z", context: {}, a2uiClientDataModel: { parcel: { value: "137" } } };
afterEach(() => vi.unstubAllGlobals());

describe("standard A2UI action admission classification", () => {
  test("retains the actual source-bound continuation and current model", async () => {
    const request = vi.fn<typeof fetch>(async () => new Response(JSON.stringify({ run_id: "source", continuation_run_id: "child", status: "continued" }), { status: 202 }));
    vi.stubGlobal("fetch", request);
    expect(await postA2uiAction("source", action)).toBe("child");
    expect(JSON.parse(request.mock.calls[0][1]?.body as string)).toEqual(action);
  });
  test("only the actual declared-surface refusal is definite at 422", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => new Response(JSON.stringify({ error: "action is not declared by the active A2UI surface" }), { status: 422 })));
    await expect(postA2uiAction("source", action)).rejects.toMatchObject({ uncertain: false });
    vi.stubGlobal("fetch", vi.fn(async () => new Response(JSON.stringify({ error: "another stage failed" }), { status: 422 })));
    await expect(postA2uiAction("source", action)).rejects.toMatchObject({ uncertain: true });
  });
  test("transport loss and malformed accepted bodies never authorize a POST retry", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => { throw new TypeError("network"); }));
    await expect(postA2uiAction("source", action)).rejects.toEqual(new A2uiActionError(true));
    vi.stubGlobal("fetch", vi.fn(async () => new Response("{}", { status: 202 })));
    await expect(postA2uiAction("source", action)).rejects.toMatchObject({ uncertain: true });
  });
});
