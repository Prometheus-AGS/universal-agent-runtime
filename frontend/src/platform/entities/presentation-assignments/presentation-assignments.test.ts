import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { PRESENTATION_ADMISSION_ID, presentationActions, presentationListKey } from "../presentations/domain";
import { PRESENTATION_CATALOG_ENTITY, PRESENTATION_ENTITY } from "../presentations/contracts";
import * as api from "./api";
import { assignmentDraft, presentationAssignmentActions } from "./domain";
import { presentationAssignmentId, type PresentationAssignment, type PresentationAssignmentTarget, type PresentationSelection } from "./contracts";

const OWNER = "assignment-owner";
const TARGET: PresentationAssignmentTarget = { scope: "agent", agentId: "agent-one" };
const selection = (ids: string[] = []): PresentationSelection => ({ mode: "selected", ids, denied_ids: [] });
function saved(intent: PresentationSelection = selection(["allowed"])): PresentationAssignment {
  return { id: presentationAssignmentId(OWNER, TARGET), owner_id: OWNER, target: TARGET,
    policy: { memory_enabled: false, presentations: intent }, selection: intent };
}
function draft() { return assignmentDraft(graphStore.getState(), TARGET); }

describe("Presentation assignment domain", () => {
  beforeEach(() => {
    graphStore.getState().replaceEntity(PRESENTATION_CATALOG_ENTITY, PRESENTATION_ADMISSION_ID, {
      id: PRESENTATION_ADMISSION_ID, owner_id: OWNER, generation: 1, status: "ready", error: null, editor_open: false,
    });
    vi.spyOn(presentationActions, "ensureLoaded").mockResolvedValue(undefined);
    vi.spyOn(api, "fetchAssignment").mockResolvedValue(saved());
    vi.spyOn(api, "saveAssignment").mockImplementation(async (_baseline, intent) => saved(intent));
  });
  afterEach(() => vi.restoreAllMocks());

  test("switching modes retains remembered IDs without activating them or clearing exclusions", async () => {
    vi.mocked(api.fetchAssignment).mockResolvedValue(saved({ ...selection(["allowed"]), denied_ids: ["excluded"] }));
    await presentationAssignmentActions.ensureLoaded(TARGET);
    presentationAssignmentActions.setMode(TARGET, "none");
    expect(draft()).toMatchObject({ selection: { mode: "none", ids: [], denied_ids: ["excluded"] }, retained_ids: ["allowed"] });
    presentationAssignmentActions.setMode(TARGET, "inherit");
    expect(draft()?.selection).toEqual({ mode: "inherit", ids: [], denied_ids: ["excluded"] });
    presentationAssignmentActions.setMode(TARGET, "selected");
    expect(draft()?.selection).toEqual({ mode: "selected", ids: ["allowed"], denied_ids: ["excluded"] });
  });

  test("reset explicitly restores inheritance and clears remembered selections and exclusions", async () => {
    vi.mocked(api.fetchAssignment).mockResolvedValue(saved({ ...selection(["allowed"]), denied_ids: ["excluded"] }));
    await presentationAssignmentActions.ensureLoaded(TARGET);
    presentationAssignmentActions.reset(TARGET);
    expect(draft()).toMatchObject({ dirty: true, retained_ids: [], selection: { mode: "inherit", ids: [], denied_ids: [] } });
  });

  test("cannot add foreign, disabled or absent templates but permits removing an unavailable selection", async () => {
    await presentationAssignmentActions.ensureLoaded(TARGET);
    // These rows model already-ingested catalog metadata; no content is rendered here.
    for (const [id, owner_id, enabled] of [["foreign", "another-owner", true], ["disabled", OWNER, false], ["not-listed", OWNER, true]] as const) {
      graphStore.getState().replaceEntity(PRESENTATION_ENTITY, id, { id, owner_id, content: { enabled } });
    }
    graphStore.getState().setListResult(presentationListKey(OWNER), ["foreign", "disabled"], { total: 2 });
    for (const id of ["foreign", "disabled", "not-listed", "missing"]) presentationAssignmentActions.toggle(TARGET, id);
    expect(draft()?.selection.ids).toEqual(["allowed"]);
    presentationAssignmentActions.toggle(TARGET, "allowed");
    expect(draft()?.selection.ids).toEqual([]);
  });

  test("catalog re-admission invalidates an older assignment even for the same owner", async () => {
    await presentationAssignmentActions.ensureLoaded(TARGET);
    graphStore.getState().replaceEntity(PRESENTATION_CATALOG_ENTITY, PRESENTATION_ADMISSION_ID, {
      id: PRESENTATION_ADMISSION_ID, owner_id: OWNER, generation: 2, status: "ready",
    });
    expect(draft()).toBeNull();
    presentationAssignmentActions.setMode(TARGET, "all");
    await expect(presentationAssignmentActions.save(TARGET)).resolves.toBe(false);
    expect(api.saveAssignment).not.toHaveBeenCalled();
  });

  test("detects a changed saved assignment before patching and retains the local draft", async () => {
    await presentationAssignmentActions.ensureLoaded(TARGET);
    presentationAssignmentActions.setMode(TARGET, "none");
    vi.mocked(api.fetchAssignment).mockResolvedValue(saved(selection(["changed-remotely"])));
    await expect(presentationAssignmentActions.save(TARGET)).resolves.toBe(false);
    expect(api.saveAssignment).not.toHaveBeenCalled();
    expect(draft()).toMatchObject({ conflict: true, dirty: true, selection: { mode: "none" } });
    presentationAssignmentActions.setMode(TARGET, "all");
    expect(draft()?.selection.mode).toBe("none");
  });

  test("sends saved intent as baseline and edited intent as the mutation", async () => {
    await presentationAssignmentActions.ensureLoaded(TARGET);
    presentationAssignmentActions.setMode(TARGET, "none");
    await expect(presentationAssignmentActions.save(TARGET)).resolves.toBe(true);
    expect(api.saveAssignment).toHaveBeenCalledWith(
      expect.objectContaining({ policy: { memory_enabled: false, presentations: selection(["allowed"]) }, selection: selection(["allowed"]) }),
      { mode: "none", ids: [], denied_ids: [] },
    );
    expect(draft()).toMatchObject({ dirty: false, uncertain: false, selection: { mode: "none" } });
  });

  test("an unknown write blocks another save until explicit saved-state reconciliation", async () => {
    await presentationAssignmentActions.ensureLoaded(TARGET);
    presentationAssignmentActions.setMode(TARGET, "none");
    vi.mocked(api.saveAssignment).mockRejectedValueOnce(new api.AssignmentApiError("unknown write", 0, true));
    await expect(presentationAssignmentActions.save(TARGET)).resolves.toBe(false);
    expect(draft()).toMatchObject({ uncertain: true, dirty: true });
    await expect(presentationAssignmentActions.save(TARGET)).resolves.toBe(false);
    expect(api.saveAssignment).toHaveBeenCalledTimes(1);
    await presentationAssignmentActions.checkSaved(TARGET);
    expect(draft()).toMatchObject({ uncertain: false, dirty: true, selection: { mode: "none" } });
    await expect(presentationAssignmentActions.save(TARGET)).resolves.toBe(true);
    expect(api.saveAssignment).toHaveBeenCalledTimes(2);
  });
});
