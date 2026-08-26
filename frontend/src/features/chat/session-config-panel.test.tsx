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
  error: null as string | null,
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
  };
  return {
    agentSessionDraftId: (sessionId: string, editorId: string) =>
      `${sessionId}:${editorId}`,
    useAgentSessionDraftActions: () => actions,
    useAgentSessionDraftError: () => mocks.error,
    useAgentSessionDraftField: (_draftId: string, field: string) => {
      if (field === "prompt_caching_enabled") return null;
      return null;
    },
    useAgentSessionDraftStatus: () => "idle",
    useSessionPromptCaching: () => mocks.effective,
  };
});

vi.mock("@/features/models/model-selector", () => ({
  ModelSelector: ({ disabled }: { disabled?: boolean }) => (
    <button type="button" disabled={disabled}>
      Select model override
    </button>
  ),
}));

describe("SessionConfigPanel prompt caching", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.loadAndOpen.mockResolvedValue("draft");
    mocks.error = null;
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
