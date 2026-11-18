import { describe, expect, test } from "vitest";

import {
  attemptView,
  canCancelAttempt,
  classifyStepFailure,
  countLanes,
  formatAge,
  humanizeToken,
  isChargedInFull,
  isLiveCategory,
  isSettledStatus,
  needsClamp,
  outputText,
  parseReservation,
  pickWaitState,
  remainingAfter,
  reservationHints,
  reservationOverrun,
  sortLanes,
  splitThinking,
  stepState,
  stepsDone,
  unreportedChargeSummary,
} from "./run-model";

describe("attemptView", () => {
  test("maps every runtime status", () => {
    expect(attemptView("queued", true)).toBe("queued");
    expect(attemptView("running", true)).toBe("running");
    expect(attemptView("cancellation_requested", true)).toBe("cancelling");
    expect(attemptView("succeeded", false)).toBe("succeeded");
    expect(attemptView("failed", false)).toBe("failed");
    expect(attemptView("cancelled", false)).toBe("cancelled");
    expect(attemptView("uncertain", false)).toBe("uncertain");
  });

  test("reads a yielded attempt through its wait", () => {
    expect(attemptView("yielded", true)).toBe("waiting");
    expect(attemptView("yielded", false, { state: "waiting", taskStatus: "waiting" })).toBe("waiting");
    expect(attemptView("yielded", false, { state: "resumed", taskStatus: "running" })).toBe("handed_off");
  });

  test("reads an invalidated wait on a cancelled task as cancelled while waiting", () => {
    expect(attemptView("yielded", false, { state: "invalidated", taskStatus: "cancelled" })).toBe("cancelled_waiting");
  });

  test("reports an invalidated wait without a cancelled task as invalidated, not cancelled", () => {
    expect(attemptView("yielded", false, { state: "invalidated", taskStatus: "waiting" })).toBe("wait_invalidated");
    expect(attemptView("yielded", false, { state: "invalidated", taskStatus: null })).toBe("wait_invalidated");
  });

  test("does not assert a hand-off for a yielded attempt with no wait record", () => {
    expect(attemptView("yielded", false)).toBe("yielded");
    expect(attemptView("yielded", false, { state: null, taskStatus: "cancelled" })).toBe("yielded");
  });

  test("reports a status it does not know instead of guessing", () => {
    expect(attemptView("paused", false)).toBe("unknown");
  });
});

describe("pickWaitState", () => {
  const waits = [
    { attemptId: "a1", state: "resumed" },
    { attemptId: "a2", state: "invalidated" },
    { attemptId: "a3", state: "" },
    { attemptId: "a4", state: "invalidated" },
    { attemptId: "a4", state: "waiting" },
  ];

  test("finds the wait that belongs to the attempt", () => {
    expect(pickWaitState(waits, "a1")).toBe("resumed");
    expect(pickWaitState(waits, "a2")).toBe("invalidated");
  });

  test("prefers a live wait and ignores a record that has not loaded", () => {
    expect(pickWaitState(waits, "a4")).toBe("waiting");
    expect(pickWaitState(waits, "a3")).toBeNull();
  });

  test("is null when the attempt has no wait record", () => {
    expect(pickWaitState(waits, "a9")).toBeNull();
  });
});

describe("lane ordering", () => {
  test("puts failed, uncertain, waiting, cancelling, running, queued before finished work", () => {
    const sorted = sortLanes([
      { id: "a", category: "succeeded" },
      { id: "b", category: "running" },
      { id: "c", category: "failed" },
      { id: "d", category: "idle" },
      { id: "e", category: "waiting" },
      { id: "f", category: "uncertain" },
      { id: "g", category: "cancelling" },
      { id: "h", category: "queued" },
      { id: "i", category: "handed_off" },
      { id: "j", category: "cancelled" },
      { id: "k", category: "cancelled_waiting" },
      { id: "l", category: "wait_invalidated" },
    ]);
    expect(sorted.map((entry) => entry.id)).toEqual(["c", "f", "l", "e", "g", "b", "h", "a", "i", "j", "k", "d"]);
  });

  test("keeps roster order inside one category", () => {
    const sorted = sortLanes([
      { id: "x", category: "running" },
      { id: "y", category: "running" },
    ]);
    expect(sorted.map((entry) => entry.id)).toEqual(["x", "y"]);
  });

  test("counts only the categories present, in attention order", () => {
    expect(
      countLanes([
        { id: "a", category: "running" },
        { id: "b", category: "failed" },
        { id: "c", category: "running" },
      ]),
    ).toEqual([
      { category: "failed", count: 1 },
      { category: "running", count: 2 },
    ]);
  });

  test("only in-flight and waiting lanes carry a live attempt", () => {
    expect(isLiveCategory("running")).toBe(true);
    expect(isLiveCategory("waiting")).toBe(true);
    expect(isLiveCategory("handed_off")).toBe(false);
    expect(isLiveCategory("idle")).toBe(false);
  });
});

