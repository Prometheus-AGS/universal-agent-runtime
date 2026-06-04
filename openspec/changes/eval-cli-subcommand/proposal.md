# eval-cli-subcommand

## Why

EH1–EH4 built the eval domain, scorers, runner, persistence, and regression logic — but nothing invokes them. This change (EH5) adds the **runner surface** (decision D2): a CLI subcommand that runs a golden suite through the real orchestrator, scores it, persists results, compares to a baseline, prints a report, and **exits non-zero on regression** (the CI gate). It completes the v1 harness.

## What Changes

- **CLI restructure:** add `#[command(subcommand)] command: Option<Command>` to `Cli` (`config.rs`) with `enum Command { Eval { action: EvalAction } }` and `EvalAction::{ Run, List, Baseline }`. Existing global flags are unchanged; `command == None` runs the server exactly as today.
- **Dispatch (`main.rs`):** take the subcommand out before `load_with_cli`; `None` ⇒ run the server (unchanged); `Some(Eval)` ⇒ run the eval path and `exit(code)`.
- **Eval CLI (`src/uar/eval/cli.rs`):**
  - `OrchestratorCompletionProvider` implementing `CompletionProvider` over a minimal `Orchestrator` (built from `config.llm` + `McpRegistry::empty()` + `NativeSkillRegistry::new()`; `chat_non_streaming`, no tools).
  - `run_eval(config, action) -> i32`:
    - **Run `<suite>`**: resolve `evals/<suite>.{yaml,json}` (or a direct path), pick a default scorer set, run via the orchestrator provider, `summarize`, `load_baseline`, `compare(threshold)`, record metrics, `save_results`, print a per-scorer report; `--update-baseline` writes the new baseline; exit `1` on regression else `0`.
    - **List**: list result files for a suite (or all) under the results dir.
    - **Baseline `<suite>`**: print the stored baseline summary.

Out of scope: LLM-as-judge (deferred), full-agent-run mode, HTTP endpoint, per-suite scorer configuration (v1 uses a default set; expected-based scorers added when the suite's cases carry `expected`).

## Capabilities

### Modified Capabilities
- **`eval-harness`** — delta `specs/eval-harness/spec.md`. Adds the runnable surface: a CLI that executes a suite, reports scores + regressions, and signals regression via exit code; the server path is unchanged when no subcommand is given.

## Impact

- **Affected code:** `src/config.rs` (`Command`/`EvalAction` + `Cli.command`), `src/main.rs` (dispatch), `src/uar/eval/cli.rs` (new — provider + `run_eval`), `src/uar/eval/mod.rs` (`pub mod cli`). Reuses EH1–EH4 + `Orchestrator::chat_non_streaming` + `McpRegistry::empty` + `NativeSkillRegistry::new`. No new dependency (clap already present).
- **Behavior preservation (Rule 32):** the default (no-subcommand) invocation runs the server exactly as before; only `eval …` triggers the new path.
- **CI value:** `uar eval run <suite>` exits non-zero on regression — a drop-in regression gate.
- **KBD workflow state:** YES — EH5, the final change of `uar-eval-harness` v1.
