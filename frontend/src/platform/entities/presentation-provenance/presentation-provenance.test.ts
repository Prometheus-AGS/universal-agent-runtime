import { afterEach, describe, expect, test, vi } from "vitest";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import type { PersistedRunEvent, PersistedRunSnapshot, PersistedRunSnapshotSubscription } from "@/platform/pglite/run-event-repository";
import * as api from "./api";
import { decodePresentationProvenance, PRESENTATION_PROVENANCE_ENTITY, type PresentationObservation, type PresentationProvenance } from "./contracts";
import { PRESENTATION_PROVENANCE_ADMISSION, presentationProvenanceActions } from "./domain";
import { registerPresentationProvenanceEntities } from "./registration";

const RUN = "presentation-test-run";
function observation(overrides: Partial<PresentationObservation> = {}): PresentationObservation {
  return {
    version: 1, requested_mode: "hybrid", effective_mode: "hybrid",
    admission_fallback_reason: null, fallback_reason: null, run_outcome: "running",
    eligible_templates: [{ presentation_id: "template-one", revision: 7 }],
    published_templates: [], surface_published: false, generation_failed: false,
    receipt_status: "available", client_display: "unconfirmed", ...overrides,
  };
}

function event(sequence: number, value: unknown, overrides: Partial<PersistedRunEvent> = {}): PersistedRunEvent {
  return {
    runId: RUN, seq: sequence, eventId: `event-${sequence}`, wireSequence: sequence,
    type: "STATE_DELTA", kind: "state", at: "2026-09-05T00:00:00Z",
    payload: { runId: RUN, delta: [{ op: "add", path: "/presentation", value }] }, ...overrides,
  };
}
function snapshot(...events: PersistedRunEvent[]): PersistedRunSnapshot { return { run: null, events }; }

describe("Presentation provenance wire projection", () => {
  test.each(["STATE_DELTA", "STATE_SNAPSHOT", "CUSTOM"])("reads the full host record from %s", (type) => {
    const value = observation({ run_outcome: "finished", surface_published: true,
      published_templates: [{ template_id: "template-one", revision: 7 }] });
    const payload = type === "STATE_SNAPSHOT" ? { snapshot: { presentation: value } }
      : type === "CUSTOM" ? { name: "uar.presentation.snapshot", value }
        : { delta: [{ op: "add", path: "/presentation", value }] };
    expect(decodePresentationProvenance(RUN, snapshot(event(10, value, { type, payload })))).toMatchObject({
      status: "ready", source_sequence: 10, observation: value,
    });
  });

  test("wire order wins over late local arrival and duplicate replay", () => {
    const old = event(10, observation(), { seq: 9 });
    const latest = event(20, observation({ run_outcome: "cancelled" }), { seq: 2 });
    expect(decodePresentationProvenance(RUN, snapshot(latest, old, { ...latest, seq: 10 }))).toMatchObject({
      source_event_id: latest.eventId, observation: { run_outcome: "cancelled" },
    });
  });

  test.each([
    { version: 2 },
    { published_templates: [{ template_id: "template-one", revision: Number.MAX_SAFE_INTEGER + 1 }] },
    { client_display: "displayed" },
    { effective_mode: "unknown" },
    { unexpected_extension: true },
  ])("never substitutes an older valid record for unsupported evidence %j", (invalid) => {
    expect(decodePresentationProvenance(RUN, snapshot(event(1, observation()), event(2, { ...observation(), ...invalid })))).toMatchObject({
      status: "unsupported", observation: null, source_sequence: 2,
    });
  });

  test("missing history and explicit removal stay unknown, not empty publication", () => {
    expect(decodePresentationProvenance(RUN, snapshot())).toMatchObject({ status: "missing", observation: null });
    const removal = event(2, undefined, { payload: { delta: [{ op: "remove", path: "/presentation" }] } });
    expect(decodePresentationProvenance(RUN, snapshot(event(1, observation()), removal))).toMatchObject({ status: "missing", observation: null });
  });

  test("rejects other run identities and invalid ordering metadata", () => {
    expect(decodePresentationProvenance(RUN, snapshot(
      event(1, observation(), { runId: "other-run" }),
      event(2, observation(), { payload: { runId: "other-run", delta: [{ op: "add", path: "/presentation", value: observation() }] } }),
      event(Number.NaN, observation()),
    ))).toMatchObject({ status: "missing", observation: null });
  });

  test("unrelated deltas do not erase independently replayed provenance", () => {
    const receipt = event(4, undefined, { type: "CUSTOM", payload: { name: "uar.presentation.snapshot", value: observation() } });
    const unrelated = event(5, undefined, { payload: { delta: [{ op: "add", path: "/other", value: 2 }] } });
    expect(decodePresentationProvenance(RUN, snapshot(receipt, unrelated))).toMatchObject({ status: "ready", source_sequence: 4 });
  });

  test("root state replacement loses provenance until the host restores it", () => {
    const root = event(2, undefined, { payload: { delta: [{ op: "replace", path: "", value: { other: 1 } }] } });
    expect(decodePresentationProvenance(RUN, snapshot(event(1, observation()), root))).toMatchObject({ status: "missing" });
    expect(decodePresentationProvenance(RUN, snapshot(root, event(3, observation())))).toMatchObject({ status: "ready", source_sequence: 3 });
  });

  test.each(["failed", "cancelled", "finished"] as const)("retains %s distinctly from publication and client display", (run_outcome) => {
    const result = decodePresentationProvenance(RUN, snapshot(event(1, observation({ run_outcome }))));
    expect(result.observation).toMatchObject({ run_outcome, surface_published: false, published_templates: [], client_display: "unconfirmed" });
  });
});

