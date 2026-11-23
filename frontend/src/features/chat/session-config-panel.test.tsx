import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { SessionConfigPanel } from "./session-config-panel";

const mocks = vi.hoisted(() => ({
  setField: vi.fn(),
  loadAndOpen: vi.fn().mockResolvedValue("draft"),
  open: vi.fn(),
  markError: vi.fn(),
  cancel: vi.fn(),
  save: vi.fn().mockResolvedValue(true),
  loadEffectiveModel: vi.fn().mockResolvedValue(undefined),
  clearEffectiveModel: vi.fn(),
  error: null as string | null,
  draftModel: null as string | null,
  savedModel: null as string | null,
  effectiveModel: {
    stored: true as boolean | undefined,
    model: "openai/gpt-5.4-mini" as string | null,
    origin: "system_default" as string | null,
    unavailable_reason: null as string | null,
    inherited_model: "openai/gpt-5.4-mini" as string | null,
    inherited_origin: "system_default" as string | null,
    inherited_unavailable_reason: null as string | null,
  },
  effective: {
    id: "session-one",
    session_id: "session-one",
    enabled: true,
    source: "user" as const,
    session_override: null,
    user_override: true,
    global_default: false,
  } as {
    id: string;
    session_id: string;
    enabled: boolean;
    source: "request" | "session" | "user" | "global";
    session_override: boolean | null;
    user_override: boolean | null;
    global_default: boolean;
  } | null,
}));

vi.mock("@/platform/entities", () => {
  const actions = {
    setField: mocks.setField,
    loadAndOpen: mocks.loadAndOpen,
    open: mocks.open,
    markError: mocks.markError,
    cancel: mocks.cancel,
    save: mocks.save,
    loadEffectiveModel: mocks.loadEffectiveModel,
    clearEffectiveModel: mocks.clearEffectiveModel,
  };
  return {
    agentSessionDraftId: (sessionId: string, editorId: string) =>
      `${sessionId}:${editorId}`,
    useAgentSessionDraftActions: () => actions,
    useAgentSessionDraftError: () => mocks.error,
    useAgentSessionDraftField: (_draftId: string, field: string) => {
      if (field === "model") return mocks.draftModel;
      return null;
    },
    useAgentSession: () => (mocks.savedModel ? { model: mocks.savedModel } : null),
    useSessionEffectiveModelField: (_sessionId: string, field: string) =>
      (mocks.effectiveModel as Record<string, unknown>)[field],
    useAgentSessionDraftStatus: () => "idle",
    useAgentSessionDraftUncertain: () => false,
    useSessionPromptCaching: () => mocks.effective,
  };
});

vi.mock("@/features/models/pickers", () => ({
  ModelPicker: ({
    disabled,
    onChange,
  }: {
    disabled?: boolean;
    onChange: (value: string) => void;
  }) => (
    <>
      <button type="button" disabled={disabled} onClick={() => onChange("bdd-provider-c/qwen-5-turbo")}>
        Select model override
      </button>
      <button type="button" disabled={disabled} onClick={() => onChange("")}>
        Clear picker
      </button>
    </>
  ),
}));

// This suite isolates prompt caching; Presentation controls have their own domain path.
vi.mock("@/features/presentations", () => ({
  SessionPresentationSelection: () => <section aria-label="Presentations" />,
}));

describe("SessionConfigPanel prompt caching", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.loadAndOpen.mockResolvedValue("draft");
    mocks.error = null;
    mocks.draftModel = null;
    mocks.savedModel = null;
    mocks.loadEffectiveModel.mockResolvedValue(undefined);
    mocks.effectiveModel = {
      stored: true,
      model: "openai/gpt-5.4-mini",
      origin: "system_default",
      unavailable_reason: null,
      inherited_model: "openai/gpt-5.4-mini",
      inherited_origin: "system_default",
      inherited_unavailable_reason: null,
    };
    mocks.effective = {
      id: "session-one",
      session_id: "session-one",
      enabled: true,
      source: "user",
      session_override: null,
      user_override: true,
      global_default: false,
    };
  });

  test("shows the authoritative effective and inherited source", () => {
    render(
      <SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />,
    );

    expect(
      screen.getByRole("combobox", { name: "Prompt Caching" }),
    ).toHaveTextContent("Inherit");
    expect(screen.getByRole("status")).toHaveTextContent(
      "Effective now: On from user override. Inherited value: On from user override.",
    );
  });

  test("persists an explicit Off session override through the draft action", async () => {
    const user = userEvent.setup();
    render(
      <SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />,
    );

    const promptCaching = screen.getByRole("combobox", {
      name: "Prompt Caching",
    });
    await vi.waitFor(() => expect(promptCaching).toBeEnabled());
    await user.click(promptCaching);
    await user.click(screen.getByRole("option", { name: "Off" }));

    expect(mocks.setField).toHaveBeenCalledWith(
      expect.stringContaining("session-one:"),
      "prompt_caching_enabled",
      false,
    );
  });

  test("blocks the override and offers Retry when effective state is unavailable", async () => {
    const user = userEvent.setup();
    mocks.effective = null;

    render(
      <SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Effective prompt-caching status is unavailable",
    );
    expect(
      screen.queryByRole("combobox", { name: "Prompt Caching" }),
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Retry" }));
    await vi.waitFor(() => expect(mocks.loadAndOpen).toHaveBeenCalledTimes(2));
  });

  test("does not present a stale effective entity after the current load fails", async () => {
    mocks.loadAndOpen.mockRejectedValueOnce(
      new Error("effective endpoint unavailable"),
    );

    render(
      <SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />,
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Effective prompt-caching status is unavailable",
    );
    expect(
      screen.queryByRole("combobox", { name: "Prompt Caching" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Select model override" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("combobox", { name: "Tool Approval" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Save Configuration" }),
    ).toBeDisabled();
  });

  test("retrying a failed refresh does not cancel the existing draft", async () => {
    const user = userEvent.setup();
    mocks.loadAndOpen.mockRejectedValueOnce(new Error("refresh failed"));

    render(
      <SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />,
    );

    await user.click(await screen.findByRole("button", { name: "Retry" }));
    await vi.waitFor(() => expect(mocks.loadAndOpen).toHaveBeenCalledTimes(2));
    expect(mocks.cancel).not.toHaveBeenCalled();
  });

  test("announces draft load errors", () => {
    mocks.error = "Session configuration failed to load";

    render(
      <SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Session configuration failed to load",
    );
  });
});

