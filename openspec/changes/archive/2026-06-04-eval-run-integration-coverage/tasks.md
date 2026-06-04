# Tasks — eval-run-integration-coverage

## 1. Fixture + test (integration_tests.rs)
- [x] 1.1 `RecordedProvider { responses: HashMap<String,String> }` impl `CompletionProvider` (miss → Err)
- [x] 1.2 End-to-end test: load_suite(temp, declared scorers) → build_scorers → Runner::run → summarize
- [x] 1.3 Persist: save_results + reload-and-assert-equal; save_baseline/load_baseline round-trip
- [x] 1.4 compare × 3: no-baseline clean, equal clean, drop>threshold regressed
- [x] 1.5 Contained-failure: a case with no recorded output yields a completion=0.0 result, run still completes

## 2. Wire (mod.rs)
- [x] 2.1 `#[cfg(test)] mod integration_tests;`

## 3. Validation (gate)
- [x] 3.1 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 3.2 `cargo clippy` — no new warnings in touched code
- [x] 3.3 `cargo test --features postgres-backend --lib eval::` green (incl. new integration tests)
- [x] 3.4 `openspec validate eval-run-integration-coverage --strict`; update progress

## Notes
- Test-only; no production change. Covers the pipeline minus live-orchestrator construction (EHH1 nightly). No new dep.
