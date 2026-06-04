# Current Waypoint

- Phase: `uar-eval-harness` **(reflect_complete)**
- Previous phase: `uar-safety-and-evals`
- Backend: OpenSpec
- Status: `complete`
- Progress: **4 / 4 changes shipped** (PRs #34–#37 merged + archived)
- Exact next command: `/kbd-new-phase`
- Reflection: [reflection.md](phases/uar-eval-harness/reflection.md)
- Updated at: 2026-06-04

## Phase arc outcome

**1 / 1 goal MET (S1 — greenfield eval harness).**

Shipped + merged + archived:

- **EH1 `eval-domain-and-rule-scorers`** — `src/uar/eval/` domain (`EvalCase`/`EvalSuite`/`Score`/`EvalResult`) + `Scorer` trait + rule scorers (`ExactMatch`/`Contains`/`JsonValid`/`NonEmpty`/`PatternMatch`/`Sycophancy`).
- **EH2 `eval-suite-loading-and-runner`** — `load_suite` (JSON/YAML) + `Runner` over a `CompletionProvider` seam (per-case errors contained).
- **EH4 `eval-persistence-and-regression`** — file results + baseline; delta-vs-baseline `compare`; eval metrics (`uar_eval_score`, `uar_eval_regressions_total`).
- **EH5 `eval-cli-subcommand`** — `eval run|list|baseline`; non-zero exit on regression (CI gate); server default preserved when no subcommand.

v1 = rule-based, file-backed, CLI-driven, **as planned**. EH3 LLM-judge intentionally deferred.

## Deviations & debt

- `PatternMatch` shipped instead of a `Regex` scorer (no new dependency, Rule 27).
- Scorer selection is a heuristic; **per-suite scorer config deferred**.
- Live `eval run <suite>` path is unit-untested (needs a configured model); pure pieces fully tested.
- No example suite shipped under `evals/`; dead `src/testing/` still present.
- Pre-existing **Rule 33** issue: `main.rs:46` logs secrets in plaintext — flagged via spawn-task.

## Recommended next phase

**`eval-harness-hardening`** fast-follow — ship a starter suite + CI gate (highest leverage), then EH3 LLM-judge, per-suite scorer config, and `run` integration coverage. See `recommendedNextPhaseSeeds` in `current-waypoint.json`.
