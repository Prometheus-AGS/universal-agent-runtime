import { describe, expect, test, vi } from "vitest";

import type { ContentBlock } from "@/shared/content";

import {
  HISTORY_PROVENANCE_KIND,
  formatReceiptTime,
  modelOriginLabel,
  parseModelReceipt,
  parseReceiptSource,
  receiptSourceOf,
} from "./model-receipt";

vi.mock("@/platform/entities", () => ({
  isModelOrigin: (value: unknown) =>
    ["system_default", "conversation", "agent", "turn", "configured_fallback"].includes(String(value)),
}));

const RECEIPT = {
  version: 1,
  served_provider: "bdd-provider-b",
  served_model: "claude-5-haiku-20261101",
  model_origin: "system_default",
  model_origin_since: "2026-11-20T10:11:12Z",
};

describe("model receipt", () => {
  test("names the provider-reported model, not a requested one", () => {
    expect(parseModelReceipt(RECEIPT)).toEqual({
      provider: "bdd-provider-b",
      model: "claude-5-haiku-20261101",
      origin: "system_default",
      since: "2026-11-20T10:11:12Z",
    });
  });

  test("keeps a time only for the system default", () => {
    expect(parseModelReceipt({ ...RECEIPT, model_origin: "conversation" })?.since).toBeNull();
  });

  test("renders no origin when the runtime could not state one", () => {
    const receipt = parseModelReceipt({ ...RECEIPT, model_origin: null, model_origin_since: null });
    expect(receipt?.origin).toBeNull();
    expect(receipt?.since).toBeNull();
  });

  test("is absent without a serving provider and model", () => {
    expect(parseModelReceipt({ ...RECEIPT, served_model: "" })).toBeNull();
    expect(parseModelReceipt(null)).toBeNull();
  });

  test("uses the console's origin words", () => {
    expect(modelOriginLabel("system_default")).toBe("system default");
    expect(modelOriginLabel("conversation")).toBe("this conversation");
    expect(modelOriginLabel("agent")).toBe("agent");
    expect(modelOriginLabel("configured_fallback")).toBe("configured fallback");
    expect(modelOriginLabel("turn")).toBe("this turn");
  });

  test("reads the live receipt artifact", () => {
    const content: ContentBlock[] = [
      { type: "text", text: "hello" },
      { type: "artifact", id: "model-receipt-run-1", kind: "text/plain", content: JSON.stringify(RECEIPT) },
    ];
    expect(parseReceiptSource(receiptSourceOf(content))?.model).toBe("claude-5-haiku-20261101");
  });

  test("reads the receipt from reloaded run history", () => {
    const content: ContentBlock[] = [
      {
        type: "artifact",
        id: "m1:history",
        kind: HISTORY_PROVENANCE_KIND,
        content: JSON.stringify({ model_receipt: RECEIPT, usage_availability: "unavailable" }),
      },
    ];
    expect(parseReceiptSource(receiptSourceOf(content))?.provider).toBe("bdd-provider-b");
  });

  test("has no receipt when history carries none", () => {
    const content: ContentBlock[] = [
      { type: "artifact", id: "m1:history", kind: HISTORY_PROVENANCE_KIND, content: JSON.stringify({ model_receipt: null }) },
    ];
    expect(receiptSourceOf(content)).toBeNull();
  });

  test("formats local HH:MM:SS", () => {
    const time = formatReceiptTime("2026-11-20T10:11:12Z");
    expect(time).toMatch(/^\d{2}:\d{2}:\d{2}$/);
    const local = new Date("2026-11-20T10:11:12Z");
    expect(time?.slice(-2)).toBe(String(local.getSeconds()).padStart(2, "0"));
    expect(formatReceiptTime("not a time")).toBeNull();
  });
});
