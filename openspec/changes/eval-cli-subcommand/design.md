## Context

EH1–EH4 provide domain, scorers, `Runner`, `CompletionProvider`, `load_suite`, `summarize`/`compare`, file persistence, and metrics. The binary's `Cli` (`config.rs`) is a flat `clap::Parser`; `main` calls `AppConfig::load_with_cli(cli)` then `start_server`. `Orchestrator::new(llm_config, Arc<McpRegistry>, Arc<NativeSkillRegistry>)` + `chat_non_streaming` exist; `McpRegistry::empty()` and `NativeSkillRegistry::new()` give a tool-less orchestrator.

## Goals / Non-Goals
**Goals:** a CLI `eval run|list|baseline`; run a suite through the real model; persist + compare + report; non-zero exit on regression; server path unchanged.
**Non-Goals:** LLM-judge, full-agent-run, HTTP endpoint, per-suite scorer config.

## Decisions
- **D1 — Cli subcommand, server default:** add `#[command(subcommand)] command: Option<Command>` to `Cli`; `Command::Eval { action: EvalAction }`; `EvalAction::{ Run { suite, --threshold (default 0.05), --results-dir (default "evals/results"), --update-baseline }, List { --suite, --results-dir }, Baseline { suite, --results-dir } }`. `Command`/`EvalAction` derive `Subcommand, Debug, Clone`. In `main`, `let command = cli.command.take();` before `load_with_cli(cli)` (no Clone needed; `load_with_cli` ignores the field).
- **D2 — dispatch + exit:** `main`: `match command { None => start_server(config).await, Some(Command::Eval{action}) => { let code = uar::eval::cli::run_eval(&config, &action).await; flush otel; std::process::exit(code); } }`.
- **D3 — provider:** `OrchestratorCompletionProvider { orch: Orchestrator }` built from `config.llm.clone()` + `McpRegistry::empty()` + `NativeSkillRegistry::new()`; `complete(input)` = `orch.chat_non_streaming(vec![user Message(input)])`.
- **D4 — suite resolution:** `run <suite>`: if `<suite>` is an existing path use it; else try `evals/<suite>.yaml`, `.yml`, `.json` (first that exists). Error (non-zero exit) if none.
- **D5 — default scorer set:** `[NonEmpty, Sycophancy]`; if EVERY case has `expected`, prepend `[ExactMatch, Contains]` (expected-based suites). Per-suite scorer config is a follow-up. (Avoids penalizing suites without expected outputs.)
- **D6 — report + metrics + exit code:** after `compare`, for each scorer `record_eval_score(suite, scorer, mean)`; if `any_regressed` `record_eval_regression()`. Print a readable per-scorer table (mean, baseline, delta, regressed). `save_results` always. `--update-baseline` ⇒ `save_baseline(summary)` + exit 0. Else exit `i32::from(report.any_regressed)`.
- **D7 — location:** `src/uar/eval/cli.rs`; `mod.rs` adds `pub mod cli;`. `run_eval` returns `i32` (no panics; errors print to stderr + return non-zero).

## Risks / Trade-offs
- **[Cli restructure]** adding a subcommand could change arg parsing → Mitigation: `command` is optional; all existing flags stay as globals; `None` path is byte-for-byte the prior behavior; verify the server still starts with no args.
- **[Eval needs network/model]** `run` calls the real LLM → Mitigation: not exercised in unit tests (the run path needs a live model — documented; pure pieces are already tested in EH1–EH4). `run_eval`'s non-LLM branches (list/baseline/suite-resolution) are testable.
- **[Default scorer choice]** a fixed default set may not fit every suite → Mitigation: D5 heuristic + documented follow-up for per-suite config.
- **[process::exit]** skips Drop → acceptable for a CLI; otel flush done before exit.

## Migration Plan
1. `config.rs`: `Command`/`EvalAction` + `Cli.command`.
2. `src/uar/eval/cli.rs`: provider + `run_eval` (+ suite resolution + scorer selection + report); `pub mod cli` in `mod.rs`.
3. `main.rs`: take command, dispatch, exit.
4. Tests: suite-path resolution + scorer-selection helpers (pure); `cargo check`/`clippy`/tests; manual `eval run` against a model (pending env); verify `--help` shows the subcommand and no-arg still starts the server.
- Rollback: additive (optional subcommand + new module); revert restores server-only binary.

## Open Questions
- Per-suite scorer configuration (declare scorers in the suite file)? Follow-up after v1.
