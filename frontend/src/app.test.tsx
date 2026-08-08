import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { describe, expect, test, vi } from "vitest";

import { AppRoutes, RouteLoadingFallback } from "./App";

vi.mock("@/hooks/use-app-bootstrap", () => ({ useAppBootstrap: vi.fn() }));
vi.mock("@/app/shell/app-shell", () => ({
  AppShell: ({ children }: { children: React.ReactNode }) => <div data-testid="app-shell">{children}</div>,
}));
vi.mock("@/components/Titlebar", () => ({ Titlebar: () => <div>Titlebar</div> }));
vi.mock("@/components/OfflineBanner", () => ({ OfflineBanner: () => null }));
vi.mock("sonner", () => ({ Toaster: () => null }));
vi.mock("@/pages/chat-page", () => ({ ChatPage: () => <div>Chat route resolved</div> }));
vi.mock("@/pages/admin-page", () => ({ AdminPage: () => <div>Admin route resolved</div> }));
vi.mock("@/pages/about-page", () => ({ AboutPage: () => <div>About route resolved</div> }));

function renderRoutes(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <AppRoutes />
    </MemoryRouter>,
  );
}

describe("application route loading boundaries", () => {
  test("keeps the default thread route in the static startup path", () => {
    renderRoutes("/threads");

    expect(screen.getByText("Chat route resolved")).toBeInTheDocument();
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  test("resolves the lazy administration route", async () => {
    renderRoutes("/admin");

    expect(await screen.findByText("Admin route resolved")).toBeInTheDocument();
  });

  test("resolves the lazy about route", async () => {
    renderRoutes("/about");

    expect(await screen.findByText("About route resolved")).toBeInTheDocument();
  });

  test("redirects unknown locations to the static thread route", async () => {
    renderRoutes("/not-a-route");

    await waitFor(() => expect(screen.getByText("Chat route resolved")).toBeInTheDocument());
  });

  test("provides an accessible shared loading state", () => {
    render(<RouteLoadingFallback label="administration" />);

    expect(screen.getByRole("status")).toHaveTextContent("Loading administration");
    expect(screen.getByRole("status")).toHaveAttribute("aria-live", "polite");
  });
});