describe("SessionConfigPanel model view", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.loadAndOpen.mockResolvedValue("draft");
    mocks.loadEffectiveModel.mockResolvedValue(undefined);
    mocks.save.mockResolvedValue(true);
    mocks.error = null;
    mocks.draftModel = "bdd-provider-c/qwen-5-turbo";
    mocks.savedModel = "bdd-provider-c/qwen-5-turbo";
    mocks.effective = {
      id: "session-one",
      session_id: "session-one",
      enabled: true,
      source: "user",
      session_override: null,
      user_override: true,
      global_default: false,
    };
    mocks.effectiveModel = {
      stored: true,
      model: "bdd-provider-c/qwen-5-turbo",
      origin: "conversation",
      unavailable_reason: null,
      inherited_model: "bdd-provider-b/claude-5-haiku",
      inherited_origin: "system_default",
      inherited_unavailable_reason: null,
    };
  });

  test("shows the requested model, the effective model and where it comes from", async () => {
    render(<SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />);

    expect(await screen.findByTestId("session-config-requested")).toHaveTextContent(
      "bdd-provider-c/qwen-5-turbo",
    );
    expect(screen.getByTestId("session-config-effective-model")).toHaveTextContent(
      "bdd-provider-c/qwen-5-turbo",
    );
    expect(screen.getByTestId("session-config-effective-origin")).toHaveTextContent(
      "this conversation",
    );
    expect(mocks.clearEffectiveModel).toHaveBeenCalledWith("session-one");
    expect(mocks.loadEffectiveModel).toHaveBeenCalledWith("session-one", expect.any(AbortSignal));
  });

  test("names the model a cleared override would inherit and changes nothing on cancel", async () => {
    const user = userEvent.setup();
    render(<SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />);

    await user.click(await screen.findByTestId("session-config-clear"));
    const preview = screen.getByTestId("session-config-clear-preview");
    expect(preview).toHaveTextContent("bdd-provider-b/claude-5-haiku");
    expect(preview).toHaveTextContent("from system default");

    await user.click(screen.getByTestId("session-config-clear-cancel"));
    expect(screen.queryByTestId("session-config-clear-preview")).not.toBeInTheDocument();
    expect(mocks.setField).not.toHaveBeenCalled();
    expect(mocks.save).not.toHaveBeenCalled();
  });

  test("the picker's own clear opens the same preview instead of clearing silently", async () => {
    const user = userEvent.setup();
    render(<SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />);

    await user.click(await screen.findByRole("button", { name: "Clear picker" }));

    expect(screen.getByTestId("session-config-clear-preview")).toBeInTheDocument();
    expect(mocks.setField).not.toHaveBeenCalled();
  });

  test("confirming clears the override through the draft and saves it", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    render(<SessionConfigPanel threadId="session-one" open onOpenChange={onOpenChange} />);

    await user.click(await screen.findByTestId("session-config-clear"));
    await user.click(screen.getByTestId("session-config-clear-confirm"));

    expect(mocks.setField).toHaveBeenCalledWith(expect.stringContaining("session-one:"), "model", null);
    await vi.waitFor(() => expect(mocks.save).toHaveBeenCalledTimes(1));
    await vi.waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false));
  });

  test("refuses the clear and shows the reason when no model could answer", async () => {
    const user = userEvent.setup();
    mocks.effectiveModel = {
      ...mocks.effectiveModel,
      inherited_model: null,
      inherited_origin: null,
      inherited_unavailable_reason: "Provider 'bdd-provider-b' is disabled",
    };
    render(<SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />);

    await user.click(await screen.findByTestId("session-config-clear"));

    expect(screen.getByTestId("session-config-clear-preview")).toHaveTextContent(
      "Provider 'bdd-provider-b' is disabled",
    );
    expect(screen.getByTestId("session-config-clear-confirm")).toBeDisabled();
  });

  test("keeps the previous effective model while a refused save shows its reason", async () => {
    mocks.error = "Not saved. This model is not offered: Provider 'bdd-provider-b' does not offer model 'claude-5-retired'";
    render(<SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("not offered");
    expect(screen.getByTestId("session-config-effective-model")).toHaveTextContent(
      "bdd-provider-c/qwen-5-turbo",
    );
  });

  test("offers Retry in its own block when the effective model cannot be read", async () => {
    const user = userEvent.setup();
    mocks.loadEffectiveModel.mockRejectedValueOnce(new Error("effective-config failed"));
    render(<SessionConfigPanel threadId="session-one" open onOpenChange={vi.fn()} />);

    const block = await screen.findByTestId("session-config-effective");
    expect(block).toHaveTextContent("Could not check which model answers");
    await user.click(screen.getByRole("button", { name: "Retry" }));
    await vi.waitFor(() => expect(mocks.loadEffectiveModel).toHaveBeenCalledTimes(2));
  });
});
