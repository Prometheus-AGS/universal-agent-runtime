# Coverage baseline

UAR uses [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) for
Rust coverage and `vitest --coverage` (v8 provider) for frontend coverage.
Both run locally with a **60% line-coverage floor** (`--fail-under-lines 60` / vitest
`thresholds`), per operator decision Q4 in
`.kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/plan.md`: start at 60%
and raise it once real usage data is in — not 80% on day one.

## Per-feature baseline

<!-- PENDING: task 3.1 (openspec/changes/coverage-cargo-llvm-cov-60pct/tasks.md)
     records the actual measured percentages here after the first local coverage
     run. Per the KBD implementation-first policy, that
     run is deferred to the phase's consolidated validation pass rather than
     run ad hoc per file edit during implementation. `tools/coverage-drift.sh`
     reads this table's `| path | pct% |` rows to compute drift once it is
     populated. -->

| Path | Line coverage |
|---|---|
| _(populate from the first retained local coverage run)_ | _TBD_ |

## How this is maintained

- `tools/coverage-drift.sh <lcov-file>` compares a fresh lcov report against
  this table and fails if any file drops more than 5 points.
- Update this file whenever the baseline intentionally moves (e.g. after
  adding a large well-tested module, or after the threshold is raised past
  60%).
- The threshold itself (currently 60%) is passed to local `cargo-llvm-cov` as
  `--fail-under-lines 60` and lives in `frontend/vitest.config.ts`
  (`coverage.thresholds`).
