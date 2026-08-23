//! Axum + Leptos + HTMX + Web Components Server
//!
//! Entry point for the agentic streaming LLM application.

use mimalloc::MiMalloc;

/// Global allocator for improved performance (M-MIMALLOC-APPS).
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use clap::Parser as _;
use dotenvy::{dotenv, from_path};
use std::path::PathBuf;
use universal_agent_runtime::config::{Cli, Command, LogFormat};
use universal_agent_runtime::config_manager::ConfigManager;
use universal_agent_runtime::server;
use universal_agent_runtime::uar;

#[tokio::main]
async fn main() {
    let _ = dotenv();

    if let Some(path) = selected_env_file() {
        if let Err(error) = from_path(&path) {
            eprintln!(
                "Failed to load selected environment file '{}': {error}",
                path.display()
            );
            std::process::exit(1);
        }
    }

    let mut cli = Cli::parse();

    // Take the subcommand out before consuming `cli`; load_with_cli ignores it.
    let command = cli.command.take();
    let strict_config = cli.strict_config;

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

    #[cfg(windows)]
    if matches!(&command, Some(Command::Service)) {
        if let Err(error) = server::run_windows_service(cli, log_format) {
            eprintln!("Windows service failed: {error:#}");
            std::process::exit(1);
        }
        return;
    }

    let otel_provider = match uar::telemetry::init(&log_format) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("Failed to initialize telemetry: {error:#}");
            std::process::exit(1);
        }
    };
    uar::telemetry::metrics::init();

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
        #[cfg(windows)]
        Some(Command::Service) => unreachable!("Windows service handled before telemetry setup"),
    }
}

fn selected_env_file() -> Option<PathBuf> {
    let mut args = std::env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--env-file" {
            return args.next().and_then(|value| {
                (!value.to_string_lossy().starts_with('-')).then(|| PathBuf::from(value))
            });
        }

        if let Some(argument) = argument.to_str()
            && let Some(path) = argument.strip_prefix("--env-file=")
            && !path.is_empty()
        {
            return Some(PathBuf::from(path));
        }
    }

    std::env::var_os("UAR_ENV_FILE").map(PathBuf::from)
}
