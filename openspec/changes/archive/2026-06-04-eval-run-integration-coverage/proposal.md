# eval-run-integration-coverage

## Why

The eval `run` pipeline (load → run → score → summarize → compare → persist) is only manually smoke-tested — its glue (gap G4) has no automated coverage, because the CLI builds a live orchestrator. This change (EHH4) adds a **recorded-fixture `CompletionProvider`** and an integration test that drives the full pipeline deterministically, with no live model. It also gives EHH1 the fixture its Tier-1 structural CI test needs.

## What Changes

- **Recorded-fixture `CompletionProvider`** (`#[cfg(test)]`) — returns canned outputs keyed by input; missing input → error (exercises the runner's contained-failure path).
- **End-to-end integration test** (`src/uar/eval/integration_tests.rs`, `#[cfg(test)]`): `load_suite` (temp file, with declared scorers) → `build_scorers` + `Runner::run` over the fixture → `summarize` → `save_results` round-trip → `save_baseline`/`load_baseline` → `compare` for all three verdicts (no-baseline clean, equal = no regression, drop > threshold = regression). Asserts the public pipeline composes correctly.

Out of scope: testing `cli::run_suite` itself (it constructs a live orchestrator — covered by the nightly real-model job in EHH1); no production code change.

## Capabilities

### Modified Capabilities
- **`eval-harness`** — delta `specs/eval-harness/spec.md`. Adds an automated end-to-end coverage requirement for the run→score→persist→compare pipeline using a deterministic provider.

## Impact

- **Affected code:** new `src/uar/eval/integration_tests.rs` (`#[cfg(test)]`); `src/uar/eval/mod.rs` (`#[cfg(test)] mod integration_tests;`). **No production logic changes** — test-only.
- **No new dependency** (Rule 27): reuses the existing public eval API + `async_trait`.
- **KBD workflow state:** YES — EHH4, round 2 of `eval-harness-hardening`.
