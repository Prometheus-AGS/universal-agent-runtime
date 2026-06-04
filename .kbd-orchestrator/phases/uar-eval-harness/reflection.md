# REFLECTION: uar-eval-harness

Project: universal-agent-runtime · Date: 2026-06-04 · Backend: OpenSpec
Reflecting model: Opus 4.8 (frontier)
Origin: goal **S1**, deferred from `uar-safety-and-evals` as greenfield/phase-sized.

---

## Goal Achievement

| Goal | Status | Evidence |
| ---- | ------ | -------- |
| **S1** — greenfield eval harness (rule-based, file-backed, CLI-driven) | **MET** | All 4 planned changes (EH1, EH2, EH4, EH5) merged to `main` and archived. The binary now runs `eval run\|list\|baseline`; `run` executes a golden suite through the orchestrator, scores it, persists results, compares to a baseline, and exits non-zero on regression (CI gate). |

**Overall: 1/1 goal MET (100%).** v1 scope as planned — rule-based scorers, file storage, CLI surface. The deferred items below were *intentional scope cuts at plan time* (D4 + the explicit DEFERRED list), not unmet work.

---

## Delivered Changes

| # | Change (OpenSpec) | What landed | Verified |
| - | ----------------- | ----------- | -------- |
| EH1 | `eval-domain-and-rule-scorers` | `src/uar/eval/mod.rs` — `EvalCase`/`EvalSuite`/`Score`/`EvalResult`, `Scorer` trait, rule scorers `ExactMatch`/`Contains`/`JsonValid`/`NonEmpty`/`PatternMatch` + `Sycophancy` adapter (`1 − quality::detect`). | unit tests (clamp, round-trip, each scorer) |
| EH2 | `eval-suite-loading-and-runner` | `runner.rs` — `load_suite` (JSON/YAML by ext), `CompletionProvider` trait, `Runner::run` (per-case errors contained as a `completion`=0.0 score). | unit tests with stub providers (Echo/Failing) |
| EH4 | `eval-persistence-and-regression` | `persistence.rs` — `summarize`, `compare` (delta-vs-baseline), `save_results`/`save_baseline`/`load_baseline`; metrics `record_eval_score` (gauge) + `record_eval_regression` (counter). | unit tests (means, regress/no-baseline, round-trip) |
| EH5 | `eval-cli-subcommand` | `config.rs` (`Command`/`EvalAction`), `main.rs` dispatch + `process::exit`, `cli.rs` (`OrchestratorCompletionProvider`, suite resolution, scorer selection, report). | unit tests (resolve/select) + manual smoke (help, exit codes, server-default preserved) |

All four archived under `openspec/changes/archive/2026-06-04-eval-*`; main specs updated (`openspec/specs/eval-harness/`).

---

## Artifact Quality Summary

| Metric | Value |
| ------ | ----- |
| Changes with artifact-refiner QA | 0/4 |
| First-pass pass rate | n/a (refiner not run for this phase) |
| Verification method | Manual gates per change |

No `.refiner/artifacts/` logs exist for the eval changes — QA was enforced via **manual verification gates** instead, applied to every change:
`SKIP_FRONTEND_BUILD=1 cargo check --features postgres-backend` (clean) · `cargo clippy` (zero new warnings in touched code) · `cargo test --features postgres-backend --lib eval::` (all green) · `openspec validate <change> --strict` (valid) · surgical-diff discipline (`rustfmt --edition 2024` on touched files only).

### Recurring constraint observations (self-enforced, not refiner)
- **Zero-new-warnings** held across all 4 — clippy nits fixed as they appeared (`let...else`, `redundant_closure_for_method_calls`, `case_sensitive_file_extension_comparisons`, `missing_debug_implementations`, `similar_names`).
- **No silent dependency introduction (Rule 27)** held — no new crates; reused `clap`, `serde`/`serde_yaml`, `async_trait`, existing `quality::detect`, `metrics`.

---

## Deviations from Plan (honest accounting)

