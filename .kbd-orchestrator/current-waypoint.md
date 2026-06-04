# Current Waypoint

- Phase: `gate-activation-and-security-cleanup` **(planned)**
- Previous phase: `eval-harness-hardening` (complete — 4/4 MET + HK1)
- Backend: OpenSpec
- Status: `planned`
- Progress: **0 / 1 change** (+ 1 verification task, no code)
- Exact next command: `/opsx:new eval-require-baseline-gate`
- Assessment: [assessment.md](phases/gate-activation-and-security-cleanup/assessment.md)
- Plan: [plan.md](phases/gate-activation-and-security-cleanup/plan.md)
- Updated at: 2026-06-04

## Phase intent (small phase)

Make the nightly eval gate *enforce* (not silently pass when unseeded) and close the carried secret-logging item.

**Key finding:** secret redaction is **already done on `main`** (`config.rs` redacting `Debug` impls) — Goal 2 essentially MET. So the only code is one small change; the rest is a runbook + verification.

## Resolved decisions

- **D-A → operator-seeds the baseline** (no agent model call; human owns the reference bar). `--require-baseline` keeps the unseeded state safe meanwhile.
- **D-B → minimal scope** (GA1 + runbook + verify security). Refiner QA-automation deferred.
- **D-C → `--require-baseline` opt-in** (default off preserves EH5 behavior; nightly opts in).

## Change list

1. **GA1 `eval-require-baseline-gate`** — `EvalAction::Run` gains `--require-baseline`; `run_suite` exits non-zero when set + no baseline (fail loudly); nightly workflow adds the flag; `evals/README.md` gets an operator runbook for seeding + activating the gate. Pure-helper unit test for the strict-missing decision.
2. **SC1 (no code)** — verify the config dump is masked at runtime; dismiss the secret-redaction spawn-task chip; record Goal 2 as pre-existing MET. Folded into reflect.

## Deferred (not this phase)

- Operator actions: configure `UAR_LLM__API_KEY` secret + first `workflow_dispatch` seed + commit baseline (human-only; documented).
- QA1 refiner QA-gate automation (own phase); per-case scorers; per-judge model; HTTP eval endpoint; SurrealDB storage.

## Next

`/opsx:new eval-require-baseline-gate` to start GA1.
