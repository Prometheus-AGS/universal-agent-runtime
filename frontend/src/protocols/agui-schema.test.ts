import fs from "node:fs";
import path from "node:path";
import { describe, expect, test } from "vitest";

import { isUarAguiEvent } from "@/protocols/agui-schema";

describe("shared UAR AG-UI golden fixture", () => {
  test("is accepted by the TypeScript runtime validator", () => {
    const fixturePath = path.resolve(
      process.cwd(),
      "../tests/fixtures/agui/uar-agui-v1.json",
    );
    const events = JSON.parse(fs.readFileSync(fixturePath, "utf8")) as unknown[];

    expect(events.length).toBeGreaterThan(0);
    expect(events.every(isUarAguiEvent)).toBe(true);
    expect(events.some((event) => String((event as { type?: string }).type).startsWith("THINKING_")))
      .toBe(false);
  });

  test("rejects malformed, legacy, and unnamed custom events", () => {
    expect(isUarAguiEvent({ type: "agui.done", profile: "uar.agui/1", eventId: "1", sequence: 1 }))
      .toBe(false);
    expect(isUarAguiEvent({ type: "CUSTOM", profile: "uar.agui/1", eventId: "1", sequence: 1, value: {} }))
      .toBe(false);
    expect(isUarAguiEvent({ type: "STATE_DELTA", profile: "uar.agui/1", eventId: "1", sequence: 1, delta: {} }))
      .toBe(false);
  });
});
