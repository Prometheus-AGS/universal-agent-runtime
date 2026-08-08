import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { AdminPage } from "./admin-page";

const useRuntimeConsoleFeeds = vi.fn();

vi.mock("@/features/agents", () => ({ AgentsPage: () => <div>Agents surface</div> }));
vi.mock("@/features/auth", () => ({ AuthPage: () => <div>Auth surface</div> }));
vi.mock("@/features/compiler", () => ({ CompilerPage: () => <div>Compiler surface</div> }));
vi.mock("@/features/cost", () => ({ CostDashboardPage: () => <div>Cost surface</div> }));
vi.mock("@/features/credentials", () => ({ CredentialsPage: () => <div>Credentials surface</div> }));
vi.mock("@/features/knowledge", () => ({ KnowledgePage: () => <div>Knowledge surface</div> }));
vi.mock("@/features/memory", () => ({ MemoryPage: () => <div>Memory surface</div> }));
vi.mock("@/features/models", () => ({ ModelsPage: () => <div>Models surface</div> }));
vi.mock("@/features/providers", () => ({
  AdminWelcome: () => <div>Provider welcome</div>,
  ProvidersPage: () => <div>Providers surface</div>,
}));
vi.mock("@/features/runtime", () => ({
  RuntimeApprovalsPage: () => <div>Approvals surface</div>,
  RuntimeCockpitPage: () => <div>Runtime surface</div>,
  RuntimeProtocolsPage: () => <div>Protocols surface</div>,
  RuntimeRunsPage: () => <div>Runs surface</div>,
}));
vi.mock("@/features/runtime/model/runtime-console-feeds", () => ({
  useRuntimeConsoleFeeds: () => useRuntimeConsoleFeeds(),
}));
vi.mock("@/features/settings", () => ({ SettingsPage: () => <div>Settings surface</div> }));
vi.mock("@/features/skills", () => ({ SkillsPage: () => <div>Skills surface</div> }));
vi.mock("@/features/tools", () => ({
  McpHealthPage: () => <div>MCP health surface</div>,
  ToolsPage: () => <div>Tools surface</div>,
}));
vi.mock("@/features/a2ui/testing", () => ({
  A2uiTestingPage: () => <div>A2UI testing surface</div>,
}));

function renderPage(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <AdminPage />
    </MemoryRouter>,
  );
}

describe("feature-owned administration route composition", () => {
  beforeEach(() => {
    useRuntimeConsoleFeeds.mockClear();
    document.documentElement.removeAttribute("data-admin-theme");
  });

  test("defaults unknown administration paths to the runtime cockpit", () => {
    renderPage("/admin/not-a-surface");

    expect(screen.getByTestId("admin-section-runtime")).toHaveTextContent("Runtime surface");
    expect(useRuntimeConsoleFeeds).toHaveBeenCalledOnce();
    expect(document.documentElement).not.toHaveAttribute("data-admin-theme");
  });

  test("renders feature routes without a nested admin shell", () => {
    renderPage("/admin/providers");

    expect(screen.getByTestId("admin-section-providers")).toHaveTextContent("Providers surface");
    expect(screen.getByText("Provider welcome")).toBeInTheDocument();
    expect(screen.queryByTestId("admin-shell")).not.toBeInTheDocument();
  });

  test("preserves the feature-owned MCP health route", () => {
    renderPage("/admin/mcp-health");

    expect(screen.getByTestId("admin-section-mcp-health")).toHaveTextContent("MCP health surface");
  });

  test("keeps the A2UI tester reachable in development", async () => {
    renderPage("/admin/a2ui-testing");

    expect(await screen.findByText("A2UI testing surface")).toBeInTheDocument();
    expect(screen.getByTestId("admin-section-a2ui-testing")).toHaveTextContent("A2UI testing surface");
  });
});
