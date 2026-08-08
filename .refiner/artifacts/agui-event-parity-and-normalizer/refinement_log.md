# Refinement Log: agui-event-parity-and-normalizer

## 2026-08-07 — Iteration 1

- Schema: artifact manifest and constraints conform to the artifact-refiner schemas.
- Files: the manifest verification receipt exists and is non-empty.
- Constraints: all four C-06 blocking constraints pass on deterministic evidence.
- Completeness: 9/9 OpenSpec tasks complete; Wave 2 boundary passes at 36 test files / 170 tests plus production build.
- External condition: the live integration target did not execute because the shared `Cli` fixture lacks `strict_config`; no passing claim is made for that test.
- Result: converged.

## 2026-08-07 — Iteration 2

- Adversarial review's sole critical was disproved: `run_updated` already maps
  to `RuntimeRun`; a direct entity test now proves `phase_timings` persistence.
- Restored legacy-only message/reasoning delta fallbacks while official frames
  continue through the typed single-pass projection.
- Removed fabricated replay tool names by buffering argument deltas until the
  real tool name is available, then emitting `START -> ARGS -> END`.
- Frontend gates pass at 3 focused files / 22 tests and 36 full files / 171
  tests; production build passes. Rust server-full check and focused tool
  projection test pass.
- Result: converged; resubmit isolated review with corrected evidence.

## 2026-08-07 — Iteration 3

- Round-two adversarial review identified a real collision between unbounded
  buffered argument ordinals and hard-coded tool lifecycle ordinals.
- Renumbered every flushed projection from emitted order and separated
  per-frame identity from per-source-event ordering, with an eight-chunk
  regression proving unique ids and a monotonic next event.
- Restored cursor-scoped history reads for legacy SSE attaches; full retained
  history remains limited to `agui_spec` snapshot reconstruction.
- The focused Rust regression, server-full check, C-06 rustfmt check, strict
  OpenSpec validation, and diff-integrity check pass.
- The third isolated review passes at 0 critical / 2 warnings / 0 suggestions
  with verified-distinct judge routing and anti-sycophancy score 0.0.
- Result: converged; independently confirmed with no unresolved C-06 critical
  finding.
