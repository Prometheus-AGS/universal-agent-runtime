import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, useLocation } from "react-router";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { useUiStore } from "@/stores/ui-store";

import { AppShell } from "./app-shell";

vi.mock("@/hooks/use-healthz", () => ({
  useHealthz: () => ({
    health: null,
    load: vi.fn(),
  }),
}));

vi.mock("@/components/ThemeToggle", () => ({
  ThemeToggle: () => <button type="button" aria-label="Theme">Theme</button>,
}));

function LocationProbe() {
  return <p data-testid="location">{useLocation().pathname}</p>;
}

function renderShell(initialPath = "/threads") {
  return render(
    <MemoryRouter initialEntries={[initialPath]}>
      <AppShell>
        <div>Feature content</div>
        <LocationProbe />
      </AppShell>
    </MemoryRouter>,
  );
}

function installDesktopQuery(initialMatches = false) {
  let matches = initialMatches;
  const listeners = new Set<() => void>();
  const originalMatchMedia = window.matchMedia;
  const desktopQuery = {
    get matches() { return matches; },
    media: "(min-width: 901px)",
    onchange: null,
    addEventListener: (_event: string, listener: () => void) => { listeners.add(listener); },
    removeEventListener: (_event: string, listener: () => void) => { listeners.delete(listener); },
    addListener: (listener: () => void) => { listeners.add(listener); },
    removeListener: (listener: () => void) => { listeners.delete(listener); },
    dispatchEvent: () => true,
  } as unknown as MediaQueryList;

  window.matchMedia = vi.fn().mockImplementation((query: string) => (
    query === desktopQuery.media
      ? desktopQuery
      : originalMatchMedia?.(query) ?? { ...desktopQuery, media: query, matches: false }
  ));

  return {
    setDesktop(nextMatches: boolean) {
      matches = nextMatches;
      listeners.forEach((listener) => listener());
    },
    restore() {
      window.matchMedia = originalMatchMedia;
    },
  };
}

beforeEach(() => {
  useUiStore.setState({
    mobileSidebarOpen: false,
    navRailCollapsed: false,
    commandPaletteOpen: false,
    shellSheet: null,
  });
});