1. **`Regex` scorer → `PatternMatch`.** EH1's plan listed a `Regex` scorer; I shipped `PatternMatch` (literal `Contains`/`StartsWith`/`EndsWith`) to avoid adding a `regex` dependency for marginal value (Rule 27). A true-regex scorer is a documented follow-up. *Net: same capability surface for the common cases, no new dep.*
2. **`Scorer::name -> &'static str`** (plan said `&str`) — required by clippy's lifetime lint; behaviorally identical.
3. **Default scorer selection is a heuristic** (`[NonEmpty, Sycophancy]`, plus `[ExactMatch, Contains]` when *every* case has `expected`). The plan implied per-suite scorers ("the suite's configured scorers"); per-suite/declared scorer config was **not** built in v1 and is a follow-up. This is the one place v1 is thinner than the plan's wording.

---

## Technical Debt Introduced / Carried

- **Live `run <suite>` path is not covered by automated tests** — it needs a configured model. Pure pieces (loader, scorers, summarize/compare, resolve/select) are unit-tested; the orchestrator-backed completion is exercised only by manual smoke (help/exit-codes). *Risk: medium — the integration glue could regress silently.* Mitigation candidate: a recorded-fixture `CompletionProvider` test, or a gated live smoke in CI.
- **No example suite shipped** under `evals/` — the first user must author one before `eval run` does anything. A 3–5 case starter suite + a README snippet would make the CI gate immediately usable.
- **Per-suite scorer configuration deferred** (see deviation #3) — suites can't yet declare which scorers apply.
- **Dead `src/testing/` tree still present** — housekeeping explicitly deferred; not deleted/compile-gated this phase.
- **Pre-existing (flagged, not introduced):** `main.rs:46` logs the full `AppConfig` at INFO, exposing LLM API keys, provider keys, and the JWT secret in plaintext — a **Rule 33** violation. Flagged as a spawn-task security follow-up (redact secrets in `Debug`); out of EH5 scope.

---

## Lessons Captured (for knowledge base)

1. **The `CompletionProvider` seam was the highest-leverage decision.** Decoupling `Runner` from `Orchestrator` behind a trait made every non-LLM piece fully unit-testable without a live model, and let EH2 land + be verified before the orchestrator wiring existed (EH5). Repeat this pattern wherever a unit needs an LLM only at the edges.
2. **Build the model bottom-up so each change verifies in isolation.** EH1→EH2→EH4→EH5 sequencing meant each PR compiled, tested, and validated on its own; no change depended on un-merged work. The shared `src/uar/eval/` module grew cleanly.
3. **`rustfmt --edition 2024 <only-my-files>` beats repo-wide `cargo fmt`** for surgical diffs — repo-wide formatting kept reflowing unrelated files (recurred across the whole KBD run). Make this the standing convention.
4. **Disk headroom is a real gate.** The final link failed with `errno 28` (No space left) even though `cargo check` passed — linking the full binary needs GBs of headroom that `check` doesn't. Stale worktrees accumulate (`~/.claude/worktrees/`); prune merged ones periodically. Reclaimed 2.5 GB this phase.
5. **Heuristic defaults are a debt signal.** The scorer-selection heuristic works for homogeneous suites but is the seam most likely to surprise users — when a v1 cut replaces a config point with a heuristic, log it loudly (done in code comments + design D5) and queue the real config.

---

## Recommended Focus for Next Phase

A short **eval-harness-hardening** fast-follow (or fold into the next safety/quality phase):

1. **Example suite + CI wiring** — ship `evals/<starter>.yaml` (3–5 cases) + a CI step running `eval run` as a regression gate. Makes the harness load-bearing. *(highest value, smallest effort)*
2. **EH3 LLM-as-judge scorer** — rubric via `chat_non_streaming` + deterministic numeric parse (the one deferred v1 scorer).
3. **Per-suite scorer configuration** — let a suite declare its scorers; removes the EH5 heuristic.
4. **Integration coverage for `run`** — recorded-fixture provider test or gated live smoke.
5. **Housekeeping:** delete/compile-gate dead `src/testing/`; address the secret-logging follow-up (already chipped).
6. **Later:** HTTP `POST /api/uar/eval/run`; SurrealDB result storage; absolute-floor gate option; true-regex scorer.

---

## Phase Status

**uar-eval-harness — COMPLETE.** Goal S1 MET; 4/4 planned changes delivered, verified, merged, archived; specs updated. Deferred items are tracked above and were intentional scope cuts, not gaps. Advance with `/kbd-new-phase`.
