# Tasks — eval-cli-subcommand

## 0. Bootstrap
- [ ] 0.1 Confirm EH1–EH4 exports + `Orchestrator::new`/`chat_non_streaming`, `McpRegistry::empty`, `NativeSkillRegistry::new`, `config.llm`
- [ ] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. CLI types (config.rs)
- [ ] 1.1 Add `#[command(subcommand)] pub command: Option<Command>` to `Cli`
- [ ] 1.2 `enum Command { Eval { action: EvalAction } }` + `enum EvalAction { Run{suite, threshold=0.05, results_dir="evals/results", update_baseline}, List{suite:Option, results_dir}, Baseline{suite, results_dir} }` (derive Subcommand, Debug, Clone)

## 2. Eval CLI module (src/uar/eval/cli.rs)
- [ ] 2.1 `OrchestratorCompletionProvider` (CompletionProvider over Orchestrator built from config.llm + empty mcp/native); `complete` = chat_non_streaming(user msg)
- [ ] 2.2 `resolve_suite_path(suite) -> Option<PathBuf>` (path or evals/<suite>.{yaml,yml,json}) — pure, tested
- [ ] 2.3 `select_scorers(&EvalSuite) -> Vec<Arc<dyn Scorer>>` ([NonEmpty,Sycophancy]; + [ExactMatch,Contains] when all cases have expected) — pure, tested
- [ ] 2.4 `pub async fn run_eval(config: &AppConfig, action: &EvalAction) -> i32` — Run/List/Baseline; Run: load → run → summarize → load_baseline → compare → metrics → save_results → report → exit code; --update-baseline saves baseline; errors → stderr + non-zero

## 3. Dispatch (main.rs)
- [ ] 3.1 `let command = cli.command.take();` before `load_with_cli`; `None` → start_server (unchanged); `Some(Eval{action})` → run_eval, flush otel, `process::exit(code)`

## 4. Validation (gate)
- [ ] 4.1 `cargo check --features postgres-backend` clean; zero new warnings
- [ ] 4.2 `cargo clippy` — no new warnings in touched code
- [ ] 4.3 `cargo test --features postgres-backend --lib eval::` — existing + new (resolve_suite_path, select_scorers) pass
- [ ] 4.4 Manual: no-arg still starts the server; `eval --help` lists run/list/baseline; `eval run <suite>` against a model reports + gates (pending live env)
- [ ] 4.5 `openspec validate eval-cli-subcommand --strict`; update `.kbd-orchestrator` progress

## Notes
- Server path unchanged when no subcommand (Rule 32). Default scorers [NonEmpty,Sycophancy] (+expected-based when all cases have expected); per-suite scorer config is a follow-up. Run needs a live model (not unit-tested); pure helpers are. No new dependency.