describe("application shell", () => {
  test("renders one main landmark, skip navigation, brand, breadcrumb, and readiness text", () => {
    renderShell();

    expect(screen.getAllByRole("main")).toHaveLength(1);
    expect(screen.getByRole("link", { name: "Skip to content" })).toHaveAttribute(
      "href",
      "#shell-main-content",
    );
    expect(screen.getByRole("main")).toHaveAttribute("id", "shell-main-content");
    expect(screen.getByRole("navigation", { name: "Breadcrumb" })).toHaveTextContent(
      "Workspace/Chat",
    );
    expect(screen.getAllByText("Unreachable").length).toBeGreaterThan(0);
    const brand = screen.getByRole("img", { name: "Universal Agent Runtime" });
    expect(brand.querySelector("svg")).toBeInTheDocument();
  });

  test("collapses the desktop rail while retaining accessible destination labels", async () => {
    const user = userEvent.setup();
    renderShell();

    const rail = screen.getByRole("navigation", { name: "Primary navigation" });
    expect(rail).toHaveClass("w-60");

    await user.click(screen.getByRole("button", { name: "Collapse navigation" }));

    expect(rail).toHaveClass("w-[60px]");
    expect(within(rail).getByRole("link", { name: "Knowledge" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Expand navigation" })).toHaveAttribute(
      "aria-expanded",
      "false",
    );
  });

  test("opens the compact Configure hub and closes it after route navigation", async () => {
    const viewport = installDesktopQuery();
    const user = userEvent.setup();
    const view = renderShell();

    try {
      await user.click(screen.getByRole("button", { name: "Configure" }));
      const dialog = await screen.findByRole("dialog", { name: "Configure" });
      expect(within(dialog).getByRole("navigation", { name: "Configure destinations" })).toBeVisible();
      expect(within(dialog).getByRole("link", { name: /Providers/ })).toBeVisible();

      await user.click(within(dialog).getByRole("link", { name: /Providers/ }));

      await waitFor(() => {
        expect(screen.getByTestId("location")).toHaveTextContent("/admin/providers");
        expect(screen.queryByRole("dialog", { name: "Configure" })).not.toBeInTheDocument();
      });
      expect(screen.getByRole("navigation", { name: "Breadcrumb" })).toHaveTextContent(
        "Configure/Providers",
      );
    } finally {
      view.unmount();
      viewport.restore();
    }
  });

  test("opens the Base UI palette with Control+K and navigates with a command", async () => {
    const user = userEvent.setup();
    renderShell();

    await user.keyboard("{Control>}k{/Control}");
    const palette = await screen.findByRole("dialog", { name: "Command palette" });
    const search = within(palette).getByRole("combobox", { name: "Search commands" });
    expect(search).toHaveFocus();

    await user.type(search, "Knowledge");
    await user.keyboard("{Enter}");

    await waitFor(() => {
      expect(screen.getByTestId("location")).toHaveTextContent("/admin/knowledge");
      expect(screen.queryByRole("dialog", { name: "Command palette" })).not.toBeInTheDocument();
    });
  });

  test("keeps non-tab work and system routes available through compact command access", async () => {
    const user = userEvent.setup();
    renderShell();

    await user.click(screen.getByRole("button", { name: "Open command palette" }));
    const palette = await screen.findByRole("dialog", { name: "Command palette" });

    expect(within(palette).getByText("Runs")).toBeVisible();
    expect(within(palette).getByText("About")).toBeVisible();
  });

  test("does not intercept the global shortcut from an editable control", async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter initialEntries={["/threads"]}>
        <input aria-label="Draft title" />
        <AppShell><div>Feature content</div></AppShell>
      </MemoryRouter>,
    );

    const editable = screen.getByRole("textbox", { name: "Draft title" });
    await user.click(editable);
    expect(fireEvent.keyDown(editable, { ctrlKey: true, key: "k" })).toBe(true);

    expect(screen.queryByRole("dialog", { name: "Command palette" })).not.toBeInTheDocument();
  });

  test("does not intercept the global shortcut from plaintext contenteditable", () => {
    render(
      <MemoryRouter initialEntries={["/threads"]}>
        <div aria-label="Draft body" contentEditable="plaintext-only" role="textbox" />
        <AppShell><div>Feature content</div></AppShell>
      </MemoryRouter>,
    );

    const editable = screen.getByRole("textbox", { name: "Draft body" });
    expect(fireEvent.keyDown(editable, { ctrlKey: true, key: "k" })).toBe(true);
    expect(screen.queryByRole("dialog", { name: "Command palette" })).not.toBeInTheDocument();
  });

  test("closes the compact Configure sheet when the viewport becomes desktop-sized", async () => {
    const viewport = installDesktopQuery();
    const user = userEvent.setup();
    const view = renderShell();
    try {
      await user.click(screen.getByRole("button", { name: "Configure" }));
      expect(await screen.findByRole("dialog", { name: "Configure" })).toBeVisible();

      viewport.setDesktop(true);
      await waitFor(() => {
        expect(screen.queryByRole("dialog", { name: "Configure" })).not.toBeInTheDocument();
      });
    } finally {
      view.unmount();
      viewport.restore();
    }
  });

  test("does not stack the global palette over an existing dialog", async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter initialEntries={["/admin/settings"]}>
        <div role="dialog" aria-modal="true" aria-label="Legacy command palette">Legacy palette</div>
        <AppShell><div>Feature content</div></AppShell>
      </MemoryRouter>,
    );

    await user.keyboard("{Control>}k{/Control}");

    expect(screen.getByRole("dialog", { name: "Legacy command palette" })).toBeVisible();
    expect(screen.queryByRole("dialog", { name: "Command palette" })).not.toBeInTheDocument();
  });

  test("toggles the global palette closed with the same shortcut", async () => {
    const user = userEvent.setup();
    renderShell();

    await user.keyboard("{Control>}k{/Control}");
    expect(await screen.findByRole("dialog", { name: "Command palette" })).toBeVisible();

    await user.keyboard("{Control>}k{/Control}");
    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: "Command palette" })).not.toBeInTheDocument();
    });
  });

  test("labels system breadcrumbs and exposes the compact Configure current state", () => {
    const { unmount } = renderShell("/about");
    expect(screen.getByRole("navigation", { name: "Breadcrumb" })).toHaveTextContent(
      "System/About",
    );

    unmount();
    renderShell("/admin/settings");
    expect(screen.getByRole("button", { name: /^Configure$/ })).toHaveAttribute(
      "aria-current",
      "true",
    );
  });

  test("marks exactly one rail destination current for admin-backed work routes", () => {
    renderShell("/admin/knowledge");
    const rail = screen.getByRole("navigation", { name: "Primary navigation" });

    expect(within(rail).getAllByRole("link").filter(
      (link) => link.getAttribute("aria-current") === "page",
    )).toHaveLength(1);
    expect(within(rail).getByRole("link", { name: "Knowledge" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(within(rail).getByRole("link", { name: "Runtime settings" })).not.toHaveAttribute(
      "aria-current",
    );
  });
});
