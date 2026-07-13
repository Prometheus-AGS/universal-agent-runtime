# Coverage baseline

## Purpose

Define the 60% coverage baseline and the per-PR drift gate for
the UAR runtime and frontend. The coverage number is enforced
in CI; the baseline is recorded in `docs/coverage-baseline.md`.

## ADDED Requirements

### Requirement: Cargo coverage with cargo-llvm-cov
The `.github/workflows/coverage.yml` job MUST run
`cargo-llvm-cov` (NOT `cargo-tarpaulin` — the 2026 best practice)
for both the `server-full` and `minimal` feature profiles. The
job MUST upload the LCOV report to Codecov.

#### Scenario: A PR maintains or improves coverage
- **WHEN** the PR is opened
- **AND** the resulting line coverage is >= 60%
- **THEN** the workflow passes
- **AND** the Codecov badge updates

#### Scenario: A PR drops coverage below 60%
- **WHEN** the PR is opened
- **AND** the resulting line coverage is < 60%
- **THEN** the workflow fails with the `--fail-under-lines 60` exit
- **AND** the PR cannot be merged

### Requirement: Frontend coverage with vitest
The frontend CI MUST run `pnpm test:coverage` with the `v8`
provider. The 60% line-coverage threshold applies to the frontend
test surface as well.

#### Scenario: Frontend coverage runs in CI
- **WHEN** a PR touches `frontend/`
- **THEN** the `frontend-coverage` job in `.github/workflows/coverage.yml`
  runs `pnpm test:coverage`
- **AND** `frontend/vitest.config.ts`'s `coverage.thresholds` (60% lines,
  statements, functions, branches) gates the vitest run itself

### Requirement: Coverage drift gate
A per-file coverage drift gate MUST be enforced: any file whose
line coverage drops by more than 5 percentage points from the
baseline recorded in `docs/coverage-baseline.md` MUST fail the PR.
The drift gate is implemented in `tools/coverage-drift.sh` and runs
as a follow-up step in the coverage workflow. Until the baseline
document is populated with real per-file percentages (see the next
requirement), the drift gate has no rows to compare against and is
a no-op that always passes — it becomes enforcing the moment the
baseline table has real entries.

#### Scenario: A file's coverage drops more than 5 points
- **WHEN** `tools/coverage-drift.sh` runs against a fresh lcov report
- **AND** `docs/coverage-baseline.md` has a populated entry for a file
- **AND** that file's current line coverage is more than 5 percentage
  points below its baseline entry
- **THEN** the script exits non-zero and the CI step fails

#### Scenario: The baseline document has no entry for a file yet
- **WHEN** `tools/coverage-drift.sh` runs against a file with no
  matching row in `docs/coverage-baseline.md`
- **THEN** the script reports the file's current coverage with no
  baseline comparison and does not fail the build for that file

### Requirement: Coverage baseline document
`docs/coverage-baseline.md` MUST exist and document the 60%
threshold and how the baseline table is maintained. The per-feature
line-coverage numbers MUST be recorded after the first coverage
workflow run on `main` (deferred from this change's implementation
pass to the phase's consolidated validation, per the KBD
implementation-first policy — the doc explicitly marks the table as
pending that run rather than fabricating numbers). The document
MUST be updated quarterly with the new baseline once populated. The
threshold for the *next* quarter is decided by the operator based on
observed usage; the threshold is recorded in the same document.

#### Scenario: The baseline document exists before numbers are recorded
- **WHEN** this change closes
- **THEN** `docs/coverage-baseline.md` exists, documents the 60%
  threshold and the drift-gate mechanism
- **AND** its per-file table is explicitly marked pending the first
  `coverage.yml` run on `main`, not populated with invented figures

### Requirement: No 80% threshold on day one
The CI gate MUST be `--fail-under-lines 60` for the first
release. The 80% threshold is a follow-up-grade target, not a
day-one requirement. The threshold lives in the coverage workflow
file and is a single-line change.

#### Scenario: A PR proposes raising the threshold to 80%
- **WHEN** someone proposes changing `--fail-under-lines 60` (or the
  vitest `coverage.thresholds`) to 80% before real usage data
  justifies it
- **THEN** that is out of scope for this change and requires a
  separate change with operator sign-off, per Q4 in
  `.kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/plan.md`
