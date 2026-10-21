#![recursion_limit = "256"]

//! Axum + Leptos + HTMX + Web Components Server
//!
//! Entry point for the agentic streaming LLM application.

use mimalloc::MiMalloc;

/// Global allocator for improved performance (M-MIMALLOC-APPS).
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use clap::Parser as _;
use dotenvy::from_path;
use futures::FutureExt as _;
use std::path::PathBuf;
use universal_agent_runtime::config::{Cli, Command, CredentialAction, LogFormat};
use universal_agent_runtime::config_manager::ConfigManager;
use universal_agent_runtime::server;
use universal_agent_runtime::uar;

fn main() -> std::process::ExitCode {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("Failed to create UAR async runtime: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    std::process::ExitCode::from(runtime.block_on(run_main()))
}

async fn run_main() -> u8 {
    // .env is NOT loaded implicitly. An unconditional dotenv() here silently
    // promoted a checkout-local .env over the operator's explicit --config /
    // UAR_LLM__* environment (issue #419): dotenvy ran before clap, clap's
    // `env = "LLM_MODEL"` binding captured it, and the CLI tier applied it as a
    // set_override that no YAML or UAR_* source could beat. Load env files only
    // through the explicit path below.
    if let Some(path) = selected_env_file() {
        if let Err(error) = from_path(&path) {
            eprintln!(
                "Failed to load selected environment file '{}': {error}",
                path.display()
            );
            return 1;
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
            return 1;
        }
        return 0;
    }

    let otel_provider =
        match uar::telemetry::init(&log_format, "info,universal_agent_runtime=debug") {
            Ok(provider) => provider,
            Err(error) => {
                eprintln!("Failed to initialize telemetry: {error:#}");
                return 1;
            }
        };
    let mut configuration_owner = None;
    let outcome = std::panic::AssertUnwindSafe(async {
        uar::telemetry::metrics::init();
        let configuration = if command.is_some() {
            // One-shot commands need a stable snapshot, not a live producer.
            ConfigManager::load_without_watcher(cli).await
        } else {
            ConfigManager::load(cli).await
        };
        let config_manager = configuration.map_err(|error| {
            anyhow::anyhow!("Failed to load configuration: {error:?}")
        })?;
        configuration_owner = Some(std::sync::Arc::clone(&config_manager));
        // Strict mode is enabled if the operator passed `--strict-config` or set
        // `UAR_STRICT_CONFIG=true`. It makes any runtime reload that would change
        // the effective configuration an error.
        if strict_config {
            config_manager.set_strict(true);
        }
        if config_manager.watched_path().is_some() {
            tracing::info!(
                "Configuration loaded and watching {}",
                config_manager.watched_path().unwrap().display()
            );
        } else {
            tracing::info!("Configuration loaded");
        }

        let code = match command {
            // Default (no subcommand): run the server, unchanged.
            None => {
                server::start_server(config_manager).await?;
                0
            }
            Some(Command::Credentials { action }) => {
                let CredentialAction::InitializeRevisions { writers_quiesced } = action;
                let result = uar::security::credentials::maintenance::initialize_revisions(
                    &config_manager.current(), writers_quiesced,
                ).await;
                match result {
                    Ok(summary) => match serde_json::to_string(&summary) {
                        Ok(summary) => { println!("{summary}"); 0 }
                        Err(_) => { eprintln!("credential_revision_output_failed"); 1 }
                    },
                    Err(_) => {
                        eprintln!("credential_revision_initialization_refused_or_unavailable");
                        1
                    }
                }
            }
            // `eval …`: preserve the harness status after owned cleanup.
            Some(Command::Eval { action }) => {
                uar::eval::cli::run_eval(&config_manager.current(), &action).await
            }
            // `compile <path>`: compile+sign a single .agent.md document (CH-15).
            Some(Command::Compile { path, out }) => {
                uar::compiler::run_compile(&path, out.as_deref()).await
            }
            #[cfg(windows)]
            Some(Command::Service) => unreachable!("Windows service handled before telemetry setup"),
        };
        Ok::<i32, anyhow::Error>(code)
    }).catch_unwind().await;

    let mut code = match outcome {
        Ok(Ok(code)) => code as u8,
        Ok(Err(error)) => {
            tracing::error!(%error, "UAR command failed");
            eprintln!("UAR command failed: {error:#}");
            1
        }
        Err(_) => {
            eprintln!("UAR command panicked; finalizing owned resources");
            1
        }
    };
    if let Some(configuration) = configuration_owner {
        match std::panic::AssertUnwindSafe(configuration.close_watchers()).catch_unwind().await {
            Ok(Ok(())) => {},
            Ok(Err(error)) => {
                eprintln!("UAR configuration cleanup failed: {error:#}");
                if code == 0 { code = 1; }
            }
            Err(_) => {
                eprintln!("UAR configuration cleanup panicked; release is unconfirmed");
                if code == 0 { code = 1; }
            }
        }
    }
    if let Some(provider) = &otel_provider {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| provider.shutdown())) {
            Ok(Ok(())) => {},
            Ok(Err(error)) => {
                eprintln!("UAR telemetry cleanup failed: {error}");
                if code == 0 { code = 1; }
            }
            Err(_) => {
                eprintln!("UAR telemetry cleanup panicked; release is unconfirmed");
                if code == 0 { code = 1; }
            }
        }
    }
    code
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
