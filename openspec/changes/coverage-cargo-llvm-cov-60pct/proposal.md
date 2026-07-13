## Why

UAR's `.grcovrc` is committed but no coverage measurement runs in
CI. Coverage is currently a black box: a regression in new code
ships unobserved. The 2026-07-13 release-readiness assessment
flagged this as **a known, fixable structural gap** — not a
defect, but missing infrastructure. The operator's 2026-07-13
analysis selected **`cargo-llvm-cov`** (the recommended Rust
coverage tool) with a **60% threshold on day one** (Q4 decision),
adjustable based on observed usage.

## What Changes

- New `.github/workflows/coverage.yml` runs `cargo-llvm-cov` on
  every PR; `--fail-under-lines 60` for the `server-full` and
  `minimal` features.
- **Codecov integration** for the per-PR coverage delta badge and
  the historical coverage graph.
- New `docs/coverage-baseline.md` records the starting coverage
  per feature.
- `frontend/` coverage via `vitest --coverage` with the `v8`
  provider; same 60% threshold.
- New `tools/coverage-drift.sh` prints the coverage delta against
  the baseline; useful in PR reviews.
- `.grcovrc` removed (cargo-llvm-cov supersedes grcov; the file
  was committed but never wired in).

## Capabilities

### New Capabilities

- `coverage-baseline`: the 60% coverage baseline + Codecov
  integration + per-PR delta.

## Impact

- **CI:** one new workflow (5 min runtime on the supported
  features); the existing `ci.yml` `check` and `test` jobs are
  unchanged.
- **Frontend:** `vitest --coverage` requires the `@vitest/coverage-v8`
  package; one new devDependency.
- **Developer workflow:** the `--fail-under-lines 60` gate will
  block PRs that drop coverage. This is the intended behavior; the
  baseline document explains how to maintain coverage.
- **Performance:** `cargo-llvm-cov` requires nightly. The CI
  workflow installs it automatically; no local toolchain change.
- **License:** no change. cargo-llvm-cov is MIT; @vitest/coverage-v8
  is MIT.

## Out of scope

- **Mutation testing** (cargo-mutants). Tracked as a separate
  change (`test-quality-mutation-fuzz-property`) in the same
  Order 2 of the grade-A plan.
- **Fuzz / property tests** (cargo-fuzz, proptest). Tracked as the
  same separate change above.
- **A 60% threshold increase to 80%.** The operator explicitly
  chose 60% on day one with the intent to adjust based on observed
  usage. Any threshold change is a separate change.
- **Frontend E2E coverage.** `playwright` coverage is in
  experimental state; out of scope.
