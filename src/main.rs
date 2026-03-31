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
use universal_agent_runtime::config::{AppConfig, Cli};
use universal_agent_runtime::server;
use universal_agent_runtime::uar;

#[tokio::main]
async fn main() {
    uar::telemetry::init();

    let cli = Cli::parse();

    let _ = dotenv();

    let config = match AppConfig::load_with_cli(cli) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to load configuration: {:?}", e);
            std::process::exit(1);
        }
    };
    tracing::info!("Configuration loaded: {:?}", config);

    if let Err(e) = server::start_server(config).await {
        tracing::error!("Server error: {:?}", e);
        std::process::exit(1);
    }
}
