import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { SettingsPage } from "./settings-page";

const mocks = vi.hoisted(() => ({
  types: vi.fn(),
}));

vi.mock("../model/use-settings-types-meta", () => ({
  useSettingsTypesMeta: mocks.types,
}));

vi.mock("../model/use-settings", () => ({
  useSettings: (namespace: string) => ({
    values: {},
    settings: {},
    dirty: {},
    conflicts: {},
    loading: false,
    saving: false,
    error: null,
    setSetting: vi.fn(),
    saveAll: vi.fn().mockResolvedValue(undefined),
    reload: vi.fn().mockResolvedValue(undefined),
    namespace,
  }),
}));

vi.mock("../model/use-onboarding", () => ({
  useOnboarding: () => ({ dismissed: true, dismiss: vi.fn() }),
}));

vi.mock("../model/use-user-jwt-settings", () => ({
  useUserJwtSettings: () => ({
    settings: null,
    loading: false,
    saving: false,
    error: null,
    load: vi.fn().mockResolvedValue(undefined),
    save: vi.fn().mockResolvedValue(undefined),
  }),
}));

const allTypes = [
  "llm",
  "provider",
  "vision",
  "context_management",
  "context_strategy",
  "rag",
  "knowledge_bases",
  "memory",
  "models",
  "file_processing",
  "unstructured",
  "mistral_ocr",
  "kreuzberg",
  "resilience",
  "server",
  "persistence",
  "sandbox",
  "intent_classifier",
  "security",
  "governance",
  "sycophancy",
  "agent_config",
  "skill_config",
  "native_tools",
  "skill_evolution",
  "acp",
  "llm_failover",
].map((key) => ({ key, schema: {} }));

describe("SettingsPage decomposition", () => {
  beforeEach(() => {
    mocks.types.mockReturnValue(allTypes);
  });

  test("keeps provider as the default custom panel", () => {
    render(<SettingsPage />);

    expect(
      screen.getByRole("heading", { name: "LLM Providers" }),
    ).toBeInTheDocument();
  });

  test("navigates to the prompt-caching custom panel", async () => {
    const user = userEvent.setup();
    render(<SettingsPage />);

    await user.click(screen.getByRole("button", { name: "Prompt Caching" }));

    expect(
      screen.getByRole("heading", { name: "Prompt Caching" }),
    ).toBeInTheDocument();
  });

  test("keeps metadata-backed unavailable items disabled", () => {
    mocks.types.mockReturnValue([{ key: "provider", schema: {} }]);
    render(<SettingsPage />);

    expect(screen.getByRole("button", { name: "Vision" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Prompt Caching" }),
    ).toBeEnabled();
  });

  test("keeps schema-driven namespace panels in the registry", async () => {
    const user = userEvent.setup();
    render(<SettingsPage />);

    await user.click(
      screen.getByRole("button", { name: "LLM Configuration" }),
    );

    expect(
      screen.getByRole("heading", { name: "LLM Configuration" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No editable settings are registered for this namespace."),
    ).toBeInTheDocument();
  });
});