describe("canCancelAttempt", () => {
  test("allows queued and running", () => {
    expect(canCancelAttempt("queued", null)).toBe(true);
    expect(canCancelAttempt("running", null)).toBe(true);
  });

  test("allows a yielded attempt only while its wait is waiting or blocked", () => {
    expect(canCancelAttempt("yielded", "waiting")).toBe(true);
    expect(canCancelAttempt("yielded", "blocked")).toBe(true);
    expect(canCancelAttempt("yielded", "yield_requested")).toBe(false);
    expect(canCancelAttempt("yielded", "resumed")).toBe(false);
    expect(canCancelAttempt("yielded", null)).toBe(false);
  });

  test("offers nothing for cancelling, uncertain or settled attempts", () => {
    for (const status of ["cancellation_requested", "uncertain", "succeeded", "failed", "cancelled"]) {
      expect(canCancelAttempt(status, null)).toBe(false);
    }
  });
});

describe("output", () => {
  test("folds think blocks out of the answer", () => {
    const folded = splitThinking("<think>plan the answer</think>\n\nBlue light scatters more.");
    expect(folded.answer).toBe("Blue light scatters more.");
    expect(folded.thoughts).toEqual(["plan the answer"]);
  });

  test("folds an unterminated think block to the end", () => {
    const folded = splitThinking("Answer.<think>still thinking");
    expect(folded.answer).toBe("Answer.");
    expect(folded.thoughts).toEqual(["still thinking"]);
  });

  test("leaves text without think blocks alone", () => {
    expect(splitThinking("plain")).toEqual({ answer: "plain", thoughts: [] });
  });

  test("shows a string as text and anything else as formatted JSON", () => {
    expect(outputText("hello")).toBe("hello");
    expect(outputText({ a: 1 })).toBe('{\n  "a": 1\n}');
    expect(outputText(null)).toBe("");
  });

  test("clamps long or tall output only", () => {
    expect(needsClamp("short")).toBe(false);
    expect(needsClamp("x".repeat(240))).toBe(false);
    expect(needsClamp("x".repeat(241))).toBe(true);
    expect(needsClamp(Array.from({ length: 3 }, () => "line").join("\n"))).toBe(false);
    expect(needsClamp(Array.from({ length: 4 }, () => "line").join("\n"))).toBe(true);
  });
});

describe("humanizeToken", () => {
  test("turns an action token into words", () => {
    expect(humanizeToken("change-settings")).toBe("Change settings");
    expect(humanizeToken("rebind")).toBe("Rebind");
  });
});

describe("formatAge", () => {
  test("reads seconds, then minutes, then hours", () => {
    expect(formatAge(4_000)).toBe("4 s ago");
    expect(formatAge(180_000)).toBe("3 min ago");
    expect(formatAge(3 * 3_600_000)).toBe("3 h ago");
    expect(formatAge(-5_000)).toBe("0 s ago");
  });
});

describe("parseReservation", () => {
  test("converts currency units to microunits", () => {
    expect(parseReservation({ tokens: "128000", cost: "1.00", elapsedSeconds: "120" })).toEqual({
      ok: true,
      value: { tokens: 128000, costMicrounits: 1_000_000, elapsedSeconds: 120 },
    });
  });

  test("names the field that is wrong", () => {
    const tokens = parseReservation({ tokens: "0", cost: "1", elapsedSeconds: "120" });
    expect(tokens.ok === false && tokens.problem).toMatch(/Tokens/);
    const seconds = parseReservation({ tokens: "10", cost: "1", elapsedSeconds: "1.5" });
    expect(seconds.ok === false && seconds.problem).toMatch(/Elapsed/);
    const cost = parseReservation({ tokens: "10", cost: "-1", elapsedSeconds: "10" });
    expect(cost.ok === false && cost.problem).toMatch(/Cost/);
  });
});

describe("reservationOverrun", () => {
  const value = { tokens: 128000, costMicrounits: 1_000_000, elapsedSeconds: 120 };
  const money = (microunits: number) => `$${microunits / 1_000_000}`;

  test("is null when everything fits or is unknown", () => {
    expect(reservationOverrun(value, { tokens: null, costMicrounits: null, elapsedSeconds: null, slots: null }, money)).toBeNull();
    expect(reservationOverrun(value, { tokens: 128000, costMicrounits: 1_000_000, elapsedSeconds: 120, slots: 3 }, money)).toBeNull();
  });

  test("names the first dimension that does not fit", () => {
    expect(reservationOverrun(value, { tokens: 100, costMicrounits: 5_000_000, elapsedSeconds: 500, slots: 1 }, money)).toMatch(/token/);
    expect(reservationOverrun(value, { tokens: 200000, costMicrounits: 500_000, elapsedSeconds: 500, slots: 1 }, money)).toMatch(/\$0.5/);
  });
});

