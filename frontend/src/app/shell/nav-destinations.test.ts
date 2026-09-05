import { describe, expect, test } from "vitest";

import {
  COMPACT_DESTINATIONS,
  buildNavigationDestinations,
  CONFIGURE_DESTINATIONS,
  findDestinationForPath,
  isConfigurePath,
  isDestinationActive,
  NAV_DESTINATIONS,
  WORK_DESTINATIONS,
} from "./nav-destinations";

describe("shell navigation inventory", () => {
  test("projects the binding work and compact destinations from one inventory", () => {
    expect(WORK_DESTINATIONS.map(({ id }) => id)).toEqual([
      "chat",
      "knowledge",
      "agents",
      "runs",
    ]);
    expect(COMPACT_DESTINATIONS.map(({ id }) => id)).toEqual([
      "chat",
      "knowledge",
      "agents",
    ]);
    expect(CONFIGURE_DESTINATIONS.map(({ label }) => label)).toEqual([
      "Providers",
      "MCP & tools",
      "Skills",
      "Presentations",
      "A2UI",
      "Runtime settings",
    ]);
    expect(new Set(NAV_DESTINATIONS.map(({ id }) => id)).size).toBe(
      NAV_DESTINATIONS.length,
    );
  });

  test.each([
    ["/threads/thread-1", "chat"],
    ["/admin/knowledge", "knowledge"],
    ["/admin/agents", "agents"],
    ["/admin/runs/run-1", "runs"],
    ["/admin/credentials", "providers"],
    ["/admin/mcp-health", "mcp-tools"],
    ["/admin/compiler", "skills"],
    ["/admin/presentations", "presentations"],
    ["/admin/a2ui-testing", "a2ui"],
    ["/admin/protocols", "runtime-settings"],
    ["/about", "about"],
  ])("matches %s to %s", (pathname, destinationId) => {
    expect(findDestinationForPath(pathname)?.id).toBe(destinationId);
  });

  test("does not let the broad Configure family claim work routes", () => {
    const runtimeSettings = NAV_DESTINATIONS.find(
      ({ id }) => id === "runtime-settings",
    );
    expect(runtimeSettings).toBeDefined();
    expect(findDestinationForPath("/admin/knowledge")?.id).toBe("knowledge");
    expect(runtimeSettings?.exactMatchPaths).toContain("/admin");
    expect(runtimeSettings && isDestinationActive(runtimeSettings, "/admin/knowledge")).toBe(false);
    expect(isConfigurePath("/admin/knowledge")).toBe(false);
    expect(isConfigurePath("/admin/agents")).toBe(false);
    expect(isConfigurePath("/admin/runs")).toBe(false);
    expect(isConfigurePath("/admin/settings")).toBe(true);
  });

  test("excludes the A2UI testing destination from production inventory and routing", () => {
    const production = buildNavigationDestinations({ includeDevelopment: false });
    expect(production.some(({ id }) => id === "a2ui")).toBe(false);
    expect(findDestinationForPath("/admin/presentations", production)?.id).toBe("presentations");
    expect(findDestinationForPath("/admin/a2ui-testing", production)).toBeUndefined();
    expect(buildNavigationDestinations({ includeDevelopment: true }).some(({ id }) => id === "a2ui")).toBe(true);
  });
});
