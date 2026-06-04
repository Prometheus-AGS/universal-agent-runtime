# Tasks — eval-starter-suite-and-ci-gate

## 1. Starter suite + docs
- [x] 1.1 `evals/starter.yaml` — model-agnostic cases w/ `expected`; declared scorers non_empty + contains + advisory llm_judge
- [x] 1.2 `evals/README.md` — two tiers + how to seed/update the committed baseline

## 2. Tier 1 (PR CI structural test)
- [x] 2.1 `integration_tests.rs`: load shipped `evals/starter.yaml` via CARGO_MANIFEST_DIR; assert parses (cases + scorers); build_scorers (stub provider) + Runner::run; assert every case scored

## 3. Tier 2 (scheduled real-model gate)
- [x] 3.1 `.github/workflows/eval-nightly.yml` — schedule + workflow_dispatch; guard on UAR_LLM__API_KEY (skip if absent); build --release; `eval run evals/starter.yaml`; non-zero exit on regression; key via env only (Rule 33)

## 4. Validation (gate)
- [x] 4.1 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 4.2 `cargo clippy` — no new warnings in touched code
- [x] 4.3 `cargo test --features postgres-backend --lib eval::` green (incl. starter-suite test)
- [x] 4.4 Workflow YAML sanity; `openspec validate eval-starter-suite-and-ci-gate --strict`; update progress

## Notes
- Two-tier gate (D-A): cheap structural test per PR; real-model run scheduled, key-guarded (fork-safe). Judge advisory (D-B). Baseline committed deliberately (D1/D4) — none shipped yet; gate is informational until seeded. No new dep.
