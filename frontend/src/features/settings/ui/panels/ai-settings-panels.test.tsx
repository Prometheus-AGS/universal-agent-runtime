import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { ProviderPanel } from "./ai-settings-panels";

const mocks = vi.hoisted(() => ({
  setSetting: vi.fn(),
  saveAll: vi.fn().mockResolvedValue(undefined),
  reload: vi.fn().mockResolvedValue(undefined),
  dirty: {} as Record<string, unknown>,
  loading: false,
  refreshing: false,
  saving: false,
  error: null as string | null,
}));

vi.mock("../../model/use-settings", () => ({
  useSettings: () => ({
    values: {
      "provider.example": {
        display_name: "Example AI",
        base_url: "https://api.example.test/v1",
        api_key: "********",
        protocol: "chat",
        enabled: true,
        default_model: "model-alpha",
        models: [
          { id: "model-alpha", display_name: "Model Alpha", enabled: true },
          { id: "model-beta", display_name: "", enabled: true },
          { id: "model-alpha", display_name: "Duplicate Alpha" },
          { id: "model-disabled", display_name: "Disabled", enabled: false },
        ],
      },
      "provider.empty": {
        display_name: "Empty AI",
        base_url: "https://empty.example.test/v1",
        api_key: "********",
        protocol: "chat",
        enabled: true,
        default_model: "retired-model",
        models: [{ id: "disabled-model", enabled: false }],
      },
      "provider.stale": {
        display_name: "Stale AI",
        base_url: "https://stale.example.test/v1",
        api_key: "********",
        protocol: "chat",
        enabled: true,
        default_model: "retired-model",
        models: [{ id: "current-model", display_name: "Current Model" }],
      },
      "provider.seven": {
        display_name: "Seven AI",
        base_url: "https://seven.example.test/v1",
        api_key: "********",
        protocol: "chat",
        enabled: true,
        default_model: "seven-model-1",
        models: Array.from({ length: 7 }, (_, index) => ({
          id: `seven-model-${index + 1}`,
          display_name: `Seven Model ${index + 1}`,
        })),
      },
      "provider.large": {
        display_name: "Large AI",
        base_url: "https://large.example.test/v1",
        api_key: "********",
        protocol: "chat",
        enabled: true,
        default_model: "large-model-1",
        models: [
          { id: "large-model-1", display_name: "Large Model 1" },
          { id: "large-model-2", display_name: "Large Model 2" },
          { id: "large-model-3", display_name: "Large Model 3" },
          { id: "large-model-4", display_name: "Large Model 4" },
          { id: "large-model-5", display_name: "Large Model 5" },
          { id: "large-model-6", display_name: "Large Model 6" },
          { id: "raw-special/v2", display_name: "Shared Label" },
          { id: "raw-special-v3", display_name: "Shared Label" },
        ],
      },
    },
    settings: {
      "provider.example": {
        id: "setting-provider-example",
        settings_type_id: "settings-type-provider",
        key: "provider.example",
        name: "Example AI",
        data: {},
        created_at: "2026-08-25T00:00:00Z",
        meta: { source: "Api", is_drift: false },
      },
      "provider.empty": {
        id: "setting-provider-empty",
        settings_type_id: "settings-type-provider",
        key: "provider.empty",
        name: "Empty AI",
        data: {},
        created_at: "2026-08-25T00:00:00Z",
        meta: { source: "Api", is_drift: false },
      },
      "provider.stale": {
        id: "setting-provider-stale",
        settings_type_id: "settings-type-provider",
        key: "provider.stale",
        name: "Stale AI",
        data: {},
        created_at: "2026-08-25T00:00:00Z",
        meta: { source: "Api", is_drift: false },
      },
      "provider.seven": {
        id: "setting-provider-seven",
        settings_type_id: "settings-type-provider",
        key: "provider.seven",
        name: "Seven AI",
        data: {},
        created_at: "2026-08-25T00:00:00Z",
        meta: { source: "Api", is_drift: false },
      },
      "provider.large": {
        id: "setting-provider-large",
        settings_type_id: "settings-type-provider",
        key: "provider.large",
        name: "Large AI",
        data: {},
        created_at: "2026-08-25T00:00:00Z",
        meta: { source: "Api", is_drift: false },
      },
    },
    dirty: mocks.dirty,
    conflicts: {},
    loading: mocks.loading,
    refreshing: mocks.refreshing,
    saving: mocks.saving,
    error: mocks.error,
    setSetting: mocks.setSetting,
    saveAll: mocks.saveAll,
    reload: mocks.reload,
  }),
}));

