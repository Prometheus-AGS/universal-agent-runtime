import { describe, expect, it } from "vitest";

import {
  buildVirtualRows,
  filterPickerOptions,
  groupPickerOptions,
  isUnavailableValue,
  pickerRenderMode,
  type PickerOption,
} from "./combobox-picker-model";

const option = (value: string, extra: Partial<PickerOption> = {}): PickerOption => ({
  value,
  label: value,
  ...extra,
});

describe("pickerRenderMode", () => {
  it("renders a simple select for fewer than eight ready options", () => {
    expect(pickerRenderMode(0, "ready")).toBe("select");
    expect(pickerRenderMode(7, "ready")).toBe("select");
    expect(pickerRenderMode(8, "ready")).toBe("list");
  });

  it("keeps the searchable list while loading or failed", () => {
    expect(pickerRenderMode(0, "loading")).toBe("list");
    expect(pickerRenderMode(3, "error")).toBe("list");
  });

  it("virtualizes only above 1,000 options", () => {
    expect(pickerRenderMode(1000, "ready")).toBe("list");
    expect(pickerRenderMode(1001, "ready")).toBe("virtual");
  });
});

describe("filterPickerOptions", () => {
  const options = [
    option("anthropic/claude-3", { label: "Claude 3", groupLabel: "GPT Hub" }),
    option("openai/gpt-4o", { label: "GPT-4o", detail: "gpt-4o" }),
  ];

  it("matches display name and raw id case-insensitively after trimming", () => {
    expect(filterPickerOptions(options, "  CLAUDE ").map((o) => o.value)).toEqual([
      "anthropic/claude-3",
    ]);
    expect(filterPickerOptions(options, "gpt-4O").map((o) => o.value)).toEqual([
      "openai/gpt-4o",
    ]);
  });

  it("does not match a group heading", () => {
    expect(filterPickerOptions(options, "hub")).toEqual([]);
  });

  it("returns every option for a blank query and none for a miss", () => {
    expect(filterPickerOptions(options, "   ")).toBe(options);
    expect(filterPickerOptions(options, "zzzz-no-match")).toEqual([]);
  });
});

describe("grouping and virtual rows", () => {
  const options = [
    option("a/1", { groupId: "a", groupLabel: "A" }),
    option("b/1", { groupId: "b", groupLabel: "B" }),
    option("a/2", { groupId: "a", groupLabel: "A" }),
  ];

  it("returns null when no option names a group", () => {
    expect(groupPickerOptions([option("x")])).toBeNull();
  });

  it("groups by first appearance and keeps option order inside a group", () => {
    const groups = groupPickerOptions(options)!;
    expect(groups.map((g) => g.id)).toEqual(["a", "b"]);
    expect(groups[0]!.items.map((o) => o.value)).toEqual(["a/1", "a/2"]);
  });

  it("indexes options among options only, skipping group headings", () => {
    const { rows, optionRowIndex } = buildVirtualRows(groupPickerOptions(options), options);
    expect(rows.map((r) => r.kind)).toEqual(["group", "option", "option", "group", "option"]);
    expect(optionRowIndex).toEqual([1, 2, 4]);
  });
});

describe("isUnavailableValue", () => {
  const options = [option("a"), option("b", { disabled: true })];

  it("flags a stored value that is absent or disabled once ready", () => {
    expect(isUnavailableValue("zzz", options, "ready")).toBe(true);
    expect(isUnavailableValue("b", options, "ready")).toBe(true);
    expect(isUnavailableValue("a", options, "ready")).toBe(false);
  });

  it("never flags an empty value or a list that is not ready", () => {
    expect(isUnavailableValue("", options, "ready")).toBe(false);
    expect(isUnavailableValue("zzz", options, "loading")).toBe(false);
  });
});
