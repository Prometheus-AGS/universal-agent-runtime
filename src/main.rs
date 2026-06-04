//! Axum + Leptos + HTMX + Web Components Server
//!
//! Entry point for the agentic streaming LLM application.

use mimalloc::MiMalloc;

/// Global allocator for improved performance (M-MIMALLOC-APPS).
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use clap::Parser as _;
use dotenvy::dotenv;
use std::sync::Arc;
use universal_agent_runtime::config::{AppConfig, Cli, Command, LogFormat};
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
    // Take the subcommand out before consuming `cli`; `load_with_cli` ignores it.
    let command = cli.command.take();

    let config = match AppConfig::load_with_cli(cli) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to load configuration: {:?}", e);
            std::process::exit(1);
        }
    };
    tracing::info!("Configuration loaded: {:?}", config);

    match command {
        // Default (no subcommand): run the server, unchanged.
        None => {
            let server_result = server::start_server(config).await;

            // Flush buffered OTLP spans before exit.
            if let Some(provider) = &otel_provider {
                let _ = provider.shutdown();
            }

            if let Err(e) = server_result {
                tracing::error!("Server error: {:?}", e);
                std::process::exit(1);
            }
        }
        // `eval …`: run the harness and exit with its status code (CI gate).
        Some(Command::Eval { action }) => {
            let code = uar::eval::cli::run_eval(&config, &action).await;

            // Flush buffered OTLP spans before the hard exit.
            if let Some(provider) = &otel_provider {
                let _ = provider.shutdown();
            }
            std::process::exit(code);
        }
    }
}
