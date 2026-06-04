# Tasks — eval-require-baseline-gate

## 1. CLI flag (config.rs)
- [x] 1.1 Add `#[arg(long)] require_baseline: bool` to `EvalAction::Run` (with doc comment)

## 2. Guard + helper (cli.rs)
- [x] 2.1 `fn baseline_missing_under_strict(require_baseline: bool, baseline: &ScoreSummary) -> bool` (true iff require && empty)
- [x] 2.2 Thread `require_baseline` through `run_eval` (destructure) → `run_suite` (new param)
- [x] 2.3 In `run_suite`, after `load_baseline`, before `compare`: if helper true → eprintln seeding hint + `return 2`
- [x] 2.4 Unit test for `baseline_missing_under_strict` (strict+empty → true; strict+non-empty → false; non-strict → false)

## 3. Nightly opt-in (eval-nightly.yml)
- [x] 3.1 Add `--require-baseline` to the gating (non-update_baseline) `eval run` invocation; leave the `--update_baseline` seeding branch unchanged

## 4. Operator runbook (evals/README.md)
- [x] 4.1 Add "Activating the gate (operator)" — set UAR_LLM__API_KEY secret (+ optional vars.UAR_EVAL_MODEL); workflow_dispatch update_baseline=true; commit evals/results/starter.baseline.json; verify a deliberate regression fails a strict run

## 5. Validation (gate)
- [x] 5.1 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 5.2 `cargo clippy` — no new warnings in touched code
- [x] 5.3 `cargo test --features postgres-backend --lib eval::` green (incl. new helper unit)
- [x] 5.4 Manual: `eval run <missing> --require-baseline` exits 2; without flag exits 0; `eval run --help` shows the flag
- [x] 5.5 `eval-nightly.yml` valid YAML; `openspec validate eval-require-baseline-gate --strict`; update progress

## Notes
- Opt-in flag, default off (Rule 32). Exit 2 = missing-baseline precondition (≠ 1 regression). Seeding the baseline + CI secret are operator actions (runbook). No new dep.
