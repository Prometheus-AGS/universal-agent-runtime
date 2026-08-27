import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, test, vi } from "vitest";
import type { GovernanceRuntimeStatus } from "../../api/settings-api";
import { GovernancePanel } from "./governance-settings-panels";

const mocks = vi.hoisted(() => ({
  values: {
    "governance.enabled": true,
    "governance.default_mode": "permit_all",
    "governance.allowed_actions": [],
    "governance.policy_reload_enabled": true,
  } as Record<string, unknown>,
  dirty: {} as Record<string, unknown>,
  loading: false,
  saving: false,
  error: null as string | null,
  setSetting: vi.fn(),
  saveAll: vi.fn(),
  reload: vi.fn().mockResolvedValue(undefined),
  status: null as GovernanceRuntimeStatus | null,
  statusLoading: false,
  statusError: null as string | null,
  refreshStatus: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../../model/use-settings", () => ({
  useSettings: () => ({
    values: mocks.values,
    settings: {},
    dirty: mocks.dirty,
    conflicts: {},
    loading: mocks.loading,
    refreshing: false,
    saving: mocks.saving,
    error: mocks.error,
    setSetting: mocks.setSetting,
    saveAll: mocks.saveAll,
    reload: mocks.reload,
  }),
}));

vi.mock("../../model/use-governance-status", () => ({
  useGovernanceStatus: () => ({
    status: mocks.status,
    loading: mocks.statusLoading,
    error: mocks.statusError,
    refresh: mocks.refreshStatus,
  }),
}));

const ELIGIBLE_ON: GovernanceRuntimeStatus = {
  boot_instance_id: "boot-a",
  revision: 4,
  phase: "on",
  effective_state: "on",
  effective_enabled: true,
  may_disable: true,
  mutation_available: true,
  configured_host: "localhost",
  bound_addresses: ["127.0.0.1:1906"],
  jwt_required: false,
  reasons: [],
};

