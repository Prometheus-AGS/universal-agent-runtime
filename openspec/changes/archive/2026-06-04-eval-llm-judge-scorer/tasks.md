# Tasks — eval-llm-judge-scorer

## 1. Judge scorer (judge.rs)
- [x] 1.1 `LlmJudge { provider: Arc<dyn CompletionProvider>, rubric: String }` impl async `Scorer` (name "llm_judge")
- [x] 1.2 `judge_prompt(rubric, input, output)` — instruct JSON-only `{score, reason}`
- [x] 1.3 `extract_json_object(&str) -> Option<String>` (first `{` … last `}`)
- [x] 1.4 `parse_verdict(&str) -> Score` — serde into `Verdict{score:f32, reason}`, clamp; failure → 0.0 + detail (no panic)
- [x] 1.5 Tests: clean JSON, JSON-in-prose, malformed, out-of-range (clamped); `LlmJudge::score` via a stub provider

## 2. Factory wiring (scorer_spec.rs)
- [x] 2.1 Add `ScorerSpec::LlmJudge { rubric: String }`
- [x] 2.2 `ScorerSpec::build(&self, provider: &Arc<dyn CompletionProvider>)` + `build_scorers(suite, provider)`
- [x] 2.3 Update existing factory tests to pass a stub provider

## 3. Re-export (mod.rs)
- [x] 3.1 `mod judge; pub use judge::LlmJudge;`

## 4. CLI (cli.rs)
- [x] 4.1 Provider as `Arc<dyn CompletionProvider>`; `build_scorers(&suite, &provider)`; `Runner.run(.., provider.as_ref(), ..)`

## 5. Validation (gate)
- [x] 5.1 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 5.2 `cargo clippy` — no new warnings in touched code
- [x] 5.3 `cargo test --features postgres-backend --lib eval::` green
- [x] 5.4 `openspec validate eval-llm-judge-scorer --strict`; update progress

## Notes
- Advisory only (D-B): judge adds a reported score; hard gate stays rule-scorer means (no `compare` change). No new dep. v1 uses the run's provider/model (no per-judge override).
