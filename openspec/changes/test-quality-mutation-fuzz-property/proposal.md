## Why

UAR has conventional unit and integration tests, but lacks the
higher-confidence test-quality gates expected of a Grade-A runtime:
mutation testing to catch weak assertions, fuzz targets to catch
parsing/edge-case bugs, and property-based tests to guard invariants.
The 2026-07-13 release-readiness assessment flagged this as a
structural gap. The operator's analysis selected **cargo-mutants**
for mutation testing, **cargo-fuzz** for fuzzing, and **proptest**
for property-based testing, with a minimum 22h investment (Q5 decision).

## What Changes

- New `.github/workflows/mutation.yml` nightly cron running
  `cargo mutants --no-shuffle`; results published to
  `docs/mutation-history/`.
- New `fuzz/` directory with 4 initial targets: `chunker`,
  `rag_verification`, `mcp_message_parser`, `json_schema_validator`.
- New `proptest` property tests for: settings store serde roundtrip,
  retrieval RRF invariants, and governance policy hot-reload semantics.
- `release-plz` configured with a conventional-commits check.
- `commitlint` + `lefthook` for the JS workspace to enforce commit
  message hygiene.

## Capabilities

### New Capabilities

- `test-quality-gates`: mutation testing, fuzz targets, property-based
  tests, and conventional-commit release checks.

## Impact

- **CI:** one new nightly workflow (`mutation.yml`); no change to the
  per-PR pipeline. The `release-plz` check is part of the release
  workflow.
- **Developer workflow:** contributors can run `cargo mutants` and
  `cargo fuzz` locally; `lefthook` enforces commit lint before push.
- **Dependencies:** `cargo-mutants` (MIT), `cargo-fuzz` (MIT/Apache),
  `proptest` (MIT/Apache), `commitlint` (MIT), `lefthook` (MIT).
- **License:** no change. All added tools are permissively licensed.

## Out of scope

- **Raising the line-coverage threshold.** Handled by the separate
  `coverage-cargo-llvm-cov-60pct` change.
- **Full formal verification** of the runtime. This change adds
  fuzz/property targets; exhaustive verification is out of scope.
- **Frontend unit-test migration.** Existing frontend tests remain
  unchanged; `commitlint`/`lefthook` apply to the JS workspace only.
