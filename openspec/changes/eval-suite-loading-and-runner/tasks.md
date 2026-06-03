# Tasks — eval-suite-loading-and-runner

## 0. Bootstrap
- [ ] 0.1 Confirm EH1 types + deps (serde_yaml 0.9, serde_json, chrono, async-trait) available
- [ ] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Suite loader
- [ ] 1.1 `pub fn load_suite(path: &Path) -> anyhow::Result<EvalSuite>` — parse by extension (.json/.yaml/.yml); error on missing/malformed/unknown-ext

## 2. CompletionProvider + Runner (src/uar/eval/runner.rs)
- [ ] 2.1 `#[async_trait] pub trait CompletionProvider: Send + Sync { async fn complete(&self, input: &str) -> anyhow::Result<String> }`
- [ ] 2.2 `pub struct Runner;` + `pub async fn run(&self, suite: &EvalSuite, scorers: &[Arc<dyn Scorer>], provider: &dyn CompletionProvider, model: Option<&str>) -> Vec<EvalResult>`
- [ ] 2.3 Per case: `provider.complete(input)` → Ok: run all scorers → `EvalResult`; Err: failed result (`Score::new("completion",0.0,Some(err))`), continue (D4). `run_at = chrono::Utc::now().to_rfc3339()`
- [ ] 2.4 Re-export from `mod.rs` (`mod runner; pub use runner::{CompletionProvider, Runner, load_suite};`)

## 3. Tests
- [ ] 3.1 Stub `CompletionProvider` (returns fixed/echo output, or error)
- [ ] 3.2 Loader: parse a JSON suite + a YAML suite from string; missing/malformed → error
- [ ] 3.3 Runner: N cases → N results, each with one score per scorer
- [ ] 3.4 Error containment: provider error → failed result + remaining cases still run

## 4. Validation (gate)
- [ ] 4.1 `cargo check --features postgres-backend` clean; zero new warnings
- [ ] 4.2 `cargo clippy` — no new warnings in the eval module
- [ ] 4.3 `cargo test --features postgres-backend --lib eval::` — new tests pass; full lib suite unaffected
- [ ] 4.4 `openspec validate eval-suite-loading-and-runner --strict`; update `.kbd-orchestrator` progress

## Notes
- Runner decoupled from the orchestrator via CompletionProvider (testable without an LLM); EH5 supplies the orchestrator-backed impl.
- One scorer set per suite in v1 (per-case selection deferred). No new dependency (serde_yaml already present).
