# Assessment: uar-eval-harness

**Phase:** `uar-eval-harness`
**Date:** 2026-06-03 · Backend: OpenSpec · base `main` `1cbcb8f`
**Origin:** goal **S1**, deferred from `uar-safety-and-evals` (decision D1) as greenfield/phase-sized.

## Goal

A usable LLM evaluation harness: run prompt/golden suites through the agent, score outputs (rule-based + LLM-as-judge), persist scores, and compare against a baseline for regression over time — with a runner surface (CLI and/or endpoint).

## Current state (grounded)

- **No eval infrastructure exists.** No `Scorer`/`EvalCase`/`EvalSuite`/judge/golden/faithfulness/toxicity/relevance/regression in compiled `src/`.
- **`src/testing/` is dead, uncompiled CI-flakiness analytics** (`alerting/analytics/monitoring/performance/reliability/…`; `mod testing` not declared) — **not** an eval harness; do not mistake it for one. (Consider deleting or compiling it separately — out of scope here.)
- **`PersistenceLayer`** (`persistence/mod.rs:35`) covers sessions/skills/knowledge only — **no run-result or eval-score storage**. Eval results need a new store (SurrealDB table or files).
- **CLI** (`config.rs` `Cli`, `clap::Parser`) is a flat flag parser — **no subcommands**. An eval runner needs a subcommand (restructure to `clap::Subcommand`), a separate bin, or an HTTP endpoint.

## Reusable building blocks (no new deps needed for v1)

- **LLM-as-judge:** `state.orchestrator.chat_non_streaming(Vec<Message>) -> Result<String>` — one-shot, no tools; feed a rubric prompt, parse a score. (Same primitive SE2 used.)
- **Rule-based scorer exemplar:** `uar::quality::detect` (sycophancy) is already a rule-based scorer producing a 0..1 score — the pattern to generalize into a `Scorer` trait; sycophancy can be one built-in scorer.
- **Metrics:** `uar/telemetry/metrics.rs` + Prometheus for eval score/regression series.
- **Run execution:** `RunManager::start_run` + `history_since` to drive a full agent run per case, or `chat_non_streaming` for a bare model-output eval.
- **Persistence:** extend `PersistenceLayer` with eval methods, or a dedicated eval store.

## Proposed architecture (v1)

- **Domain:** `EvalCase { id, input, expected: Option, metadata }`, `EvalSuite { name, cases }`, `trait Scorer { fn name(); async fn score(case, output) -> Score }`, `Score { scorer, value: f32 (0..1), detail }`, `EvalResult { suite, case_id, scores, model, run_at }`.
- **Scorers (v1):** rule-based (exact/contains/regex match, JSON-valid, non-empty, sycophancy via `quality::detect`) + one **LLM-as-judge** (rubric → numeric score via `chat_non_streaming`).
- **Suites:** golden files (`evals/*.{yaml,json}`) loaded at runtime.
- **Runner:** for each case → produce output (bare LLM call or full agent run) → run scorers → collect `EvalResult`s → persist → compare to baseline → report pass/fail + regressions.
- **Surface:** a CLI subcommand (`eval run/list/baseline`) and/or `POST /api/uar/eval/run`.

## Recommended decomposition (likely 4–5 changes)

1. **EH1 — eval domain + `Scorer` trait + rule-based scorers** (incl. sycophancy adapter). Foundation; pure + unit-testable.
2. **EH2 — suite loading + runner** (golden files → execute cases → collect results), bare-LLM-output mode first.
3. **EH3 — LLM-as-judge scorer** (rubric prompt via `chat_non_streaming`; deterministic parsing; cost/latency noted).
4. **EH4 — persistence + regression comparison** (store `EvalResult`s; diff vs a named baseline; emit metrics).
5. **EH5 — surface** (CLI subcommand and/or HTTP endpoint + report).

A tight **v1 could be EH1+EH2+EH4+EH5** (rule-based + persisted + runnable), with EH3 (LLM-judge) as a fast-follow.

## Key product decisions (for `/kbd-plan`)

- **D1 — Result/suite storage:** files (simple, git-friendly, no DB dep) vs SurrealDB (queryable history, needs `PersistenceLayer` extension). Suites likely files either way; results are the question.
- **D2 — Runner surface:** CLI subcommand (restructure `Cli` to subcommands — touches the binary's arg parsing) vs HTTP endpoint (`POST /api/uar/eval/run`) vs a separate `eval` bin. (CLI is most natural for CI regression gates.)
- **D3 — What the runner executes per case:** a full agent run (`start_run`, exercises tools/memory/policy — realistic, heavier) vs a bare model completion (`chat_non_streaming` — fast, isolates the model). v1 likely bare-output; full-run as an option.
- **D4 — LLM-judge in v1 or fast-follow?** (Judge adds a model dependency + cost/determinism care; rule-based + sycophancy gives a useful v1 without it.)
- **D5 — Regression gate semantics:** compare to a stored baseline; fail threshold (absolute score floor vs delta-vs-baseline). For CI.

## Complexity & risk

**LARGE / Medium risk.** A new subsystem spanning domain types, scorers, a runner, persistence, and a surface. No new dependency required for a rule-based v1; LLM-judge reuses `chat_non_streaming`. Risk concentrated in: surface choice (CLI restructure), result persistence design, and judge determinism. Additive (new module + optional surface) — low risk to existing runtime.

## Assessment status

- Greenfield confirmed; reusable primitives identified; v1 architecture + 5-change decomposition proposed.
- Decisions D1–D5 surfaced (storage, surface, run mode, judge timing, regression semantics).
- Ready for `/kbd-plan uar-eval-harness` (resolve D1–D5 first; recommend a rule-based persisted v1 with EH3/judge as fast-follow).
