import budgetInventory from "../../performance-budgets.json" with { type: "json" };

export const PERFORMANCE_BUDGETS = budgetInventory.latencyMs;

export type PerformanceBudgetName = keyof typeof PERFORMANCE_BUDGETS;

export interface PerformanceBudgetResult {
  schemaVersion: 1;
  name: PerformanceBudgetName;
  observedMs: number;
  limitMs: number;
  verdict: "pass";
}

/** Validate and record one browser performance measurement. */
export function assertPerformanceBudget(
  name: PerformanceBudgetName,
  observedMs: number,
): PerformanceBudgetResult {
  const limitMs = PERFORMANCE_BUDGETS[name];
  if (!Number.isFinite(observedMs) || observedMs < 0) {
    throw new Error(`${name} produced an invalid duration: ${String(observedMs)}ms`);
  }
  if (observedMs > limitMs) {
    throw new Error(`${name} exceeded its ${limitMs}ms budget: ${observedMs}ms`);
  }

  return {
    schemaVersion: 1,
    name,
    observedMs,
    limitMs,
    verdict: "pass",
  };
}