describe("classifyStepFailure", () => {
  test("separates a runtime refusal, an unknown outcome and a step that was never sent", () => {
    expect(classifyStepFailure({ kind: "conflict", uncertain: false })).toBe("refused");
    expect(classifyStepFailure({ kind: "unavailable", uncertain: true })).toBe("unknown_outcome");
    expect(classifyStepFailure({ kind: "network", uncertain: false })).toBe("unknown_outcome");
    expect(classifyStepFailure({ kind: "precondition", uncertain: false })).toBe("not_sent");
  });
});

describe("routing steps", () => {
  test("counts steps reached from the stored progress", () => {
    expect(stepsDone("pending", null)).toBe(0);
    expect(stepsDone("created", null)).toBe(1);
    expect(stepsDone("claimed", null)).toBe(2);
    expect(stepsDone("admitted", null)).toBe(3);
    expect(stepsDone("failed", "claim")).toBe(1);
    expect(stepsDone("failed", "admit")).toBe(2);
  });

  test("marks the step in flight, the one that failed and the ones after it", () => {
    expect(stepState(0, 1, "failed", null)).toBe("done");
    expect(stepState(1, 1, "failed", null)).toBe("failed");
    expect(stepState(1, 1, "created", "claim")).toBe("running");
    expect(stepState(1, 1, "created", null)).toBe("next");
    expect(stepState(2, 1, "failed", null)).toBe("later");
  });
});

describe("charged in full", () => {
  test("a settled attempt without a usage report keeps its whole reservation", () => {
    expect(isChargedInFull("cancelled", null)).toBe(true);
    expect(isChargedInFull("succeeded", undefined)).toBe(true);
    expect(isChargedInFull("failed", null)).toBe(true);
    expect(isChargedInFull("uncertain", null)).toBe(true);
  });

  test("is not claimed while the attempt is live or when usage was reported", () => {
    expect(isChargedInFull("running", null)).toBe(false);
    expect(isChargedInFull("queued", null)).toBe(false);
    expect(isChargedInFull("yielded", null)).toBe(false);
    expect(isChargedInFull(undefined, null)).toBe(false);
    expect(isChargedInFull("cancelled", { tokens: 1, costMicrounits: 1, elapsedSeconds: 1 })).toBe(false);
  });

  test("names the settled statuses", () => {
    expect(isSettledStatus("cancelled")).toBe(true);
    expect(isSettledStatus("cancellation_requested")).toBe(false);
  });

  test("summarises the charge in one line", () => {
    expect(unreportedChargeSummary("Cancelled", "$1.00")).toBe("Cancelled · usage unknown · $1.00 charged (full reservation)");
    expect(unreportedChargeSummary("Failed", null)).toBe("Failed · usage unknown · full reservation charged");
  });
});

describe("reservation hints and consequence", () => {
  test("reads each field on its own", () => {
    expect(reservationHints({ tokens: "128000", cost: "1", elapsedSeconds: "120" })).toEqual({
      tokens: 128000,
      costMicrounits: 1_000_000,
      elapsedSeconds: 120,
    });
    expect(reservationHints({ tokens: "abc", cost: "", elapsedSeconds: "1.5" })).toEqual({
      tokens: null,
      costMicrounits: null,
      elapsedSeconds: null,
    });
    expect(reservationHints({ tokens: "5", cost: "-1", elapsedSeconds: "5" }).costMicrounits).toBeNull();
  });

  test("takes the reservation and one task slot off what is left", () => {
    const value = { tokens: 128000, costMicrounits: 1_000_000, elapsedSeconds: 120 };
    expect(remainingAfter(value, { tokens: 1_000_000, costMicrounits: 3_000_000, elapsedSeconds: 600, slots: 6 })).toEqual({
      tokens: 872_000,
      costMicrounits: 2_000_000,
      elapsedSeconds: 480,
      slots: 5,
    });
  });

  test("keeps unknown figures unknown and never goes below zero", () => {
    const value = { tokens: 10, costMicrounits: 10, elapsedSeconds: 10 };
    expect(remainingAfter(value, { tokens: null, costMicrounits: 5, elapsedSeconds: null, slots: 0 })).toEqual({
      tokens: null,
      costMicrounits: 0,
      elapsedSeconds: null,
      slots: 0,
    });
  });
});
