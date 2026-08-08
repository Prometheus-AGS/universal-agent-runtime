import { describe, expect, test } from "vitest";

import {
  assertPerformanceBudget,
  PERFORMANCE_BUDGETS,
  type PerformanceBudgetName,
} from "./performance-budget";

const names = Object.keys(PERFORMANCE_BUDGETS) as PerformanceBudgetName[];

describe("frontend performance budgets", () => {
  test.each(names)("accepts below-limit and exact-limit %s observations", (name) => {
    const limitMs = PERFORMANCE_BUDGETS[name];
    expect(assertPerformanceBudget(name, limitMs - 0.01)).toMatchObject({
      name,
      limitMs,
      verdict: "pass",
    });
    expect(assertPerformanceBudget(name, limitMs)).toMatchObject({
      name,
      observedMs: limitMs,
      verdict: "pass",
    });
  });

  test.each(names)("rejects over-limit %s observations", (name) => {
    const limitMs = PERFORMANCE_BUDGETS[name];
    expect(() => assertPerformanceBudget(name, limitMs + 0.01)).toThrow(
      `${name} exceeded its ${limitMs}ms budget`,
    );
  });

  test.each(names)("rejects invalid %s observations", (name) => {
    expect(() => assertPerformanceBudget(name, -1)).toThrow("invalid duration");
    expect(() => assertPerformanceBudget(name, Number.NaN)).toThrow("invalid duration");
    expect(() => assertPerformanceBudget(name, Number.POSITIVE_INFINITY)).toThrow("invalid duration");
  });
});
