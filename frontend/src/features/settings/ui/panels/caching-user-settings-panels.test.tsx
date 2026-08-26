import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { PromptCachingPanel } from "./caching-user-settings-panels";

const mocks = vi.hoisted(() => ({
  values: {} as Record<string, unknown>,
  settings: {} as Record<string, unknown>,
  dirty: {} as Record<string, unknown>,
  loading: false,
  refreshing: false,
  saving: false,
  error: null as string | null,
  setSetting: vi.fn(),
  saveAll: vi.fn().mockResolvedValue(undefined),
  reload: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../../model/use-settings", () => ({
  useSettings: () => ({
    ...mocks,
    conflicts: {},
  }),
}));

vi.mock("../../model/use-onboarding", () => ({
  useOnboarding: () => ({ dismissed: true, dismiss: vi.fn() }),
}));

vi.mock("../../model/use-user-jwt-settings", () => ({
  useUserJwtSettings: () => ({
    settings: null,
    loading: false,
    saving: false,
    error: null,
    load: vi.fn(),
    save: vi.fn(),
  }),
}));

describe("PromptCachingPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.values = {};
    mocks.settings = {};
    mocks.dirty = {};
    mocks.loading = false;
    mocks.refreshing = false;
    mocks.saving = false;
    mocks.error = null;
  });

  test("blocks editing after an initial load failure and offers Retry", async () => {
    const user = userEvent.setup();
    mocks.error = "404";

    render(<PromptCachingPanel />);

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Prompt-caching settings are unavailable",
    );
    expect(screen.queryByRole("switch")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();

    await user.click(screen.getByRole("button", { name: "Retry" }));
    expect(mocks.reload).toHaveBeenCalledTimes(1);
  });

  test("renders only the authoritative value and exposes accessible dirty state", async () => {
    const user = userEvent.setup();
    mocks.values = { "prompt_caching.enabled": true };
    mocks.settings = {
      "prompt_caching.enabled": {
        key: "prompt_caching.enabled",
        data: false,
      },
    };
    mocks.dirty = { "prompt_caching.enabled": true };

    render(<PromptCachingPanel />);

    const toggle = screen.getByRole("switch", {
      name: "Enable Prompt Caching (Global Default)",
    });
    expect(toggle).toBeChecked();
    expect(screen.getByRole("status")).toHaveTextContent("Unsaved changes");
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();

    await user.click(toggle);
    expect(mocks.setSetting).toHaveBeenCalledWith(
      "prompt_caching.enabled",
      false,
    );
  });

  test("preserves editable server values when a refresh fails", () => {
    mocks.values = { "prompt_caching.enabled": true };
    mocks.settings = {
      "prompt_caching.enabled": {
        key: "prompt_caching.enabled",
        data: false,
      },
    };
    mocks.dirty = { "prompt_caching.enabled": true };
    mocks.error = "network unavailable";

    render(<PromptCachingPanel />);

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Can’t reach the server",
    );
    expect(screen.getByRole("switch")).toBeChecked();
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
  });
});
