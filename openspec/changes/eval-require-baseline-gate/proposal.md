# eval-require-baseline-gate

## Why

The Tier-2 nightly eval gate (EHH1) only enforces a regression bar **when a baseline file is present**. With no committed baseline, `eval run` does `load_baseline(...).unwrap_or_default()` → empty baseline → `compare` finds no regression → **exit 0**. The gate silently passes, and nothing signals that it isn't actually gating. No baseline ships, so today the nightly is a no-op smoke test wearing a gate's clothes.

This change (GA1) makes that state honest: an opt-in `--require-baseline` flag turns a missing baseline into a **loud non-zero failure**. The nightly opts in, so until an operator seeds the baseline the scheduled job *fails* ("blocked until seeded") instead of *passing* ("looks green, gates nothing"). An operator runbook documents the human-only activation steps (CI secret + first seed run).

## What Changes

- **`EvalAction::Run` gains `--require-baseline`** (`bool`, default `false`).
- **`run_suite` guard:** when `require_baseline` is set and no baseline exists for the suite, print a clear message and return a non-zero exit **before** comparing. When the flag is off, behavior is unchanged (empty baseline → clean → exit 0).
- **Nightly workflow** (`eval-nightly.yml`): the gating (non-`update_baseline`) branch adds `--require-baseline`, so a missing committed baseline fails the scheduled job. The seeding branch (`--update_baseline`) is untouched.
- **Operator runbook** in `evals/README.md`: configure the `UAR_LLM__API_KEY` secret (+ optional `vars.UAR_EVAL_MODEL`), run the workflow with `update_baseline=true`, commit `evals/results/starter.baseline.json`, then verify a deliberate regression fails a normal run.

Out of scope: actually seeding the baseline (operator action — needs a real model + human judgment on the reference bar); auto-committing baselines from CI; secret redaction (already done on `main`).

## Capabilities

### Modified Capabilities
- **`eval-harness`** — delta `specs/eval-harness/spec.md`. Strengthens the CI-gate requirement: the scheduled tier fails when strict mode is set and no baseline exists.

## Impact

- **Affected code:** `src/config.rs` (`EvalAction::Run` field), `src/uar/eval/cli.rs` (guard + a pure `baseline_missing_under_strict` helper + unit test), `.github/workflows/eval-nightly.yml` (add the flag), `evals/README.md` (runbook).
- **Behavior preservation (Rule 32):** the flag defaults off; plain `eval run` is unchanged. Only the nightly opts in.
- **No new dependency** (Rule 27).
- **KBD workflow state:** YES — GA1, the sole change of `gate-activation-and-security-cleanup`.
