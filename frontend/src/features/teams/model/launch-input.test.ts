import { describe, expect, test } from "vitest";

import { parseInput, slotProblem } from "./launch-input";

describe("launch input", () => {
  test("keeps the parse error so it can be shown inline", () => {
    expect(parseInput('{"a":1}')).toEqual({ ok: true, value: { a: 1 } });
    const bad = parseInput("{oops");
    expect(bad.ok).toBe(false);
  });

  test("holds a role's count inside its declared range", () => {
    const range = { min: 1, max: 3 };
    expect(slotProblem("2", range)).toBeNull();
    expect(slotProblem("0", range)).toContain("between 1 and 3");
    expect(slotProblem("4", range)).toContain("between 1 and 3");
    expect(slotProblem("two", range)).toContain("whole number");
    expect(slotProblem("2", { min: 1, max: 1 })).toContain("exactly 1");
  });
});
