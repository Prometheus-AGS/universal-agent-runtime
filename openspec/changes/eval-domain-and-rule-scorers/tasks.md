# Tasks — eval-domain-and-rule-scorers

## 0. Bootstrap
- [ ] 0.1 Confirm reuse: `quality::detect` (+ `SycophancyOutcome.score`), `serde_json` dep, `async_trait` availability; no `regex` crate
- [ ] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Module + domain (src/uar/eval/)
- [ ] 1.1 Create `src/uar/eval/mod.rs` (+ submodules); register `pub mod eval;` in `src/uar/mod.rs`
- [ ] 1.2 Domain types: `EvalCase`, `EvalSuite`, `Score` (with clamping `Score::new`), `EvalResult` — all `Serialize`/`Deserialize`

## 2. Scorer trait + rule scorers
- [ ] 2.1 `#[async_trait] pub trait Scorer { fn name(&self) -> &str; async fn score(&self, case: &EvalCase, output: &str) -> Score }`
- [ ] 2.2 Rule scorers: `ExactMatch`, `Contains`, `PatternMatch` (substring/anchors — no regex dep), `JsonValid`, `NonEmpty`
- [ ] 2.3 `Sycophancy` adapter: `quality::detect(&SycophancyConfig::default(), output)`; value = `1.0 - score` (None ⇒ 1.0); detail = pattern ids

## 3. Tests
- [ ] 3.1 Domain serde round-trip (EvalResult preserves case_id/scores/metadata)
- [ ] 3.2 Each scorer positive + negative; Score value always in 0.0–1.0
- [ ] 3.3 Sycophancy adapter: clean output ≈1.0, flagged output lower

## 4. Validation (gate)
- [ ] 4.1 `cargo check --features postgres-backend` clean; zero new warnings
- [ ] 4.2 `cargo clippy` — no new warnings in the new module
- [ ] 4.3 `cargo test --features postgres-backend --lib eval::` — new tests pass; full lib suite unaffected
- [ ] 4.4 `openspec validate eval-domain-and-rule-scorers --strict`; update `.kbd-orchestrator` progress

## Notes
- Foundation only — no runner/IO/CLI/LLM (EH2/EH4/EH5). Nothing invokes it yet ⇒ zero runtime change.
- No new dependency (substring "regex"); true regex deferred. Sycophancy scoring uses default config (independent of runtime gating).
