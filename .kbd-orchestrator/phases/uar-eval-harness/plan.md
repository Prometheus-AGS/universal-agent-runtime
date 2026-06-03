PLAN: uar-eval-harness
Project: universal-agent-runtime · Date: 2026-06-03 · OpenSpec: YES
Planning model: Opus 4.8 (frontier)
Changes to implement: 4 (LLM-judge EH3 deferred to fast-follow)

---

## Decisions resolved

- **D1 storage → files** (git-friendly): suites as `evals/<suite>.{yaml,json}`; results + baselines as JSON under `evals/results/`. No DB dependency.
- **D2 surface → CLI subcommand**: restructure the flat `Cli` into an optional `#[command(subcommand)]`; default (none) runs the server, `eval run|list|baseline` runs the harness.
- **D3 run mode → bare completion** in v1 (`state.orchestrator.chat_non_streaming`); full-agent-run (`start_run`) mode is a later option.
- **D4 LLM-judge → deferred** (EH3 fast-follow). v1 is rule-based only (incl. sycophancy adapter).
- **D5 regression gate → delta-vs-baseline**: fail when a scorer's mean drops more than a configured threshold vs the stored baseline.

Scope cut (S-07/S-03): v1 = rule-based, file-backed, CLI-driven. LLM-judge, full-agent-run mode, an HTTP endpoint, and SurrealDB result storage are explicitly OUT (fast-follow / later).

---

## CHANGE LIST (ordered — sequential dependencies)

1. **eval-domain-and-rule-scorers** (EH1): eval domain types + `Scorer` trait + rule-based scorers.
   - Scope: new `src/uar/eval/` module (domain + scorers) | reuse `quality::detect`
   - Depends on: NONE
   - Agent: Claude Code · Complexity: M · Score: Medium · Model: medium · Value: foundational
   - Details: `EvalCase { id, input, expected: Option<String>, metadata }`, `EvalSuite { name, cases }`, `Score { scorer, value: f32, detail }`, `EvalResult { suite, case_id, model, scores, run_at }`; `trait Scorer { fn name(&self) -> &str; async fn score(&self, case: &EvalCase, output: &str) -> Score }`. Built-in rule scorers: `ExactMatch`, `Contains`, `Regex`, `JsonValid`, `NonEmpty`, and a `Sycophancy` adapter wrapping `quality::detect` (score = 1 − sycophancy_score). Pure + unit-tested. Register `pub mod eval;` in `uar/mod.rs`.

2. **eval-suite-loading-and-runner** (EH2): load golden suites from files + run cases via bare completion.
   - Scope: `src/uar/eval/` (suite loader + runner)
   - Depends on: EH1
   - Agent: Claude Code · Complexity: M · Score: Medium · Model: medium · Value: HIGH
   - Details: load `EvalSuite` from `evals/<suite>.{yaml,json}` (serde); a `Runner` that, per case, calls `orchestrator.chat_non_streaming([user=input])` to get the output, runs the suite's configured scorers, and collects `EvalResult`s in memory. Unit-test the loader + scorer aggregation with a stub output (no live LLM). Errors per case contained (recorded as a failed result, not aborting the suite).

3. **eval-persistence-and-regression** (EH4): persist results to files + compare to a baseline.
   - Scope: `src/uar/eval/` (file store + regression) | `metrics.rs`
   - Depends on: EH2
   - Agent: Claude Code · Complexity: M · Score: Medium · Model: medium · Value: HIGH
   - Details: write `EvalResult`s to `evals/results/<suite>-<ts>.json`; load/save a named baseline (`evals/results/<suite>.baseline.json`); compute per-scorer mean and a regression verdict = fail when `(baseline_mean − current_mean) > threshold` (delta-vs-baseline, D5). Emit `uar_eval_score{suite,scorer}` + `uar_eval_regressions_total`. Pure comparison fn unit-tested (pass / regress / no-baseline).

4. **eval-cli-subcommand** (EH5): runnable end-to-end via the binary.
   - Scope: `src/config.rs` (`Cli` → subcommands) | `src/main.rs` (dispatch) | `src/uar/eval` (report)
   - Depends on: EH1+EH2+EH4
   - Agent: Claude Code · Complexity: M · Score: Medium · Model: frontier · Value: HIGH
   - Details: add `#[command(subcommand)] command: Option<Command>` to `Cli` with `enum Command { Eval(EvalArgs) }` (`run <suite> [--update-baseline]`, `list`, `baseline <suite>`); `main` dispatches: `None` ⇒ run server (unchanged), `Some(Eval)` ⇒ build the app config + orchestrator, run the suite, print a pass/fail report + regression summary, exit non-zero on regression (CI gate). Preserve all existing global flags.

---

## EXECUTION ROUND ORDER

- Sequential: **EH1 → EH2 → EH4 → EH5** (each depends on the prior). No parallelism (shared `src/uar/eval/` module grows per change).

## DEFERRED (fast-follow / later)

- **EH3 LLM-as-judge scorer** — rubric via `chat_non_streaming` + deterministic numeric parse. Own change after v1 lands.
- Full-agent-run mode (`start_run`); HTTP `POST /api/uar/eval/run`; SurrealDB result storage; absolute-floor gate option.
- (Housekeeping) delete or compile-gate the dead `src/testing/` tree — separate cleanup.

## COMMANDS TO RUN

```
/opsx:new eval-domain-and-rule-scorers
/opsx:new eval-suite-loading-and-runner
/opsx:new eval-persistence-and-regression
/opsx:new eval-cli-subcommand
```

## Sycophancy self-check
- S-02: every change names a concrete reuse (`quality::detect`, `chat_non_streaming`, `metrics.rs`, the `Cli` parser) grounded in the assessment; no invented infra.
- S-07: v1 held to rule-based + file + CLI; judge, full-run, endpoint, DB all explicitly deferred — no scope creep into the greenfield space.
- S-03: trade-offs surfaced — bare-completion vs full-run fidelity (D3), delta-vs-baseline vs absolute floor (D5), CLI restructure touches the binary, judge deferred.

PLAN COMPLETE