describe("ProviderPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.dirty = {};
    mocks.loading = false;
    mocks.refreshing = false;
    mocks.saving = false;
    mocks.error = null;
  });

  test("selects from every enabled provider-owned model", async () => {
    const user = userEvent.setup();
    render(<ProviderPanel />);

    const trigger = screen.getByRole("combobox", {
      name: "Example AI default model",
    });
    expect(trigger).toHaveTextContent("Model Alpha");

    await user.click(trigger);

    expect(
      await screen.findByRole("option", { name: "Model Alpha" }),
    ).toBeVisible();
    expect(screen.getByRole("option", { name: "model-beta" })).toBeVisible();
    expect(
      screen.queryByRole("option", { name: "Disabled" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("option", { name: "Duplicate Alpha" }),
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole("option", { name: "model-beta" }));

    expect(mocks.setSetting).toHaveBeenCalledWith(
      "provider.example",
      expect.objectContaining({ default_model: "model-beta" }),
    );

    const emptyTrigger = screen.getByRole("combobox", {
      name: "Empty AI default model",
    });
    expect(emptyTrigger).toBeDisabled();
    expect(emptyTrigger).not.toHaveAttribute("aria-invalid");
    expect(emptyTrigger).toHaveTextContent("No enabled models");
    expect(
      screen.getByText("No enabled models are available for this provider."),
    ).toBeVisible();

    const staleTrigger = screen.getByRole("combobox", {
      name: "Stale AI default model",
    });
    expect(staleTrigger).not.toBeDisabled();
    expect(staleTrigger).toHaveAttribute("aria-invalid", "true");
    expect(staleTrigger).toHaveTextContent("Select a model");
    expect(
      screen.getByText("Current model is unavailable. Choose an enabled model."),
    ).toBeVisible();
  });

  test("keeps seven models simple and makes eight or more searchable", async () => {
    const user = userEvent.setup();
    render(<ProviderPanel />);

    const shortTrigger = screen.getByRole("combobox", {
      name: "Seven AI default model",
    });
    await user.click(shortTrigger);
    expect(
      screen.queryByRole("combobox", { name: "Search Seven AI models" }),
    ).not.toBeInTheDocument();
    await user.keyboard("{Escape}");

    const largeTrigger = screen.getByRole("combobox", {
      name: "Large AI default model",
    });
    await user.click(largeTrigger);

    const search = await screen.findByRole("combobox", {
      name: "Search Large AI models",
    });
    expect(search).toHaveFocus();

    await user.type(search, "  RAW-SPECIAL/V2  ");
    expect(
      await screen.findByRole("option", {
        name: /Shared Label raw-special\/v2/i,
      }),
    ).toBeVisible();
    expect(
      screen.queryByRole("option", { name: /Large Model 1/i }),
    ).not.toBeInTheDocument();

    await user.clear(search);
    await user.type(search, "not-a-real-model");
    expect(screen.getByText("No matching models.")).toBeVisible();
  });

  test("selects a searched provider model with the keyboard", async () => {
    const user = userEvent.setup();
    render(<ProviderPanel />);

    const trigger = screen.getByRole("combobox", {
      name: "Large AI default model",
    });
    await user.click(trigger);
    const search = await screen.findByRole("combobox", {
      name: "Search Large AI models",
    });

    await user.type(search, "raw-special-v3");
    await user.keyboard("{ArrowDown}{Enter}");

    expect(mocks.setSetting).toHaveBeenCalledTimes(1);
    expect(mocks.setSetting).toHaveBeenCalledWith(
      "provider.large",
      expect.objectContaining({ default_model: "raw-special-v3" }),
    );
    expect(
      screen.queryByRole("combobox", { name: "Search Large AI models" }),
    ).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  test("associates provider controls and recovery guidance", () => {
    render(<ProviderPanel />);

    const example = screen.getByRole("group", { name: "Example AI" });
    expect(within(example).getByLabelText("Base URL")).toBeVisible();
    expect(within(example).getByLabelText("Protocol")).toBeVisible();
    expect(within(example).getByLabelText("API Key")).toBeVisible();
    expect(within(example).getByLabelText("Default Model")).toBeVisible();
    expect(
      within(example).getByRole("switch", {
        name: "Enable Example AI provider",
      }),
    ).toBeVisible();
    expect(
      within(example).getByRole("button", {
        name: "Show Example AI API key",
      }),
    ).toBeVisible();

    const stale = screen.getByRole("group", { name: "Stale AI" });
    expect(within(stale).getByLabelText("Default Model")).toHaveAccessibleDescription(
      "Current model is unavailable. Choose an enabled model.",
    );
  });

  test("protects dirty provider drafts and reports modified state", () => {
    mocks.dirty = { "provider.example": { default_model: "model-beta" } };
    render(<ProviderPanel />);

    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeDisabled();
    expect(screen.getByText("Unsaved changes")).toBeVisible();
    expect(screen.getByText("Save changes before refreshing.")).toBeVisible();
    expect(
      within(screen.getByRole("group", { name: "Example AI" })).getByText(
        "Modified",
      ),
    ).toBeVisible();
    expect(
      within(screen.getByRole("group", { name: "Empty AI" })).queryByText(
        "Modified",
      ),
    ).not.toBeInTheDocument();

    const event = new Event("beforeunload", { cancelable: true });
    window.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });

  test("keeps clean actions honest and exposes live feedback", async () => {
    const user = userEvent.setup();
    mocks.error = "credential rejected";
    render(<ProviderPanel />);

    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeEnabled();
    expect(screen.getByRole("alert")).toHaveTextContent("credential rejected");

    const cleanUnload = new Event("beforeunload", { cancelable: true });
    window.dispatchEvent(cleanUnload);
    expect(cleanUnload.defaultPrevented).toBe(false);

    await user.click(screen.getByRole("button", { name: "Refresh" }));
    expect(mocks.reload).toHaveBeenCalledTimes(1);
  });

  test("announces a successful provider save", async () => {
    const user = userEvent.setup();
    mocks.dirty = { "provider.example": { default_model: "model-beta" } };
    render(<ProviderPanel />);

    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(mocks.saveAll).toHaveBeenCalledTimes(1);
    const saved = await screen.findByText("Settings saved");
    expect(saved.closest('[role="status"]')).toHaveAttribute(
      "aria-live",
      "polite",
    );
  });

  test("does not announce success after a rejected provider save", async () => {
    const user = userEvent.setup();
    mocks.dirty = { "provider.example": { default_model: "model-beta" } };
    mocks.error = "credential rejected";
    mocks.saveAll.mockRejectedValueOnce(new Error("credential rejected"));
    render(<ProviderPanel />);

    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(mocks.saveAll).toHaveBeenCalledTimes(1);
    expect(screen.queryByText("Settings saved")).not.toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("credential rejected");
    expect(screen.getByText("Unsaved changes")).toBeVisible();
  });

  test("uses status semantics and responsive provider field structure", () => {
    mocks.loading = true;
    mocks.refreshing = true;
    render(<ProviderPanel />);

    expect(screen.getByRole("status")).toHaveTextContent("Loading…");
    expect(screen.getByRole("button", { name: "Refresh" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Refresh" })).toHaveAccessibleDescription(
      "Refreshing settings…",
    );
    const example = screen.getByRole("group", { name: "Example AI" });
    expect(example.querySelector(".grid")).toHaveClass(
      "grid-cols-1",
      "lg:grid-cols-2",
    );
  });

  test("disables provider actions while a save is in flight", () => {
    mocks.dirty = { "provider.example": { default_model: "model-beta" } };
    mocks.saving = true;
    render(<ProviderPanel />);

    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Refresh" })).toHaveAccessibleDescription(
      "Save changes before refreshing.",
    );
  });

  test("keeps dirty drafts saveable during a background refresh", () => {
    mocks.dirty = { "provider.example": { default_model: "model-beta" } };
    mocks.refreshing = true;
    render(<ProviderPanel />);

    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeDisabled();
  });
});
