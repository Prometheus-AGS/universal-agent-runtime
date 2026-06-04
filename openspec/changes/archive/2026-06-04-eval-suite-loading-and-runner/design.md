## Context

EH1 provides `EvalCase/EvalSuite/Score/EvalResult` + `Scorer` (`src/uar/eval/mod.rs`). Deps available: `serde_yaml` 0.9, `serde_json`, `chrono`, `async-trait`, `tokio`. The orchestrator's one-shot is `Orchestrator::chat_non_streaming(Vec<Message>) -> Result<String>` (wired in EH5, not here).

## Goals / Non-Goals
**Goals:** load suites from JSON/YAML; a runner that scores each case via a pluggable provider; contain per-case errors; unit-testable without an LLM.
**Non-Goals:** persistence/regression (EH4), CLI/HTTP (EH5), LLM-judge, full-agent-run, suite-level per-case scorer selection (runner applies one scorer set to all cases in v1).

## Decisions
- **D1 — `CompletionProvider` trait** (`#[async_trait] async fn complete(&self, input: &str) -> anyhow::Result<String>`). The runner takes `&dyn CompletionProvider`. Decouples from the orchestrator → tests use a stub; EH5 supplies an orchestrator-backed impl. (Key testability decision.)
- **D2 — loader by extension:** `.json` → `serde_json::from_str`, `.yaml`/`.yml` → `serde_yaml::from_str`; unknown extension → error. Read via `std::fs`/`tokio::fs`. Missing/parse failure → `anyhow::Result` error.
- **D3 — Runner signature:** `Runner::run(&EvalSuite, scorers: &[Arc<dyn Scorer>], provider: &dyn CompletionProvider, model: Option<&str>) -> Vec<EvalResult>`. One scorer set applied to every case (suite-scoped scorer config is a later refinement). `run_at = chrono::Utc::now().to_rfc3339()`.
- **D4 — error containment:** on `provider.complete` Err, push an `EvalResult` with a single `Score::new("completion", 0.0, Some(err))` and continue; scorers are not run (no output). On Ok, run all scorers.
- **D5 — file layout:** add `src/uar/eval/runner.rs`; `mod.rs` declares `mod runner; pub use runner::{CompletionProvider, Runner, load_suite};`.

## Risks / Trade-offs
- **[serde_yaml maintenance]** serde_yaml 0.9 is in maintenance mode → Mitigation: already a project dep; acceptable; JSON is the primary path.
- **[One scorer set per suite]** no per-case scorer selection in v1 → Mitigation: documented; sufficient for v1 golden suites; refine later.
- **[run_at non-determinism]** timestamp varies → Mitigation: not asserted in unit tests (tests check counts/scores, not run_at).

## Migration Plan
1. Add `runner.rs`: `CompletionProvider`, `load_suite`, `Runner` + re-exports in `mod.rs`.
2. Unit tests: a stub provider; loader JSON + YAML (from string/temp); N cases → N results; error containment.
3. `cargo check`/`clippy`/tests.
- Rollback: additive; revert removes the runner. EH1 unaffected.

## Open Questions
- Suite-level scorer selection (which scorers per suite/case)? Defer to a later refinement; v1 caller picks one set.
