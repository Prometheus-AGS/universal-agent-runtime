# Tasks — eval-domain-and-rule-scorers

## 0. Bootstrap
- [x] 0.1 Confirm reuse: `quality::detect` (+ `SycophancyOutcome.score`), `serde_json` dep, `async_trait` availability; no `regex` crate
- [x] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Module + domain (src/uar/eval/)
- [x] 1.1 Create `src/uar/eval/mod.rs` (+ submodules); register `pub mod eval;` in `src/uar/mod.rs`
- [x] 1.2 Domain types: `EvalCase`, `EvalSuite`, `Score` (with clamping `Score::new`), `EvalResult` — all `Serialize`/`Deserialize`

## 2. Scorer trait + rule scorers
- [x] 2.1 `#[async_trait] pub trait Scorer { fn name(&self) -> &str; async fn score(&self, case: &EvalCase, output: &str) -> Score }`
- [x] 2.2 Rule scorers: `ExactMatch`, `Contains`, `PatternMatch` (substring/anchors — no regex dep), `JsonValid`, `NonEmpty`
- [x] 2.3 `Sycophancy` adapter: `quality::detect(&SycophancyConfig::default(), output)`; value = `1.0 - score` (None ⇒ 1.0); detail = pattern ids

## 3. Tests
- [x] 3.1 Domain serde round-trip (EvalResult preserves case_id/scores/metadata)
- [x] 3.2 Each scorer positive + negative; Score value always in 0.0–1.0
- [x] 3.3 Sycophancy adapter: clean output ≈1.0, flagged output lower

## 4. Validation (gate)
- [x] 4.1 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 4.2 `cargo clippy` — no new warnings in the new module
- [x] 4.3 `cargo test --features postgres-backend --lib eval::` — new tests pass; full lib suite unaffected
- [x] 4.4 `openspec validate eval-domain-and-rule-scorers --strict`; update `.kbd-orchestrator` progress

## Notes
- Foundation only — no runner/IO/CLI/LLM (EH2/EH4/EH5). Nothing invokes it yet ⇒ zero runtime change.
- No new dependency (substring "regex"); true regex deferred. Sycophancy scoring uses default config (independent of runtime gating).