describe("GovernancePanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.values = {
      "governance.enabled": true,
      "governance.default_mode": "permit_all",
      "governance.allowed_actions": [],
      "governance.policy_reload_enabled": true,
    };
    mocks.dirty = {};
    mocks.loading = false;
    mocks.saving = false;
    mocks.error = null;
    mocks.status = ELIGIBLE_ON;
    mocks.statusLoading = false;
    mocks.statusError = null;
    mocks.saveAll.mockResolvedValue(null);
  });

  test("renders Unknown without making an editable boolean claim", () => {
    mocks.status = null;
    mocks.statusError = "Governance status unavailable: 404";

    render(<GovernancePanel />);

    expect(screen.getByText("Unknown")).toBeVisible();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Runtime governance status could not be verified",
    );
    expect(
      screen.queryByRole("switch", { name: "Enforce tool governance" }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Unavailable")).toBeVisible();
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  test("invalidates a cached boolean claim when status refresh fails", () => {
    mocks.statusError = "Governance request timed out after 10 seconds";

    render(<GovernancePanel />);

    expect(screen.getByText("Unknown")).toBeVisible();
    expect(
      screen.queryByRole("switch", { name: "Enforce tool governance" }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  test("keeps status visible but locks fallback policy values when settings fail", () => {
    mocks.error = "500";
    mocks.dirty = { "governance.default_mode": "deny_all" };

    render(<GovernancePanel />);

    expect(screen.getByText("On")).toBeVisible();
    expect(
      screen.getByRole("switch", { name: "Enforce tool governance" }),
    ).toHaveAttribute("aria-disabled", "true");
    expect(
      screen.getByRole("combobox", { name: "Default Authorization Mode" }),
    ).toBeDisabled();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Governance settings could not be loaded or saved",
    );
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  test("keeps a Required master focusable and guards Space and Enter", async () => {
    const user = userEvent.setup();
    mocks.status = {
      ...ELIGIBLE_ON,
      effective_state: "required",
      may_disable: false,
      reasons: ["jwt_required", "configured_host_not_allowed"],
    };

    render(<GovernancePanel />);

    const master = screen.getByRole("switch", {
      name: "Enforce tool governance",
    });
    expect(master).toBeChecked();
    expect(master).toHaveAttribute("aria-disabled", "true");
    expect(master).toHaveAccessibleDescription(
      expect.stringContaining("JWT authentication is active"),
    );
    master.focus();
    await user.keyboard(" {Enter}");
    expect(master).toHaveFocus();
    expect(mocks.setSetting).not.toHaveBeenCalled();
  });

  test("Required overrides an existing Off draft and keeps policy controls enabled", () => {
    mocks.status = {
      ...ELIGIBLE_ON,
      effective_state: "required",
      may_disable: false,
      reasons: ["jwt_required"],
    };
    mocks.values = { ...mocks.values, "governance.enabled": false };
    mocks.dirty = { "governance.enabled": false };

    render(<GovernancePanel />);

    expect(
      screen.getByRole("switch", { name: "Enforce tool governance" }),
    ).toBeChecked();
    expect(
      screen.getByRole("combobox", { name: "Default Authorization Mode" }),
    ).toBeEnabled();
    expect(screen.getByText(/restart UAR before turning governance Off/)).toBeVisible();
  });

  test("keeps authoritative On visible while persistence makes mutation unavailable", () => {
    mocks.status = {
      ...ELIGIBLE_ON,
      mutation_available: false,
      reasons: ["persistence_unavailable"],
    };

    render(<GovernancePanel />);

    expect(screen.getByText("On")).toBeVisible();
    expect(
      screen.getByRole("switch", { name: "Enforce tool governance" }),
    ).toHaveAttribute("aria-disabled", "true");
    expect(
      screen.getByRole("combobox", { name: "Default Authorization Mode" }),
    ).toBeDisabled();
    expect(screen.getByText(/settings are unavailable/)).toBeVisible();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeEnabled();
  });

  test("separates an On draft from the authoritative Off warning", () => {
    mocks.status = {
      ...ELIGIBLE_ON,
      phase: "off",
      effective_state: "off",
      effective_enabled: false,
    };
    mocks.values = { ...mocks.values, "governance.enabled": true };
    mocks.dirty = { "governance.enabled": true };

    render(<GovernancePanel />);

    expect(screen.getByText("Off")).toBeVisible();
    expect(
      screen.getByText(/All available tools can run without Cedar policies/),
    ).toBeVisible();
    expect(
      screen.getByText(/After Save, policy checks and approval prompts resume/),
    ).toBeVisible();
    expect(
      screen.getByRole("switch", { name: "Enforce tool governance" }),
    ).toBeChecked();
  });

  test("reports partial application without claiming complete success", async () => {
    const user = userEvent.setup();
    mocks.dirty = {
      "governance.default_mode": "deny_all",
      "governance.enabled": true,
    };
    mocks.saveAll.mockResolvedValue({
      status: "partial",
      results: [
        { key: "governance.default_mode", status: "updated" },
        { key: "governance.enabled", status: "dependency_failed" },
      ],
      applied_status: { boot_instance_id: "boot-a", revision: 4 },
      governance_status: ELIGIBLE_ON,
      governance_outcome: "partial",
      observed_governance_status: ELIGIBLE_ON,
      retained_draft_keys: ["governance.enabled"],
    });

    render(<GovernancePanel />);
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Applied: Default authorization mode. Still drafts: Enforce tool governance.",
    );
    expect(screen.queryByText("Governance settings saved and confirmed.")).not.toBeInTheDocument();
  });

  test("announces the master transition while Saving", () => {
    mocks.values = { ...mocks.values, "governance.enabled": false };
    mocks.dirty = { "governance.enabled": false };
    const { rerender } = render(<GovernancePanel />);

    mocks.saving = true;
    rerender(<GovernancePanel />);

    expect(screen.getByText("Turning tool governance off…")).toBeInTheDocument();
  });

  test("reports a newer authoritative revision as changed elsewhere", async () => {
    const user = userEvent.setup();
    mocks.dirty = { "governance.enabled": true };
    const newerOff = {
      ...ELIGIBLE_ON,
      revision: 5,
      phase: "off" as const,
      effective_state: "off" as const,
      effective_enabled: false,
    };
    mocks.saveAll.mockResolvedValue({
      status: "updated",
      results: [{ key: "governance.enabled", status: "updated" }],
      applied_status: { boot_instance_id: "boot-a", revision: 4 },
      governance_status: ELIGIBLE_ON,
      governance_outcome: "changed_elsewhere",
      observed_governance_status: newerOff,
    });

    render(<GovernancePanel />);
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Tool governance is now Off.",
    );
    expect(screen.getByText(
      "Settings saved, then changed elsewhere. Tool governance is now Off.",
    )).toBeVisible();
  });

  test("announces only effective-state changes across remote revisions", () => {
    const { rerender } = render(<GovernancePanel />);

    mocks.status = { ...ELIGIBLE_ON, revision: 5 };
    rerender(<GovernancePanel />);
    expect(screen.getByRole("status")).toHaveTextContent("");

    mocks.status = {
      ...ELIGIBLE_ON,
      revision: 6,
      phase: "off",
      effective_state: "off",
      effective_enabled: false,
    };
    rerender(<GovernancePanel />);
    expect(screen.getByRole("status")).toHaveTextContent(
      "Tool governance is now Off.",
    );
  });

  test("names the authoritative state after a confirmed master save", async () => {
    const user = userEvent.setup();
    const confirmedOff = {
      ...ELIGIBLE_ON,
      revision: 5,
      phase: "off" as const,
      effective_state: "off" as const,
      effective_enabled: false,
    };
    mocks.values = { ...mocks.values, "governance.enabled": false };
    mocks.dirty = { "governance.enabled": false };
    mocks.saveAll.mockResolvedValue({
      status: "updated",
      results: [{ key: "governance.enabled", status: "updated" }],
      applied_status: { boot_instance_id: "boot-a", revision: 5 },
      governance_status: confirmedOff,
      governance_outcome: "confirmed",
      observed_governance_status: confirmedOff,
    });

    render(<GovernancePanel />);
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Tool governance is now Off.",
    );
    expect(
      screen.getByText("Governance settings saved and confirmed."),
    ).toBeVisible();
  });

  test("does not call an idempotent confirmed master save a transition", async () => {
    const user = userEvent.setup();
    mocks.dirty = { "governance.enabled": true };
    mocks.saveAll.mockResolvedValue({
      status: "updated",
      results: [{ key: "governance.enabled", status: "updated" }],
      applied_status: { boot_instance_id: "boot-a", revision: 5 },
      governance_status: { ...ELIGIBLE_ON, revision: 5 },
      governance_outcome: "confirmed",
      observed_governance_status: { ...ELIGIBLE_ON, revision: 5 },
    });

    render(<GovernancePanel />);
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Governance settings saved and confirmed.",
    );
    expect(screen.getByRole("status")).not.toHaveTextContent(
      /Tool governance is now/,
    );
  });

  test("uses an assertive error for an authoritative rejection", async () => {
    const user = userEvent.setup();
    mocks.dirty = { "governance.enabled": false };
    mocks.saveAll.mockResolvedValue({
      status: "partial",
      results: [
        { key: "governance.enabled", status: "validation_rejected" },
      ],
      applied_status: { boot_instance_id: "boot-a", revision: 5 },
      governance_status: {
        ...ELIGIBLE_ON,
        revision: 5,
        effective_state: "required",
        may_disable: false,
        reasons: ["jwt_required"],
      },
      governance_outcome: "rejected",
      retained_draft_keys: [],
    });

    render(<GovernancePanel />);
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "No governance settings were applied. No submitted drafts remain.",
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Tool governance is now Required.",
    );
    expect(screen.queryByText(/saved and confirmed/)).not.toBeInTheDocument();
  });

  test("reports an unverifiable restart as Unknown without success", async () => {
    const user = userEvent.setup();
    mocks.dirty = { "governance.enabled": true };
    mocks.saveAll.mockResolvedValue({
      status: "updated",
      results: [{ key: "governance.enabled", status: "updated" }],
      applied_status: { boot_instance_id: "boot-a", revision: 4 },
      governance_status: ELIGIBLE_ON,
      governance_outcome: "unknown",
    });

    render(<GovernancePanel />);
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The previous runtime stopped before the save outcome could be verified",
    );
    expect(screen.queryByText(/saved and confirmed/)).not.toBeInTheDocument();
  });

  test("announces a policy-only save without claiming a master transition", () => {
    mocks.dirty = { "governance.default_mode": "deny_all" };
    const { rerender } = render(<GovernancePanel />);

    mocks.saving = true;
    rerender(<GovernancePanel />);

    expect(screen.getByText("Saving governance settings…")).toBeInTheDocument();
    expect(screen.queryByText(/Turning tool governance/)).not.toBeInTheDocument();
  });
});
