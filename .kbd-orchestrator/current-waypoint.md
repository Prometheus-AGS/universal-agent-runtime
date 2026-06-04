# Current Waypoint

- Phase: `eval-harness-hardening` **(reflect_complete)**
- Previous phase: `uar-eval-harness` (complete — S1 MET)
- Backend: OpenSpec
- Status: `complete`
- Progress: **5 / 5 changes shipped** (PRs #38–#42 merged; 4 archived + HK1 chore)
- Exact next command: `/kbd-new-phase`
- Reflection: [reflection.md](phases/eval-harness-hardening/reflection.md)
- Updated at: 2026-06-04

## Phase arc outcome

**4 / 4 goals MET + 1 housekeeping (HK1).**

- **EHH3 `eval-suite-scorer-config`** (#38) — `ScorerSpec` + `EvalSuite.scorers` (serde-default) + `build_scorers` factory; CLI uses it (heuristic fallback retained).
- **EHH2 `eval-llm-judge-scorer`** (#39) — `LlmJudge` async scorer; rubric prompt; deterministic JSON-verdict parse; advisory (D-B).
- **EHH4 `eval-run-integration-coverage`** (#40) — recorded-fixture provider + end-to-end pipeline tests (no live model).
- **HK1 `remove-dead-testing-tree`** (#41) — deleted uncompiled `src/testing/` (27 files, ~22.7k lines).
- **EHH1 `eval-starter-suite-and-ci-gate`** (#42) — `evals/starter.yaml` + two-tier CI gate (Tier-1 keyless structural test per PR; Tier-2 nightly real-model gated workflow, fork-safe, Rule 33).

35 eval lib tests green. The harness is now **load-bearing** (suite + gate), **trustworthy** (judge + integration coverage), and **configurable** (suite-declared scorers).

## Debt / follow-ups

- **P0 — seed + prove the gate:** no baseline shipped, so Tier-2 is *informational until seeded*. Configure `UAR_LLM__API_KEY`, run nightly via `workflow_dispatch --update_baseline`, commit `evals/results/starter.baseline.json`, confirm a deliberate regression fails the job. Tier-2 has not run in CI yet.
- **P1 — secret-redaction chip** (`main.rs:46`, Rule 33) still open.
- **P1 — artifact-refiner QA-gate automation** still not wired (carried 2 phases).
- **P2** — per-case scorer overrides; per-judge model override.
- **Later** — HTTP `POST /api/uar/eval/run`; SurrealDB result storage; true-regex scorer; expand starter suite.

## Next

`/kbd-new-phase` — recommended next is a short gate-activation + security-cleanup phase (seed the baseline, redact secrets, automate the QA gate).
