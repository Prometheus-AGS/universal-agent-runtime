//! Eval harness CLI surface (EH5).
//!
//! Wires the runner + persistence + regression into the `eval` subcommand:
//! `run` executes a suite through a tool-less orchestrator, scores it, persists
//! results, compares to a baseline, prints a report, and returns a non-zero exit
//! code on regression (a CI gate). `list`/`baseline` inspect stored data.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;

use crate::config::{AppConfig, EvalAction};
use crate::llm::orchestrator::Orchestrator;
use crate::llm::{Message, MessageContent, MessageRole};
use crate::mcp::registry::McpRegistry;
use crate::uar::eval::{
    CompletionProvider, Runner, ScoreSummary, build_scorers, compare, load_baseline, load_suite,
    save_baseline, save_results, summarize,
};
use crate::uar::runtime::native_skill::NativeSkillRegistry;
use crate::uar::telemetry::metrics;

/// A [`CompletionProvider`] backed by a tool-less [`Orchestrator`] (one-shot,
/// non-streaming) — the real-model output source for `eval run`.
struct OrchestratorCompletionProvider {
    orchestrator: Orchestrator,
}

#[async_trait]
impl CompletionProvider for OrchestratorCompletionProvider {
    async fn complete(&self, input: &str) -> anyhow::Result<String> {
        let messages = vec![Message {
            role: MessageRole::User,
            content: MessageContent::text(input),
            tool_call_id: None,
            tool_calls: None,
        }];
        self.orchestrator.chat_non_streaming(messages).await
    }
}

/// Resolve a suite argument to a file path: a direct path if it exists, else
/// `evals/<suite>.{yaml,yml,json}` (first that exists).
fn resolve_suite_path(suite: &str) -> Option<PathBuf> {
    let direct = Path::new(suite);
    if direct.is_file() {
        return Some(direct.to_path_buf());
    }
    for ext in ["yaml", "yml", "json"] {
        let p = PathBuf::from("evals").join(format!("{suite}.{ext}"));
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn print_summary(summary: &ScoreSummary) {
    for (scorer, mean) in summary {
        println!("  {scorer:<16} {mean:.3}");
    }
}

/// Execute an `eval` subcommand. Returns a process exit code (0 ok, 1 regression,
/// 2 usage/IO error). Never panics.
pub async fn run_eval(config: &AppConfig, action: &EvalAction) -> i32 {
    match action {
        EvalAction::Run {
            suite,
            threshold,
            results_dir,
            update_baseline,
        } => run_suite(config, suite, *threshold, results_dir, *update_baseline).await,
        EvalAction::List { suite, results_dir } => list_results(results_dir, suite.as_deref()),
        EvalAction::Baseline { suite, results_dir } => print_baseline(results_dir, suite),
    }
}

async fn run_suite(
    config: &AppConfig,
    suite: &str,
    threshold: f32,
    results_dir: &str,
    update_baseline: bool,
) -> i32 {
    let Some(path) = resolve_suite_path(suite) else {
        eprintln!(
            "eval: suite '{suite}' not found (looked for a path or evals/{suite}.{{yaml,yml,json}})"
        );
        return 2;
    };
    let suite_obj = match load_suite(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("eval: {e}");
            return 2;
        }
    };

    let orchestrator = match Orchestrator::new(
        config.llm.clone(),
        Arc::new(McpRegistry::empty()),
        Arc::new(NativeSkillRegistry::new()),
    ) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("eval: failed to build orchestrator: {e}");
            return 2;
        }
    };
    let provider: Arc<dyn CompletionProvider> =
        Arc::new(OrchestratorCompletionProvider { orchestrator });
    let scorers = build_scorers(&suite_obj, &provider);

    let results = Runner
        .run(
            &suite_obj,
            &scorers,
            provider.as_ref(),
            Some(&config.llm.model),
        )
        .await;
    let summary = summarize(&results);

    let dir = Path::new(results_dir);
    let ts = chrono::Utc::now().to_rfc3339();
    if let Err(e) = save_results(dir, &suite_obj.name, &results, &ts) {
        eprintln!("eval: warning — failed to save results: {e}");
    }
    for (scorer, mean) in &summary {
        metrics::record_eval_score(&suite_obj.name, scorer, f64::from(*mean));
    }

    if update_baseline {
        return match save_baseline(dir, &suite_obj.name, &summary) {
            Ok(()) => {
                println!("Baseline updated for suite '{}':", suite_obj.name);
                print_summary(&summary);
                0
            }
            Err(e) => {
                eprintln!("eval: failed to save baseline: {e}");
                2
            }
        };
    }

    let baseline = load_baseline(dir, &suite_obj.name)
        .unwrap_or(None)
        .unwrap_or_default();
    let report = compare(&summary, &baseline, threshold);
    println!(
        "Eval '{}' ({} cases):",
        suite_obj.name,
        suite_obj.cases.len()
    );
    for e in &report.entries {
        let base = e
            .baseline_mean
            .map_or_else(|| "—".to_string(), |b| format!("{b:.3}"));
        let delta = e
            .delta
            .map_or_else(String::new, |d| format!(" (Δ {d:+.3})"));
        let flag = if e.regressed { "  REGRESSED" } else { "" };
        println!(
            "  {:<16} {:.3}  baseline {}{}{}",
            e.scorer, e.current_mean, base, delta, flag
        );
    }
    if report.any_regressed {
        metrics::record_eval_regression();
        eprintln!("eval: regression detected vs baseline");
        1
    } else {
        0
    }
}

fn list_results(results_dir: &str, suite: Option<&str>) -> i32 {
    let dir = Path::new(results_dir);
    let Ok(entries) = std::fs::read_dir(dir) else {
        println!("(no results in '{results_dir}')");
        return 0;
    };
    let mut names: Vec<String> = entries
        .filter_map(std::result::Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| {
            Path::new(n)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("json"))
                && !n.ends_with(".baseline.json")
        })
        .filter(|n| suite.is_none_or(|s| n.starts_with(s)))
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(no matching result files)");
    } else {
        for n in names {
            println!("{n}");
        }
    }
    0
}

fn print_baseline(results_dir: &str, suite: &str) -> i32 {
    match load_baseline(Path::new(results_dir), suite) {
        Ok(Some(summary)) => {
            println!("Baseline for suite '{suite}':");
            print_summary(&summary);
            0
        }
        Ok(None) => {
            println!("(no baseline for suite '{suite}')");
            0
        }
        Err(e) => {
            eprintln!("eval: {e}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_suite_path;

    #[test]
    fn resolve_direct_path() {
        let dir = std::env::temp_dir();
        let path = dir.join("uar_eval_cli_direct.json");
        std::fs::write(&path, r#"{"name":"x","cases":[]}"#).unwrap();
        let resolved = resolve_suite_path(path.to_str().unwrap());
        assert_eq!(resolved.as_deref(), Some(path.as_path()));
        let _ = std::fs::remove_file(&path);
        assert!(resolve_suite_path("definitely-missing-suite-xyz").is_none());
    }
}
