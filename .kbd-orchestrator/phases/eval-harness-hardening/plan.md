PLAN: eval-harness-hardening
Project: universal-agent-runtime · Date: 2026-06-04 · OpenSpec: YES
Planning model: Opus 4.8 (frontier)
Changes to implement: 4 + 1 housekeeping

---

## Decisions resolved (from assessment D-A…D-E)

- **D-A CI gate → two-tier.** PR CI runs a *deterministic structural* eval test (recorded-fixture provider + rule scorers — no API key, no token spend, no fork-PR secret risk). A *separate nightly / main-only* job runs the real model against `evals/starter.yaml` with a repo secret and gates on regression.
- **D-B judge in gate → advisory only.** The hard regression gate uses deterministic rule scorers. LLM-judge scores are computed, persisted, and reported but **do not fail CI** (absorbs judge run-to-run variance).
- **D-C judge verdict → JSON** `{score: 0.0–1.0, reason}`; clamp; parse-failure → `0.0` + detail (never panics).
- **D-D scorer config → suite-level.** A suite declares `scorers` applying to all its cases; per-case overrides deferred.
- **D-E housekeeping → fold in** the dead `src/testing/` deletion; secret redaction (`main.rs:46`) **stays with its spawn-task chip** (out of this phase).

---

## CHANGE LIST (ordered — dependency-driven)

1. **eval-suite-scorer-config** (EHH3): suites declare their scorers; CLI builds from the spec.
   - Scope: `src/uar/eval/mod.rs` (+ maybe new `scorer_spec.rs`), `src/uar/eval/cli.rs`
   - Depends on: NONE
   - Agent: Claude Code · Complexity: M · Model: medium · Value: foundational (unblocks EHH2 + EHH1)
   - Details: add `ScorerSpec` tagged enum — `exact_match`, `contains`, `json_valid`, `non_empty`, `pattern_match { pattern, mode }`, `sycophancy`, `llm_judge { rubric, model? }` (the `llm_judge` variant is **declared here, implemented in EHH2**). Add `EvalSuite.scorers: Vec<ScorerSpec>` with `#[serde(default)]` (backward-compatible — existing suites/tests deserialize unchanged). Add a `build_scorers(suite, provider: &Arc<dyn CompletionProvider>) -> Vec<Arc<dyn Scorer>>` factory: when `suite.scorers` is empty, fall back to today's `select_scorers` heuristic (preserves behavior); else map each spec to its scorer. CLI `run_suite` calls the factory instead of `select_scorers`. Unit-test: spec→scorer mapping, serde default (no `scorers` field → heuristic), round-trip.

2. **eval-llm-judge-scorer** (EHH2): the deferred LLM-as-judge scorer.
   - Scope: `src/uar/eval/` (new `judge.rs` or in `mod.rs`), wire into the EHH3 factory
   - Depends on: EHH3 (factory + `ScorerSpec::LlmJudge`)
   - Agent: Claude Code · Complexity: M (parse robustness is the risk) · Model: frontier · Value: HIGH
   - Details: `LlmJudge { provider: Arc<dyn CompletionProvider>, rubric: String, model: Option<String> }` implementing the async `Scorer` trait. `score()` builds a judge prompt (rubric + case input + candidate output, instructing JSON-only output), calls `provider.complete(prompt)`, parses a JSON verdict `{score: f32 0..1, reason: String}` (tolerant: extract the first JSON object; clamp; on any failure → `Score::new("llm_judge", 0.0, Some(detail))`). Never panics, no `unwrap`. The factory builds it with the run's `CompletionProvider` (judge defaults to `config.llm`). Unit-test the parser against clean JSON, JSON-in-prose, malformed, and out-of-range with a stub provider. **Advisory (D-B):** judge is not part of the hard gate — document + ensure `compare`'s gate path is unaffected by the judge scorer (it just appears in the report/metrics).

3. **eval-run-integration-coverage** (EHH4): cover the end-to-end `run` path without a live model.
   - Scope: `src/uar/eval/` (test-only fixture provider + integration test)
   - Depends on: NONE (parallel with EHH2; lands cleanest after EHH3 so it can assert scorer-config behavior)
   - Agent: Claude Code · Complexity: S–M · Model: medium · Value: HIGH
   - Details: a `#[cfg(test)]` recorded-fixture `CompletionProvider` (canned output per input). An integration test driving the full pure pipeline: `load_suite` (temp file) → `Runner::run` with the fixture + `build_scorers` → `summarize` → `compare` (pass + regress + no-baseline) → `save_results`/`load_baseline` round-trip. Asserts the run path is wired correctly (the gap G4 left uncovered). No orchestrator/live model.

