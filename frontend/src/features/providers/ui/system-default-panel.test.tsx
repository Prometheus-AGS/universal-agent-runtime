import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { PinnedAgentsSnapshot } from "../model/pinned-agents";
import { SystemDefaultPanel } from "./system-default-panel";

const mocks = vi.hoisted(() => ({
  FakeSystemDefaultError: class extends Error {},
  setSystemDefaultModel: vi.fn(),
  loadPinnedAgents: vi.fn(),
  meta: { default_id: "openai" as string | null, default_model: "gpt-5.4-mini" as string | null },
  pinned: null as PinnedAgentsSnapshot | null,
  registryError: null as string | null,
}));

vi.mock("@/platform/entities", () => ({
  SystemDefaultModelError: mocks.FakeSystemDefaultError,
  setSystemDefaultModel: mocks.setSystemDefaultModel,
  useProviderMetaField: (field: "default_id" | "default_model") => mocks.meta[field],
  useConfiguredModels: () => ({
    items: [{ id: "openai/gpt-5.4-mini" }, { id: "bdd-provider-b/claude-5-haiku" }],
    isLoading: false,
  }),
  useModelCatalogLoaded: () => true,
  useModelCatalogLoadError: (id: string) => (id === "providers" ? mocks.registryError : null),
  loadProviderRegistry: vi.fn().mockResolvedValue(undefined),
  loadModelCatalog: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../model/pinned-agents", () => ({
  loadPinnedAgents: mocks.loadPinnedAgents,
  usePinnedAgents: () => mocks.pinned,
}));

vi.mock("@/features/models/pickers", () => ({
  CatalogModelPicker: ({ onChange, disabled }: { onChange: (route: string) => void; disabled?: boolean }) => (
    <button type="button" disabled={disabled} onClick={() => onChange("bdd-provider-b/claude-5-haiku")}>
      Choose model
    </button>
  ),
}));

function renderPanel() {
  return render(
    <MemoryRouter>
      <SystemDefaultPanel />
    </MemoryRouter>,
  );
}

describe("SystemDefaultPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.meta = { default_id: "openai", default_model: "gpt-5.4-mini" };
    mocks.registryError = null;
    mocks.pinned = {
      id: "current",
      status: "ready",
      error: null,
      pinned: [
        { id: "a", title: "BDD Agent Aurora", model: "openai/gpt-5.4-mini" },
        { id: "c", title: "BDD Agent Cirrus", model: "bdd-provider-c/qwen-5-turbo" },
      ],
      follow_count: 2,
    };
    mocks.loadPinnedAgents.mockResolvedValue(undefined);
  });

  test("shows the live pair and no confirmation before any change", () => {
    renderPanel();

    expect(screen.getByTestId("system-default-live-pair")).toHaveTextContent("openai/gpt-5.4-mini");
    expect(screen.queryByTestId("system-default-confirmation")).not.toBeInTheDocument();
    expect(screen.getByTestId("system-default-confirm")).toBeDisabled();
  });

  test("confirms only after the runtime answered, with the requested pair and live", async () => {
    const user = userEvent.setup();
    let acknowledge: () => void = () => undefined;
    mocks.setSystemDefaultModel.mockReturnValue(new Promise<void>((resolve) => (acknowledge = resolve)));
    renderPanel();

    await user.click(screen.getByRole("button", { name: "Choose model" }));
    await user.click(screen.getByTestId("system-default-confirm"));

    expect(mocks.setSystemDefaultModel).toHaveBeenCalledWith("bdd-provider-b", "claude-5-haiku");
    expect(screen.queryByTestId("system-default-confirmation")).not.toBeInTheDocument();

    acknowledge();
    const confirmation = await screen.findByTestId("system-default-confirmation");
    expect(confirmation).toHaveTextContent("bdd-provider-b/claude-5-haiku");
    expect(confirmation).toHaveTextContent("live");
    expect(mocks.loadPinnedAgents).toHaveBeenCalledTimes(2);
  });

  test("shows the runtime's refusal and no confirmation", async () => {
    const user = userEvent.setup();
    mocks.setSystemDefaultModel.mockRejectedValue(
      new mocks.FakeSystemDefaultError("Model saved, system default not changed. Provider 'x' is disabled."),
    );
    renderPanel();

    await user.click(screen.getByRole("button", { name: "Choose model" }));
    await user.click(screen.getByTestId("system-default-confirm"));

    expect(await screen.findByTestId("system-default-error")).toHaveTextContent("system default not changed");
    expect(screen.queryByTestId("system-default-confirmation")).not.toBeInTheDocument();
    expect(screen.getByTestId("system-default-live-pair")).toHaveTextContent("openai/gpt-5.4-mini");
  });

  test("lists pinned agents with their models and how many follow the default", () => {
    renderPanel();

    const listed = screen.getAllByTestId("system-default-pinned-agent");
    expect(listed).toHaveLength(2);
    expect(listed[0]).toHaveTextContent("BDD Agent Aurora");
    expect(listed[1]).toHaveTextContent("bdd-provider-c/qwen-5-turbo");
    expect(screen.getByTestId("system-default-follow-count")).toHaveTextContent("2");
  });

  test("says pinned agents are unknown when the agent list failed, never that all follow", () => {
    mocks.pinned = { id: "current", status: "error", error: "500", pinned: [], follow_count: 0 };
    renderPanel();

    expect(screen.getByRole("alert")).toHaveTextContent("Pinned agents unknown");
    expect(screen.queryByTestId("system-default-follow-count")).not.toBeInTheDocument();
  });

  test("names the missing part when there is no usable default", () => {
    mocks.meta = { default_id: "openai", default_model: null };
    renderPanel();

    expect(screen.queryByTestId("system-default-live-pair")).not.toBeInTheDocument();
    expect(screen.getByText(/has no default model/)).toBeInTheDocument();
  });
});
