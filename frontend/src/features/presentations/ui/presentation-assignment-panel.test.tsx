import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import * as catalogApi from "@/platform/entities/presentations/api/presentations-api";
import { presentationActions } from "@/platform/entities/presentations/domain";
import { registerPresentationEntities } from "@/platform/entities/presentations/registration";
import * as assignmentApi from "@/platform/entities/presentation-assignments/api";
import { presentationAssignmentId, type PresentationAssignmentTarget } from "@/platform/entities/presentation-assignments/contracts";
import { presentationAssignmentActions } from "@/platform/entities/presentation-assignments/domain";
import { registerPresentationAssignmentEntities } from "@/platform/entities/presentation-assignments/registration";
import { PresentationAssignmentPanel } from "./presentation-assignment-panel";

const owner = "assignment-label-test-owner";
const target: PresentationAssignmentTarget = { scope: "agent", agentId: "label-test-agent" };

beforeEach(async () => {
  registerPresentationEntities();
  registerPresentationAssignmentEntities();
  vi.spyOn(catalogApi, "fetchPresentations").mockResolvedValue({ owner_id: owner, presentations: [] });
  vi.spyOn(assignmentApi, "fetchAssignment");
  await presentationActions.reload();
});
afterEach(() => vi.restoreAllMocks());

describe("Presentation assignment's selected label", () => {
  test.each([
    ["inherit", false, "Inherit"],
    ["inherit", true, "Inherit, with exclusions"],
    ["auto", false, "Automatic"],
    ["all", false, "All allowed"],
    ["selected", false, "Selected templates"],
    ["none", false, "None"],
  ] as const)("shows the authored label for %s (exclusions: %s)", async (mode, excluded, label) => {
    const selection = { mode, ids: [], denied_ids: excluded ? ["unavailable-template"] : [] };
    vi.mocked(assignmentApi.fetchAssignment).mockResolvedValue({
      id: presentationAssignmentId(owner, target), owner_id: owner, target,
      policy: { presentations: selection }, selection,
    });
    await presentationAssignmentActions.reload(target);
    render(<PresentationAssignmentPanel target={target} />);
    const trigger = await screen.findByRole("combobox", { name: "Assignment mode" });
    expect(trigger.querySelector('[data-slot="select-value"]')?.textContent).toBe(label);
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  });
});