4. **eval-starter-suite-and-ci-gate** (EHH1): make the harness load-bearing.
   - Scope: `evals/starter.{yaml}` (new), `.github/workflows/ci.yml` (structural step) + a nightly workflow (new), maybe a tiny test entrypoint
   - Depends on: EHH3 + EHH4
   - Agent: Claude Code · Complexity: M (CI + secret) · Model: frontier · Value: HIGH (the point of the phase)
   - Details: ship `evals/starter.yaml` — a handful of cases with `expected` + an explicit `scorers:` list (rule scorers for the gate, optionally an advisory `llm_judge`). **Tier 1 (PR CI):** a deterministic structural eval test (uses the EHH4 fixture provider + the starter suite's rule scorers) added to the existing `test` job — no key, no cost, fails if wiring rots. **Tier 2 (nightly/main-only):** a new workflow (`eval-nightly.yml`, `schedule:` + `workflow_dispatch`) that builds the binary and runs `eval run evals/starter.yaml` against the real model using a repo secret (`UAR_LLM__API_KEY`), exiting non-zero on regression. Guard: skip gracefully if the secret is absent (no hard failure on forks). Document the two tiers in the suite/workflow comments + spec.

— **Housekeeping (D-E), standalone, no dependency** —

5. **remove-dead-testing-tree** (HK1): delete the uncompiled `src/testing/` tree.
   - Scope: delete `src/testing/` (8 submodules, not declared in `lib.rs`/`main.rs`)
   - Depends on: NONE (can land anytime)
   - Agent: Claude Code · Complexity: trivial · Model: medium · Value: cleanup
   - Details: confirm no `mod testing;` / path references anywhere (`grep`), `git rm -r src/testing/`, verify `SKIP_FRONTEND_BUILD=1 cargo check --features postgres-backend` still clean. Pure deletion of dead code — cannot change behavior (Rule 32). Spec impact: none (no spec covers it) → archive with `--skip-specs`.

---

## EXECUTION ROUND ORDER

- **Round 1:** EHH3 `eval-suite-scorer-config` (foundational — unblocks EHH2 + EHH1).
- **Round 2 (parallelizable):** EHH2 `eval-llm-judge-scorer` ∥ EHH4 `eval-run-integration-coverage`. Both touch `src/uar/eval/` but in distinct files (`judge.rs` vs test module) — sequence them if diffs collide. HK1 `remove-dead-testing-tree` can also land here (independent, different files).
- **Round 3:** EHH1 `eval-starter-suite-and-ci-gate` (needs EHH3 scorer config + EHH4 fixture).

Per-change workflow (established): branch in main checkout → author OpenSpec artifacts (`/opsx:new` → continue) → implement (`/opsx:apply`) → verify gates → PR → await merge → archive. One PR per change.

## VERIFICATION GATES (every change)

```
SKIP_FRONTEND_BUILD=1 cargo check --features postgres-backend     # clean
SKIP_FRONTEND_BUILD=1 cargo clippy --features postgres-backend    # zero new warnings in touched code
SKIP_FRONTEND_BUILD=1 cargo test --features postgres-backend --lib eval::   # all green
openspec validate <change> --strict                               # valid
rustfmt --edition 2024 <only-touched-files>                       # surgical diffs
```
For EHH1: additionally validate the workflow YAML and dry-run the structural eval test locally; the real-model nightly is verified by manual `workflow_dispatch` post-merge (documented, gated on the secret).

## DEFERRED (not this phase)

- Per-case scorer overrides (D-D); HTTP `POST /api/uar/eval/run`; SurrealDB eval result storage; true-regex scorer.
- Secret redaction at `main.rs:46` — stays with its spawn-task chip (D-E).

## COMMANDS TO RUN

```
/opsx:new eval-suite-scorer-config        # EHH3 (round 1)
/opsx:new eval-llm-judge-scorer           # EHH2 (round 2)
/opsx:new eval-run-integration-coverage   # EHH4 (round 2)
/opsx:new remove-dead-testing-tree        # HK1  (round 2, independent)
/opsx:new eval-starter-suite-and-ci-gate  # EHH1 (round 3)
```
