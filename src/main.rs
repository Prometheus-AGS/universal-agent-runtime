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
use universal_agent_runtime::config::{AppConfig, Cli, load_llm_settings};
use universal_agent_runtime::server;
use universal_agent_runtime::uar;

#[tokio::main]
async fn main() {
    // Initialize Telemetry (Logging, Tracing, Metrics)
    uar::telemetry::init();

    // Parse CLI args first — this lets clap handle --version, --help, and
    // other early-exit flags BEFORE any config loading or env-var validation.
    let cli = Cli::parse();

    // Load .env (if present)
    let _ = dotenv();

    // Load Configuration (CLI > Env > File)
    let config = match AppConfig::load_with_cli(cli) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to load configuration: {:?}", e);
            std::process::exit(1);
        }
    };
    tracing::info!("Configuration loaded: {:?}", config);

    // Load LLM settings
    let settings = match load_llm_settings() {
        Ok(s) => s,
        Err(msg) => {
            eprintln!("Configuration error: {msg}");
            std::process::exit(1);
        }
    };

    if let Err(e) = server::start_server(config, settings).await {
        tracing::error!("Server error: {:?}", e);
        std::process::exit(1);
    }
}
