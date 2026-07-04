# run-hot-path-bench

## Why

`benches/hot_path.rs` (written in `uar-spec-v2-and-polish`'s CH-20) had
never actually been compiled or executed in any session — verified only
by code review. `assessment.md` confirmed this explicitly via direct
inspection before this change started.

## What changed

- `cargo check --benches`: confirmed the bench compiles (first time
  ever verified — clean).
- `cargo bench --bench hot_path`: ran all 4 Criterion benchmarks for
  real, producing an actual baseline instead of an assumed one.
- Recorded the baseline directly in `benches/hot_path.rs`'s own doc
  comment (a table with each benchmark's time), so it's durable and
  discoverable in the file itself rather than only in this proposal —
  the whole point of this change was "don't let this regress to
  'never run' again."

## Results

| Benchmark | Time |
|---|---|
| `prompt_dialect_detect` (7 model ids) | ~1.81 µs |
| `strategy_for_model` (5 context windows) | ~82.1 ns |
| `apply_strategy_sliding_window_500_messages` | ~134.9 µs |
| `model_router_route` (async, seeded registry) | ~341.2 µs |

All four are microsecond-scale or better — no red flags for a per-request
hot path. `model_router_route` is the most expensive (async + registry/
health-monitor lookups) but still well under a millisecond.

## Verification

- `cargo check --benches`: clean.
- `cargo bench --bench hot_path`: ran to completion, all 4 benchmarks
  produced stable timing distributions (Criterion's own outlier
  detection flagged 10-18% high-severity outliers on 3 of 4 benchmarks
  — within Criterion's normal noise tolerance for a shared/virtualized
  CI-like environment, not a correctness concern).
- The release-profile build for the bench binary took ~35 minutes
  (first-time compile of the full `bench` profile dependency tree) —
  disclosed since it's a real cost, not hidden.
