# Current Waypoint

- Phase: `eval-harness-hardening` **(planned)**
- Previous phase: `uar-eval-harness` (complete — S1 MET)
- Backend: OpenSpec
- Status: `planned`
- Progress: **0 / 5 changes** (4 + 1 housekeeping)
- Exact next command: `/opsx:new eval-suite-scorer-config`
- Assessment: [assessment.md](phases/eval-harness-hardening/assessment.md)
- Plan: [plan.md](phases/eval-harness-hardening/plan.md)
- Updated at: 2026-06-04

## Phase intent

Fast-follow on the v1 eval harness: make it **load-bearing and trustworthy** — a real suite gates CI, the LLM-judge scorer exists, suites declare their own scorers, and the `run` path has automated coverage.

## Resolved decisions

- **CI gate → two-tier:** PR CI runs a deterministic structural eval test (fixture provider + rule scorers, no key/cost); a nightly/main-only job runs the real model vs the starter suite with a repo secret and gates on regression.
- **LLM-judge → advisory only:** hard gate uses rule scorers; judge scores reported but don't fail CI.
- **Judge verdict → JSON** `{score 0..1, reason}`; clamp; parse-failure → 0.0 + detail.
- **Scorer config → suite-level** (per-case deferred).
- **Housekeeping:** delete dead `src/testing/` here; secret redaction (`main.rs:46`) stays with its spawn-task chip.

## Change list (ordered)

1. **EHH3 `eval-suite-scorer-config`** (R1) — `ScorerSpec` enum + `EvalSuite.scorers` (serde default) + `build_scorers` factory; CLI uses it (heuristic fallback). *foundational.*
2. **EHH2 `eval-llm-judge-scorer`** (R2) — `LlmJudge` async `Scorer` over `CompletionProvider`; JSON verdict parse; wired into the factory; advisory.
3. **EHH4 `eval-run-integration-coverage`** (R2) — recorded-fixture provider + end-to-end `run` pipeline integration test (no live model).
4. **HK1 `remove-dead-testing-tree`** (R2, independent) — `git rm -r src/testing/` (uncompiled dead code).
5. **EHH1 `eval-starter-suite-and-ci-gate`** (R3) — `evals/starter.yaml` + Tier-1 structural CI test + Tier-2 nightly real-model gated job.

**Rounds:** R1 EHH3 → R2 (EHH2 ∥ EHH4 ∥ HK1) → R3 EHH1.

## Next

`/opsx:new eval-suite-scorer-config` to start EHH3.
