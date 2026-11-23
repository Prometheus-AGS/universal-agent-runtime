import { afterEach, describe, expect, test, vi } from "vitest";

import {
  decodeSessionEffectiveModel,
  emptySessionEffectiveModel,
  fetchSessionEffectiveModel,
} from "./effective-model";

afterEach(() => {
  vi.unstubAllGlobals();
});

const BODY = {
  session_id: "s1",
  requested_policy: { model: { provider_id: "bdd-provider-c", model_id: "qwen-5-turbo" } },
  effective_policy: {
    model: { provider_id: "bdd-provider-c", model_id: "qwen-5-turbo" },
    model_origin: "conversation",
  },
  model_unavailable_reason: null,
  inherited_model: {
    model: { provider_id: "bdd-provider-b", model_id: "claude-5-haiku" },
    model_origin: "system_default",
    model_origin_since: "2026-11-20T10:11:12Z",
    unavailable_reason: null,
  },
};

describe("decodeSessionEffectiveModel", () => {
  test("qualifies the effective and the inherited model and carries their origins", () => {
    expect(decodeSessionEffectiveModel("s1", BODY)).toMatchObject({
      stored: true,
      model: "bdd-provider-c/qwen-5-turbo",
      origin: "conversation",
      since: null,
      inherited_model: "bdd-provider-b/claude-5-haiku",
      inherited_origin: "system_default",
      inherited_since: "2026-11-20T10:11:12Z",
      inherited_unavailable_reason: null,
    });
  });

  test("names the configured fallback when that is what would answer", () => {
    const decoded = decodeSessionEffectiveModel("s1", {
      ...BODY,
      inherited_model: {
        model: { provider_id: "openai", model_id: "gpt-5.4-mini" },
        model_origin: "configured_fallback",
        model_origin_since: null,
        unavailable_reason: null,
      },
    });
    expect(decoded.inherited_model).toBe("openai/gpt-5.4-mini");
    expect(decoded.inherited_origin).toBe("configured_fallback");
  });

  test("carries the reason, and no model, when admission would refuse", () => {
    const decoded = decodeSessionEffectiveModel("s1", {
      session_id: "s1",
      effective_policy: { model: null },
      model_unavailable_reason: "Provider 'x' is disabled",
      inherited_model: { model: null, unavailable_reason: "Provider 'x' is disabled" },
    });
    expect(decoded.model).toBeNull();
    expect(decoded.origin).toBeNull();
    expect(decoded.unavailable_reason).toBe("Provider 'x' is disabled");
    expect(decoded.inherited_model).toBeNull();
    expect(decoded.inherited_unavailable_reason).toBe("Provider 'x' is disabled");
  });

  test("ignores an origin it does not know rather than guessing", () => {
    const decoded = decodeSessionEffectiveModel("s1", {
      ...BODY,
      effective_policy: { model: BODY.effective_policy.model, model_origin: "mystery" },
    });
    expect(decoded.origin).toBeNull();
  });

  test("an unstored conversation has nothing to state", () => {
    expect(emptySessionEffectiveModel("s1")).toMatchObject({ stored: false, model: null, inherited_model: null });
  });

  test("rejects a body that is not an object", () => {
    expect(() => decodeSessionEffectiveModel("s1", "nope")).toThrow();
  });
});

describe("fetchSessionEffectiveModel", () => {
  function stubResponse(status: number, body: unknown) {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(new Response(JSON.stringify(body), { status })),
    );
  }

  test("a conversation with nothing saved is still answered by the runtime", async () => {
    stubResponse(200, {
      session_id: "s1",
      requested_policy: null,
      effective_policy: {
        model: { provider_id: "openai", model_id: "gpt-5.4-mini" },
        model_origin: "system_default",
        model_origin_since: "2026-11-20T10:11:12Z",
      },
      model_unavailable_reason: null,
      inherited_model: {
        model: { provider_id: "openai", model_id: "gpt-5.4-mini" },
        model_origin: "system_default",
        model_origin_since: "2026-11-20T10:11:12Z",
        unavailable_reason: null,
      },
    });

    expect(await fetchSessionEffectiveModel("s1")).toMatchObject({
      stored: false,
      model: "openai/gpt-5.4-mini",
      origin: "system_default",
      since: "2026-11-20T10:11:12Z",
      inherited_model: "openai/gpt-5.4-mini",
    });
  });

  test("a stored conversation is marked stored", async () => {
    stubResponse(200, { ...BODY, requested_policy: { model: null } });

    expect((await fetchSessionEffectiveModel("s1")).stored).toBe(true);
  });

  test.each([503, 404, 500])("status %s is a load error, never a guessed model", async (status) => {
    stubResponse(status, { error: "unavailable" });

    await expect(fetchSessionEffectiveModel("s1")).rejects.toThrow(String(status));
  });
});