describe("Presentation provenance subscription domain", () => {
  const releases: Array<() => void> = [];
  function subscribe() { const release = presentationProvenanceActions.subscribe(RUN); releases.push(release); return release; }
  function read() { return graphStore.getState().readEntity<PresentationProvenance>(PRESENTATION_PROVENANCE_ENTITY, RUN); }
  afterEach(() => {
    releases.splice(0).forEach((release) => release());
    vi.restoreAllMocks();
  });

  test("clears old hydrated authority before reading local history", async () => {
    registerPresentationProvenanceEntities();
    graphStore.getState().replaceEntity(PRESENTATION_PROVENANCE_ENTITY, RUN, {
      id: RUN, admission_id: "previous-runtime", status: "ready", observation: observation({ surface_published: true }),
    });
    vi.spyOn(api, "subscribePresentationHistory").mockResolvedValue({ initialSnapshot: snapshot(), unsubscribe: vi.fn().mockResolvedValue(undefined) });
    subscribe();
    expect(read()).toMatchObject({ status: "loading", observation: null, admission_id: PRESENTATION_PROVENANCE_ADMISSION });
    await vi.waitFor(() => expect(read()).toMatchObject({ status: "missing", observation: null }));
  });

  test("live evidence arriving before initial resolution is not overwritten", async () => {
    vi.spyOn(api, "subscribePresentationHistory").mockImplementation(async (_runId, callback) => {
      callback(snapshot(event(2, observation({ run_outcome: "finished" }))));
      return { initialSnapshot: snapshot(event(1, observation())), unsubscribe: vi.fn().mockResolvedValue(undefined) };
    });
    subscribe();
    await vi.waitFor(() => expect(read()).toMatchObject({ source_sequence: 2, observation: { run_outcome: "finished" } }));
  });

  test("shares one subscription and releases it only with the final inspector", async () => {
    const unsubscribe = vi.fn().mockResolvedValue(undefined);
    const transport = vi.spyOn(api, "subscribePresentationHistory").mockResolvedValue({ initialSnapshot: snapshot(event(1, observation())), unsubscribe });
    const first = subscribe();
    const second = subscribe();
    await vi.waitFor(() => expect(read()?.status).toBe("ready"));
    expect(transport).toHaveBeenCalledTimes(1);
    first(); releases.splice(releases.indexOf(first), 1);
    expect(unsubscribe).not.toHaveBeenCalled();
    second(); releases.splice(releases.indexOf(second), 1);
    expect(unsubscribe).toHaveBeenCalledTimes(1);
    expect(read()?.status).toBe("idle");
  });

  test("retry invalidates old callbacks and preserves stable receipt identity", async () => {
    const callbacks: Array<(value: PersistedRunSnapshot) => void> = [];
    vi.spyOn(api, "subscribePresentationHistory").mockImplementation(async (_runId, callback) => {
      callbacks.push(callback);
      return { initialSnapshot: snapshot(event(1, observation())), unsubscribe: vi.fn().mockResolvedValue(undefined) };
    });
    subscribe();
    await vi.waitFor(() => expect(read()?.status).toBe("ready"));
    presentationProvenanceActions.retry(RUN);
    await vi.waitFor(() => expect(callbacks).toHaveLength(2));
    const published_templates = [{ template_id: "template-one", revision: 7 }];
    callbacks[1](snapshot(event(3, observation({ published_templates, surface_published: true }))));
    const retained = read()?.observation?.published_templates;
    callbacks[1](snapshot(event(4, observation({ published_templates, surface_published: true, run_outcome: "finished" }))));
    expect(read()?.observation?.published_templates).toBe(retained);
    callbacks[0](snapshot(event(99, observation({ run_outcome: "failed" }))));
    expect(read()).toMatchObject({ source_sequence: 4, observation: { run_outcome: "finished" } });
  });

  test("unmount before subscription setup resolves releases the pending handle", async () => {
    let resolve: ((value: PersistedRunSnapshotSubscription) => void) | undefined;
    const unsubscribe = vi.fn().mockResolvedValue(undefined);
    vi.spyOn(api, "subscribePresentationHistory").mockImplementation(() => new Promise((done) => { resolve = done; }));
    const release = subscribe();
    await vi.waitFor(() => expect(resolve).toBeDefined());
    release(); releases.splice(releases.indexOf(release), 1);
    resolve?.({ initialSnapshot: snapshot(event(1, observation())), unsubscribe });
    await vi.waitFor(() => expect(unsubscribe).toHaveBeenCalledTimes(1));
    expect(read()).toMatchObject({ status: "idle", observation: null });
  });

  test("a local-history failure remains recoverable through explicit retry", async () => {
    vi.spyOn(api, "subscribePresentationHistory").mockRejectedValueOnce(new Error("database unavailable"))
      .mockResolvedValueOnce({ initialSnapshot: snapshot(event(1, observation())), unsubscribe: vi.fn().mockResolvedValue(undefined) });
    subscribe();
    await vi.waitFor(() => expect(read()?.status).toBe("error"));
    presentationProvenanceActions.retry(RUN);
    await vi.waitFor(() => expect(read()?.status).toBe("ready"));
  });
});
