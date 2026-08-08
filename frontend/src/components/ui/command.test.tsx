import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "./command";

function CommandFixture({ onSelect = vi.fn() }: { onSelect?: (value: string) => void }) {
  return (
    <Command>
      <CommandInput aria-label="Search actions" />
      <CommandList>
        <CommandEmpty>No actions found.</CommandEmpty>
        <CommandGroup heading="Actions">
          <CommandItem value="Alpha action" onSelect={onSelect}>
            Alpha
          </CommandItem>
          <CommandItem value="Beta action" onSelect={onSelect}>Beta</CommandItem>
        </CommandGroup>
      </CommandList>
    </Command>
  );
}

describe("Command Base UI compatibility facade", () => {
  it("filters static action items and exposes the empty state", async () => {
    const user = userEvent.setup();
    render(<CommandFixture />);

    const input = screen.getByRole("combobox", { name: "Search actions" });
    await user.type(input, "beta");

    expect(screen.queryByText("Alpha")).not.toBeInTheDocument();
    expect(screen.getByText("Beta")).toBeInTheDocument();

    await user.clear(input);
    await user.type(input, "missing");
    expect(screen.getByText("No actions found.")).toBeInTheDocument();
  });

  it("translates pointer activation to the stable onSelect callback", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(<CommandFixture onSelect={onSelect} />);

    await user.click(screen.getByText("Alpha"));

    expect(onSelect).toHaveBeenCalledWith("Alpha action");
  });

  it("translates Enter on the highlighted match to onSelect", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(<CommandFixture onSelect={onSelect} />);

    const input = screen.getByRole("combobox", { name: "Search actions" });
    await user.type(input, "alpha");
    await user.keyboard("{Enter}");

    expect(onSelect).toHaveBeenCalledWith("Alpha action");
  });

  it("remains an action filter across repeated selections", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(<CommandFixture onSelect={onSelect} />);

    await user.click(screen.getByText("Alpha"));
    expect(screen.getByRole("combobox", { name: "Search actions" })).toHaveValue("");
    await user.click(screen.getByText("Beta"));

    expect(onSelect).toHaveBeenNthCalledWith(1, "Alpha action");
    expect(onSelect).toHaveBeenNthCalledWith(2, "Beta action");
  });
});
