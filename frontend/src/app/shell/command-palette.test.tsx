import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, useLocation } from "react-router";
import { beforeEach, describe, expect, test } from "vitest";

import { useUiStore } from "@/stores/ui-store";

import { CommandPalette } from "./command-palette";

function LocationProbe() {
  const location = useLocation();
  return <p data-testid="location">{`${location.pathname}${location.hash}`}</p>;
}

function renderPalette() {
  return render(
    <MemoryRouter initialEntries={["/threads"]}>
      <CommandPalette />
      <LocationProbe />
    </MemoryRouter>,
  );
}

describe("CommandPalette system default command", () => {
  beforeEach(() => {
    useUiStore.getState().setCommandPaletteOpen(true);
  });

  test("offers Change system default model and opens the Providers panel", async () => {
    const user = userEvent.setup();
    renderPalette();

    const palette = await screen.findByRole("dialog", { name: "Command palette" });
    const search = within(palette).getByRole("combobox", { name: "Search commands" });
    await user.type(search, "system default");
    expect(within(palette).getByText("Change system default model")).toBeVisible();

    await user.keyboard("{Enter}");

    await waitFor(() => {
      expect(screen.getByTestId("location")).toHaveTextContent("/admin/providers#system-default-model");
    });
  });

  test("a plain destination still navigates without a hash", async () => {
    const user = userEvent.setup();
    renderPalette();

    const palette = await screen.findByRole("dialog", { name: "Command palette" });
    await user.type(within(palette).getByRole("combobox", { name: "Search commands" }), "Knowledge");
    await user.keyboard("{Enter}");

    await waitFor(() => {
      expect(screen.getByTestId("location")).toHaveTextContent(/^\/admin\/knowledge$/);
    });
  });
});
