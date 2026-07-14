//! Axum + Leptos + HTMX + Web Components Server
//!
//! Entry point for the agentic streaming LLM application.

use mimalloc::MiMalloc;

/// Global allocator for improved performance (M-MIMALLOC-APPS).
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use clap::Parser as _;
use dotenvy::dotenv;
use universal_agent_runtime::config::{Cli, Command, LogFormat};
use universal_agent_runtime::config_manager::ConfigManager;
use universal_agent_runtime::server;
use universal_agent_runtime::uar;

#[tokio::main]
async fn main() {
    let _ = dotenv();

    // Resolve log format early (before full config load) so telemetry
    // is initialized with the correct format from the start.
    let log_format = std::env::var("UAR_SERVER__LOG_FORMAT")
        .ok()
        .and_then(|v| match v.to_lowercase().as_str() {
            "json" => Some(LogFormat::Json),
            "compact" => Some(LogFormat::Compact),
            "pretty" => Some(LogFormat::Pretty),
            _ => None,
        })
        .unwrap_or_default();

    let otel_provider = uar::telemetry::init(&log_format);
    uar::telemetry::metrics::init();

    let mut cli = Cli::parse();
    // Take the subcommand out before consuming `cli`; load_with_cli ignores it.
    let command = cli.command.take();
    let strict_config = cli.strict_config;

    let config_manager = match ConfigManager::load(cli).await {
        Ok(m) => {
            // Strict mode is enabled if the operator passed `--strict-config` or set
            // `UAR_STRICT_CONFIG=true`. It makes any runtime reload that would change
            // the effective configuration an error.
            if strict_config {
                m.set_strict(true);
            }
            m
        }
        Err(e) => {
            tracing::error!("Failed to load configuration: {:?}", e);
            eprintln!("Failed to load configuration: {e:?}");
            std::process::exit(1);
        }
    };
    if config_manager.watched_path().is_some() {
        tracing::info!(
            "Configuration loaded and watching {}",
            config_manager.watched_path().unwrap().display()
        );
    } else {
        tracing::info!("Configuration loaded");
    }

    match command {
        // Default (no subcommand): run the server, unchanged.
        None => {
            let server_result = server::start_server(config_manager).await;

            // Flush buffered OTLP spans before exit.
            if let Some(provider) = &otel_provider {
                let _ = provider.shutdown();
            }

            if let Err(e) = server_result {
                tracing::error!("Server error: {:?}", e);
                eprintln!("Server error: {e:?}");
                std::process::exit(1);
            }
        }
        // `eval …`: run the harness and exit with its status code (CI gate).
        Some(Command::Eval { action }) => {
            let code = uar::eval::cli::run_eval(&config_manager.current(), &action).await;

            // Flush buffered OTLP spans before the hard exit.
            if let Some(provider) = &otel_provider {
                let _ = provider.shutdown();
            }
            std::process::exit(code);
        }
        // `compile <path>`: compile+sign a single .agent.md document (CH-15).
        Some(Command::Compile { path, out }) => {
            let code = uar::compiler::run_compile(&path, out.as_deref()).await;

            if let Some(provider) = &otel_provider {
                let _ = provider.shutdown();
            }
            std::process::exit(code);
        }
    }
}
