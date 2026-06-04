PLAN: gate-activation-and-security-cleanup
Project: universal-agent-runtime · Date: 2026-06-04 · OpenSpec: YES
Planning model: Opus 4.8 (frontier)
Changes to implement: 1 (+ 1 verification task, no code)

---

## Decisions resolved (from assessment D-A…D-C)

- **D-A → operator-seeds the baseline.** The agent does NOT make a real model call to generate `starter.baseline.json`; seeding the reference bar is a human judgment call. `--require-baseline` keeps the unseeded state safe (fails loudly) meanwhile. *(No token spend, Rule 8 — no avoidable irreversible/cost action.)*
- **D-B → minimal scope.** GA1 (`--require-baseline` + runbook) + verify/close security. Artifact-refiner QA-gate automation (QA1) is unrelated process tooling — tracked as deferred, not this phase.
- **D-C → opt-in flag.** `--require-baseline` defaults off (preserves EH5 `eval run` behavior, Rule 32); the nightly workflow opts in.

## Honest scope note

Security cleanup (the carried Rule 33 item) is **already implemented on `main`** (verified in assessment) — so this phase is small: one cohesive code+docs change and one verification. No busywork invented.

---

## CHANGE LIST

1. **eval-require-baseline-gate** (GA1): make the unseeded nightly gate fail loudly + document activation.
   - Scope: `src/config.rs` (`EvalAction::Run` gains `require_baseline`), `src/uar/eval/cli.rs` (guard in `run_suite`), `.github/workflows/eval-nightly.yml` (add `--require-baseline`), `evals/README.md` (operator runbook).
   - Depends on: NONE (builds on merged EHH1).
   - Agent: Claude Code · Complexity: S · Model: medium · Value: HIGH (makes the gate's state honest/enforcing).
   - Details:
     - `EvalAction::Run` → add `#[arg(long)] require_baseline: bool` (default false).
     - `run_suite`: thread `require_baseline`; after `load_baseline`, if `require_baseline && baseline.is_empty()` → print a clear error (`eval: --require-baseline set but no baseline for '<suite>' — seed it with --update-baseline`) and return a non-zero exit (use `2`, the usage/IO code) **before** the compare. When not set, behavior is unchanged (empty baseline → clean, exit 0).
     - Nightly Tier-2 gating step: add `--require-baseline` to the non-`update_baseline` branch so a missing committed baseline fails the scheduled job (turns "informational" into "blocked until seeded"). The `--update_baseline` branch stays as-is (it's the seeding path).
     - `evals/README.md`: add an **"Activating the gate (operator runbook)"** section — set `UAR_LLM__API_KEY` secret (+ optional `vars.UAR_EVAL_MODEL`), run `eval-nightly` via `workflow_dispatch` with `update_baseline=true`, commit `evals/results/starter.baseline.json`, then confirm a deliberate regression fails a normal run.
     - Test: extend `integration_tests.rs` — a `require_baseline`-style guard test at the pure level is awkward (the flag lives in `run_suite`, which builds an orchestrator). Instead add a focused unit covering the *decision* — e.g. factor the "missing baseline under strict" check into a tiny pure helper `fn baseline_missing_under_strict(require: bool, baseline: &ScoreSummary) -> bool` in `cli.rs` and unit-test it (true only when `require && empty`). Keeps the gate logic tested without a live model.
   - OpenSpec: MODIFIED `eval-harness` requirement — the scheduled tier SHALL fail when strict-mode is set and no baseline exists.

— **Verification task (no code, not an OpenSpec change)** —

2. **SC1 — verify + close security.**
   - Confirm the config dump is masked: build the binary and run `eval list` (or any startup) and grep the `Configuration loaded` line for `***redacted***` on `api_key`/`provider_keys`/`jwt_secret` and **no** plaintext key material. Already structurally verified by reading `config.rs`; this is the runtime confirmation.
   - Dismiss the still-open "Redact secrets from config startup log" spawn-task chip (the work is already on `main`).
   - Record in the phase reflection that Goal 2 was MET (pre-existing).

---

## EXECUTION ROUND ORDER

- **Round 1:** GA1 `eval-require-baseline-gate` (single PR — code + workflow + docs + test).
- **Round 2 (no PR):** SC1 verification + chip dismissal, folded into reflect.

Per-change workflow (established): branch in main checkout → author OpenSpec artifacts → implement (`/opsx:apply`) → verify gates → PR → await merge → archive.

## VERIFICATION GATES (GA1)

```
SKIP_FRONTEND_BUILD=1 cargo check --features postgres-backend         # clean
SKIP_FRONTEND_BUILD=1 cargo clippy --features postgres-backend        # zero new warnings in touched code
SKIP_FRONTEND_BUILD=1 cargo test --features postgres-backend --lib eval::   # green (incl. new guard unit)
# manual: `eval run <missing-baseline-suite> --require-baseline` exits non-zero; without the flag exits 0
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/eval-nightly.yml'))"   # YAML still valid
openspec validate eval-require-baseline-gate --strict
rustfmt --edition 2024 <only-touched-files>
```

## DEFERRED (not this phase)

- **D-A operator actions:** configure `UAR_LLM__API_KEY` secret + first `workflow_dispatch` seed run + commit baseline (human-only; documented by the runbook).
- **QA1:** artifact-refiner QA-gate automation (carried 3 phases — own phase).
- Per-case scorer overrides; per-judge model override; HTTP `POST /api/uar/eval/run`; SurrealDB result storage; true-regex scorer; expand starter suite.

## COMMANDS TO RUN

```
/opsx:new eval-require-baseline-gate        # GA1 (round 1)
# then implement, PR, merge; SC1 verification happens at reflect time
```
