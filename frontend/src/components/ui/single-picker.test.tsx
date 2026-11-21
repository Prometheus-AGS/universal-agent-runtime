import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, test, vi } from "vitest";

import type { PickerOption } from "./combobox-picker-model";
import { SinglePicker } from "./single-picker";

function options(count: number): PickerOption[] {
  return Array.from({ length: count }, (_, index) => ({
    value: `model-${index + 1}`,
    label: `Model ${index + 1}`,
    detail: `raw-${index + 1}`,
    groupId: index % 2 === 0 ? "even" : "odd",
    groupLabel: index % 2 === 0 ? "Even" : "Odd",
  }));
}

describe("SinglePicker", () => {
  test("renders a simple select for seven options and a search box for eight", async () => {
    const user = userEvent.setup();
    const { rerender } = render(
      <SinglePicker value="" onValueChange={vi.fn()} options={options(7)} ariaLabel="Model" searchAriaLabel="Search models" />,
    );
    await user.click(screen.getByRole("combobox", { name: "Model" }));
    expect(screen.queryByRole("combobox", { name: "Search models" })).not.toBeInTheDocument();
    await user.keyboard("{Escape}");

    rerender(
      <SinglePicker value="" onValueChange={vi.fn()} options={options(8)} ariaLabel="Model" searchAriaLabel="Search models" />,
    );
    await user.click(screen.getByRole("combobox", { name: "Model" }));
    expect(await screen.findByRole("combobox", { name: "Search models" })).toHaveFocus();
    expect(screen.getByText("8 options")).toBeVisible();
  });

  test("narrows by display name or raw id and reports no match", async () => {
    const user = userEvent.setup();
    render(
      <SinglePicker value="" onValueChange={vi.fn()} options={options(12)} ariaLabel="Model" searchAriaLabel="Search models" />,
    );
    await user.click(screen.getByRole("combobox", { name: "Model" }));
    const search = await screen.findByRole("combobox", { name: "Search models" });
    await user.type(search, "RAW-12");
    expect(await screen.findByRole("option", { name: /Model 12/ })).toBeVisible();
    expect(screen.queryByRole("option", { name: /Model 1 / })).not.toBeInTheDocument();
    await user.clear(search);
    await user.type(search, "zzzz-no-match");
    expect(screen.getByText("No matching options.")).toBeVisible();
  });

  test("shows a stored value that is no longer offered as unavailable without clearing it", () => {
    const onValueChange = vi.fn();
    render(
      <SinglePicker value="retired" onValueChange={onValueChange} options={options(10)} ariaLabel="Model" searchAriaLabel="Search models" />,
    );
    const trigger = screen.getByRole("combobox", { name: "Model" });
    expect(trigger).toHaveTextContent("Unavailable: retired");
    expect(trigger).toHaveAttribute("aria-invalid", "true");
    expect(trigger).toHaveAccessibleDescription(/no longer available/);
    expect(onValueChange).not.toHaveBeenCalled();
  });

  test("names loading and error states", () => {
    const { rerender } = render(
      <SinglePicker value="" onValueChange={vi.fn()} options={[]} status="loading" ariaLabel="Model" searchAriaLabel="Search models" />,
    );
    expect(screen.getByRole("combobox", { name: "Model" })).toHaveAttribute("aria-busy", "true");
    rerender(
      <SinglePicker value="" onValueChange={vi.fn()} options={[]} status="error" errorMessage="Catalog failed" ariaLabel="Model" searchAriaLabel="Search models" />,
    );
    expect(screen.getByRole("combobox", { name: "Model" })).toHaveTextContent("Catalog failed");
  });
});
