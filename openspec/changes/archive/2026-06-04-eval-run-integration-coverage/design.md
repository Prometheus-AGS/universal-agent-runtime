## Context

`cli::run_suite` composes the public eval API (`load_suite`, `build_scorers`, `Runner::run`, `summarize`, `save_results`, `load_baseline`, `compare`) around a live-orchestrator provider. The composition is untested; the pieces are individually tested. `CompletionProvider` is the seam that lets a test inject a deterministic provider.

## Goals / Non-Goals
**Goals:** deterministic end-to-end coverage of the pipeline; a reusable fixture provider for EHH1.
**Non-Goals:** testing the live-orchestrator construction; production changes.

## Decisions
- **D1 — recorded-fixture provider:** `RecordedProvider { responses: HashMap<String,String> }` impl `CompletionProvider`; `complete(input)` returns the mapped output or `Err` (drives the runner's contained-failure path). `#[cfg(test)]`.
- **D2 — placement:** a new `#[cfg(test)] mod integration_tests;` file in `src/uar/eval/`. In-crate (not `tests/`) so it reaches the public API quickly and shares the build; no separate integration binary.
- **D3 — pipeline under test:** mirror `run_suite`'s data flow minus orchestrator construction — `load_suite(temp file)` → `build_scorers(&suite, &provider)` → `Runner::run` → `summarize` → `save_results` + reload assert → `save_baseline`/`load_baseline` → `compare` × 3 verdicts. Suite declares scorers (`exact_match`, `contains`, `non_empty`) so scores are deterministic given recorded outputs.
- **D4 — temp isolation:** write the suite + results under a unique temp subdir (process id in the name); clean up at the end.

## Risks / Trade-offs
- **[doesn't cover `run_suite` directly]** the orchestrator-construction line stays uncovered → accepted; that path needs a live model and is the EHH1 nightly job's domain. The test covers the rest of the pipeline 1:1.
- **[temp-file flakiness]** → unique subdir + cleanup; failures are contained to the test.

## Migration Plan
1. `integration_tests.rs`: `RecordedProvider` + the end-to-end test(s).
2. `mod.rs`: `#[cfg(test)] mod integration_tests;`.
3. Verify: `cargo test --lib eval::integration`; check/clippy; `openspec validate --strict`.
- Rollback: test-only; delete the module.

## Open Questions
- None.
