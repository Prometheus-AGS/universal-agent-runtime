# 3. Set a 60% coverage baseline on day one

Date: 2026-07-13

## Status

Accepted

The 60% local target remains the recorded intent. The statement below that CI
enforces it is superseded by the 2026-08-09 deployment-only GitHub Actions
decision. Coverage checks run locally, and `docs/coverage-baseline.md` still has
no populated measured baseline; this ADR is not evidence that 60% was observed.

## Context

Test coverage was ungated and varied across the Rust workspace and the React frontend. The operator wanted a measurable floor that could be raised once usage data was available, rather than an aspirational target that blocked the release.

## Decision

- Require `60%` line coverage for both the Rust workspace (`cargo-llvm-cov`) and the React frontend (`vitest --coverage`).
- Store the starting baseline in `docs/coverage-baseline.md`.
- Provide `tools/coverage-drift.sh` to show delta vs baseline.

## Consequences

- Local coverage validation is intended to fail below 60%; GitHub Actions do not
  run routine coverage checks.
- The baseline is deliberately conservative; the next quarter may raise it to 70–75% based on actual usage.
- Removes `grcov` in favor of `cargo-llvm-cov`.

## Alternatives considered

- 70% on day one: rejected because the operator preferred to calibrate against real usage.
- Coverage by module: rejected for simplicity; may be revisited after the baseline is established.
