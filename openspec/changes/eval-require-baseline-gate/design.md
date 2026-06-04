## Context

`EvalAction::Run` (config.rs:111) has `suite`/`threshold`/`results_dir`/`update_baseline`. `cli::run_suite` loads the baseline with `load_baseline(...).unwrap_or(None).unwrap_or_default()` → an empty `ScoreSummary` when absent → `compare` finds no regression → exit 0. `load_baseline` returns `Ok(None)` when the file is absent. The nightly workflow's gating step runs `eval run evals/starter.yaml --results-dir evals/results` (no strictness). No baseline ships.

## Goals / Non-Goals
**Goals:** an opt-in strict mode that fails on a missing baseline; the nightly opts in; an operator runbook.
**Non-Goals:** seeding the baseline (operator); auto-commit from CI; secret redaction (already done).

## Decisions
- **D1 — flag:** `EvalAction::Run` gains `#[arg(long)] require_baseline: bool` (clap defaults bool to `false`). Opt-in preserves EH5 behavior (Rule 32).
- **D2 — pure decision helper:** add `fn baseline_missing_under_strict(require_baseline: bool, baseline: &ScoreSummary) -> bool { require_baseline && baseline.is_empty() }` in `cli.rs` — unit-testable without an orchestrator or model. `run_suite`'s strictness reduces to this one predicate.
- **D3 — guard placement (fail fast):** load the baseline **up front** (right after `load_suite`, before building the orchestrator) and, when `!update_baseline && baseline_missing_under_strict(require_baseline, &baseline)`, `eprintln!` the seeding hint and `return 2` **before any model call** — a CI gate with no committed baseline blocks loudly without spending tokens. The single early load is reused for the later `compare` (no double read). `2` = usage/precondition (≠ `1` regression). The `--update-baseline` seeding path is excluded. *(Refined during implementation from a post-run guard to a pre-run one — cheaper + smoke-testable without a live model.)*
- **D4 — threading:** `run_eval` already destructures `EvalAction::Run { .. }`; add `require_baseline` and pass it to `run_suite` (new param). Single call site.
- **D5 — nightly opt-in:** in `eval-nightly.yml`, the gating (non-`update_baseline`) invocation adds `--require-baseline`. The seeding branch (`--update-baseline`) does not (it's establishing the baseline). Net: a scheduled run with no committed baseline now fails (visible "blocked until seeded") instead of passing.
- **D6 — runbook:** `evals/README.md` gains an "Activating the gate (operator)" section: set `UAR_LLM__API_KEY` secret (+ optional `vars.UAR_EVAL_MODEL`); run `eval-nightly` via `workflow_dispatch` with `update_baseline=true`; commit `evals/results/starter.baseline.json`; confirm a deliberate regression fails a normal (strict) run.

## Risks / Trade-offs
- **[exit-code semantics]** `2` (usage/IO) vs `1` (regression): chose `2` because "no baseline" is a precondition failure, not a measured regression — keeps CI logs unambiguous. Documented.
- **[behavior change for the nightly]** the scheduled job will start failing until a baseline is committed — that is the intended, visible signal (not silent green). Documented in the runbook.
- **[testability]** `run_suite` needs an orchestrator, so the guard is tested via the extracted `baseline_missing_under_strict` predicate (pure) rather than end-to-end. The flag plumbing is exercised by `--help`/manual run.

## Migration Plan
1. `config.rs`: add `require_baseline` to `EvalAction::Run`.
2. `cli.rs`: `baseline_missing_under_strict` helper + unit test; thread the flag through `run_eval`→`run_suite`; guard before `compare`.
3. `eval-nightly.yml`: add `--require-baseline` to the gating step.
4. `evals/README.md`: operator runbook section.
5. Verify: check/clippy/test `eval::`; manual `--help` + missing-baseline exit; YAML sanity; `openspec validate --strict`.
- Rollback: additive flag + a workflow arg + docs; revert restores prior gating.

## Open Questions
- None. Auto-baseline-commit remains a deferred follow-up.
