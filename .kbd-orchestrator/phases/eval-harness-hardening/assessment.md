# ASSESSMENT: eval-harness-hardening

Project: universal-agent-runtime · Date: 2026-06-04 · Backend: OpenSpec
Assessing model: Opus 4.8 (frontier)
**Origin:** fast-follow on `uar-eval-harness` (S1 MET v1). Make the harness load-bearing and close v1 debt.

---

## Goal

Turn the v1 eval harness from "exists and is unit-tested" into "**load-bearing and trustworthy**":
1. A real suite runs in CI as a regression gate.
2. The one deferred scorer (LLM-as-judge) exists.
3. Suites declare their own scorers (remove the EH5 heuristic).
4. The end-to-end `run` path has automated coverage (today it's only manually smoke-tested).

---

## Current state (grounded)

**Shipped & merged (uar-eval-harness, on `main`):**
- `src/uar/eval/mod.rs` — domain (`EvalCase`/`EvalSuite`/`Score`/`EvalResult`) + `Scorer` trait (already **`async`** — designed for an LLM-judge) + rule scorers (`ExactMatch`/`Contains`/`JsonValid`/`NonEmpty`/`PatternMatch`/`Sycophancy`).
- `src/uar/eval/runner.rs` — `load_suite` (JSON/YAML), `CompletionProvider` trait, `Runner::run`.
- `src/uar/eval/persistence.rs` — `summarize`, `compare` (delta-vs-baseline), file results + baseline, metrics.
- `src/uar/eval/cli.rs` — `eval run|list|baseline`; `OrchestratorCompletionProvider` over `chat_non_streaming`; **`select_scorers` is a hardcoded heuristic** (`[NonEmpty, Sycophancy]` + `[ExactMatch, Contains]` when all cases have `expected`).
- Spec promoted to `openspec/specs/eval-harness/spec.md`.

**Gaps confirmed by inspection:**

| # | Gap | Evidence |
| - | --- | -------- |
| G1 | **No suite ships, no CI gate.** `evals/` does not exist; `.github/workflows/ci.yml` runs `fmt`/`clippy`/`check`/`test` only — no `eval run`. The harness is dead weight until a suite + gate exist. | `ls evals/` → absent; `ci.yml` has `check` + `test` jobs only |
| G2 | **No LLM-as-judge scorer.** Only rule scorers exist. The `Scorer` trait is async and `chat_non_streaming(messages) -> Result<String>` exists, so the seam is ready — but nothing implements a judge. | `grep` shows no judge; `orchestrator.rs:931` |
| G3 | **Suites can't declare scorers.** `EvalSuite { name, cases }` has no scorer field; the CLI picks scorers by heuristic. A suite that wants `json_valid` or `llm_judge` can't ask for it. | `EvalSuite` struct; `cli.rs::select_scorers` |
| G4 | **`run` path is unit-untested.** `OrchestratorCompletionProvider` + `run_suite` (load→run→summarize→compare→persist→report) are exercised only by manual smoke; no fixture/recorded provider test. | `cli.rs` tests cover only `resolve_suite_path`/`select_scorers` |
| G5 | **(carried) dead `src/testing/` tree.** 8 submodules under `src/testing/`, **not declared** in `lib.rs`/`main.rs` → never compiled. Pure dead weight. | `grep "mod testing" src/lib.rs src/main.rs` → none |
| G6 | **(carried, security) secrets logged.** `main.rs:46` logs full `AppConfig` (LLM/provider keys, JWT secret) at INFO — Rule 33. spawn-task chip filed. | `main.rs:46` |

---

## Reusable building blocks (no new deps needed)

- **Judge:** `Orchestrator::chat_non_streaming` + the existing `CompletionProvider` trait + the async `Scorer` trait → a `LlmJudge` scorer holding `Arc<dyn CompletionProvider>` fits with zero trait churn.
- **Scorer config:** `serde` with `#[serde(default)]` makes a new optional `scorers` field on `EvalSuite` backward-compatible (existing suites/tests unaffected).
- **Fixtures:** the `CompletionProvider` seam already lets tests inject a deterministic provider (EH2's `Echo`/`Failing` stubs are the pattern) — a recorded-fixture provider needs no production change.
- **CI:** `ci.yml` is the obvious host; a structural eval test runs under the existing `test` job; a real-model run needs a separate gated job + repo secret.
- **Parse:** `serde_json` (already used) for the judge's JSON verdict.

---

## Proposed architecture (v1.1)

1. **Scorer factory** — `ScorerSpec` tagged enum (`exact_match`, `contains`, `json_valid`, `non_empty`, `pattern_match{pattern,mode}`, `sycophancy`, `llm_judge{rubric, …}`) + `build_scorers(suite, provider) -> Vec<Arc<dyn Scorer>>`. `EvalSuite.scorers: Vec<ScorerSpec>` (`#[serde(default)]`); when empty, fall back to today's heuristic (preserves behavior). CLI calls the factory instead of `select_scorers`.
2. **`LlmJudge` scorer** — `{ provider: Arc<dyn CompletionProvider>, rubric, model? }`; builds a judge prompt (rubric + case input + candidate output), expects a JSON verdict `{ "score": 0.0–1.0, "reason": "…" }`, parses deterministically, clamps, and on parse failure returns `0.0` + detail (never panics). Judge defaults to `config.llm`.
3. **Recorded-fixture provider** — an in-repo `CompletionProvider` returning canned outputs keyed by input, for an end-to-end `run` integration test (no live model, deterministic).
4. **Starter suite + CI** — `evals/starter.yaml` (a handful of cases); a PR-CI **structural** eval test (fixture provider, rule scorers only — cheap, no key, deterministic); a **nightly** real-model `eval run evals/starter.yaml` job gated by a repo secret that posts the regression verdict.

---

## Recommended decomposition (4 changes)

1. **`eval-suite-scorer-config`** (EHH3) — `ScorerSpec` + `build_scorers` factory + `EvalSuite.scorers` (serde default) + CLI uses the factory (heuristic fallback retained). Pure + unit-tested. *Depends on: none. Complexity: M.*
2. **`eval-llm-judge-scorer`** (EHH2) — `LlmJudge` `Scorer` over `CompletionProvider`; JSON verdict parse; wire `llm_judge` into the factory. Unit-test the parser with a stub provider. *Depends on: EHH3 (factory). Complexity: M (parse robustness is the risk).*
3. **`eval-run-integration-coverage`** (EHH4) — recorded-fixture provider + an integration test for the full `run_suite` path (load→run→summarize→compare→persist). *Depends on: none (parallel with EHH2). Complexity: S–M.*
4. **`eval-starter-suite-and-ci-gate`** (EHH1) — ship `evals/starter.yaml`; PR-CI structural eval test (fixture, no key); nightly real-model gated job. *Depends on: EHH3 + EHH4. Complexity: M (CI + secret handling).*

**Order:** EHH3 → (EHH2 ∥ EHH4) → EHH1. *(Optional bundle: G5 dead-`src/testing/` deletion as a tiny standalone cleanup; G6 secret redaction already chipped — both can ride along or stay separate.)*

---

## Key product decisions (for `/kbd-plan`)

- **D-A — CI gate strategy (the crux).** `eval run` needs a live model → can't run on every PR (cost, secrets, fork PRs). Recommend **two-tier**: (1) PR CI runs a *deterministic structural* eval test (fixture provider, rule scorers) so the wiring can't rot; (2) a *nightly/main-only* job runs the real model against `evals/starter.yaml` with a repo secret and gates on regression. Alternative: main-only real run on every push (more token spend). **Pick the tier model + which branch/event triggers the real run.**
- **D-B — LLM-judge non-determinism in a gate.** A judge score varies run-to-run → can cause flaky regression failures. Recommend the **hard gate uses rule scorers; judge scores are advisory/reported** in v1.1 (or gated with a wider threshold). **Confirm judge-in-gate posture.**
- **D-C — judge verdict contract.** Recommend require JSON `{score: 0..1, reason}`; clamp; parse-failure → `0.0` + detail. **Confirm format + failure semantics.**
- **D-D — scorer-config scope.** Suite-level `scorers` (applies to all cases) in v1.1; per-case overrides deferred. **Confirm suite-level is enough.**
- **D-E — housekeeping bundle.** Fold G5 (delete dead `src/testing/`) and/or G6 (secret redaction) into this phase, or leave G6 to its chip and G5 to a standalone? **Confirm.**

---

## Complexity & risk

- **Highest risk: D-A (CI gate).** Getting a real-model gate that's useful without being flaky, expensive, or leaking secrets is the hard part — hence the two-tier recommendation. Surface cost/secret handling explicitly.
- **Medium: LLM-judge parse robustness** — models wander off-format; the parser must be defensive and tested against messy outputs.
- **Low: scorer config + fixture coverage** — additive, serde-default-backward-compatible, no live model.
- **Trivial: G5 deletion** — `src/testing/` is uncompiled; removal cannot break the build (verify with `cargo check`).
- No new dependencies anticipated (Rule 27): reuse `serde`/`serde_json`/`async_trait`/`clap`/existing orchestrator + provider seams.

---

## Assessment status

**COMPLETE.** 6 gaps identified (G1–G4 primary, G5–G6 carried). 4 changes proposed (EHH3 → EHH2 ∥ EHH4 → EHH1) + optional housekeeping. 5 decisions (D-A…D-E) require resolution in `/kbd-plan` — D-A (CI gate strategy) is the pivotal one. No new deps expected. Next: `/kbd-plan eval-harness-hardening`.
