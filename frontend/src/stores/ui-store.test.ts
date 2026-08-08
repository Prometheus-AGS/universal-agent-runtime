import { beforeEach, describe, expect, test } from "vitest";

import { useUiStore } from "./ui-store";

beforeEach(() => {
  useUiStore.setState({
    mobileSidebarOpen: false,
    navRailCollapsed: false,
    commandPaletteOpen: false,
    shellSheet: null,
  });
});

describe("UI store shell state", () => {
  test("shares serializable shell state across sibling controls", () => {
    const actions = useUiStore.getState();

    actions.toggleNavRail();
    actions.setCommandPaletteOpen(true);
    actions.setShellSheet("configure");

    expect(useUiStore.getState()).toMatchObject({
      navRailCollapsed: true,
      commandPaletteOpen: true,
      shellSheet: "configure",
    });

    useUiStore.getState().closeShellOverlays();
    expect(useUiStore.getState()).toMatchObject({
      navRailCollapsed: true,
      commandPaletteOpen: false,
      shellSheet: null,
    });
  });
});
