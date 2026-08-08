import { describe, expect, test } from "vitest";

import {
  filterModelRows,
  projectModelRows,
  selectComparisonModels,
} from "./model-catalog-view";

const rows = projectModelRows([
  {
    id: "alpha/tool-model",
    provider_id: "alpha",
    provider_name: "Alpha",
    provider_configured: true,
    model_id: "tool-model",
    name: "Tool Model",
    context: 100,
    tool_call: true,
    reasoning: false,
    vision: false,
  },
  {
    id: "beta/vision-reasoner",
    provider_id: "beta",
    model_id: "vision-reasoner",
    name: "Vision Reasoner",
    tool_call: false,
    reasoning: true,
    vision: true,
  },
]);

describe("model catalog presentation", () => {
  test("projects missing metadata to deterministic display defaults", () => {
    expect(rows[1]).toMatchObject({
      provider_name: "beta",
      context: 0,
      cost_input: 0,
      cost_output: 0,
      benchmarks: [],
    });
  });

  test("filters by provider, capabilities, key, and display name", () => {
    expect(
      filterModelRows(rows, "alpha", { tools: true, reasoning: false, vision: false }, ""),
    ).toEqual([rows[0]]);
    expect(
      filterModelRows(rows, "all", { tools: false, reasoning: true, vision: true }, "vision"),
    ).toEqual([rows[1]]);
    expect(
      filterModelRows(rows, "all", { tools: false, reasoning: false, vision: false }, "ALPHA/TOOL"),
    ).toEqual([rows[0]]);
  });

  test("preserves compare pick order and ignores missing keys", () => {
    expect(
      selectComparisonModels(["beta/vision-reasoner", "missing", "alpha/tool-model"], rows)
        .map((model) => model.key),
    ).toEqual(["beta/vision-reasoner", "alpha/tool-model"]);
  });
});
