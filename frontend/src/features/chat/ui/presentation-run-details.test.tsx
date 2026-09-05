import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { PersistedRunSnapshot } from "@/platform/pglite/run-event-repository";
import { graphStore, type PresentationObservation } from "@/platform/entities";
import { PRESENTATION_PROVENANCE_ENTITY } from "@/platform/entities/presentation-provenance/contracts";
import * as history from "@/platform/entities/presentation-provenance/api";
import { PresentationRunDetails } from "./presentation-run-details";

const RUN = "ui-provenance-run";
function observation(overrides: Partial<PresentationObservation> = {}): PresentationObservation {
  return {
    version: 1, requested_mode: "hybrid", effective_mode: "hybrid",
    admission_fallback_reason: null, fallback_reason: null, run_outcome: "running",
    eligible_templates: [{ presentation_id: "template-one", revision: 7 }],
    published_templates: [], surface_published: false, generation_failed: false,
    receipt_status: "available", client_display: "unconfirmed", ...overrides,
  };
}
function snapshot(value?: unknown, runId = RUN): PersistedRunSnapshot {
  return { run: null, events: value === undefined ? [] : [{
    runId, seq: 1, eventId: `${runId}-1`, wireSequence: 16, type: "CUSTOM", kind: "custom",
    at: "2026-09-05T00:00:00Z", payload: { runId, name: "uar.presentation.snapshot", value },
  }] };
}
function historyWith(value?: unknown) {
  return vi.spyOn(history, "subscribePresentationHistory").mockResolvedValue({
    initialSnapshot: snapshot(value), unsubscribe: vi.fn().mockResolvedValue(undefined),
  });
}
afterEach(() => vi.restoreAllMocks());

describe("Presentation run details", () => {
  test("marks its run-wide time scope and keeps loading contextual", async () => {
    historyWith();
    render(<PresentationRunDetails runId={RUN} />);
    expect(screen.getByRole("heading", { name: "Presentation" })).toBeVisible();
    expect(screen.getByText("Latest recorded details for this run.")).toBeVisible();
    expect(screen.getByRole("status")).toHaveTextContent("Loading Presentation details");
    expect(await screen.findByText("Presentation details were not recorded for this run.")).toBeVisible();
    expect(screen.queryByText("No template published.")).not.toBeInTheDocument();
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  test("unsupported records never claim empty or successful publication", async () => {
    historyWith({ ...observation(), version: 2 });
    render(<PresentationRunDetails runId={RUN} />);
    expect(await screen.findByText(/This client cannot read the recorded Presentation details/)).toBeVisible();
    expect(screen.queryByText("No template published.")).not.toBeInTheDocument();
    expect(screen.queryByText("Finished")).not.toBeInTheDocument();
  });

  test("local-history failure offers an explicit working retry", async () => {
    historyWith(observation()).mockRejectedValueOnce(new Error("database unavailable"));
    render(<PresentationRunDetails runId={RUN} />);
    fireEvent.click(await screen.findByRole("button", { name: "Retry details" }));
    expect(await screen.findByText("Running")).toBeVisible();
    expect(screen.queryByRole("button", { name: "Retry details" })).not.toBeInTheDocument();
  });

  test("separates permission, pending publication and client display", async () => {
    historyWith(observation());
    render(<PresentationRunDetails runId={RUN} />);
    expect(await screen.findByText("Running")).toBeVisible();
    expect(screen.getByText("Output permitted by the host; publication is recorded below.")).toBeVisible();
    expect(screen.getByText("No generated UI surface published yet.")).toBeVisible();
    expect(screen.getByText("Policy summaries do not count as generated UI surfaces.")).toBeVisible();
    expect(screen.getByText("No template published yet.")).toBeVisible();
    expect(screen.getByText("Publication does not confirm client display.")).toBeVisible();
  });

  test.each(["failed", "cancelled", "finished"] as const)("renders %s without presenting it as a successful surface", async (run_outcome) => {
    historyWith(observation({ run_outcome }));
    render(<PresentationRunDetails runId={RUN} />);
    expect(await screen.findByText(run_outcome[0].toUpperCase() + run_outcome.slice(1))).toBeVisible();
    expect(screen.getByText("No generated UI surface published.")).toBeVisible();
    expect(screen.getByText("No template published.")).toBeVisible();
    expect(screen.queryByText("No template published yet.")).not.toBeInTheDocument();
  });

  test("manual legacy surface publication is not a template receipt", async () => {
    historyWith(observation({ requested_mode: null, effective_mode: "legacy", run_outcome: "finished", surface_published: true }));
    render(<PresentationRunDetails runId={RUN} />);
    expect(await screen.findByText("Generated UI surface published.")).toBeVisible();
    expect(screen.getAllByText("Legacy (not negotiated)")).toHaveLength(2);
    expect(screen.getByText("No template published.")).toBeVisible();
  });

  test("keeps full frozen template identity and revision behind a native disclosure", async () => {
    const template_id = "long-template-identity-that-must-not-be-truncated-0123456789";
    historyWith(observation({ surface_published: true, published_templates: [{ template_id, revision: 17 }] }));
    render(<PresentationRunDetails runId={RUN} />);
    const summary = await screen.findByText("Published templates (1)");
    expect(summary.tagName).toBe("SUMMARY");
    expect(summary.parentElement?.tagName).toBe("DETAILS");
    expect(screen.getByText(template_id)).toHaveTextContent(template_id);
    expect(screen.getByText("17")).toBeInTheDocument();
    expect(screen.queryByText("No template published yet.")).not.toBeInTheDocument();
  });

  test("unavailable receipts do not masquerade as an empty list", async () => {
    historyWith(observation({ receipt_status: "unavailable", run_outcome: "finished" }));
    render(<PresentationRunDetails runId={RUN} />);
    expect(await screen.findByText("Template publication receipts are unavailable for this run.")).toBeVisible();
    expect(screen.queryByText("No template published.")).not.toBeInTheDocument();
  });

  test("renders a fallback reason independently of the admitted mode", async () => {
    historyWith(observation({ effective_mode: "text", fallback_reason: "parent_text_ceiling" }));
    render(<PresentationRunDetails runId={RUN} />);
    expect(await screen.findByText("Text fallback: the parent run permits text only.")).toBeVisible();
    expect(screen.getByText("Text only")).toBeVisible();
    expect(screen.getByText("Text and UI")).toBeVisible();
  });

  test("switching runs clears old details and ignores late callbacks", async () => {
    const callbacks = new Map<string, (value: PersistedRunSnapshot) => void>();
    vi.spyOn(history, "subscribePresentationHistory").mockImplementation(async (runId, callback) => {
      callbacks.set(runId, callback);
      return { initialSnapshot: snapshot(observation({ run_outcome: runId === RUN ? "finished" : "cancelled" }), runId), unsubscribe: vi.fn().mockResolvedValue(undefined) };
    });
    const view = render(<PresentationRunDetails runId={RUN} />);
    expect(await screen.findByText("Finished")).toBeVisible();
    view.rerender(<PresentationRunDetails runId="another-run" />);
    expect(await screen.findByText("Cancelled")).toBeVisible();
    act(() => callbacks.get(RUN)?.(snapshot(observation({ run_outcome: "failed" }))));
    expect(graphStore.getState().readEntity(PRESENTATION_PROVENANCE_ENTITY, RUN)).toMatchObject({
      status: "idle", observation: null,
    });
    await waitFor(() => expect(screen.queryByText("Failed")).not.toBeInTheDocument());
    expect(screen.getByText("Cancelled")).toBeVisible();
  });
});
